//! Contracts for enforcing `full-monty` fork genericity rules.
//!
//! This module encodes the non-generic semantic guardrails from
//! `docs/adr-001-monty-ifc-vm-hooks.md` so CI can fail closed when a patch adds
//! Zamburak semantics to Track A API surface.

/// Allowed high-level change categories for the `full-monty` fork.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MontyForkChangeCategory {
    /// Stable, host-only runtime IDs.
    StableRuntimeIds,
    /// Generic observer/event hook substrate.
    GenericObserverHooks,
    /// Generic snapshot-extension seam owned by the embedder.
    GenericSnapshotExtension,
    /// Narrow refactors required to enable approved categories.
    EnablingRefactor,
}

/// Machine-readable violation identifiers from fork policy checks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MontyForkViolationCode {
    /// Added API surface line includes forbidden semantics.
    ForbiddenSemanticTokenInApi,
}

/// One policy violation emitted by the fork guardrails.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MontyForkViolation {
    /// Stable violation code.
    pub code: MontyForkViolationCode,
    /// File path extracted from the patch where the violation was detected.
    pub path: Box<str>,
    /// 1-based patch line index for diagnostics.
    pub line_number: usize,
    /// Line content that triggered the violation.
    pub line: Box<str>,
    /// Lower-case forbidden token that matched.
    pub matched_token: &'static str,
}

const FORBIDDEN_SEMANTIC_TOKENS: [&str; 4] = ["zamburak", "taint", "policy", "capabilit"];

/// Returns all allowed fork categories in deterministic order.
///
/// # Examples
///
/// ```rust
/// use zamburak::monty_fork_policy_contract::{
///     MontyForkChangeCategory, allowed_change_categories,
/// };
///
/// assert_eq!(
///     allowed_change_categories(),
///     [
///         MontyForkChangeCategory::StableRuntimeIds,
///         MontyForkChangeCategory::GenericObserverHooks,
///         MontyForkChangeCategory::GenericSnapshotExtension,
///         MontyForkChangeCategory::EnablingRefactor,
///     ]
/// );
/// ```
#[must_use]
pub const fn allowed_change_categories() -> [MontyForkChangeCategory; 4] {
    [
        MontyForkChangeCategory::StableRuntimeIds,
        MontyForkChangeCategory::GenericObserverHooks,
        MontyForkChangeCategory::GenericSnapshotExtension,
        MontyForkChangeCategory::EnablingRefactor,
    ]
}

/// Evaluates added lines and returns non-generic API-surface violations.
///
/// This helper is intended for focused checks where only added source lines are
/// available and file context is not required.
///
/// # Examples
///
/// ```rust
/// use zamburak::monty_fork_policy_contract::evaluate_added_lines;
///
/// let lines = ["pub enum RuntimeEvent { ValueCreated }"];
/// assert!(evaluate_added_lines(&lines).is_empty());
/// ```
#[must_use]
pub fn evaluate_added_lines(added_lines: &[&str]) -> Vec<MontyForkViolation> {
    added_lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| violation_for_added_line("<unknown>", index + 1, line))
        .collect()
}

/// Evaluates unified patch text for non-generic API-surface additions.
///
/// The check is fail-closed and case-insensitive for forbidden terms from ADR
/// 001, currently `zamburak`, `taint`, `policy`, and capability-family terms.
///
/// # Examples
///
/// ```rust
/// use zamburak::monty_fork_policy_contract::evaluate_patch_text;
///
/// let patch = concat!(
///     "diff --git a/src/run.rs b/src/run.rs\n",
///     "+++ b/src/run.rs\n",
///     "+pub struct RuntimeObserver;\n",
/// );
///
/// assert!(evaluate_patch_text(patch).is_empty());
/// ```
#[must_use]
pub fn evaluate_patch_text(patch_text: &str) -> Vec<MontyForkViolation> {
    let mut current_path = "<unknown>";

    patch_text
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            if let Some(path) = line.strip_prefix("+++ b/") {
                current_path = path;
                return None;
            }

            line.strip_prefix('+').and_then(|added_line| {
                if line.starts_with("+++") {
                    return None;
                }
                violation_for_added_line(current_path, index + 1, added_line)
            })
        })
        .collect()
}

