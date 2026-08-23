"""End-to-end checks that the tracked ``typos.toml`` enforces the intended policy.

The shared dictionary ignores fenced code blocks wholesale but checks inline
backtick spans, so intentionally US-spelled identifiers rely on narrow
``typos.local.toml`` patterns. These tests exercise the real ``typos`` binary
against a toy corpus to keep that boundary honest.
"""

from __future__ import annotations

import json
import os
import re
import shutil
import subprocess
import typing as typ
from pathlib import Path

import pytest

REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
CONFIG = REPOSITORY_ROOT / "typos.toml"
MAKEFILE = REPOSITORY_ROOT / "Makefile"
TYPOS_VERSION_PATTERN = re.compile(r"^TYPOS_VERSION\s*\??=\s*(\S+)", re.MULTILINE)


def _typos_version() -> str:
    """Return the ``typos`` version pinned by the Makefile."""
    match = TYPOS_VERSION_PATTERN.search(MAKEFILE.read_text(encoding="utf-8"))
    if match is None:
        pytest.fail(f"no TYPOS_VERSION pin found in {MAKEFILE}")
    return match.group(1)


def _run_typos(corpus: Path) -> list[dict[str, typ.Any]]:
    """Spell-check ``corpus`` with the tracked config and return typo records."""
    if shutil.which("uv") is None:
        pytest.skip("uv is unavailable; cannot run the pinned typos binary")
    environment = os.environ | {
        "UV_CACHE_DIR": str(REPOSITORY_ROOT / ".uv-cache"),
        "UV_TOOL_DIR": str(REPOSITORY_ROOT / ".uv-tools"),
    }
    command = [
        "uv",
        "tool",
        "run",
        f"typos@{_typos_version()}",
        "--config",
        str(CONFIG),
        "--force-exclude",
        "--format",
        "json",
        str(corpus),
    ]
    try:
        completed = subprocess.run(  # noqa: S603
            command,
            capture_output=True,
            text=True,
            check=False,
            cwd=REPOSITORY_ROOT,
            env=environment,
        )
    except OSError as error:  # pragma: no cover - environment failure
        pytest.skip(f"could not invoke typos: {error}")
    if completed.returncode not in {0, 2}:
        pytest.skip(f"typos could not run: {completed.stderr.strip()}")
    records = [json.loads(line) for line in completed.stdout.splitlines() if line]
    return [record for record in records if record.get("type") == "typo"]


def _write_corpus(directory: Path, body: str) -> Path:
    """Write ``body`` to a Markdown file inside ``directory``."""
    corpus = directory / "corpus.md"
    corpus.write_text(body, encoding="utf-8")
    return corpus


def _typos_found(directory: Path, body: str) -> set[str]:
    """Return the distinct misspellings reported for ``body``."""
    return {record["typo"] for record in _run_typos(_write_corpus(directory, body))}


@pytest.mark.parametrize("identifier", ["color", "mold"])
def test_inline_backticked_identifiers_are_accepted(
    tmp_path: Path,
    identifier: str,
) -> None:
    """Narrow local patterns exempt the quoted US-spelled identifiers."""
    body = f"The `{identifier}` identifier keeps its upstream spelling.\n"
    assert _typos_found(tmp_path, body) == set()


@pytest.mark.parametrize("identifier", ["color", "mold"])
def test_plain_prose_uses_are_still_reported(
    tmp_path: Path,
    identifier: str,
) -> None:
    """The exemptions are span-scoped and do not excuse ordinary prose."""
    body = f"Ordinary prose about {identifier} must still be corrected.\n"
    assert _typos_found(tmp_path, body) == {identifier}


def test_inline_spans_do_not_exempt_arbitrary_words(tmp_path: Path) -> None:
    """Inline backticks are checked; only the listed spans are exempt."""
    body = "An unlisted `behavior` span is not exempt.\n"
    assert _typos_found(tmp_path, body) == {"behavior"}


def test_fenced_code_blocks_are_ignored_wholesale(tmp_path: Path) -> None:
    """Fenced blocks remain exempt without any local pattern."""
    body = "```rust\nlet behavior = flavor();\n```\n"
    assert _typos_found(tmp_path, body) == set()
