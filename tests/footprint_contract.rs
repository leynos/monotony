//! Contract for the development build footprint (Concordat rule
//! `rust-dev-build-footprint`): the default development profile keeps no
//! incremental state and no debug information, and Cargo coordinates metadata
//! output through its native key rather than a raw rustc flag.

const MANIFEST: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"));
const CONFIG: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/.cargo/config.toml"));

/// Returns whether a line carries content: it is neither blank nor a comment.
fn is_content(line: &str) -> bool { !line.is_empty() && !line.starts_with('#') }

/// Returns the non-comment lines of the named top-level table.
fn table<'a>(text: &'a str, header: &str) -> Vec<&'a str> {
    let mut in_table = false;
    let mut lines = Vec::new();
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') {
            in_table = line == header;
        } else if in_table && is_content(line) {
            lines.push(line);
        }
    }
    lines
}

fn sets(lines: &[&str], key: &str, value: &str) -> bool {
    lines.iter().any(|line| {
        line.split_once('=')
            .is_some_and(|(k, v)| k.trim() == key && v.trim() == value)
    })
}

#[test]
fn the_development_profile_disables_incremental_state_and_debug_information() {
    let dev = table(MANIFEST, "[profile.dev]");
    assert!(
        sets(&dev, "incremental", "false"),
        "[profile.dev] must set incremental = false"
    );
    assert!(sets(&dev, "debug", "0"), "[profile.dev] must set debug = 0");
    assert!(
        table(MANIFEST, "[profile.test]").is_empty(),
        "[profile.test] must inherit from [profile.dev], not re-enable either key"
    );
}

#[test]
fn the_configuration_uses_the_native_key_for_metadata_embedding() {
    let unstable = table(CONFIG, "[unstable]");
    assert!(
        sets(&unstable, "embed-metadata", "false"),
        "[unstable] must set embed-metadata = false on a nightly from 2026-07-31"
    );
    assert!(
        !CONFIG
            .lines()
            .filter(|line| !line.trim_start().starts_with('#'))
            .any(|line| line.contains("no-embed-metadata") || line.contains("-Zembed-metadata")),
        "metadata embedding is set through Cargo, not a raw rustc flag"
    );
}
