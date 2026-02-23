//! Review-policy checker for `full-monty` submodule deltas.

use std::env;
use std::io::{self, Write};
use std::process::{Command, ExitCode};

use camino::{Utf8Path, Utf8PathBuf};
use cap_std::{ambient_authority, fs_utf8};
use zamburak::monty_fork_policy_contract::{MontyForkViolation, evaluate_patch_text};

const DEFAULT_SUBMODULE_PATH: &str = "third_party/full-monty";

type SuperprojectRev = String;

#[derive(Debug)]
struct CliArgs {
    diff_file: Option<Utf8PathBuf>,
    submodule_path: Utf8PathBuf,
    base_superproject_rev: Option<SuperprojectRev>,
    head_superproject_rev: Option<SuperprojectRev>,
}

#[derive(Debug)]
enum ReviewError {
    InvalidArgument(Box<str>),
    MissingArgumentValue(Box<str>),
    MissingRevisionPair,
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
            discard_write_result(writeln!(stderr, "monty-fork-review error: {error}"));
            if let ReviewError::Violations { violations } = error {
                emit_violations(&mut stderr, &violations);
            }
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), ReviewError> {
    let cli_args = parse_cli_args(env::args().skip(1).collect())?;
    let patch_text = resolve_patch_text(&cli_args)?;
    let violations = evaluate_patch_text(&patch_text);

    if violations.is_empty() {
        let mut stdout = io::stdout().lock();
        discard_write_result(writeln!(stdout, "monty-fork-review: pass (0 violation(s))"));
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
                base_superproject_rev = Some(rev);
            }
            "--head-superproject-rev" => {
                let Some(rev) = args.next() else {
                    return Err(ReviewError::MissingArgumentValue(flag.into_boxed_str()));
                };
                head_superproject_rev = Some(rev);
            }
            "--help" | "-h" => {
                print_usage();
                return Ok(CliArgs {
                    diff_file: Some(Utf8PathBuf::from("/dev/null")),
                    submodule_path,
                    base_superproject_rev,
                    head_superproject_rev,
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
    })
}

fn resolve_patch_text(cli_args: &CliArgs) -> Result<String, ReviewError> {
    if let Some(diff_file) = &cli_args.diff_file {
        return read_patch_from_file(diff_file);
    }

    let Some(base_rev) = &cli_args.base_superproject_rev else {
        return Err(ReviewError::MissingRevisionPair);
    };
    let Some(head_rev) = &cli_args.head_superproject_rev else {
        return Err(ReviewError::MissingRevisionPair);
    };

    build_patch_from_submodule_range(&cli_args.submodule_path, base_rev, head_rev)
}

fn read_patch_from_file(path: &Utf8Path) -> Result<String, ReviewError> {
    if path.is_absolute() {
        let root_dir =
            fs_utf8::Dir::open_ambient_dir("/", ambient_authority()).map_err(|source| {
                ReviewError::Io {
                    path: Utf8PathBuf::from("/"),
                    source,
                }
            })?;
        let relative_path = path.strip_prefix("/").map_err(|source| ReviewError::Io {
            path: path.to_path_buf(),
            source: io::Error::other(source.to_string()),
        })?;

        return root_dir
            .read_to_string(relative_path)
            .map_err(|source| ReviewError::Io {
                path: path.to_path_buf(),
                source,
            });
    }

    let current_dir =
        fs_utf8::Dir::open_ambient_dir(".", ambient_authority()).map_err(|source| {
            ReviewError::Io {
                path: Utf8PathBuf::from("."),
                source,
            }
        })?;

    current_dir
        .read_to_string(path)
        .map_err(|source| ReviewError::Io {
            path: path.to_path_buf(),
            source,
        })
}

fn build_patch_from_submodule_range(
    submodule_path: &Utf8Path,
    base_superproject_rev: &str,
    head_superproject_rev: &str,
) -> Result<String, ReviewError> {
    let base_pointer_option = try_resolve_submodule_pointer(base_superproject_rev, submodule_path)?;
    let head_pointer_option = try_resolve_submodule_pointer(head_superproject_rev, submodule_path)?;

    let Some(base_pointer) = base_pointer_option else {
        return Ok(String::new());
    };
    let Some(head_pointer) = head_pointer_option else {
        return Ok(String::new());
    };

    if base_pointer == head_pointer {
        return Ok(String::new());
    }

    fetch_submodule_commits(submodule_path, &base_pointer, &head_pointer)?;
    diff_submodule_commits(submodule_path, &base_pointer, &head_pointer)
}

fn try_resolve_submodule_pointer(
    superproject_rev: &str,
    submodule_path: &Utf8Path,
) -> Result<Option<String>, ReviewError> {
    let output = run_submodule_pointer_command(superproject_rev, submodule_path)?;

    if output.status.success() {
        return Ok(Some(
            String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        ));
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if pointer_not_present_in_revision(stderr.as_ref()) {
        return Ok(None);
    }

    Err(ReviewError::CommandFailed {
        cmd: format!(
            "git rev-parse {superproject_rev}:{}",
            submodule_path.as_str()
        )
        .into_boxed_str(),
        stderr: stderr.trim().into(),
    })
}

fn run_submodule_pointer_command(
    superproject_rev: &str,
    submodule_path: &Utf8Path,
) -> Result<std::process::Output, ReviewError> {
    let object_spec = format!("{superproject_rev}:{}", submodule_path.as_str());
    Command::new("git")
        .args(["rev-parse", &object_spec])
        .output()
        .map_err(|source| ReviewError::Command {
            cmd: format!("git rev-parse {object_spec}").into_boxed_str(),
            source,
        })
}

fn pointer_not_present_in_revision(stderr: &str) -> bool {
    stderr.contains("exists on disk, but not in")
        || stderr.contains("unknown revision or path not in the working tree")
        || stderr.contains("pathspec")
}

fn fetch_submodule_commits(
    submodule_path: &Utf8Path,
    base_pointer: &str,
    head_pointer: &str,
) -> Result<(), ReviewError> {
    let submodule_arg = submodule_path.as_str();
    let cmd = format!(
        "git -C {submodule_arg} fetch --quiet --depth=1 origin {base_pointer} {head_pointer}"
    );
    let output = Command::new("git")
        .args([
            "-C",
            submodule_arg,
            "fetch",
            "--quiet",
            "--depth=1",
            "origin",
            base_pointer,
            head_pointer,
        ])
        .output()
        .map_err(|source| ReviewError::Command {
            cmd: cmd.clone().into_boxed_str(),
            source,
        })?;

    if output.status.success() {
        return Ok(());
    }

    Err(ReviewError::CommandFailed {
        cmd: cmd.into_boxed_str(),
        stderr: String::from_utf8_lossy(&output.stderr).trim().into(),
    })
}

fn diff_submodule_commits(
    submodule_path: &Utf8Path,
    base_pointer: &str,
    head_pointer: &str,
) -> Result<String, ReviewError> {
    let submodule_arg = submodule_path.as_str();
    let cmd = format!("git -C {submodule_arg} diff --unified=0 {base_pointer} {head_pointer}");
    let output = Command::new("git")
        .args([
            "-C",
            submodule_arg,
            "diff",
            "--unified=0",
            base_pointer,
            head_pointer,
        ])
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

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn emit_violations(stderr: &mut io::StderrLock<'_>, violations: &[MontyForkViolation]) {
    discard_write_result(writeln!(
        stderr,
        "monty-fork-review: fail ({})",
        violations.len()
    ));

    for violation in violations {
        discard_write_result(writeln!(
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

fn print_usage() {
    let mut stdout = io::stdout().lock();
    discard_write_result(writeln!(
        stdout,
        concat!(
            "Usage:\n",
            "  monty_fork_review --diff-file <PATH>\n",
            "  monty_fork_review --base-superproject-rev <REV> ",
            "--head-superproject-rev <REV> [--submodule-path <PATH>]"
        )
    ));
}

fn discard_write_result(write_result: io::Result<()>) {
    drop(write_result);
}
