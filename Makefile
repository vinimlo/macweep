.PHONY: dev dev-frontend dev-tauri build build-dmg lint test clean help

.DEFAULT_GOAL := help

help: ## Show this help
	@echo "macweep — macOS storage cleanup tool"
	@echo ""
	@echo "Usage: make <target>"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-16s\033[0m %s\n", $$1, $$2}'
	@echo ""

dev: ## Start frontend (Docker) + Tauri dev mode
	docker compose up -d frontend
	source $$HOME/.cargo/env && cargo tauri dev

dev-frontend: ## Start frontend only (Docker)
	docker compose up frontend

dev-tauri: ## Start Tauri dev mode only (no Docker)
	source $$HOME/.cargo/env && cargo tauri dev

build: ## Build frontend + Tauri for production
	npm run build
	source $$HOME/.cargo/env && cargo tauri build

build-dmg: ## Build macOS .dmg bundle (dmgbuild, bypasses broken AppleScript on macOS 26)
	source $$HOME/.cargo/env && cargo tauri build --bundles app
	mkdir -p src-tauri/target/release/bundle/dmg
	dmgbuild -s scripts/dmg-settings.py "macweep" \
		src-tauri/target/release/bundle/dmg/macweep_0.1.0_aarch64.dmg

lint: ## Run clippy (Rust) + svelte check
	cd src-tauri && source $$HOME/.cargo/env && cargo clippy -- -D warnings
	npm run check

test: ## Run Rust tests + svelte check
	cd src-tauri && source $$HOME/.cargo/env && cargo test
	npm run check

clean: ## Stop Docker, clean all build artifacts
	docker compose down
	cd src-tauri && source $$HOME/.cargo/env && cargo clean
	rm -rf build .svelte-kit node_modules
