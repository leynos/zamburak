.PHONY: help all clean test build release lint typecheck fmt check-fmt \
        markdownlint nixie spelling phase-gate script-baseline \
        script-typecheck script-test test-workflow-contracts monty-sync \
        lint-full-monty-local


TARGET ?= libzamburak.rlib

CARGO ?= cargo
WHITAKER ?= whitaker
BUILD_JOBS ?=
RUST_FLAGS ?= -D warnings
RUSTDOC_FLAGS ?= -D warnings
CARGO_FLAGS ?= --all-targets --all-features
CLIPPY_FLAGS ?= $(CARGO_FLAGS) -- $(RUST_FLAGS)
TEST_FLAGS ?= $(CARGO_FLAGS)
MDLINT ?= markdownlint-cli2
# `make fmt` and `make check-fmt` call mdtablefix directly. `--git` selects the
# Markdown files Git tracks and `--include-untracked` adds the untracked files
# Git does not ignore, so a new document is formatted before it is staged.
# Both modes need mdtablefix 0.6.0 or later; CI pins the version at the
# install-mdtablefix step.
MDTABLEFIX ?= mdtablefix
MDTABLEFIX_SELECT = --git --include-untracked
MDTABLEFIX_RULES = --wrap --renumber --breaks --ellipsis --fences
NIXIE ?= nixie
UV ?= uv
UV_ENV = UV_CACHE_DIR=.uv-cache UV_TOOL_DIR=.uv-tools
PATHSPEC_VERSION ?= 1.1.1
TYPOS_CONFIG_BUILDER_VERSION ?= v0.1.1
TYPOS_CONFIG_BUILDER = $(UV_ENV) $(UV) tool run --python 3.14 --from \
	"git+https://github.com/leynos/typos-config-builder.git@$(TYPOS_CONFIG_BUILDER_VERSION)" \
	typos-config-builder
PHASE_GATE_TARGET_FILE ?= .github/phase-gate-target.txt
SCRIPT_UV_DEPS ?= --with pytest --with pytest-bdd --with pytest-mock \
	--with cmd-mox --with astroid --with cuprum==0.1.0 \
	--with pathspec==$(PATHSPEC_VERSION)
SCRIPT_TYPECHECK_FLAGS ?= --ignore unresolved-import

build: target/debug/$(TARGET) ## Build debug binary
release: target/release/$(TARGET) ## Build release binary

all: check-fmt lint test spelling ## Perform a comprehensive check of code

clean: ## Remove build artefacts
	$(CARGO) clean
	rm -rf .uv-cache .uv-tools

test: ## Run tests with warnings treated as errors
	RUSTFLAGS="$(RUST_FLAGS)" $(CARGO) test --workspace $(TEST_FLAGS) $(BUILD_JOBS)

phase-gate: ## Evaluate phase-gate verification suites for configured target
	RUSTFLAGS="$(RUST_FLAGS)" $(CARGO) run --bin phase_gate -- --target-file $(PHASE_GATE_TARGET_FILE)

target/%/$(TARGET): ## Build binary in debug or release mode
	$(CARGO) build $(BUILD_JOBS) $(if $(findstring release,$(@)),--release)

lint: ## Run Clippy and the Whitaker Dylint suite with warnings denied
	RUSTDOCFLAGS="$(RUSTDOC_FLAGS)" $(CARGO) doc --workspace --no-deps
	$(CARGO) clippy --workspace $(CLIPPY_FLAGS)
	RUSTFLAGS="$(RUST_FLAGS)" $(WHITAKER) --all -- $(CARGO_FLAGS)

typecheck: script-typecheck ## Run compile-time type checks
	$(CARGO) check --workspace $(CARGO_FLAGS) $(BUILD_JOBS)

fmt: ## Format Rust and Markdown sources
	$(CARGO) fmt --all
	$(MDTABLEFIX) --in-place $(MDTABLEFIX_SELECT) $(MDTABLEFIX_RULES)
	$(MDLINT) --fix "**/*.md"

check-fmt: ## Verify formatting
	$(CARGO) fmt --all -- --check
	$(MDTABLEFIX) --check $(MDTABLEFIX_SELECT) $(MDTABLEFIX_RULES)

markdownlint: spelling ## Lint Markdown files and enforce spelling
	$(MDLINT) "**/*.md" "#.uv-cache" "#.uv-tools"

spelling: ## Enforce en-GB-oxendict policy in tracked text
	$(TYPOS_CONFIG_BUILDER) gate --repository .

nixie: ## Validate Mermaid diagrams
	$(NIXIE) --no-sandbox

script-baseline: ## Validate roadmap script baseline contracts
	uv run $(SCRIPT_UV_DEPS) scripts/verify_script_baseline.py

script-typecheck: ## Run script type checks with ty
	uv run --with ty ty check $(SCRIPT_TYPECHECK_FLAGS) scripts

script-test: ## Run script baseline test suite
	uv run $(SCRIPT_UV_DEPS) pytest scripts/tests

test-workflow-contracts: ## Validate the workflow contracts, including the CV-005 CodeScene shape
	uv run --with 'pytest>=8' --with 'pyyaml>=6' pytest tests/workflow_contracts -q

monty-sync: ## Sync full-monty fork branch with upstream and run verification gates
	uv run scripts/monty_sync.py

lint-full-monty-local: ## Run full-monty Rust lint with nested-checkout safe defaults
	$(MAKE) -C third_party/full-monty lint-rs-local

help: ## Show available targets
	@grep -E '^[a-zA-Z_-]+:.*?##' $(MAKEFILE_LIST) | \
	awk 'BEGIN {FS=":"; printf "Available targets:\n"} {printf "  %-20s %s\n", $$1, $$2}'
