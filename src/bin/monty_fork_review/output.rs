//! Output helpers for the `monty_fork_review` binary.

use std::io::{self, Write};

pub(crate) fn print_usage() {
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

pub(crate) fn discard_write_result(write_result: io::Result<()>) {
    drop(write_result);
}
