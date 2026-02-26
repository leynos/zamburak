.PHONY: help all clean test build release lint typecheck fmt check-fmt markdownlint nixie phase-gate script-baseline script-typecheck script-test monty-sync


TARGET ?= libzamburak.rlib

CARGO ?= cargo
BUILD_JOBS ?=
RUST_FLAGS ?= -D warnings
RUSTDOC_FLAGS ?= -D warnings
CARGO_FLAGS ?= --all-targets --all-features
CLIPPY_FLAGS ?= $(CARGO_FLAGS) -- $(RUST_FLAGS)
TEST_FLAGS ?= $(CARGO_FLAGS)
MDLINT ?= markdownlint-cli2
NIXIE ?= nixie
PHASE_GATE_TARGET_FILE ?= .github/phase-gate-target.txt
SCRIPT_UV_DEPS ?= --with pytest --with pytest-bdd --with pytest-mock --with cmd-mox --with astroid --with cuprum
SCRIPT_TYPECHECK_FLAGS ?= --ignore unresolved-import

build: target/debug/$(TARGET) ## Build debug binary
release: target/release/$(TARGET) ## Build release binary

all: check-fmt lint test ## Perform a comprehensive check of code

clean: ## Remove build artifacts
	$(CARGO) clean

test: ## Run tests with warnings treated as errors
	RUSTFLAGS="$(RUST_FLAGS)" $(CARGO) test --workspace $(TEST_FLAGS) $(BUILD_JOBS)

phase-gate: ## Evaluate phase-gate verification suites for configured target
	RUSTFLAGS="$(RUST_FLAGS)" $(CARGO) run --bin phase_gate -- --target-file $(PHASE_GATE_TARGET_FILE)

target/%/$(TARGET): ## Build binary in debug or release mode
	$(CARGO) build $(BUILD_JOBS) $(if $(findstring release,$(@)),--release)

lint: ## Run Clippy with warnings denied
	RUSTDOCFLAGS="$(RUSTDOC_FLAGS)" $(CARGO) doc --workspace --no-deps
	$(CARGO) clippy --workspace $(CLIPPY_FLAGS)

typecheck: script-typecheck ## Run compile-time type checks
	$(CARGO) check --workspace $(CARGO_FLAGS) $(BUILD_JOBS)

fmt: ## Format Rust and Markdown sources
	$(CARGO) fmt --all
	mdformat-all

check-fmt: ## Verify formatting
	$(CARGO) fmt --all -- --check

markdownlint: ## Lint Markdown files
	$(MDLINT) '**/*.md'

nixie: ## Validate Mermaid diagrams
	$(NIXIE) --no-sandbox

script-baseline: ## Validate roadmap script baseline contracts
	uv run $(SCRIPT_UV_DEPS) scripts/verify_script_baseline.py

script-typecheck: ## Run script type checks with ty
	uv run --with ty ty check $(SCRIPT_TYPECHECK_FLAGS) scripts

script-test: ## Run script baseline test suite
	uv run $(SCRIPT_UV_DEPS) pytest scripts/tests

monty-sync: ## Sync full-monty fork branch with upstream and run verification gates
	uv run scripts/monty_sync.py

help: ## Show available targets
	@grep -E '^[a-zA-Z_-]+:.*?##' $(MAKEFILE_LIST) | \
	awk 'BEGIN {FS=":"; printf "Available targets:\n"} {printf "  %-20s %s\n", $$1, $$2}'
