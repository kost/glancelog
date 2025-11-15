# Makefile for glancelog cross-compilation (static builds)
#
# Usage:
#   make dist          - Build for all platforms
#   make dist-linux    - Build for Linux only (x64 + arm64) - static with musl
#   make dist-macos    - Build for macOS only (x64 + arm64)
#   make dist-windows  - Build for Windows only (x64 + arm64) - static
#   make clean         - Remove dist directory
#
# Static Build Strategy:
#   - Linux: Uses musl libc for fully static binaries
#   - Windows: Uses static CRT and links statically where possible
#   - macOS: Limited static linking (system frameworks are dynamic)
#
# Requirements:
#   - rustup with target support (installed via 'make install-targets')
#   - For Linux arm64 musl: musl-tools and gcc-aarch64-linux-gnu
#   - For Windows: MSVC (Visual Studio Build Tools) or use GitHub Actions
#   - For macOS: Xcode command line tools (on macOS host)
#
# Note: Some targets may fail if cross-compilation toolchains are not installed.
#       The build will continue for other targets.

# Version (update this for releases)
VERSION ?= 1.0.2

# Directories
DIST_DIR = dist
TARGET_DIR = target

# Binary name
BINARY_NAME = glancelog

# Target triples (using musl for Linux static builds)
LINUX_X64 = x86_64-unknown-linux-musl
LINUX_ARM64 = aarch64-unknown-linux-musl
MACOS_X64 = x86_64-apple-darwin
MACOS_ARM64 = aarch64-apple-darwin
WINDOWS_X64 = x86_64-pc-windows-msvc
WINDOWS_X86 = i686-pc-windows-msvc
WINDOWS_ARM64 = aarch64-pc-windows-msvc

# All targets
ALL_TARGETS = $(LINUX_X64) $(LINUX_ARM64) $(MACOS_X64) $(MACOS_ARM64) $(WINDOWS_X64) $(WINDOWS_X86) $(WINDOWS_ARM64)

.PHONY: all dist dist-linux dist-macos dist-windows clean install-targets help

# Default target
all: dist

# Help target
help:
	@echo "glancelog cross-compilation Makefile (static builds)"
	@echo ""
	@echo "Targets:"
	@echo "  dist           - Build for all platforms (Linux, macOS, Windows)"
	@echo "  dist-linux     - Build for Linux (x64 + arm64) - static with musl"
	@echo "  dist-macos     - Build for macOS (x64 + arm64)"
	@echo "  dist-windows   - Build for Windows (x64 + x86 + arm64) - static"
	@echo "  install-targets - Install all rustup targets"
	@echo "  clean          - Remove dist directory"
	@echo ""
	@echo "Static Build Strategy:"
	@echo "  - Linux: musl libc for fully static binaries"
	@echo "  - Windows: static CRT linking"
	@echo "  - macOS: limited static linking (system frameworks are dynamic)"
	@echo ""
	@echo "Output directory: $(DIST_DIR)/"

# Build for all platforms
dist: install-targets dist-linux dist-macos dist-windows
	@echo ""
	@echo "==================================================================="
	@echo "All binaries built successfully in $(DIST_DIR)/"
	@echo "==================================================================="
	@ls -lh $(DIST_DIR)/

# Linux builds
dist-linux: dist-linux-x64 dist-linux-arm64
	@echo "Linux builds complete"

dist-linux-x64: create-dist-dir
	@echo "Building for Linux x64 (static with musl)..."
	RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target $(LINUX_X64)
	cp $(TARGET_DIR)/$(LINUX_X64)/release/$(BINARY_NAME) $(DIST_DIR)/$(BINARY_NAME)-linux-x64
	@echo "✓ Linux x64 static build complete"

dist-linux-arm64: create-dist-dir
	@echo "Building for Linux arm64 (static with musl)..."
	-RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target $(LINUX_ARM64)
	-cp $(TARGET_DIR)/$(LINUX_ARM64)/release/$(BINARY_NAME) $(DIST_DIR)/$(BINARY_NAME)-linux-arm64
	@if [ -f $(DIST_DIR)/$(BINARY_NAME)-linux-arm64 ]; then \
		echo "✓ Linux arm64 static build complete"; \
	else \
		echo "⚠ Linux arm64 build failed (musl cross-compiler may not be installed)"; \
	fi

