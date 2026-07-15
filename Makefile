.PHONY: help all clean test build release lint typecheck fmt check-fmt \
        markdownlint nixie spelling spelling-config spelling-config-write \
        spelling-phrase-check spelling-helper-test phase-gate script-baseline \
        script-typecheck script-test test-workflow-contracts monty-sync \
        lint-full-monty-local


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
UV ?= uv
UV_ENV = UV_CACHE_DIR=.uv-cache UV_TOOL_DIR=.uv-tools
RUFF_VERSION ?= 0.15.12
PATHSPEC_VERSION ?= 1.1.1
TYPOS_VERSION ?= 1.48.0
TYPOS_CONFIG_BUILDER_COMMIT := d6da92f02240a79a945c835f69bdd08a888da1d0
TYPOS_CONFIG_BUILDER_SOURCE := git+https://github.com/leynos/typos-config-builder.git@$(TYPOS_CONFIG_BUILDER_COMMIT)
TYPOS_CONFIG_BUILDER := $(UV_ENV) $(UV) tool run --python 3.14 \
	--from "$(TYPOS_CONFIG_BUILDER_SOURCE)" typos-config-builder
SPELLING_PY_SRCS := \
	scripts/typos_rollout_check.py scripts/tests/test_typos_rollout_check.py
SPELLING_PY_TESTS := scripts/tests/test_typos_rollout_check.py
SPELLING_COVERAGE_ARGS := --cov=typos_rollout_check --cov-fail-under=90
SPELLING_HELPER_PYTEST = PYTHONPATH=scripts $(UV_ENV) $(UV) run --no-project \
	--python 3.14 --with cuprum==0.1.0 --with pathspec==$(PATHSPEC_VERSION) \
	--with pytest==9.0.2 \
	--with pytest-cov==7.0.0 python -m pytest
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

markdownlint: spelling ## Lint Markdown files and enforce spelling
	$(MDLINT) "**/*.md" "#.uv-cache" "#.uv-tools"

spelling: spelling-phrase-check ## Enforce en-GB-oxendict policy in tracked text
	@git ls-files -z '*.md' | xargs -0 -r env $(UV_ENV) \
		$(UV) tool run typos@$(TYPOS_VERSION) --config typos.toml --force-exclude

spelling-phrase-check: spelling-config ## Reject prohibited spelling phrases
	@PYTHONPATH=scripts $(UV_ENV) $(UV) run --no-project --python 3.14 scripts/typos_rollout_check.py --repository .

spelling-config: spelling-helper-test ## Verify generated spelling configuration
	@git ls-files --error-unmatch typos.toml >/dev/null
	@$(TYPOS_CONFIG_BUILDER) --repository . --check

spelling-config-write: spelling-helper-test ## Generate spelling configuration
	@$(TYPOS_CONFIG_BUILDER) --repository .

spelling-helper-test: ## Validate the shared spelling-policy integration
	@$(UV_ENV) $(UV) tool run ruff@$(RUFF_VERSION) format --isolated --target-version py313 --check $(SPELLING_PY_SRCS)
	@$(UV_ENV) $(UV) tool run ruff@$(RUFF_VERSION) check --isolated --target-version py313 $(SPELLING_PY_SRCS)
	@$(SPELLING_HELPER_PYTEST) $(SPELLING_PY_TESTS) -c /dev/null --rootdir=. -p no:cacheprovider $(SPELLING_COVERAGE_ARGS)

nixie: ## Validate Mermaid diagrams
	$(NIXIE) --no-sandbox

script-baseline: ## Validate roadmap script baseline contracts
	uv run $(SCRIPT_UV_DEPS) scripts/verify_script_baseline.py

script-typecheck: ## Run script type checks with ty
	uv run --with ty ty check $(SCRIPT_TYPECHECK_FLAGS) scripts

script-test: ## Run script baseline test suite
	uv run $(SCRIPT_UV_DEPS) pytest scripts/tests

test-workflow-contracts: ## Validate the mutation-testing caller contract
	uv run --with 'pytest>=8' --with 'pyyaml>=6' pytest tests/workflow_contracts -q

monty-sync: ## Sync full-monty fork branch with upstream and run verification gates
	uv run scripts/monty_sync.py

lint-full-monty-local: ## Run full-monty Rust lint with nested-checkout safe defaults
	$(MAKE) -C third_party/full-monty lint-rs-local

help: ## Show available targets
	@grep -E '^[a-zA-Z_-]+:.*?##' $(MAKEFILE_LIST) | \
	awk 'BEGIN {FS=":"; printf "Available targets:\n"} {printf "  %-20s %s\n", $$1, $$2}'
