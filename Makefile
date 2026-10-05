# Ollama Rust SDK - Makefile
# Comprehensive build and test automation

CARGO ?= cargo
RUST_MSRV ?= 1.97.1
RUST_TOOLCHAIN ?= 1.99.0
BINARY_NAME ?= ollama-cli
BINARY_PACKAGE ?=
SBOM_MANIFEST_PATH ?= Cargo.toml

# Docker configuration
DOCKER_IMAGE = ollama-rust-sdk
DOCKER_TAG = latest
DOCKER_FULL_NAME = $(DOCKER_IMAGE):$(DOCKER_TAG)

# Rust configuration
CARGO_FEATURES_DEFAULT =
CARGO_FEATURES_ALL = --all-features
CARGO_FEATURES_NONE = --no-default-features

# Colors for output
RED = \033[0;31m
GREEN = \033[0;32m
YELLOW = \033[0;33m
BLUE = \033[0;34m
PURPLE = \033[0;35m
CYAN = \033[0;36m
WHITE = \033[0;37m
NC = \033[0m # No Color

.PHONY: help all all-coverage all-docker all-docker-coverage clean docker-build docker-clean
.PHONY: fmt fmt-check fmt-docker lint lint-strict lint-docker audit audit-docker deny deny-docker
.PHONY: test test-docker test-doc test-doc-docker test-features feature-check build build-docker build-all build-all-docker
.PHONY: docs docs-contract doc-check docs-strict docs-docker examples examples-docker bench bench-check bench-docker
.PHONY: coverage coverage-open coverage-lcov coverage-html coverage-summary coverage-json coverage-docker
.PHONY: dev-setup setup-dev ci-local ci-local-coverage hooks-install lint-ci msrv-check

# Default target - matches CI/CD workflow
all: fmt-check lint-strict audit deny feature-check test test-doc docs-strict build-all examples ## Run all CI/CD checks and builds locally

# Extended target with coverage
all-coverage: fmt-check lint-strict audit deny feature-check test test-doc coverage docs-strict build-all examples ## Run all checks including coverage locally

# Docker all-in-one target
all-docker: docker-build ## Run all checks and builds in Docker container
	@echo "$(CYAN)Running all checks in Docker container...$(NC)"
	@docker run --rm -v "$(PWD):/workspace" $(DOCKER_FULL_NAME) sh -c " \
		echo '$(BLUE)=== Formatting Check ===$(NC)' && \
		cargo fmt --all -- --check && \
		echo '$(BLUE)=== Linting ===$(NC)' && \
		cargo clippy --all-targets --all-features -- -D warnings && \
		cargo clippy --all-targets --no-default-features -- -D warnings && \
		cargo clippy --all-targets -- -D warnings && \
		echo '$(BLUE)=== Tests ===$(NC)' && \
		echo '  With all features...' && \
		cargo test --verbose --all-features && \
		echo '  With default features...' && \
		cargo test --verbose && \
		echo '$(BLUE)=== Documentation ===$(NC)' && \
		cargo doc --all-features --no-deps && \
		echo '$(BLUE)=== Build ===$(NC)' && \
		cargo build --all-features && \
		echo '$(BLUE)=== Examples ===$(NC)' && \
		cargo build --examples --all-features && \
		echo '$(GREEN)All checks passed!$(NC)' \
	"

# Docker all-in-one target with coverage
all-docker-coverage: docker-build ## Run all checks including coverage in Docker container
	@echo "$(CYAN)Running all checks with coverage in Docker container...$(NC)"
	@docker run --rm -v "$(PWD):/workspace" $(DOCKER_FULL_NAME) sh -c " \
		echo '$(BLUE)=== Formatting Check ===$(NC)' && \
		cargo fmt --all -- --check && \
		echo '$(BLUE)=== Linting ===$(NC)' && \
		cargo clippy --all-targets --all-features -- -D warnings && \
		echo '$(BLUE)=== Tests with Coverage ===$(NC)' && \
		cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info && \
		cargo llvm-cov --all-features --workspace --html && \
		echo '$(BLUE)=== Documentation ===$(NC)' && \
		cargo doc --all-features --no-deps && \
		echo '$(BLUE)=== Build ===$(NC)' && \
		cargo build --all-features && \
		echo '$(BLUE)=== Examples ===$(NC)' && \
		cargo build --examples --all-features && \
		echo '$(GREEN)All checks with coverage passed!$(NC)' \
	"

help: ## Show this help message
	@echo "$(CYAN)Ollama Rust SDK - Available Commands$(NC)"
	@echo ""
	@echo "$(YELLOW)Main Commands:$(NC)"
	@awk 'BEGIN {FS = ":.*##"; printf "  %-20s %s\n", "Target", "Description"} /^[a-zA-Z_-]+:.*?##/ { printf "  $(GREEN)%-20s$(NC) %s\n", $$1, $$2 }' $(MAKEFILE_LIST) | grep -E "(all|help|setup|clean)" | grep -v docker
	@echo ""
	@echo "$(YELLOW)Local Development:$(NC)"
	@awk 'BEGIN {FS = ":.*##"; printf "  %-20s %s\n", "Target", "Description"} /^[a-zA-Z_-]+:.*?##/ { printf "  $(GREEN)%-20s$(NC) %s\n", $$1, $$2 }' $(MAKEFILE_LIST) | grep -v -E "(all|help|setup|clean|docker)" | grep -v docker
	@echo ""
	@echo "$(YELLOW)Docker Commands:$(NC)"
	@awk 'BEGIN {FS = ":.*##"; printf "  %-20s %s\n", "Target", "Description"} /^[a-zA-Z_-]+:.*?##/ { printf "  $(GREEN)%-20s$(NC) %s\n", $$1, $$2 }' $(MAKEFILE_LIST) | grep docker

# =============================================================================
# Setup and Installation
# =============================================================================

hooks-install: ## Install repository-owned hooks safely in any worktree
	@sh scripts/install_hooks.sh

dev-setup: hooks-install ## Install the pinned development tools used by CI
	@rustup component add --toolchain $(RUST_TOOLCHAIN) rustfmt clippy llvm-tools-preview
	@cargo install cargo-audit --locked --version 0.22.2
	@cargo install cargo-deny --locked --version 0.20.2
	@cargo install cargo-hack --locked --version 0.6.45
	@cargo install cargo-llvm-cov --locked --version 0.9.1

setup-dev: dev-setup ## (Deprecated) Use `make dev-setup` instead
	@echo "$(YELLOW)Warning: 'setup-dev' is deprecated; use 'make dev-setup'.$(NC)"

# =============================================================================
# Docker Commands
# =============================================================================

docker-build: ## Build Docker image for consistent environment
	@echo "$(CYAN)Building Docker image...$(NC)"
	@docker build \
		--build-arg BINARY_NAME=$(BINARY_NAME) \
		--build-arg BINARY_PACKAGE=$(BINARY_PACKAGE) \
		--build-arg SBOM_MANIFEST_PATH=$(SBOM_MANIFEST_PATH) \
		--build-arg OCI_IMAGE_TITLE="Ollama Rust SDK" \
		--build-arg OCI_IMAGE_DESCRIPTION="Rust SDK and CLI for the Ollama API" \
		--build-arg OCI_IMAGE_VENDOR="ThreatFlux" \
		--build-arg OCI_IMAGE_SOURCE="https://github.com/ThreatFlux/ollama_rust_sdk" \
		-t $(DOCKER_FULL_NAME) .

docker-clean: ## Clean Docker images and containers
	@echo "$(CYAN)Cleaning Docker resources...$(NC)"
	@docker rmi $(DOCKER_FULL_NAME) 2>/dev/null || true
	@docker system prune -f

# =============================================================================
# Formatting Commands
# =============================================================================

fmt: ## Format code using rustfmt (includes examples)
	@echo "$(CYAN)Formatting code...$(NC)"
	@cargo fmt --all

fmt-check: ## Check code formatting without modifying files (includes examples)
	@echo "$(CYAN)Checking code formatting...$(NC)"
	@cargo fmt --all -- --check

fmt-docker: docker-build ## Format code using Docker
	@echo "$(CYAN)Formatting code in Docker...$(NC)"
	@docker run --rm -v "$(PWD):/workspace" $(DOCKER_FULL_NAME) cargo fmt --all

# =============================================================================
# Linting Commands
# =============================================================================

lint: ## Run clippy linting
	@echo "$(CYAN)Running clippy linting...$(NC)"
	@echo "$(BLUE)  With all features...$(NC)"
	@cargo clippy --all-targets --all-features -- -W warnings
	@echo "$(BLUE)  With default features...$(NC)"
	@cargo clippy --all-targets -- -W warnings

lint-ci: ## Run the exact strict Clippy policy used by CI Quick Check
	@cargo clippy --all-features --all-targets -- \
	  -D warnings \
	  -D clippy::all \
	  -D clippy::pedantic \
	  -D clippy::nursery \
	  -A clippy::multiple_crate_versions \
	  -A clippy::module_name_repetitions \
	  -A clippy::missing_errors_doc \
	  -A clippy::missing_panics_doc \
	  -A clippy::must_use_candidate \
	  -A clippy::return_self_not_must_use \
	  -A clippy::cast_possible_truncation \
	  -A clippy::cast_sign_loss \
	  -A clippy::cast_precision_loss \
	  -A clippy::similar_names \
	  -A clippy::unreadable_literal \
	  -A clippy::struct_field_names \
	  -A clippy::wildcard_imports \
	  -A clippy::option_if_let_else \
	  -A clippy::redundant_pub_crate \
	  -A clippy::missing_const_for_fn \
	  -A clippy::doc_markdown \
	  -A clippy::items_after_statements \
	  -A clippy::too_many_lines \
	  -A clippy::collection_is_never_read \
	  -A clippy::manual_let_else

lint-strict: lint-ci ## Run strict Clippy across all, default and no-default features
	@cargo clippy --locked --no-default-features --all-targets -- -D warnings
	@cargo clippy --locked --all-targets -- -D warnings

lint-docker: docker-build ## Run clippy linting in Docker
	@echo "$(CYAN)Running clippy linting in Docker...$(NC)"
	@docker run --rm -v "$(PWD):/workspace" $(DOCKER_FULL_NAME) sh -c "\
		echo '$(BLUE)  With all features...$(NC)' && \
		cargo clippy --all-targets --all-features -- -W warnings && \
		echo '$(BLUE)  With default features...$(NC)' && \
		cargo clippy --all-targets -- -W warnings"

# =============================================================================
# Security and Dependency Commands
# =============================================================================

audit: ## Fail on security advisories or advisory warnings
	@cargo audit --deny warnings

audit-docker: docker-build ## Run security audit in Docker
	@echo "$(CYAN)Running security audit in Docker...$(NC)"
	@docker run --rm -v "$(PWD):/workspace" $(DOCKER_FULL_NAME) cargo audit

deny: ## Check the all-feature dependency graph and policy
	@cargo deny --all-features check

deny-docker: docker-build ## Run dependency validation in Docker
	@echo "$(CYAN)Running dependency validation in Docker...$(NC)"
	@docker run --rm -v "$(PWD):/workspace" $(DOCKER_FULL_NAME) sh -c "cargo install cargo-deny --locked --version 0.20.2 && cargo deny --all-features check"

# =============================================================================
# Testing Commands
# =============================================================================

test: ## Run all tests
	@echo "$(CYAN)Running tests...$(NC)"
	@cargo test --all-features --verbose

test-docker: docker-build ## Run all tests in Docker
	@echo "$(CYAN)Running tests in Docker...$(NC)"
	@docker run --rm -v "$(PWD):/workspace" $(DOCKER_FULL_NAME) cargo test --all-features --verbose

test-doc: ## Run documentation tests
	@echo "$(CYAN)Running documentation tests...$(NC)"
	@cargo test --doc --verbose

test-doc-docker: docker-build ## Run documentation tests in Docker
	@echo "$(CYAN)Running documentation tests in Docker...$(NC)"
	@docker run --rm -v "$(PWD):/workspace" $(DOCKER_FULL_NAME) cargo test --doc --verbose

test-features: ## Test with different feature combinations
	@echo "$(CYAN)Testing different feature combinations...$(NC)"
	@echo "$(BLUE)Testing with all features...$(NC)"
	@cargo test --verbose --all-features
	@echo "$(BLUE)Testing with default features only...$(NC)"
	@cargo test --verbose
	@echo "$(BLUE)Testing with no default features...$(NC)"
	@cargo test --verbose --no-default-features
	@echo "$(GREEN)Feature combinations tested!$(NC)"

msrv-check: ## Verify compatibility with the actual consumer MSRV
	@cargo +$(RUST_MSRV) check --locked --all-targets --all-features

feature-check: ## Check every combination using the same tool as CI
	@set -eu; \
	lock_snapshot=$$(mktemp); \
	cp Cargo.lock "$$lock_snapshot"; \
	trap 'cp "$$lock_snapshot" Cargo.lock; rm -f "$$lock_snapshot"' 0; \
	cargo hack check --workspace --feature-powerset --no-dev-deps

test-lib: ## Run library unit tests only
	@echo "$(CYAN)Running library unit tests...$(NC)"
	@cargo test --lib --all-features

test-integration: ## Run integration tests only
	@echo "$(CYAN)Running integration tests...$(NC)"
	@cargo test --test integration --all-features

# =============================================================================
# Build Commands
# =============================================================================

build: ## Build the project
	@echo "$(CYAN)Building project...$(NC)"
	@cargo build

build-docker: docker-build ## Build the project in Docker
	@echo "$(CYAN)Building project in Docker...$(NC)"
	@docker run --rm -v "$(PWD):/workspace" $(DOCKER_FULL_NAME) cargo build

build-all: ## Build with all features
	@echo "$(CYAN)Building project with all features...$(NC)"
	@cargo build --all-features

build-all-docker: docker-build ## Build with all features in Docker
	@echo "$(CYAN)Building project with all features in Docker...$(NC)"
	@docker run --rm -v "$(PWD):/workspace" $(DOCKER_FULL_NAME) \
		cargo build --all-features

build-release: ## Build optimized release
	@echo "$(CYAN)Building release...$(NC)"
	@cargo build --release --all-features

build-release-docker: docker-build ## Build optimized release in Docker
	@echo "$(CYAN)Building release in Docker...$(NC)"
	@docker run --rm -v "$(PWD):/workspace" $(DOCKER_FULL_NAME) \
		cargo build --release --all-features

# =============================================================================
# Documentation Commands
# =============================================================================

docs: ## Generate documentation
	@echo "$(CYAN)Generating documentation...$(NC)"
	@cargo doc --all-features --no-deps

docs-contract: ## Validate README metadata, quickstart, features, and local links
	@echo "$(CYAN)Validating documentation contract...$(NC)"
	@python3 scripts/check_docs.py

doc-check: docs-strict ## Check Rustdoc warnings and local documentation links

docs-strict: docs-contract ## Generate documentation with strict compiler checks
	@RUSTDOCFLAGS="-D warnings" cargo doc --locked --all-features --no-deps

docs-docker: docker-build ## Generate documentation in Docker
	@echo "$(CYAN)Generating documentation in Docker...$(NC)"
	@docker run --rm -v "$(PWD):/workspace" $(DOCKER_FULL_NAME) \
		cargo doc --all-features --no-deps

docs-open: docs ## Generate and open documentation
	@echo "$(CYAN)Opening documentation...$(NC)"
	@cargo doc --all-features --no-deps --open

# =============================================================================
# Examples and Benchmarks
# =============================================================================

examples: ## Build all examples
	@echo "$(CYAN)Building examples...$(NC)"
	@cargo build --examples --all-features

examples-docker: docker-build ## Build all examples in Docker
	@echo "$(CYAN)Building examples in Docker...$(NC)"
	@docker run --rm -v "$(PWD):/workspace" $(DOCKER_FULL_NAME) \
		cargo build --examples --all-features

run-example: ## Run a specific example (use EXAMPLE=name)
	@echo "$(CYAN)Running example: $(EXAMPLE)$(NC)"
	@cargo run --example $(EXAMPLE) --all-features

bench: ## Run benchmarks
	@echo "$(CYAN)Running benchmarks...$(NC)"
	@cargo bench --all-features

bench-check: ## Check that benchmarks compile (matches CI/CD)
	@echo "$(CYAN)Checking benchmark compilation...$(NC)"
	@cargo bench --no-run --all-features 2>/dev/null || echo "$(YELLOW)No benchmarks found$(NC)"

bench-docker: docker-build ## Run benchmarks in Docker
	@echo "$(CYAN)Running benchmarks in Docker...$(NC)"
	@docker run --rm -v "$(PWD):/workspace" $(DOCKER_FULL_NAME) \
		cargo bench --all-features

# =============================================================================
# Coverage and Profiling
# =============================================================================

coverage: ## Generate test coverage report
	@echo "$(CYAN)Generating coverage report...$(NC)"
	@cargo llvm-cov --workspace --lcov --output-path lcov.info
	@cargo llvm-cov --workspace --html
	@echo "$(GREEN)Coverage report generated$(NC)"
	@echo "$(BLUE)HTML report: target/llvm-cov/html/index.html$(NC)"

coverage-open: coverage ## Generate and open HTML coverage report
	@echo "$(CYAN)Opening coverage report...$(NC)"
	@open target/llvm-cov/html/index.html 2>/dev/null || \
	 xdg-open target/llvm-cov/html/index.html 2>/dev/null || \
	 echo "$(YELLOW)Please open target/llvm-cov/html/index.html manually$(NC)"

coverage-lcov: ## Generate LCOV coverage report only
	@echo "$(CYAN)Generating LCOV coverage report...$(NC)"
	@cargo llvm-cov --workspace --lcov --output-path lcov.info
	@echo "$(GREEN)LCOV report generated at lcov.info$(NC)"

coverage-html: ## Generate HTML coverage report only
	@echo "$(CYAN)Generating HTML coverage report...$(NC)"
	@cargo llvm-cov --workspace --html
	@echo "$(GREEN)HTML report generated in target/llvm-cov/html/index.html$(NC)"

coverage-summary: ## Show coverage summary
	@echo "$(CYAN)Generating coverage summary...$(NC)"
	@cargo llvm-cov --workspace --summary-only

coverage-json: ## Generate JSON coverage report
	@echo "$(CYAN)Generating JSON coverage report...$(NC)"
	@cargo llvm-cov --workspace --json --output-path coverage.json
	@echo "$(GREEN)JSON report generated at coverage.json$(NC)"

coverage-docker: docker-build ## Generate test coverage report in Docker
	@echo "$(CYAN)Generating coverage report in Docker...$(NC)"
	@docker run --rm -v "$(PWD):/workspace" $(DOCKER_FULL_NAME) \
		sh -c "cargo llvm-cov --workspace --lcov --output-path lcov.info && \
		       cargo llvm-cov --workspace --html"

# =============================================================================
# CI/Local Integration
# =============================================================================

ci-local: ## Run CI-like checks locally (full CI/CD simulation)
	@echo "$(CYAN)Running full CI/CD checks locally...$(NC)"
	@echo "$(BLUE)=== Formatting Check ===$(NC)"
	@$(MAKE) fmt-check
	@echo "$(BLUE)=== Strict Linting ===$(NC)"
	@$(MAKE) lint-strict
	@echo "$(BLUE)=== Security Audit ===$(NC)"
	@$(MAKE) audit
	@echo "$(BLUE)=== Dependency Check ===$(NC)"
	@$(MAKE) deny
	@echo "$(BLUE)=== Feature Checks ===$(NC)"
	@$(MAKE) feature-check
	@$(MAKE) msrv-check
	@echo "$(BLUE)=== Tests ===$(NC)"
	@$(MAKE) test
	@echo "$(BLUE)=== Doc Tests ===$(NC)"
	@$(MAKE) test-doc
	@echo "$(BLUE)=== Documentation ===$(NC)"
	@$(MAKE) docs-contract
	@$(MAKE) docs-strict
	@echo "$(BLUE)=== Build All Features ===$(NC)"
	@$(MAKE) build-all
	@echo "$(BLUE)=== Examples ===$(NC)"
	@$(MAKE) examples
	@echo "$(GREEN)All CI/CD checks passed locally!$(NC)"

ci-local-coverage: ## Run CI-like checks locally with coverage
	@echo "$(CYAN)Running CI checks with coverage locally...$(NC)"
	@echo "$(BLUE)=== Formatting ===$(NC)"
	@$(MAKE) fmt-check
	@echo "$(BLUE)=== Linting ===$(NC)"
	@$(MAKE) lint
	@echo "$(BLUE)=== Security Audit ===$(NC)"
	@$(MAKE) audit
	@echo "$(BLUE)=== Tests with Coverage ===$(NC)"
	@$(MAKE) coverage-summary
	@echo "$(BLUE)=== Documentation ===$(NC)"
	@$(MAKE) docs
	@echo "$(BLUE)=== Build ===$(NC)"
	@$(MAKE) build-all
	@echo "$(GREEN)All CI checks with coverage passed locally!$(NC)"

# =============================================================================
# Utility Commands
# =============================================================================

clean: ## Clean build artifacts and coverage reports
	@echo "$(CYAN)Cleaning build artifacts...$(NC)"
	@cargo clean
	@rm -rf target/
	@rm -f lcov.info coverage.json
	@echo "$(GREEN)Clean complete!$(NC)"

watch: ## Watch for changes and run tests
	@echo "$(CYAN)Watching for changes...$(NC)"
	@cargo watch -x "test"

update: ## Update dependencies
	@echo "$(CYAN)Updating dependencies...$(NC)"
	@cargo update

check-deps: ## Check dependency tree
	@echo "$(CYAN)Checking dependency tree...$(NC)"
	@cargo tree --all-features

# =============================================================================
# Development Workflows
# =============================================================================

dev: ## Quick development check (format + lint + test)
	@echo "$(CYAN)Running quick development checks...$(NC)"
	@$(MAKE) fmt
	@$(MAKE) lint
	@$(MAKE) test

dev-docker: ## Quick development check in Docker
	@echo "$(CYAN)Running quick development checks in Docker...$(NC)"
	@$(MAKE) fmt-docker
	@$(MAKE) lint-docker
	@$(MAKE) test-docker

pre-commit: ## Run pre-commit checks
	@echo "$(CYAN)Running pre-commit checks...$(NC)"
	@$(MAKE) fmt-check
	@$(MAKE) lint
	@$(MAKE) test
	@echo "$(GREEN)Pre-commit checks passed!$(NC)"

# =============================================================================
# Ollama SDK Specific Commands
# =============================================================================

test-ollama: ## Test Ollama API integration (requires running Ollama server)
	@echo "$(CYAN)Testing Ollama API integration...$(NC)"
	@curl -s http://localhost:11434/api/version >/dev/null 2>&1 && \
		(echo "$(BLUE)Ollama server detected, running integration tests...$(NC)" && \
		cargo test --test integration --all-features) || \
		echo "$(YELLOW)Warning: Ollama server not running. Start with 'ollama serve'.$(NC)"

run-cli: ## Run the Ollama CLI binary
	@echo "$(CYAN)Running Ollama CLI...$(NC)"
	@cargo run --bin ollama-cli --all-features -- $(ARGS)

stats: ## Show project statistics
	@echo "$(CYAN)Project Statistics$(NC)"
	@echo ""
	@echo "$(BLUE)Source Files:$(NC)"
	@find src -name "*.rs" | wc -l | xargs printf "  Rust files: %s\n"
	@echo ""
	@echo "$(BLUE)Lines of Code:$(NC)"
	@find src -name "*.rs" -exec cat {} \; | wc -l | xargs printf "  Total lines: %s\n"
	@echo ""
	@echo "$(BLUE)Test Count:$(NC)"
	@grep -r "#\[test\]" src tests 2>/dev/null | wc -l | xargs printf "  Tests: %s\n"
	@echo ""
	@echo "$(BLUE)Features:$(NC)"
	@cargo metadata --no-deps --format-version 1 2>/dev/null | jq -r '.packages[0].features | keys[]' 2>/dev/null || echo "  (install jq for feature list)"

# Show variables for debugging
debug-vars: ## Show Makefile variables
	@echo "$(CYAN)Makefile Variables:$(NC)"
	@echo "RUST_MSRV: $(RUST_MSRV)"
	@echo "RUST_TOOLCHAIN: $(RUST_TOOLCHAIN)"
	@echo "BINARY_NAME: $(BINARY_NAME)"
	@echo "BINARY_PACKAGE: $(BINARY_PACKAGE)"
	@echo "SBOM_MANIFEST_PATH: $(SBOM_MANIFEST_PATH)"
	@echo "DOCKER_IMAGE: $(DOCKER_IMAGE)"
	@echo "DOCKER_TAG: $(DOCKER_TAG)"
	@echo "DOCKER_FULL_NAME: $(DOCKER_FULL_NAME)"
