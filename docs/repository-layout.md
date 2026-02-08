# Zamburak repository layout

This document defines the proposed repository layout and file-purpose
documentation for Zamburak.

Design semantics, trust boundaries, and policy invariants are defined in
`docs/zamburak-design-document.md`.

Implementation sequencing is defined in `docs/roadmap.md`.

Engineering process and quality gates are defined in
`docs/zamburak-engineering-standards.md` and `AGENTS.md`.

Technology and tooling baseline constraints are defined in
`docs/tech-baseline.md`.

Verification target matrices are defined in `docs/verification-targets.md`.

## Layout goals

The repository layout is designed to:

- separate security-critical runtime concerns by crate boundary,
- keep policy, verifier, and tool contracts explicit,
- support deterministic testing and adversarial evaluation,
- keep contributor onboarding practical with predictable file locations.

## Top-level structure

```plaintext
/
├── crates/
│   ├── zamburak-core/
│   ├── zamburak-interpreter/
│   ├── zamburak-policy/
│   ├── zamburak-sanitizers/
│   ├── zamburak-tools/
│   ├── zamburak-agent/
│   └── zamburak-cli/
├── policies/
├── tests/
│   ├── integration/
│   ├── security/
│   ├── property/
│   ├── compatibility/
│   └── benchmarks/
├── fuzz/
├── docs/
├── scripts/
├── .github/workflows/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── .env.example
├── LICENSE
└── README.md
```

## Crate boundaries and responsibilities

| Crate                  | Primary responsibility                                  | Security-critical invariants                             |
| ---------------------- | ------------------------------------------------------- | -------------------------------------------------------- |
| `zamburak-core`        | Value tagging, dependency graph, propagation, summaries | Complete IFC propagation and monotonic dependencies      |
| `zamburak-interpreter` | Monty integration and opcode hook execution             | No effect path bypasses policy interception              |
| `zamburak-policy`      | Policy definitions, evaluation, and decisions           | Unknown states fail closed                               |
| `zamburak-sanitizers`  | Deterministic verifiers and sanitizers                  | Verified labels are non-forgeable                        |
| `zamburak-tools`       | Tool adapters and MCP bridge boundaries                 | Every effectful call has signature and policy gate       |
| `zamburak-agent`       | Planner loop, repair loop, and confirmations            | Redacted feedback and controlled commit flows            |
| `zamburak-cli`         | Operational commands and validation tooling             | Administrative actions respect policy and audit controls |

_Table 1: Crate boundaries and security-critical responsibilities._

## Crate file-purpose reference

The paths below are the intended locations for core implementation units.

### `crates/zamburak-core`

| Path                                           | Purpose                                              |
| ---------------------------------------------- | ---------------------------------------------------- |
| `crates/zamburak-core/src/lib.rs`              | Public exports for core runtime types and interfaces |
| `crates/zamburak-core/src/tagged_value.rs`     | `TaggedValue` model with value identity and labels   |
| `crates/zamburak-core/src/value_id.rs`         | Stable `ValueId` generation and handling             |
| `crates/zamburak-core/src/dependency_graph.rs` | Graph state and edge insertion semantics             |
| `crates/zamburak-core/src/propagation.rs`      | Normal and strict propagation rules                  |
| `crates/zamburak-core/src/control_context.rs`  | Control-context stack and strict-mode helpers        |
| `crates/zamburak-core/src/summary.rs`          | Bounded transitive summarization and caching         |
| `crates/zamburak-core/src/trust.rs`            | Integrity label and verification kinds               |
| `crates/zamburak-core/src/capability.rs`       | Authority token and label-set helpers                |
| `crates/zamburak-core/src/errors.rs`           | Core domain and runtime error types                  |

_Table 2: Core crate file-purpose mapping._

### `crates/zamburak-interpreter`

| Path                                                 | Purpose                                          |
| ---------------------------------------------------- | ------------------------------------------------ |
| `crates/zamburak-interpreter/src/lib.rs`             | Interpreter crate entry point                    |
| `crates/zamburak-interpreter/src/vm.rs`              | Monty VM wrapper and execution coordinator       |
| `crates/zamburak-interpreter/src/opcodes.rs`         | Opcode-level IFC propagation integration         |
| `crates/zamburak-interpreter/src/external_call.rs`   | External call interception and policy gateway    |
| `crates/zamburak-interpreter/src/snapshot.rs`        | Snapshot and resume semantics for IFC state      |
| `crates/zamburak-interpreter/src/redaction.rs`       | Error and prompt-feedback redaction functions    |
| `crates/zamburak-interpreter/src/resource_limits.rs` | Runtime budgets for values, graph, and summaries |

