//! Review-policy checker for `full-monty` submodule deltas.

use std::env;
use std::io::{self, Write};
use std::process::{Command, ExitCode};

use camino::{Utf8Path, Utf8PathBuf};
use zamburak::monty_fork_policy_contract::{MontyForkViolation, evaluate_patch_text};

#[path = "monty_fork_review/io_utils.rs"]
mod io_utils;
#[path = "monty_fork_review/output.rs"]
mod output;

const DEFAULT_SUBMODULE_PATH: &str = "third_party/full-monty";

#[derive(Debug, Clone, PartialEq, Eq)]
struct GitRevision(String);

impl GitRevision {
    fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
    const fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl AsRef<str> for GitRevision {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SubmodulePointer(String);

impl SubmodulePointer {
    fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
    const fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl AsRef<str> for SubmodulePointer {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

type SuperprojectRev = GitRevision;

#[derive(Debug)]
struct CliArgs {
    diff_file: Option<Utf8PathBuf>,
    submodule_path: Utf8PathBuf,
    base_superproject_rev: Option<SuperprojectRev>,
    head_superproject_rev: Option<SuperprojectRev>,
    show_help: bool,
}

#[derive(Debug)]
enum ReviewError {
    InvalidArgument(Box<str>),
    MissingArgumentValue(Box<str>),
    MissingRevisionPair,
    SubmodulePointerStateChanged {
        submodule_path: Utf8PathBuf,
        base_superproject_rev: GitRevision,
        head_superproject_rev: GitRevision,
    },
    Io {
        path: Utf8PathBuf,
        source: io::Error,
    },
    Command {
        cmd: Box<str>,
        source: io::Error,
    },
    CommandFailed {
        cmd: Box<str>,
        stderr: Box<str>,
    },
    Violations {
        violations: Vec<MontyForkViolation>,
    },
}

impl std::fmt::Display for ReviewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArgument(flag) => write!(f, "unsupported argument `{flag}`"),
            Self::MissingArgumentValue(flag) => {
                write!(f, "missing value for argument `{flag}`")
            }
            Self::MissingRevisionPair => {
                write!(
                    f,
                    "either --diff-file or both --base-superproject-rev and --head-superproject-rev are required"
                )
            }
            Self::SubmodulePointerStateChanged {
                submodule_path,
                base_superproject_rev,
                head_superproject_rev,
            } => write!(
                f,
                "submodule `{submodule_path}` is not present at both revisions (`{}` and `{}`)",
                base_superproject_rev.as_str(),
                head_superproject_rev.as_str(),
            ),
            Self::Io { path, source } => write!(f, "I/O error for `{path}`: {source}"),
            Self::Command { cmd, source } => {
                write!(f, "failed to run command `{cmd}`: {source}")
            }
            Self::CommandFailed { cmd, stderr } => {
                write!(f, "command `{cmd}` failed: {stderr}")
            }
            Self::Violations { violations } => {
                write!(f, "found {} fork-policy violation(s)", violations.len())
            }
        }
    }
}

impl std::error::Error for ReviewError {}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let mut stderr = io::stderr().lock();
            output::discard_write_result(writeln!(stderr, "monty-fork-review error: {error}"));
            if let ReviewError::Violations { violations } = error {
                emit_violations(&mut stderr, &violations);
            }
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), ReviewError> {
    let cli_args = parse_cli_args(env::args().skip(1).collect())?;
    if cli_args.show_help {
        return Ok(());
    }
    let patch_text = resolve_patch_text(&cli_args)?;
    let violations = evaluate_patch_text(&patch_text);

    if violations.is_empty() {
        let mut stdout = io::stdout().lock();
        output::discard_write_result(writeln!(stdout, "monty-fork-review: pass (0 violation(s))"));
        Ok(())
    } else {
        Err(ReviewError::Violations { violations })
    }
}

fn parse_cli_args(raw_args: Vec<String>) -> Result<CliArgs, ReviewError> {
    let mut diff_file = None::<Utf8PathBuf>;
    let mut submodule_path = Utf8PathBuf::from(DEFAULT_SUBMODULE_PATH);
    let mut base_superproject_rev = None::<SuperprojectRev>;
    let mut head_superproject_rev = None::<SuperprojectRev>;
    let mut args = raw_args.into_iter();

    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--diff-file" => {
                let Some(path) = args.next() else {
                    return Err(ReviewError::MissingArgumentValue(flag.into_boxed_str()));
                };
                diff_file = Some(Utf8PathBuf::from(path));
            }
            "--submodule-path" => {
                let Some(path) = args.next() else {
                    return Err(ReviewError::MissingArgumentValue(flag.into_boxed_str()));
                };
                submodule_path = Utf8PathBuf::from(path);
            }
            "--base-superproject-rev" => {
                let Some(rev) = args.next() else {
                    return Err(ReviewError::MissingArgumentValue(flag.into_boxed_str()));
                };
                base_superproject_rev = Some(GitRevision::new(rev));
            }
            "--head-superproject-rev" => {
                let Some(rev) = args.next() else {
                    return Err(ReviewError::MissingArgumentValue(flag.into_boxed_str()));
                };
                head_superproject_rev = Some(GitRevision::new(rev));
            }
            "--help" | "-h" => {
                output::print_usage();
                return Ok(CliArgs {
                    diff_file,
                    submodule_path,
                    base_superproject_rev,
                    head_superproject_rev,
                    show_help: true,
                });
            }
            _ => return Err(ReviewError::InvalidArgument(flag.into_boxed_str())),
        }
    }

    Ok(CliArgs {
        diff_file,
        submodule_path,
        base_superproject_rev,
        head_superproject_rev,
        show_help: false,
    })
}

