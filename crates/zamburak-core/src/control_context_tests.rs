//! Unit tests for execution context summary and effect counters.

use crate::summary::DependencySummary;
use crate::trust::{AuthoritySet, DataLabel, DataLabels, IntegrityLabel};
use crate::value_id::ValueId;

use super::{EffectCounters, ExecutionContextSummary};

// ---------------------------------------------------------------------------
// EffectCounters
// ---------------------------------------------------------------------------

#[test]
fn effect_counters_start_at_zero() {
    let counters = EffectCounters::new();
    assert_eq!(counters.total_effects(), 0);
    assert_eq!(counters.tool_effects("file_write"), 0);
}

fn make_mixed_effect_counters() -> EffectCounters {
    let mut counters = EffectCounters::new();
    counters.record("file_write");
    counters.record("file_write");
    counters.record("http_get");
    counters
}

#[test]
fn effect_counters_total_reflects_all_recordings() {
    let counters = make_mixed_effect_counters();
    assert_eq!(counters.total_effects(), 3);
}

#[test]
fn effect_counters_per_tool_counts_match_recordings() {
    let counters = make_mixed_effect_counters();
    assert_eq!(counters.tool_effects("file_write"), 2);
    assert_eq!(counters.tool_effects("http_get"), 1);
    assert_eq!(counters.tool_effects("unknown_tool"), 0);
}

// ---------------------------------------------------------------------------
// ExecutionContextSummary::new
// ---------------------------------------------------------------------------

#[test]
fn new_context_has_verified_integrity_and_empty_confidentiality() {
    let ctx = ExecutionContextSummary::new();
    assert_eq!(ctx.pc_integrity(), IntegrityLabel::Verified);
    assert!(ctx.pc_confidentiality().is_empty());
}

#[test]
fn new_context_has_no_control_dependencies_or_effects() {
    let ctx = ExecutionContextSummary::new();
    assert!(ctx.control_dependencies().is_empty());
    assert_eq!(ctx.effect_counters().total_effects(), 0);
}

// ---------------------------------------------------------------------------
// push_condition
// ---------------------------------------------------------------------------

#[test]
fn push_condition_degrades_integrity() {
    let mut ctx = ExecutionContextSummary::new();

    let condition = DependencySummary {
        integrity_join: IntegrityLabel::Untrusted,
        confidentiality_join: DataLabels::new(),
        authority_join: AuthoritySet::new(),
        origin_count: 1,
        truncated: false,
    };

    ctx.push_condition(ValueId::new(1), &condition);
    assert_eq!(ctx.pc_integrity(), IntegrityLabel::Untrusted);
}

#[test]
fn push_condition_accumulates_confidentiality() {
    let mut ctx = ExecutionContextSummary::new();

    let cond_a = DependencySummary {
        integrity_join: IntegrityLabel::Trusted,
        confidentiality_join: DataLabels::from_iter([DataLabel::Pii]),
        authority_join: AuthoritySet::new(),
        origin_count: 1,
        truncated: false,
    };
    let cond_b = DependencySummary {
        integrity_join: IntegrityLabel::Trusted,
        confidentiality_join: DataLabels::from_iter([DataLabel::AuthSecret]),
        authority_join: AuthoritySet::new(),
        origin_count: 1,
        truncated: false,
    };

    ctx.push_condition(ValueId::new(1), &cond_a);
    ctx.push_condition(ValueId::new(2), &cond_b);

    assert!(ctx.pc_confidentiality().contains(&DataLabel::Pii));
    assert!(ctx.pc_confidentiality().contains(&DataLabel::AuthSecret));
}

#[test]
fn push_condition_tracks_control_dependencies() {
    let mut ctx = ExecutionContextSummary::new();

    let condition = DependencySummary {
        integrity_join: IntegrityLabel::Trusted,
        confidentiality_join: DataLabels::new(),
        authority_join: AuthoritySet::new(),
        origin_count: 1,
        truncated: false,
    };

    ctx.push_condition(ValueId::new(10), &condition);
    ctx.push_condition(ValueId::new(20), &condition);

    assert_eq!(ctx.control_dependencies().len(), 2);
    assert_eq!(ctx.control_dependencies()[0], ValueId::new(10));
    assert_eq!(ctx.control_dependencies()[1], ValueId::new(20));
}

// ---------------------------------------------------------------------------
// as_summary
// ---------------------------------------------------------------------------

fn make_untrusted_pii_condition() -> DependencySummary {
    DependencySummary {
        integrity_join: IntegrityLabel::Untrusted,
        confidentiality_join: DataLabels::from_iter([DataLabel::Pii]),
        authority_join: AuthoritySet::new(),
        origin_count: 1,
        truncated: false,
    }
}

#[test]
fn as_summary_reflects_integrity_and_confidentiality() {
    let mut ctx = ExecutionContextSummary::new();
    let condition = make_untrusted_pii_condition();
    ctx.push_condition(ValueId::new(1), &condition);

    let summary = ctx.as_summary();

    assert_eq!(summary.integrity_join, IntegrityLabel::Untrusted);
    assert!(summary.confidentiality_join.contains(&DataLabel::Pii));
}

#[test]
fn as_summary_has_full_authority_correct_origin_and_is_not_truncated() {
    let mut ctx = ExecutionContextSummary::new();
    let condition = make_untrusted_pii_condition();
    ctx.push_condition(ValueId::new(1), &condition);

    let summary = ctx.as_summary();

    // Control context uses full (universe) authority — the identity for
    // intersection — so it does not erase operand capabilities.
    assert!(summary.authority_join.is_full());
    // Origin count = number of control dependencies.
    assert_eq!(summary.origin_count, 1);
    assert!(!summary.truncated);
}

#[test]
fn as_summary_fresh_context_has_clean_labels() {
    let ctx = ExecutionContextSummary::new();
    let summary = ctx.as_summary();

    assert_eq!(summary.integrity_join, IntegrityLabel::Verified);
    assert!(summary.confidentiality_join.is_empty());
    assert!(summary.authority_join.is_full());
}

#[test]
fn as_summary_fresh_context_has_zero_origin_count_and_is_not_truncated() {
    let ctx = ExecutionContextSummary::new();
    let summary = ctx.as_summary();

    assert_eq!(summary.origin_count, 0);
    assert!(!summary.truncated);
}

// ---------------------------------------------------------------------------
// record_effect
// ---------------------------------------------------------------------------

#[test]
fn record_effect_updates_counters() {
    let mut ctx = ExecutionContextSummary::new();

    ctx.record_effect("file_write");
    ctx.record_effect("file_write");
    ctx.record_effect("http_get");

    assert_eq!(ctx.effect_counters().total_effects(), 3);
    assert_eq!(ctx.effect_counters().tool_effects("file_write"), 2);
    assert_eq!(ctx.effect_counters().tool_effects("http_get"), 1);
}
