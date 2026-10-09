//! The coverage recipe's own environment: the linker, the backend and the link arguments a
//! measurement needs, read from the command `make -n coverage` prints.
//!
//! Coverage takes neither standard flag, so its `RUSTFLAGS` is checked as a held-out command by
//! `make`; what this module adds is the rest of what that recipe assigns before `cargo llvm-cov`,
//! as recorded when the contract was written: the LLVM backend (Cranelift cannot instrument code),
//! the clang linker and the lld arguments LLVM's coverage tools expect. Losing or changing any of
//! them, or the link argument in `RUSTFLAGS`, fails the contract. A repository with no local
//! coverage recipe records nothing, and this checks nothing.

use super::{
    config::Problems,
    make::{Command, Host, MakeRunner, Target, make_commands},
    shell::leading_assignments,
};

/// One environment variable the coverage recipe assigns.
#[derive(Clone, Copy)]
pub struct Env {
    pub name: &'static str,
    pub value: &'static str,
}

/// The link argument the coverage `RUSTFLAGS` must carry, such as `-C link-arg=-fuse-ld=lld`.
#[derive(Clone, Copy)]
pub struct LinkArg(pub &'static str);

/// What the coverage recipe assigned when the contract was written, besides `RUSTFLAGS`.
const COVERAGE_ENV: &[Env] = &[
    Env {
        name: "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER",
        value: "clang",
    },
    Env {
        name: "CARGO_PROFILE_DEV_CODEGEN_BACKEND",
        value: "llvm",
    },
    Env {
        name: "CFLAGS",
        value: "-fuse-ld=lld",
    },
    Env {
        name: "LDFLAGS",
        value: "-fuse-ld=lld",
    },
];
/// The link argument its `RUSTFLAGS` carried, or the empty string.
const COVERAGE_LINK: LinkArg = LinkArg("-C link-arg=-fuse-ld=lld");

/// Returns the last value a command assigns to a variable.
fn assigned(assignments: &[(String, String)], name: &str) -> Option<String> {
    assignments
        .iter()
        .rev()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.clone())
}

/// Returns the complaints about the command that runs `cargo llvm-cov`: each recorded variable must
/// be assigned its recorded value, and `RUSTFLAGS` must carry the recorded link argument.
pub fn coverage_env_problems(command: &Command, expected: &[Env], link: LinkArg) -> Problems {
    let assignments = leading_assignments(&command.text);
    let mut problems: Problems = expected
        .iter()
        .filter(|env| assigned(&assignments, env.name).as_deref() != Some(env.value))
        .map(|env| {
            let found = assigned(&assignments, env.name);
            format!(
                "`make coverage` assigns {}={found:?}, not {:?}",
                env.name, env.value
            )
        })
        .collect();
    let flags = assigned(&assignments, "RUSTFLAGS").unwrap_or_default();
    if !link.0.is_empty() && !flags.contains(link.0) {
        problems.push(format!(
            "`make coverage` RUSTFLAGS {flags:?} does not carry `{}`",
            link.0
        ));
    }
    problems
}

/// Returns the complaints about a list of coverage commands: exactly one must run `cargo llvm-cov`.
fn coverage_command_problems(commands: &[Command], expected: &[Env], link: LinkArg) -> Problems {
    let runs: Vec<&Command> = commands
        .iter()
        .filter(|command| command.text.contains("llvm-cov"))
        .collect();
    match runs.as_slice() {
        [command] => coverage_env_problems(command, expected, link),
        other => vec![format!(
            "`make coverage` should run `cargo llvm-cov` once, but runs it {} times",
            other.len()
        )],
    }
}

/// Returns the complaints about the repository's coverage recipe: nothing when it records no
/// environment.
///
/// # Errors
///
/// Returns the reason when the coverage target is not defined or unreadable.
pub fn coverage_recipe_problems(runner: MakeRunner) -> Result<Problems, String> {
    if COVERAGE_ENV.is_empty() && COVERAGE_LINK.0.is_empty() {
        return Ok(Vec::new());
    }
    let commands = make_commands(runner, Target("coverage"), Host::Linux)?;
    Ok(coverage_command_problems(
        &commands,
        COVERAGE_ENV,
        COVERAGE_LINK,
    ))
}

#[cfg(test)]
mod tests {
    //! The coverage environment reader over fixtures: a compliant command passes, each lost or
    //! changed variable and a lost link argument are refused, and the number of coverage
    //! commands is pinned.

    use super::{
        super::make::Assignment,
        Command,
        Env,
        LinkArg,
        coverage_command_problems,
        coverage_env_problems,
    };

    const EXPECTED: &[Env] = &[
        Env {
            name: "CARGO_PROFILE_DEV_CODEGEN_BACKEND",
            value: "llvm",
        },
        Env {
            name: "CFLAGS",
            value: "-fuse-ld=lld",
        },
    ];
    const LINK: LinkArg = LinkArg("-C link-arg=-fuse-ld=lld");

    fn command(text: &str) -> Command {
        Command {
            text: text.to_owned(),
            assignment: Assignment::Unassigned,
        }
    }

    fn problems(text: &str) -> usize { coverage_env_problems(&command(text), EXPECTED, LINK).len() }

    const OK: &str = "CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm RUSTFLAGS=\"-D warnings -C \
                      link-arg=-fuse-ld=lld\" CFLAGS=\"-fuse-ld=lld\" cargo llvm-cov --lcov";

    #[test]
    fn a_command_that_assigns_everything_recorded_is_accepted() {
        assert_eq!(problems(OK), 0);
    }

    #[test]
    fn a_lost_or_changed_variable_is_refused() {
        for lost in [
            OK.replace("CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm ", ""),
            OK.replace("=llvm", "=cranelift"),
            OK.replace("CFLAGS=\"-fuse-ld=lld\" ", ""),
            OK.replace("CFLAGS=\"-fuse-ld=lld\"", "CFLAGS=\"-fuse-ld=gold\""),
        ] {
            assert_eq!(problems(&lost), 1, "{lost}");
        }
    }

    #[test]
    fn a_lost_link_argument_is_refused() {
        assert_eq!(problems(&OK.replace(" -C link-arg=-fuse-ld=lld", "")), 1);
    }

    #[test]
    fn the_last_assignment_of_a_variable_wins() {
        assert_eq!(problems(&format!("CFLAGS=\"-fuse-ld=gold\" {OK}")), 0);
        assert_eq!(
            problems(&OK.replace(" --lcov", " --lcov CFLAGS=gold")),
            0,
            "after the command word is not an assignment"
        );
    }

    #[test]
    fn exactly_one_coverage_command_is_required() {
        assert_eq!(coverage_command_problems(&[], EXPECTED, LINK).len(), 1);
        assert_eq!(
            coverage_command_problems(&[command("echo hi")], EXPECTED, LINK).len(),
            1
        );
        assert_eq!(
            coverage_command_problems(&[command(OK), command(OK)], EXPECTED, LINK).len(),
            1
        );
        assert!(
            coverage_command_problems(&[command("echo hi"), command(OK)], EXPECTED, LINK)
                .is_empty()
        );
    }
}
