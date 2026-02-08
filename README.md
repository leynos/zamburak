# Zamburak

*Capability-governed runtime foundations for safer tool-using agents.*

Zamburak is a Rust-first project exploring how to reduce prompt-injection
impact in tool-using agent systems. The design moves enforcement out of prompt
text and into runtime policy checks, provenance tracking, and explicit trust
boundaries.

______________________________________________________________________

## Why zamburak?

- **Fail-closed policy mediation**: high-risk side effects are intended to be
  checked at runtime before tool or model calls proceed.
- **Tracked provenance**: value and control dependencies are part of the
  security model, helping unsafe flows stay visible and enforceable.
- **Deterministic verification paths**: trust upgrades are intended to come
  from explicit, auditable verification and endorsement steps.
- **Security-led delivery**: roadmap phases are tied to verification targets so
  safeguards are implemented with evidence, not assumptions.

______________________________________________________________________

## Quick start

### Installation

```bash
# Ensure the pinned Rust toolchain is installed
rustup toolchain install nightly-2026-01-30

# Build the library crate
make build
```

### Basic usage

```rust
use zamburak::greet;

fn main() {
    assert_eq!(greet(), "Hello from Zamburak!");
}
```

______________________________________________________________________

## Features

- Rust 2024 library scaffold with strict lint and documentation gates.
- Security-first architecture defined in an authoritative design document.
- Phased implementation roadmap mapped to explicit verification targets.
- Engineering standards focused on fail-closed behaviour and test evidence.

______________________________________________________________________

## Learn more

- [System design](docs/zamburak-design-document.md) - semantics, invariants,
  and trust boundaries.
- [Engineering standards](docs/zamburak-engineering-standards.md) -
  implementation and quality requirements.
- [Roadmap](docs/roadmap.md) - phased delivery plan and completion criteria.
- [Documentation index](docs/contents.md) - central catalogue of project docs.

______________________________________________________________________

## Licence

ISC - see [LICENSE](LICENSE) for details.

______________________________________________________________________

## Contributing

Contributions are welcome. Please follow the repository workflow and quality
gates in [AGENTS.md](AGENTS.md).
