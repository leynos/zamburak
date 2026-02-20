//! Phase-gate contract evaluation for CI verification suites.

use std::collections::BTreeSet;

/// A verification suite required by a phase gate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VerificationSuite {
    /// Stable suite identifier used in reports and CI output.
    pub id: &'static str,
    /// Subsystem label used to tie failures to escalation scope.
    pub subsystem: &'static str,
    /// Substring used to match tests from `cargo test -- --list` output.
    pub test_filter: &'static str,
}

/// Phase target for gate enforcement.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PhaseGateTarget {
    /// No acceptance gate is required before Phase 0.
    Phase0,
    /// Gate required before Phase 1 begins.
    Phase1,
    /// Gate required before Phase 2 begins.
    Phase2,
    /// Gate required before Phase 3 begins.
    Phase3,
    /// Gate required before Phase 4 begins.
    Phase4,
    /// Gate required before Phase 5 begins.
    Phase5,
    /// Gate required before roadmap completion.
    Completion,
}

impl PhaseGateTarget {
    /// Returns the stable string label used in CLI output and policy files.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use zamburak::phase_gate_contract::PhaseGateTarget;
    ///
    /// assert_eq!(PhaseGateTarget::Phase1.as_str(), "phase1");
    /// ```
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Phase0 => "phase0",
            Self::Phase1 => "phase1",
            Self::Phase2 => "phase2",
            Self::Phase3 => "phase3",
            Self::Phase4 => "phase4",
            Self::Phase5 => "phase5",
            Self::Completion => "completion",
        }
    }
}

/// Gate status outcome for a target phase.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PhaseGateStatus {
    /// All required suites exist and passed.
    Passed,
    /// One or more required suites are missing from the catalogue.
    MissingSuites,
    /// One or more required suites failed execution.
    FailingSuites,
}

/// Deterministic gate report used by CLI output and tests.
#[derive(Debug, Eq, PartialEq)]
pub struct PhaseGateReport {
    /// Evaluated phase target.
    pub target: PhaseGateTarget,
    /// Final gate status.
    pub status: PhaseGateStatus,
    /// Required suite ids for the target in deterministic order.
    pub required_suite_ids: Vec<&'static str>,
    /// Missing suite ids for the target in deterministic order.
    pub missing_suite_ids: Vec<&'static str>,
    /// Failing suite ids for the target in deterministic order.
    pub failing_suite_ids: Vec<&'static str>,
}

/// Release-blocking causes from the verification target policy.
pub const RELEASE_BLOCKING_CAUSES: [&str; 4] = [
    "security invariant enforcement",
    "policy decision determinism",
    "fail-closed semantics",
    "audit confidentiality constraints",
];

/// Escalation steps from the verification target policy.
pub const ESCALATION_STEPS: [&str; 3] = [
    "freeze merges affecting the failing subsystem",
    "add or update a regression test reproducing the failure",
    "restore gate green status before continuing feature work",
];

const PHASE1_SUITES: [VerificationSuite; 4] = [
    VerificationSuite {
        id: "policy-schema-contract",
        subsystem: "policy schema loader",
        test_filter: "policy_schema_bdd::",
    },
    VerificationSuite {
        id: "llm-sink-enforcement",
        subsystem: "LLM sink enforcement",
        test_filter: "llm_sink_enforcement::",
    },
    VerificationSuite {
        id: "authority-lifecycle",
        subsystem: "authority lifecycle",
        test_filter: "authority_lifecycle_bdd::",
    },
    VerificationSuite {
        id: "localization-contract",
        subsystem: "localization contract",
        test_filter: "localization_contract::",
    },
];

const PHASE2_SUITES: [VerificationSuite; 3] = [
    VerificationSuite {
        id: "container-mutation-fail-closed",
        subsystem: "mutable-container soundness",
        test_filter: "container_mutation_fail_closed::",
    },
    VerificationSuite {
        id: "aliasing-fail-closed",
        subsystem: "aliasing soundness",
        test_filter: "aliasing_fail_closed::",
    },
    VerificationSuite {
        id: "budget-overflow-fail-closed",
        subsystem: "provenance budget overflow",
        test_filter: "budget_overflow_fail_closed::",
    },
];

const PHASE3_SUITES: [VerificationSuite; 2] = [
    VerificationSuite {
        id: "tool-catalogue-pinning",
        subsystem: "tool catalogue boundary",
        test_filter: "tool_catalogue_pinning::",
    },
    VerificationSuite {
        id: "mcp-trust-class",
        subsystem: "MCP trust-class boundary",
        test_filter: "mcp_trust_class::",
    },
];

const PHASE4_SUITES: [VerificationSuite; 1] = [VerificationSuite {
    id: "llm-sink-privacy-boundary-integration",
    subsystem: "LLM sink privacy boundary integration",
    test_filter: "llm_sink_privacy_boundary::",
}];

