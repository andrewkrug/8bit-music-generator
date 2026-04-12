.PHONY: build run test lint fmt check clean help

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}'

build: ## Build the project
	cargo build

run: ## Run the MCP server
	cargo run

test: ## Run all tests
	cargo test --all-targets

lint: ## Run clippy linter
	cargo clippy --all-targets -- -D warnings

fmt: ## Format code
	cargo fmt --all

fmt-check: ## Check formatting without modifying files
	cargo fmt --all -- --check

check: fmt-check lint test ## Run all checks (format, lint, test)

clean: ## Remove build artifacts
	cargo clean
	rm -rf sample-output/

release: ## Build optimized release binary
	cargo build --release
