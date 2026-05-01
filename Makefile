SHELL := /bin/bash
.DEFAULT_GOAL := help

# ── Paths ──────────────────────────────────────────────────────────────────────
CAPTURE_DIR  := capture
VIEWER_DIR   := app
CAPTURE_SRCS := $(wildcard $(CAPTURE_DIR)/*.swift)
CAPTURE_BIN  := $(CAPTURE_DIR)/kosmos-capture
SOCKET       := /tmp/kosmos.sock
DMG_DIR      := target/release/bundle/dmg

# Tauri requires the sidecar binary to be named <name>-<target-triple>.
# Detect arch at make-time so the copy step uses the right suffix.
ARCH := $(shell uname -m)
ifeq ($(ARCH),arm64)
  TARGET_TRIPLE := aarch64-apple-darwin
  SWIFT_TARGET  := aarch64-apple-macos11.0
else
  TARGET_TRIPLE := x86_64-apple-darwin
  SWIFT_TARGET  := x86_64-apple-macos11.0
endif

# macOS 11 minimum so os.Logger is available without availability guards.
SWIFT_FLAGS := -O -wmo -target $(SWIFT_TARGET)

SIDECAR_DIR := $(VIEWER_DIR)/src-tauri/binaries
SIDECAR_BIN := $(SIDECAR_DIR)/kosmos-capture-$(TARGET_TRIPLE)

# ── Colours ────────────────────────────────────────────────────────────────────
B := $(shell tput bold 2>/dev/null)
C := $(shell tput setaf 6 2>/dev/null)
G := $(shell tput setaf 2 2>/dev/null)
Y := $(shell tput setaf 3 2>/dev/null)
E := $(shell tput setaf 1 2>/dev/null)
R := $(shell tput sgr0 2>/dev/null)

# ── Targets ────────────────────────────────────────────────────────────────────

.PHONY: all build dev capture swift log check install dmg clean clean-all help
.PHONY: build-capture build-viewer deps

all: build

## build          Build everything for production (Swift + Tauri, single .app)
build: .check-tools deps build-capture build-viewer

## dev            Compile capture, then start Tauri dev mode (spawns capture automatically)
dev: .check-tools deps $(CAPTURE_BIN)
	@printf "$(C)→$(R) Starting dev — viewer will auto-launch capture\n"
	cd $(VIEWER_DIR) && npm run tauri dev

## capture        Build Swift and run the capture daemon in this terminal (foreground)
capture: $(CAPTURE_BIN)
	$(CAPTURE_BIN)

## swift          Compile Swift capture binary only (no run, no sidecar copy)
swift: $(CAPTURE_BIN)
	@printf "$(G)✓$(R) Binary: $(CAPTURE_BIN)\n"

## log            Stream live os.log output from the capture daemon (open a second terminal)
log:
	@log stream --predicate 'subsystem == "dev.kosmos.capture"' --level debug

## check          Type-check Swift + Rust without a full build
check: .check-tools
	@printf "$(C)→$(R) Checking Swift...\n"
	@swiftc -typecheck $(SWIFT_FLAGS) $(CAPTURE_SRCS) && printf "$(G)✓$(R) Swift OK\n"
	@printf "$(C)→$(R) Checking Rust...\n"
	@cargo check --quiet                               && printf "$(G)✓$(R) Rust OK\n"

## dmg            Build release DMG (uses hdiutil, no create-dmg dependency)
dmg: build-capture build-viewer
	$(eval APP    := $(shell find target/release/bundle/macos -name "*.app" -type d | head -1))
	$(eval OUT    := $(DMG_DIR)/Kosmos_$(shell cat $(VIEWER_DIR)/src-tauri/tauri.conf.json | python3 -c "import sys,json;print(json.load(sys.stdin)['version'])")_$(TARGET_TRIPLE).dmg)
	@[ -n "$(APP)" ] || { printf "$(E)✗$(R) .app not found — run make build-viewer first\n"; exit 1; }
	@mkdir -p $(DMG_DIR)
	@printf "$(C)→$(R) Creating DMG...\n"
	@STAGING=$$(mktemp -d) && \
	  cp -R "$(APP)" "$$STAGING/" && \
	  ln -s /Applications "$$STAGING/Applications" && \
	  hdiutil create -volname "Kosmos" -srcfolder "$$STAGING" -ov -format UDZO "$(OUT)" && \
	  rm -rf "$$STAGING"
	@printf "$(G)✓$(R) $(OUT) ($$(du -sh $(OUT) | cut -f1))\n"
	@open $(DMG_DIR)

## install        Copy Kosmos.app to /Applications
install:
	$(eval APP := $(shell find . -name "Kosmos.app" -type d 2>/dev/null | head -1))
	@[ -n "$(APP)" ] || { printf "$(E)✗$(R) Run 'make build-viewer' first\n"; exit 1; }
	@printf "$(C)→$(R) Installing $(APP) → /Applications/Kosmos.app\n"
	@cp -R "$(APP)" /Applications/Kosmos.app
	@printf "$(G)✓$(R) Installed\n"

## clean          Remove compiled binaries, socket, sidecar copy, Tauri build output
clean:
	@printf "$(C)→$(R) Cleaning...\n"
	@rm -f $(CAPTURE_BIN) $(SOCKET)
	@rm -rf $(SIDECAR_DIR)
	@cd $(VIEWER_DIR) && rm -rf dist
	@cargo clean --quiet
	@printf "$(G)✓$(R) Clean\n"

## clean-all      clean + remove node_modules (forces full npm reinstall)
clean-all: clean
	@cd $(VIEWER_DIR) && rm -rf node_modules

# ── Individual build steps ─────────────────────────────────────────────────────

## build-capture  Compile Swift binary and copy as Tauri sidecar
build-capture: $(CAPTURE_BIN) $(SIDECAR_BIN)

$(CAPTURE_BIN): $(CAPTURE_SRCS)
	@printf "$(C)→$(R) Compiling Swift capture ($(SWIFT_TARGET))...\n"
	@swiftc $(SWIFT_FLAGS) $(CAPTURE_SRCS) -o $@
	@printf "$(G)✓$(R) $@\n"

# Copy into binaries/ with the target-triple suffix Tauri expects.
# Runs whenever the source binary is newer than the sidecar copy.
$(SIDECAR_BIN): $(CAPTURE_BIN)
	@mkdir -p $(SIDECAR_DIR)
	@cp $< $@
	@printf "$(G)✓$(R) sidecar → $@\n"

## build-viewer   Build Tauri app for release (bundles Swift sidecar → .app + .dmg)
build-viewer: deps $(SIDECAR_BIN)
	@printf "$(C)→$(R) Building Tauri viewer (release)...\n"
	@cd $(VIEWER_DIR) && npm run tauri build
	@printf "$(G)✓$(R) Bundle: $(shell find . -name 'Kosmos.app' -type d 2>/dev/null | head -1)\n"

## deps           Install npm dependencies (skipped if node_modules is fresh)
deps: $(VIEWER_DIR)/node_modules

$(VIEWER_DIR)/node_modules: $(VIEWER_DIR)/package.json
	@printf "$(C)→$(R) Installing npm dependencies...\n"
	@cd $(VIEWER_DIR) && npm install --silent
	@touch $@

# ── Guards ─────────────────────────────────────────────────────────────────────

.check-tools:
	@command -v swiftc >/dev/null 2>&1 || { printf "$(E)✗$(R) swiftc not found — install Xcode Command Line Tools\n"; exit 1; }
	@command -v cargo  >/dev/null 2>&1 || { printf "$(E)✗$(R) cargo not found  — install via rustup.rs\n";            exit 1; }
	@command -v npm    >/dev/null 2>&1 || { printf "$(E)✗$(R) npm not found    — install Node.js from nodejs.org\n";  exit 1; }

# ── Help ───────────────────────────────────────────────────────────────────────

help:
	@printf "\n$(B)Kosmos$(R) — screen-reading AI assistant\n\n"
	@grep -E '^## ' $(MAKEFILE_LIST) \
		| sed 's/## //' \
		| awk -F'  +' '{printf "  $(C)%-18s$(R) %s\n", $$1, $$2}'
	@printf "\n$(Y)Tip:$(R) Start here → $(B)make dev$(R)\n\n"