const PHASE5_SUITES: [VerificationSuite; 2] = [
    VerificationSuite {
        id: "audit-confidentiality",
        subsystem: "audit confidentiality",
        test_filter: "audit_confidentiality::",
    },
    VerificationSuite {
        id: "audit-tamper-evidence",
        subsystem: "audit tamper evidence",
        test_filter: "audit_tamper_evidence::",
    },
];

const COMPLETION_SUITES: [VerificationSuite; 2] = [
    VerificationSuite {
        id: "localization-fallback-ordering",
        subsystem: "localization fallback ordering",
        test_filter: "localization_fallback_ordering::",
    },
    VerificationSuite {
        id: "localization-no-global-state",
        subsystem: "localization no-global-state contract",
        test_filter: "localization_no_global_state::",
    },
];

/// Parses a raw phase-gate label into a typed target.
///
/// # Examples
///
/// ```rust
/// use zamburak::phase_gate_contract::{PhaseGateTarget, parse_phase_gate_target};
///
/// assert_eq!(parse_phase_gate_target("phase1"), Some(PhaseGateTarget::Phase1));
/// assert_eq!(parse_phase_gate_target("invalid"), None);
/// ```
#[must_use]
pub fn parse_phase_gate_target(raw_target: &str) -> Option<PhaseGateTarget> {
    match raw_target.trim() {
        "phase0" | "0" => Some(PhaseGateTarget::Phase0),
        "phase1" | "1" => Some(PhaseGateTarget::Phase1),
        "phase2" | "2" => Some(PhaseGateTarget::Phase2),
        "phase3" | "3" => Some(PhaseGateTarget::Phase3),
        "phase4" | "4" => Some(PhaseGateTarget::Phase4),
        "phase5" | "5" => Some(PhaseGateTarget::Phase5),
        "completion" | "roadmap-completion" => Some(PhaseGateTarget::Completion),
        _ => None,
    }
}

/// Returns the ordered verification suites mandated for the given target.
///
/// # Examples
///
/// ```rust
/// use zamburak::phase_gate_contract::{PhaseGateTarget, required_suites_for_target};
///
/// assert_eq!(required_suites_for_target(PhaseGateTarget::Phase0).len(), 0);
/// assert_eq!(required_suites_for_target(PhaseGateTarget::Phase1).len(), 4);
/// ```
#[must_use]
pub const fn required_suites_for_target(target: PhaseGateTarget) -> &'static [VerificationSuite] {
    match target {
        PhaseGateTarget::Phase0 => &[],
        PhaseGateTarget::Phase1 => &PHASE1_SUITES,
        PhaseGateTarget::Phase2 => &PHASE2_SUITES,
        PhaseGateTarget::Phase3 => &PHASE3_SUITES,
        PhaseGateTarget::Phase4 => &PHASE4_SUITES,
        PhaseGateTarget::Phase5 => &PHASE5_SUITES,
        PhaseGateTarget::Completion => &COMPLETION_SUITES,
    }
}

/// Resolves a suite by its stable identifier.
///
/// # Examples
///
/// ```rust
/// use zamburak::phase_gate_contract::suite_by_id;
///
/// let suite = suite_by_id("authority-lifecycle")
///     .expect("authority lifecycle suite should exist");
/// assert_eq!(suite.id, "authority-lifecycle");
/// ```
#[must_use]
pub fn suite_by_id(suite_id: &str) -> Option<&'static VerificationSuite> {
    required_suites_for_target(PhaseGateTarget::Phase0)
        .iter()
        .chain(required_suites_for_target(PhaseGateTarget::Phase1).iter())
        .chain(required_suites_for_target(PhaseGateTarget::Phase2).iter())
        .chain(required_suites_for_target(PhaseGateTarget::Phase3).iter())
        .chain(required_suites_for_target(PhaseGateTarget::Phase4).iter())
        .chain(required_suites_for_target(PhaseGateTarget::Phase5).iter())
        .chain(required_suites_for_target(PhaseGateTarget::Completion).iter())
        .find(|suite| suite.id == suite_id)
}

