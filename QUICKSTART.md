# Spaces CLI - Quick Start

## Installation

```bash
# Build
zig build

# Install to ~/.local
zig build install --prefix ~/.local

# Add shell integration to ~/.zshrc or ~/.bashrc
source /path/to/spaces/spaces-enter.sh
```

## Basic Usage

```bash
# List worktrees
spaces list

# Create a new worktree (creates new branch)
spaces create feature-auth

# Create worktree from existing branch
spaces create feature-auth -b develop

# Enter a worktree (cd to it)
spaces-enter feature-auth
# Or use alias: se feature-auth

# Show worktree info
spaces info feature-auth

# Remove a worktree
spaces remove feature-auth
```

## Shell Aliases

After sourcing `spaces-enter.sh`:

| Alias | Command | Description |
|-------|---------|-------------|
| `se` | `spaces-enter` | Enter worktree (cd) |
| `sl` | `spaces list` | List worktrees |
| `sc` | `spaces create` | Create worktree |
| `sr` | `spaces remove` | Remove worktree |
| `si` | `spaces info` | Show worktree info |
| `shk` | `spaces hook` | Run hooks |

## Hooks

Create hooks in `.spaces/hooks/` directory:

```bash
mkdir -p .spaces/hooks

# Create a post-create hook
cat > .spaces/hooks/post-create << 'EOF'
#!/bin/bash
echo "Setting up new worktree..."
# Your setup commands here
EOF

chmod +x .spaces/hooks/post-create
```

### Available Hook Events

- `pre-create` - Before worktree creation
- `post-create` - After worktree creation
- `pre-enter` - Before entering worktree
- `post-enter` - After entering worktree
- `pre-remove` - Before worktree removal
- `post-remove` - After worktree removal

## Workflows

### Feature Development

```bash
# Start new feature
sc feature-user-auth

# Work on it
se feature-user-auth
# ... make changes ...

# When done
cd ../..
sr feature-user-auth
```

### Testing Against Multiple Branches

```bash
# Create worktrees for testing
sc test-main -b main
sc test-develop -b develop

# Switch between contexts easily
se test-main  # Test on main
se test-develop  # Test on develop
```

### Bug Fixing

```bash
# Create worktree from issue branch
sc fix-issue-123 -b issue/123

# Fix the bug
se fix-issue-123
# ... fix bug ...

# Clean up after PR merges
sr fix-issue-123
```

## Tips

1. **Agent-Friendly**: The `enter` command outputs just the path, making it easy for AI agents to use:
   ```bash
   cd "$(spaces enter feature-x)"
   ```

2. **Hook Automation**: Use hooks for setup, dependency installation, etc.

3. **Parallel Development**: Keep multiple features active simultaneously.

4. **Clean Worktrees**: Worktrees are isolated; commits don't affect each other.

## Examples

See `examples/hooks/` for hook examples.