fn resolve_patch_text(cli_args: &CliArgs) -> Result<String, ReviewError> {
    if let Some(diff_file) = &cli_args.diff_file {
        return io_utils::read_patch_from_file(diff_file);
    }

    let Some(base_rev) = &cli_args.base_superproject_rev else {
        return Err(ReviewError::MissingRevisionPair);
    };
    let Some(head_rev) = &cli_args.head_superproject_rev else {
        return Err(ReviewError::MissingRevisionPair);
    };

    build_patch_from_submodule_range(&cli_args.submodule_path, base_rev, head_rev)
}

fn build_patch_from_submodule_range(
    submodule_path: &Utf8Path,
    base_superproject_rev: &GitRevision,
    head_superproject_rev: &GitRevision,
) -> Result<String, ReviewError> {
    let base_pointer_option = try_resolve_submodule_pointer(base_superproject_rev, submodule_path)?;
    let head_pointer_option = try_resolve_submodule_pointer(head_superproject_rev, submodule_path)?;

    let (base_pointer, head_pointer) = match (base_pointer_option, head_pointer_option) {
        (Some(base_pointer), Some(head_pointer)) => (base_pointer, head_pointer),
        (None, None) => return Ok(String::new()),
        _ => {
            return Err(ReviewError::SubmodulePointerStateChanged {
                submodule_path: submodule_path.to_path_buf(),
                base_superproject_rev: base_superproject_rev.clone(),
                head_superproject_rev: head_superproject_rev.clone(),
            });
        }
    };

    if base_pointer == head_pointer {
        return Ok(String::new());
    }

    fetch_submodule_commits(submodule_path, &base_pointer, &head_pointer)?;
    diff_submodule_commits(submodule_path, &base_pointer, &head_pointer)
}

fn try_resolve_submodule_pointer(
    superproject_rev: &GitRevision,
    submodule_path: &Utf8Path,
) -> Result<Option<SubmodulePointer>, ReviewError> {
    let output = run_submodule_pointer_command(superproject_rev, submodule_path)?;

    if output.status.success() {
        return Ok(Some(SubmodulePointer::new(
            String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        )));
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if pointer_not_present_in_revision(stderr.as_ref()) {
        return Ok(None);
    }

    Err(ReviewError::CommandFailed {
        cmd: format!(
            "git rev-parse {}:{}",
            superproject_rev.as_str(),
            submodule_path.as_str()
        )
        .into_boxed_str(),
        stderr: stderr.trim().into(),
    })
}

fn run_submodule_pointer_command(
    superproject_rev: &GitRevision,
    submodule_path: &Utf8Path,
) -> Result<std::process::Output, ReviewError> {
    let object_spec = format!("{}:{}", superproject_rev.as_str(), submodule_path.as_str());
    Command::new("git")
        .args(["rev-parse", &object_spec])
        .output()
        .map_err(|source| ReviewError::Command {
            cmd: format!("git rev-parse {object_spec}").into_boxed_str(),
            source,
        })
}

fn pointer_not_present_in_revision(stderr: &str) -> bool {
    stderr.contains("exists on disk, but not in") || stderr.contains("does not exist in")
}

fn run_git_command_in_submodule(
    submodule_path: &Utf8Path,
    args: &[&str],
    capture_stdout: bool,
) -> Result<String, ReviewError> {
    let submodule_arg = submodule_path.as_str();
    let joined_args = args.join(" ");
    let cmd = format!("git -C {submodule_arg} {joined_args}");
    let output = Command::new("git")
        .arg("-C")
        .arg(submodule_arg)
        .args(args)
        .output()
        .map_err(|source| ReviewError::Command {
            cmd: cmd.clone().into_boxed_str(),
            source,
        })?;

    if !output.status.success() {
        return Err(ReviewError::CommandFailed {
            cmd: cmd.into_boxed_str(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().into(),
        });
    }

    if capture_stdout {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Ok(String::new())
    }
}

fn fetch_submodule_commits(
    submodule_path: &Utf8Path,
    base_pointer: &SubmodulePointer,
    head_pointer: &SubmodulePointer,
) -> Result<(), ReviewError> {
    let fetch_args = [
        "fetch",
        "--quiet",
        "--depth=1",
        "origin",
        base_pointer.as_str(),
        head_pointer.as_str(),
    ];
    run_git_command_in_submodule(submodule_path, &fetch_args, false).map(|_| ())
}

fn diff_submodule_commits(
    submodule_path: &Utf8Path,
    base_pointer: &SubmodulePointer,
    head_pointer: &SubmodulePointer,
) -> Result<String, ReviewError> {
    let diff_args = [
        "diff",
        "--unified=0",
        base_pointer.as_str(),
        head_pointer.as_str(),
    ];
    run_git_command_in_submodule(submodule_path, &diff_args, true)
}

fn emit_violations(stderr: &mut io::StderrLock<'_>, violations: &[MontyForkViolation]) {
    output::discard_write_result(writeln!(
        stderr,
        "monty-fork-review: fail ({})",
        violations.len()
    ));

    for violation in violations {
        output::discard_write_result(writeln!(
            stderr,
            "- {:?} at {}:{} matched `{}` in `{}`",
            violation.code,
            violation.path,
            violation.line_number,
            violation.matched_token,
            violation.line,
        ));
    }
}