/// Evaluates whether a phase target passes, is missing suites, or has failures.
///
/// # Examples
///
/// ```rust
/// use std::collections::BTreeSet;
/// use zamburak::phase_gate_contract::{PhaseGateStatus, PhaseGateTarget, evaluate_phase_gate};
///
/// let report = evaluate_phase_gate(PhaseGateTarget::Phase0, &[], &BTreeSet::new());
/// assert_eq!(report.status, PhaseGateStatus::Passed);
/// ```
#[must_use]
pub fn evaluate_phase_gate(
    target: PhaseGateTarget,
    available_test_names: &[String],
    failing_suite_id_set: &BTreeSet<&'static str>,
) -> PhaseGateReport {
    let suites = required_suites_for_target(target);
    let required_suite_ids = suites.iter().map(|suite| suite.id).collect::<Vec<_>>();

    let missing_suite_ids = suites
        .iter()
        .filter(|suite| {
            !available_test_names
                .iter()
                .any(|name| name.contains(suite.test_filter))
        })
        .map(|suite| suite.id)
        .collect::<Vec<_>>();

    let failing_suite_ids = suites
        .iter()
        .filter(|suite| failing_suite_id_set.contains(suite.id))
        .map(|suite| suite.id)
        .collect::<Vec<_>>();

    let status = compute_phase_gate_status(&missing_suite_ids, &failing_suite_ids);

    PhaseGateReport {
        target,
        status,
        required_suite_ids,
        missing_suite_ids,
        failing_suite_ids,
    }
}

const fn compute_phase_gate_status(
    missing_suite_ids: &[&'static str],
    failing_suite_ids: &[&'static str],
) -> PhaseGateStatus {
    match (missing_suite_ids.is_empty(), failing_suite_ids.is_empty()) {
        (false, _) => PhaseGateStatus::MissingSuites,
        (true, false) => PhaseGateStatus::FailingSuites,
        (true, true) => PhaseGateStatus::Passed,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PhaseGateStatus, PhaseGateTarget, evaluate_phase_gate, parse_phase_gate_target,
        required_suites_for_target,
    };
    use std::collections::BTreeSet;

    use rstest::rstest;

    #[rstest]
    #[case("phase0", Some(PhaseGateTarget::Phase0))]
    #[case("0", Some(PhaseGateTarget::Phase0))]
    #[case("phase1", Some(PhaseGateTarget::Phase1))]
    #[case("phase5", Some(PhaseGateTarget::Phase5))]
    #[case("completion", Some(PhaseGateTarget::Completion))]
    #[case("roadmap-completion", Some(PhaseGateTarget::Completion))]
    #[case("phase-99", None)]
    fn parses_phase_targets(#[case] raw: &str, #[case] expected: Option<PhaseGateTarget>) {
        let parsed = parse_phase_gate_target(raw);
        assert_eq!(parsed, expected);
    }

    #[test]
    fn phase_zero_passes_without_required_suites() {
        let report = evaluate_phase_gate(PhaseGateTarget::Phase0, &[], &BTreeSet::new());

        assert_eq!(report.status, PhaseGateStatus::Passed);
        assert!(report.required_suite_ids.is_empty());
        assert!(report.missing_suite_ids.is_empty());
        assert!(report.failing_suite_ids.is_empty());
    }

    #[test]
    fn phase_one_reports_missing_mandated_suites() {
        let available_tests = vec![
            "policy_schema_bdd::load_canonical_schema_policy".to_owned(),
            "authority_lifecycle_bdd::mint_host_trusted".to_owned(),
        ];
        let report =
            evaluate_phase_gate(PhaseGateTarget::Phase1, &available_tests, &BTreeSet::new());

        assert_eq!(report.status, PhaseGateStatus::MissingSuites);
        assert_eq!(
            report.missing_suite_ids,
            vec!["llm-sink-enforcement", "localization-contract"]
        );
        assert!(report.failing_suite_ids.is_empty());
    }

    #[test]
    fn phase_one_reports_failing_suites_when_catalog_is_complete() {
        let available_tests = vec![
            "policy_schema_bdd::load_canonical_schema_policy".to_owned(),
            "authority_lifecycle_bdd::mint_host_trusted".to_owned(),
            "llm_sink_enforcement::pre_dispatch".to_owned(),
            "localization_contract::explicit_localizer".to_owned(),
        ];
        let failing_suites = BTreeSet::from(["authority-lifecycle"]);
        let report =
            evaluate_phase_gate(PhaseGateTarget::Phase1, &available_tests, &failing_suites);

        assert_eq!(report.status, PhaseGateStatus::FailingSuites);
        assert!(report.missing_suite_ids.is_empty());
        assert_eq!(report.failing_suite_ids, vec!["authority-lifecycle"]);
    }

    #[test]
    fn phase_suite_mapping_matches_acceptance_targets() {
        assert_eq!(required_suites_for_target(PhaseGateTarget::Phase1).len(), 4);
        assert_eq!(required_suites_for_target(PhaseGateTarget::Phase2).len(), 3);
        assert_eq!(required_suites_for_target(PhaseGateTarget::Phase3).len(), 2);
        assert_eq!(required_suites_for_target(PhaseGateTarget::Phase4).len(), 1);
        assert_eq!(required_suites_for_target(PhaseGateTarget::Phase5).len(), 2);
        assert_eq!(
            required_suites_for_target(PhaseGateTarget::Completion).len(),
            2
        );
    }
}
