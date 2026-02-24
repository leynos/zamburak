//! I/O helpers for the `monty_fork_review` binary.

use std::io;

use camino::{Utf8Path, Utf8PathBuf};
use cap_std::{ambient_authority, fs_utf8};

use super::ReviewError;

/// Reads a unified patch file from either an absolute or relative path.
///
/// Absolute paths are resolved by opening the path's parent directory and then
/// reading the file by name. Relative paths are resolved against the current
/// working directory (`.`), not an inferred repository root. Missing files,
/// unreadable files, and invalid absolute-path shapes (for example, no parent
/// directory or file name) are reported as [`ReviewError::Io`].
pub(super) fn read_patch_from_file(path: &Utf8Path) -> Result<String, ReviewError> {
    if path.is_absolute() {
        return read_patch_from_absolute_path(path);
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

fn read_patch_from_absolute_path(path: &Utf8Path) -> Result<String, ReviewError> {
    let Some(parent_dir_path) = path.parent() else {
        return Err(ReviewError::Io {
            path: path.to_path_buf(),
            source: io::Error::other("absolute path has no parent directory"),
        });
    };
    let Some(file_name) = path.file_name() else {
        return Err(ReviewError::Io {
            path: path.to_path_buf(),
            source: io::Error::other("absolute path has no file name"),
        });
    };
    let parent_dir =
        fs_utf8::Dir::open_ambient_dir(parent_dir_path, ambient_authority()).map_err(|source| {
            ReviewError::Io {
                path: parent_dir_path.to_path_buf(),
                source,
            }
        })?;

    parent_dir
        .read_to_string(file_name)
        .map_err(|source| ReviewError::Io {
            path: path.to_path_buf(),
            source,
        })
}
