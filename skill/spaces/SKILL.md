---
name: spaces
description: Git worktree manager for parallel development. Use when managing multiple worktrees, switching between branches, or isolating concurrent work.
---

# Spaces Best Practices

Spaces manages git worktrees for parallel, isolated development. Worktrees live in `.spaces/worktrees/`.

## When to Use Spaces

- **Parallel development**: Working on multiple features simultaneously
- **Context switching**: Quickly switch between branches without stashing
- **Isolated testing**: Test different branches/commits independently
- **Code review**: Review PRs in isolated worktrees
- **Bug fixing**: Fix bugs in separate worktrees to avoid disrupting main work

## Naming Conventions

Prefix worktree names with intent:

```bash
# Features
spaces create feat-auth-flow           # New feature
spaces create feat-user-profile        # Another feature

# Fixes
spaces create fix-login-crash          # Bug fix
spaces create fix-memory-leak          # Another bug fix

# Testing
spaces create test-main                # Test main branch
spaces create test-pr-123              # Test PR #123

# Experiments
spaces create exp-new-parser           # Experimental work
spaces create refactor-db              # Refactoring work
```

**Rules:**
- Use lowercase letters, numbers, hyphens
- Max 64 characters (git branch limit)
- Keep names descriptive but concise
- Prefix with category (`feat-`, `fix-`, `test-`, `exp-`, `refactor-`)

## Workflow Patterns

### Feature Development

```bash
# Start feature
spaces create feat-user-auth
spaces enter feat-user-auth
# ... work ...
cd ../.. && spaces remove feat-user-auth    # Clean up when done
```

### PR Review

```bash
# Fetch PR branch first
gh pr checkout 123

# Create worktree from PR branch
spaces create review-123 pr/123
spaces enter review-123
# ... review ...
cd ../.. && spaces remove review-123
```

### Parallel Features

```bash
# Create multiple worktrees
spaces create feat-auth
spaces create feat-payments
spaces create feat-notifications

# Switch as needed
spaces enter feat-auth          # Work on auth
# ... later ...
spaces enter feat-payments      # Switch to payments
```

### Bug Investigation

```bash
# Isolate bug investigation
spaces create investigate-crash main
spaces enter investigate-crash
# ... debug ...
cd ../.. && spaces remove investigate-crash
```

## Hook Best Practices

Use hooks for automation and consistency:

### post-create Hook

```bash
# .spaces/hooks/post-create
#!/bin/bash
WORKTREE_NAME="$1"
WORKTREE_PATH="$2"

echo "Setting up $WORKTREE_NAME..."
cd "$WORKTREE_PATH"

# Install dependencies if needed
if [ -f "package.json" ]; then
    npm install
elif [ -f "requirements.txt" ]; then
    pip install -r requirements.txt
fi

# Copy local config if exists
if [ -f ".env.example" ]; then
    cp .env.example .env
fi
```

### pre-enter Hook

```bash
# .spaces/hooks/pre-enter
#!/bin/bash
WORKTREE_NAME="$1"

echo "Entering $WORKTREE_NAME..."
# Remind about context
echo "Branch: $(git -C "$(spaces enter $WORKTREE_NAME)" branch --show-current)"
```

### post-enter Hook

```bash
# .spaces/hooks/post-enter
#!/bin/bash
WORKTREE_PATH="$2"

cd "$WORKTREE_PATH"

# Activate venv if exists
if [ -f "venv/bin/activate" ]; then
    source venv/bin/activate
elif [ -f ".venv/bin/activate" ]; then
    source .venv/bin/activate
fi

# Show git status
git status --short
```

**Hook tips:**
- Keep hooks idempotent (safe to run multiple times)
- Fail gracefully if dependencies missing
- Use absolute paths when executing commands in worktrees
- Keep hooks fast (they run on every enter/create)

## Common Pitfalls

### Forgetting to Remove Worktrees

Worktrees consume disk space. Clean up regularly:

```bash
# List all worktrees
spaces list

# Remove old/unused worktrees
spaces remove old-feature
spaces remove exp-failed-idea

# Check disk usage
du -sh .spaces/worktrees/*
```

### Naming Conflicts

Avoid names that conflict with git branches:

```bash
# Bad: ambiguous with branch name
spaces create main           # Confusing: which "main"?

# Good: prefix for clarity
spaces create test-main      # Clear: this is a worktree, not the main branch
spaces create review-main    # Clear: reviewing main branch
```

### Working in Wrong Directory

Always use `spaces enter` to enter worktrees, not manual `cd`:

```bash
# Good: uses spaces-enter
spaces enter feat-auth

# Bad: manual cd
cd .spaces/worktrees/feat-auth    # Error-prone
```

### Committing to Wrong Branch

Check branch before working:

```bash
spaces enter feat-auth
git branch --show-current          # Verify you're on the right branch
```

## Agent-Specific Considerations

### Agent Usage Pattern

Agents should use `spaces enter` for programmatic path output:

```bash
# Get worktree path (for scripting)
PATH=$(spaces enter feat-auth)
cd "$PATH"

# Or use the shell integration (after sourcing spaces-enter.sh)
spaces-enter feat-auth
```

### Managing Agent Worktrees

Create dedicated worktrees for agent tasks:

```bash
# Agent worktree naming
spaces create agent-task-001          # Specific task
spaces create agent-experiment-alpha  # Agent experiments
spaces create agent-refactor-x        # Refactoring work
```

### Cleanup Strategy

Clean up agent worktrees after task completion:

```bash
# After agent completes task
spaces remove agent-task-001
```

## Best Practices Summary

1. **Name intentionally**: Use descriptive prefixes (`feat-`, `fix-`, `test-`, `exp-`)
2. **Clean up**: Remove unused worktrees to save disk space
3. **Use hooks**: Automate setup/teardown with hooks
4. **Stay organized**: Keep related worktrees grouped by naming convention
5. **Check context**: Verify branch before committing work
6. **Automate**: Use `--quiet` mode and JSON output for scripting when available
7. **Isolate work**: Use worktrees for feature branches, bug fixes, experiments
8. **Limit concurrent worktrees**: 3-5 active worktrees max to avoid confusion

## Quick Reference

```bash
spaces create <name> [branch]    # Create worktree (optional: from branch)
spaces enter <name>              # Enter worktree (outputs path for scripting)
spaces list                      # List all worktrees
spaces remove <name>             # Remove worktree
spaces info <name>               # Show worktree details
spaces hook list                 # List available hooks
spaces hook <name> <event>       # Run hook manually
```

## Getting Help

```bash
spaces --help              # General help
spaces <command> --help    # Command-specific help
```