_Table 3: Interpreter crate file-purpose mapping._

### `crates/zamburak-policy`

| Path                                           | Purpose                                            |
| ---------------------------------------------- | -------------------------------------------------- |
| `crates/zamburak-policy/src/lib.rs`            | Policy crate entry point                           |
| `crates/zamburak-policy/src/engine.rs`         | `PolicyEngine` implementation and evaluation order |
| `crates/zamburak-policy/src/policy_def.rs`     | YAML and JSON policy schema models                 |
| `crates/zamburak-policy/src/tool_signature.rs` | Tool-side effect signatures and constraints        |
| `crates/zamburak-policy/src/decision.rs`       | Decision enums and reason payloads                 |
| `crates/zamburak-policy/src/audit.rs`          | Decision-to-audit record transformations           |
| `crates/zamburak-policy/src/defaults.rs`       | Baseline policy presets and safe defaults          |

_Table 4: Policy crate file-purpose mapping._

### `crates/zamburak-sanitizers`

| Path                                         | Purpose                                         |
| -------------------------------------------- | ----------------------------------------------- |
| `crates/zamburak-sanitizers/src/lib.rs`      | Sanitizer crate entry point                     |
| `crates/zamburak-sanitizers/src/traits.rs`   | `Verifier` and `Sanitizer` trait contracts      |
| `crates/zamburak-sanitizers/src/email.rs`    | Recipient validation and allowlist verification |
| `crates/zamburak-sanitizers/src/url.rs`      | URL parsing and allowlist verification          |
| `crates/zamburak-sanitizers/src/numeric.rs`  | Numeric bounds and amount verification          |
| `crates/zamburak-sanitizers/src/template.rs` | Safe templating and escaping transforms         |

_Table 5: Sanitizer crate file-purpose mapping._

### `crates/zamburak-tools`

| Path                                          | Purpose                                            |
| --------------------------------------------- | -------------------------------------------------- |
| `crates/zamburak-tools/src/lib.rs`            | Tool crate entry point                             |
| `crates/zamburak-tools/src/traits.rs`         | `ExternalFunction` trait and shared request models |
| `crates/zamburak-tools/src/email_read.rs`     | Untrusted email ingestion adapter                  |
| `crates/zamburak-tools/src/email_send.rs`     | Egress email adapter with draft and commit support |
| `crates/zamburak-tools/src/calendar_read.rs`  | Calendar read adapter                              |
| `crates/zamburak-tools/src/calendar_write.rs` | Calendar write adapter with confirmation flow      |
| `crates/zamburak-tools/src/web_fetch.rs`      | Web-fetch untrusted source adapter                 |
| `crates/zamburak-tools/src/mcp_bridge.rs`     | MCP server transport and trust-boundary wrapper    |

_Table 6: Tool crate file-purpose mapping._

### `crates/zamburak-agent`

| Path                                        | Purpose                                            |
| ------------------------------------------- | -------------------------------------------------- |
| `crates/zamburak-agent/src/lib.rs`          | Agent integration entry point                      |
| `crates/zamburak-agent/src/planner.rs`      | Planner prompts and trusted query shaping          |
| `crates/zamburak-agent/src/repair_loop.rs`  | Repair loop with redacted error feedback           |
| `crates/zamburak-agent/src/confirmation.rs` | Confirmation workflow and typed claim display      |
| `crates/zamburak-agent/src/state.rs`        | Execution state, snapshots, and continuation logic |

_Table 7: Agent crate file-purpose mapping._

### `crates/zamburak-cli`

