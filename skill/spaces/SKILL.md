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
sc feat-auth-flow           # New feature
sc feat-user-profile        # Another feature

# Fixes
sc fix-login-crash          # Bug fix
sc fix-memory-leak          # Another bug fix

# Testing
sc test-main                # Test main branch
sc test-pr-123              # Test PR #123

# Experiments
sc exp-new-parser           # Experimental work
sc refactor-db              # Refactoring work
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
sc feat-user-auth
se feat-user-auth
# ... work ...
cd ../.. && sr feat-user-auth    # Clean up when done
```

### PR Review

```bash
# Fetch PR branch first
gh pr checkout 123

# Create worktree from PR
sc review-123 -b pr/123
se review-123
# ... review ...
cd ../.. && sr review-123
```

### Parallel Features

```bash
# Create multiple worktrees
sc feat-auth
sc feat-payments
sc feat-notifications

# Switch as needed
se feat-auth          # Work on auth
# ... later ...
se feat-payments      # Switch to payments
```

### Bug Investigation

```bash
# Isolate bug investigation
sc investigate-crash -b main
se investigate-crash
# ... debug ...
cd ../.. && sr investigate-crash
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
sl

# Remove old/unused worktrees
sr old-feature
sr exp-failed-idea

# Check disk usage
du -sh .spaces/worktrees/*
```

### Naming Conflicts

Avoid names that conflict with git branches:

```bash
# Bad: ambiguous with branch name
sc main           # Confusing: which "main"?

# Good: prefix for clarity
sc test-main      # Clear: this is a worktree, not the main branch
sc review-main    # Clear: reviewing main branch
```

### Working in Wrong Directory

Always use `se` (spaces-enter) to enter worktrees, not manual `cd`:

```bash
# Good: uses spaces-enter
se feat-auth

# Bad: manual cd
cd .spaces/worktrees/feat-auth    # Error-prone
```

### Committing to Wrong Branch

Check branch before working:

```bash
se feat-auth
git branch --show-current          # Verify you're on the right branch
```

## Agent-Specific Considerations

### Agent Usage Pattern

Agents should use `spaces enter` for programmatic path output:

```bash
# Get worktree path (for scripting)
PATH=$(spaces enter feat-auth)
cd "$PATH"

# Or use the shell integration
se feat-auth
```

### Managing Agent Worktrees

Create dedicated worktrees for agent tasks:

```bash
# Agent worktree naming
sc agent-task-001          # Specific task
sc agent-experiment-alpha  # Agent experiments
sc agent-refactor-x        # Refactoring work
```

### Cleanup Strategy

Clean up agent worktrees after task completion:

```bash
# After agent completes task
sr agent-task-001
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
# Shell aliases (source spaces-enter.sh)
se <name>    # Enter worktree (cd to it)
sl           # List worktrees
sc <name>    # Create worktree
sr <name>    # Remove worktree
si <name>    # Show worktree info
shk          # Run hooks

# List hooks
spaces hook list

# Run hook manually
spaces hook <name> <event>
```

## Getting Help

```bash
spaces --help              # General help
spaces <command> --help    # Command-specific help
```