# macOS builds
dist-macos: dist-macos-x64 dist-macos-arm64
	@echo "macOS builds complete"

dist-macos-x64: create-dist-dir
	@echo "Building for macOS x64..."
	-cargo build --release --target $(MACOS_X64)
	-cp $(TARGET_DIR)/$(MACOS_X64)/release/$(BINARY_NAME) $(DIST_DIR)/$(BINARY_NAME)-macos-x64
	@if [ -f $(DIST_DIR)/$(BINARY_NAME)-macos-x64 ]; then \
		echo "✓ macOS x64 build complete"; \
	else \
		echo "⚠ macOS x64 build failed (requires macOS host or cross-compiler)"; \
	fi

dist-macos-arm64: create-dist-dir
	@echo "Building for macOS arm64..."
	-cargo build --release --target $(MACOS_ARM64)
	-cp $(TARGET_DIR)/$(MACOS_ARM64)/release/$(BINARY_NAME) $(DIST_DIR)/$(BINARY_NAME)-macos-arm64
	@if [ -f $(DIST_DIR)/$(BINARY_NAME)-macos-arm64 ]; then \
		echo "✓ macOS arm64 build complete"; \
	else \
		echo "⚠ macOS arm64 build failed (requires macOS host or cross-compiler)"; \
	fi

# Windows builds
dist-windows: dist-windows-x64 dist-windows-x86 dist-windows-arm64
	@echo "Windows builds complete"

dist-windows-x64: create-dist-dir
	@echo "Building for Windows x64 (static)..."
	-RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target $(WINDOWS_X64)
	-cp $(TARGET_DIR)/$(WINDOWS_X64)/release/$(BINARY_NAME).exe $(DIST_DIR)/$(BINARY_NAME)-windows-x64.exe
	@if [ -f $(DIST_DIR)/$(BINARY_NAME)-windows-x64.exe ]; then \
		echo "✓ Windows x64 static build complete"; \
	else \
		echo "⚠ Windows x64 build failed (MSVC may not be installed)"; \
	fi

dist-windows-x86: create-dist-dir
	@echo "Building for Windows x86/32-bit (static)..."
	-RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target $(WINDOWS_X86)
	-cp $(TARGET_DIR)/$(WINDOWS_X86)/release/$(BINARY_NAME).exe $(DIST_DIR)/$(BINARY_NAME)-windows-x86.exe
	@if [ -f $(DIST_DIR)/$(BINARY_NAME)-windows-x86.exe ]; then \
		echo "✓ Windows x86 static build complete"; \
	else \
		echo "⚠ Windows x86 build failed (MSVC may not be installed)"; \
	fi

dist-windows-arm64: create-dist-dir
	@echo "Building for Windows arm64 (static)..."
	-RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target $(WINDOWS_ARM64)
	-cp $(TARGET_DIR)/$(WINDOWS_ARM64)/release/$(BINARY_NAME).exe $(DIST_DIR)/$(BINARY_NAME)-windows-arm64.exe
	@if [ -f $(DIST_DIR)/$(BINARY_NAME)-windows-arm64.exe ]; then \
		echo "✓ Windows arm64 static build complete"; \
	else \
		echo "⚠ Windows arm64 build failed (Windows host or cross-compiler required)"; \
	fi

# Install all required rustup targets
install-targets:
	@echo "Installing rustup targets..."
	@for target in $(ALL_TARGETS); do \
		echo "Installing $$target..."; \
		rustup target add $$target || true; \
	done
	@echo "✓ All targets installed"

# Create dist directory
create-dist-dir:
	@mkdir -p $(DIST_DIR)

# Clean build artifacts
clean:
	@echo "Cleaning dist directory..."
	rm -rf $(DIST_DIR)
	@echo "✓ Clean complete"

# Clean everything including cargo build artifacts
clean-all: clean
	@echo "Cleaning all build artifacts..."
	cargo clean
	@echo "✓ Clean all complete"
