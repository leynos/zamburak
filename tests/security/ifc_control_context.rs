//! Security regression for strict-mode control-context propagation.

use std::sync::{Arc, Mutex};

use monty::{MontyObject, MontyRun, PrintWriter};
use zamburak_core::propagation::PropagationMode;
use zamburak_core::{AuthoritySet, DataLabels, IntegrityLabel, ValueLabels};
use zamburak_monty::{
    CallContext, ExternalCallMediator, GovernedIfcConfig, GovernedRunProgress, GovernedRunner,
    IfcValueSeedConfig, MediationDecision,
};

struct StrictPcDenyMediator;

impl ExternalCallMediator for StrictPcDenyMediator {
    fn mediate(&mut self, context: &CallContext) -> MediationDecision {
        if context.function_name == "effect"
            && context.ifc.control_context.pc_integrity() == IntegrityLabel::Untrusted
        {
            return MediationDecision::Deny {
                reason: "strict-mode PC integrity is untrusted".to_owned(),
            };
        }
        MediationDecision::Allow
    }
}

fn strict_ifc_config() -> GovernedIfcConfig {
    GovernedIfcConfig {
        propagation_mode: PropagationMode::Strict,
        value_seeds: IfcValueSeedConfig {
            internal_values: ValueLabels {
                integrity: IntegrityLabel::Trusted,
                confidentiality: DataLabels::new(),
                authority: AuthoritySet::full(),
            },
            resumed_external_returns: ValueLabels {
                integrity: IntegrityLabel::Untrusted,
                confidentiality: DataLabels::new(),
                authority: AuthoritySet::full(),
            },
        },
        ..GovernedIfcConfig::default()
    }
}

#[test]
fn strict_mode_denies_constant_effect_under_untrusted_control_flow() {
    let monty_run = MontyRun::new(
        "condition = gate()\nif condition:\n    effect(\"constant\")\nelse:\n    effect(\"constant\")"
            .to_owned(),
        "test.py",
        vec![],
    )
    .expect("MontyRun should be created");
    let mediator: Arc<Mutex<dyn ExternalCallMediator>> = Arc::new(Mutex::new(StrictPcDenyMediator));
    let runner = GovernedRunner::new(monty_run, mediator).with_ifc_config(strict_ifc_config());

    let first_progress = runner
        .run_no_limits(vec![])
        .expect("first governed step should succeed");
    let first_suspended = match first_progress {
        GovernedRunProgress::ExternalCallPending { suspended, .. } => suspended,
        other => panic!("expected first ExternalCallPending, got {other:?}"),
    };

    let second_progress = first_suspended
        .resume(MontyObject::String("yes".to_owned()), PrintWriter::Stdout)
        .expect("resume should continue governed execution");
    match second_progress {
        GovernedRunProgress::Denied {
            reason,
            function_name,
            ..
        } => {
            assert_eq!(function_name, "effect");
            assert!(reason.contains("untrusted"));
        }
        other => panic!("expected Denied, got {other:?}"),
    }
}
