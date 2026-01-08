# Build and install commands for spaces

# Default: show available recipes
default:
    @just --list

# Build the project
build:
    zig build

# Build with debug
build-debug:
    zig build -Doptimize=Debug

# Build release-fast
build-release-fast:
    zig build -Doptimize=ReleaseFast

# Build release-safe
build-release-safe:
    zig build -Doptimize=ReleaseSafe

# Build release-small
build-release-small:
    zig build -Doptimize=ReleaseSmall

# Run the executable (with optional args)
run *args:
    zig build run -- {{args}}

# Run tests
test:
    zig build test

# Install to local bin
install: build
    zig build install --prefix ~/.local

# Install to specific prefix
install-prefix prefix="~/.local": build
    zig build install --prefix {{prefix}}

# Uninstall
uninstall:
    rm -f ~/.local/bin/spaces

# Clean build artifacts
clean:
    rm -rf zig-cache zig-out

# Clean everything including dependencies
clean-all: clean
    rm -rf .zig-cache

# Build and test
check: build test

# Format code
fmt:
    zig fmt src/

# Check code formatting (no changes)
fmt-check:
    zig fmt src/ --check

# Build and create symlink for development
dev: build
    ln -sf $(pwd)/zig-out/bin/spaces ~/.local/bin/spaces

# Show version info
version:
    @zig version
