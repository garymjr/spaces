# spaces

Git worktree manager for AI agents. Create, manage, and switch between worktrees with hooks support.

## Installation

### Using just (recommended)

```bash
# Install just: https://github.com/casey/just#installation
# Then build and install
just install
```

### Using zig directly

```bash
# Build from source
zig build
zig build install --prefix ~/.local

# Or add to PATH
export PATH="$HOME/.local/bin:$PATH"
```

### Development setup

```bash
# Build and symlink for quick iteration
just dev
```

## Usage

```bash
spaces <command> [args...]
```

## Commands

| Command | Description |
|---------|-------------|
| `list` | List all worktrees |
| `create <name> [branch]` | Create a new worktree (use `-b <branch>` for custom branch name) |
| `enter <name>` | Enter a worktree (cd) |
| `remove <name>` | Remove a worktree |
| `info <name>` | Show worktree details |
| `hook list` | List available hooks |
| `hook <name> <event>` | Run a hook manually |

## Shell Integration

Add to your `~/.zshrc` or `~/.bashrc`:

```bash
# spaces-enter function for proper cd support
spaces-enter() {
    local path
    path=$(spaces enter "$@") || return $?
    cd "$path"
}

# Alias for convenience
alias se='spaces-enter'
alias sl='spaces list'
alias sc='spaces create'
alias sr='spaces remove'
alias si='spaces info'
```

Now use `se <name>` to enter worktrees.

## Build & Development

```bash
just --list           # Show all available recipes
just build            # Build the project
just test             # Run tests
just check            # Build + test
just fmt              # Format code
just clean            # Clean build artifacts
just run <args>       # Build and run with args
```

## Hooks

Hooks are executable scripts in `.spaces/hooks/`. Supported events:

| Event | Description |
|-------|-------------|
| `pre-create` | Before worktree creation |
| `post-create` | After worktree creation |
| `pre-enter` | Before entering worktree |
| `post-enter` | After entering worktree |
| `pre-remove` | Before worktree removal |
| `post-remove` | After worktree removal |

### Creating a Hook

```bash
mkdir -p .spaces/hooks

# Example: post-create hook
cat > .spaces/hooks/post-create << 'EOF'
#!/bin/bash
echo "Setting up new worktree..."
# Your setup commands here
EOF

chmod +x .spaces/hooks/post-create
```

### Hook Environment

Hooks receive context about the event. Hook data will be serialized as JSON and passed to the hook script.

## Examples

```bash
# List worktrees
spaces list

# Create a worktree with new branch (branch name = worktree name)
spaces create feature-auth

# Create worktree from existing branch
spaces create test-main main

# Create worktree with custom new branch name
spaces create wt-1 -b feature-x

# Enter a worktree
spaces-enter feature-auth

# Show worktree info
spaces info feature-auth

# Remove a worktree
spaces remove feature-auth

# List available hooks
spaces hook list

# Run a hook manually
spaces hook feature-auth post-create
```

## Architecture

```
spaces/
├── src/
│   ├── main.zig           # CLI entry point
│   ├── root.zig           # Library exports
│   ├── commands/          # Command implementations
│   │   ├── command.zig    # Command registry
│   │   ├── list.zig
│   │   ├── create.zig
│   │   ├── enter.zig
│   │   ├── remove.zig
│   │   ├── info.zig
│   │   └── hook.zig
│   └── lib/
│       ├── worktree.zig   # Worktree management
│       └── hooks.zig      # Hook system
```

## Library Usage

```zig
const spaces = @import("spaces");

// Initialize manager
const manager = try spaces.WorktreeManager.init(allocator, repo_path);
defer manager.deinit();

// List worktrees
const worktrees = try manager.list();

// Create worktree
const worktree = try manager.create("feature-x", null);

// Enter worktree
const path = try manager.getWorktreePath("feature-x");

// Remove worktree
try manager.remove("feature-x");
```

## License

MIT