| Path                                              | Purpose                                  |
| ------------------------------------------------- | ---------------------------------------- |
| `crates/zamburak-cli/src/main.rs`                 | CLI entry point and command dispatch     |
| `crates/zamburak-cli/src/commands/mod.rs`         | Command module root                      |
| `crates/zamburak-cli/src/commands/run.rs`         | Plan execution command                   |
| `crates/zamburak-cli/src/commands/validate.rs`    | Policy validation command                |
| `crates/zamburak-cli/src/commands/audit.rs`       | Audit exploration and query command      |
| `crates/zamburak-cli/src/commands/test_policy.rs` | Policy scenario and regression execution |
| `crates/zamburak-cli/src/config.rs`               | CLI configuration loading                |

_Table 8: CLI crate file-purpose mapping._

## Shared directories and file purposes

### `policies/`

| Path                                         | Purpose                                         |
| -------------------------------------------- | ----------------------------------------------- |
| `policies/default.yaml`                      | Baseline policy profile for assistant workflows |
| `policies/strict.yaml`                       | High-restriction profile with tighter defaults  |
| `policies/examples/email_triage.yaml`        | Example policy for inbox workflows              |
| `policies/examples/calendar_scheduling.yaml` | Example policy for scheduling workflows         |
| `policies/schema.json`                       | Policy schema for structural validation         |

_Table 9: Policy directory artefacts and purposes._

### `tests/`

| Path                   | Purpose                                                     |
| ---------------------- | ----------------------------------------------------------- |
| `tests/integration/`   | End-to-end crate integration and call-path tests            |
| `tests/security/`      | Prompt-injection, exfiltration, and bypass regression tests |
| `tests/property/`      | Property tests for monotonicity and closure invariants      |
| `tests/compatibility/` | Behavioural comparisons against upstream Monty              |
| `tests/benchmarks/`    | Performance and overhead measurement tests                  |

_Table 10: Test suite directories and purposes._

### `fuzz/`

| Path                                 | Purpose                             |
| ------------------------------------ | ----------------------------------- |
| `fuzz/fuzz_targets/fuzz_parser.rs`   | Parser fuzz target                  |
| `fuzz/fuzz_targets/fuzz_bytecode.rs` | VM and opcode execution fuzz target |
| `fuzz/fuzz_targets/fuzz_policy.rs`   | Policy evaluation fuzz target       |
| `fuzz/Cargo.toml`                    | Fuzz target workspace configuration |

_Table 11: Fuzzing artefacts and purposes._

### `docs/`

| Path                                     | Purpose                                                  |
| ---------------------------------------- | -------------------------------------------------------- |
| `docs/zamburak-design-document.md`       | Authoritative system semantics and interfaces            |
| `docs/roadmap.md`                        | High-level implementation phases, steps, and tasks       |
| `docs/zamburak-engineering-standards.md` | Project-specific engineering standards                   |
| `docs/repository-layout.md`              | Proposed repository structure and file-purpose reference |
| `docs/tech-baseline.md`                  | Toolchain and quality-gate baseline with rationale       |
| `docs/verification-targets.md`           | Verification target matrix and evidence requirements     |

_Table 12: Core documentation artefacts and ownership._

### Root and operational files

| Path                  | Purpose                                           |
| --------------------- | ------------------------------------------------- |
| `Cargo.toml`          | Workspace member registration and shared settings |
| `Cargo.lock`          | Reproducible dependency resolution                |
| `rust-toolchain.toml` | Toolchain pinning and compatibility               |
| `.env.example`        | Configuration template for local integrations     |
| `README.md`           | Project entry document and orientation            |
| `LICENSE`             | Project licence                                   |
| `scripts/`            | Operational helper scripts for local workflows    |
| `.github/workflows/`  | CI and automation workflow definitions            |

_Table 13: Root and operational artefacts with purposes._

## Layout governance rules

- Place new code in the crate owning the domain concern, not by technical
  layer alone.
- Add new side effect adapters only under `zamburak-tools` with corresponding
  policy signatures.
- Add or change any policy schema in `policies/schema.json` and update examples
  in the same change.
- Add tests in the narrowest suite that proves the behaviour; mirror any
  security fix with a regression test under `tests/security`.
- Keep `docs/zamburak-design-document.md` focused on semantics and interfaces.
  Use this document for file locations and ownership mapping.

## Relationship to implementation planning

The roadmap references design sections for semantics, and this layout document
provides the canonical path mapping used by those tasks.

Where roadmap tasks introduce new modules, update this document in the same
change set to keep repository ownership and discoverability current.