fn violation_for_added_line(
    path: &str,
    line_number: usize,
    added_line: &str,
) -> Option<MontyForkViolation> {
    if !is_api_surface_line(added_line) {
        return None;
    }

    let normalized_line = added_line.to_ascii_lowercase();
    FORBIDDEN_SEMANTIC_TOKENS
        .iter()
        .find(|token| normalized_line.contains(**token))
        .copied()
        .map(|matched_token| MontyForkViolation {
            code: MontyForkViolationCode::ForbiddenSemanticTokenInApi,
            path: path.into(),
            line_number,
            line: added_line.into(),
            matched_token,
        })
}

fn is_api_surface_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("pub ")
        || trimmed.starts_with("pub(")
        || trimmed.starts_with("trait ")
        || trimmed.starts_with("pub trait ")
        || trimmed.starts_with("///")
}

#[cfg(test)]
mod tests {
    use super::{
        MontyForkViolationCode, allowed_change_categories, evaluate_added_lines,
        evaluate_patch_text,
    };

    #[test]
    fn allowed_categories_are_stable_and_complete() {
        let categories = allowed_change_categories();
        assert_eq!(categories.len(), 4);
    }

    #[test]
    fn generic_observer_api_addition_passes() {
        let patch = concat!(
            "diff --git a/src/run.rs b/src/run.rs\n",
            "+++ b/src/run.rs\n",
            "+pub enum RuntimeEvent { ValueCreated }\n",
        );

        assert!(evaluate_patch_text(patch).is_empty());
    }

    #[test]
    fn non_api_addition_with_forbidden_term_is_ignored() {
        let patch = concat!(
            "diff --git a/src/run.rs b/src/run.rs\n",
            "+++ b/src/run.rs\n",
            "+let zamburak_marker = 1_u8;\n",
        );

        assert!(evaluate_patch_text(patch).is_empty());
    }

    #[test]
    fn public_api_addition_with_zamburak_term_is_rejected() {
        let patch = concat!(
            "diff --git a/src/run.rs b/src/run.rs\n",
            "+++ b/src/run.rs\n",
            "+pub struct ZamburakObserver;\n",
        );

        let violations = evaluate_patch_text(patch);
        assert_eq!(violations.len(), 1);
        let Some(first_violation) = violations.first() else {
            panic!("expected one violation for zamburak term");
        };
        assert_eq!(
            first_violation.code,
            MontyForkViolationCode::ForbiddenSemanticTokenInApi
        );
        assert_eq!(first_violation.matched_token, "zamburak");
    }

    #[test]
    fn check_is_case_insensitive_for_policy_token() {
        let added_lines = ["pub fn ApplyPolicy() {}"];
        let violations = evaluate_added_lines(&added_lines);
        assert_eq!(violations.len(), 1);
        let Some(first_violation) = violations.first() else {
            panic!("expected one violation for policy token");
        };
        assert_eq!(first_violation.matched_token, "policy");
    }

    #[test]
    fn api_doc_comment_with_capability_term_is_rejected() {
        let patch = concat!(
            "diff --git a/src/run.rs b/src/run.rs\n",
            "+++ b/src/run.rs\n",
            "+/// Capability mapping for host integration.\n",
        );

        let violations = evaluate_patch_text(patch);
        assert_eq!(violations.len(), 1);
        let Some(first_violation) = violations.first() else {
            panic!("expected one violation for capability token");
        };
        assert_eq!(first_violation.matched_token, "capabilit");
    }

    #[test]
    fn mixed_patch_reports_only_api_surface_violations() {
        let patch = concat!(
            "diff --git a/src/run.rs b/src/run.rs\n",
            "+++ b/src/run.rs\n",
            "+let zamburak_marker = 1_u8;\n",
            "+pub enum PolicyEvent { Started }\n",
        );

        let violations = evaluate_patch_text(patch);
        assert_eq!(violations.len(), 1);
        let Some(first_violation) = violations.first() else {
            panic!("expected one violation for mixed patch");
        };
        assert_eq!(first_violation.matched_token, "policy");
        assert_eq!(first_violation.path.as_ref(), "src/run.rs");
    }
}
