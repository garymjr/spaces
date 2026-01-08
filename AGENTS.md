# spaces - AGENTS.md

Git worktree manager for AI agents. Zig CLI tool for managing git worktrees with hooks support.

## Build, Lint, Test Commands

### Build System

Uses `just` task runner with `build.zig` (Zig build system).

```bash
# Show all available recipes
just --list

# Build the project
just build                    # or: zig build
just build-debug              # or: zig build -Doptimize=Debug
just build-release-fast       # or: zig build -Doptimize=ReleaseFast
just build-release-safe       # or: zig build -Doptimize=ReleaseSafe
just build-release-small      # or: zig build -Doptimize=ReleaseSmall

# Run the executable (with optional args)
just run <args>               # or: zig build run -- <args>

# Run tests
just test                     # or: zig build test

# Build and test
just check                    # alias for build + test

# Format code
just fmt                      # or: zig fmt src/
just fmt-check                # or: zig fmt src/ --check

# Clean build artifacts
just clean                    # rm -rf zig-cache zig-out
just clean-all                # clean + .zig-cache

# Install
just install                  # Build and install to ~/.local
just install-prefix <path>    # Install to specific prefix

# Development workflow
just dev                      # Build and symlink to ~/.local/bin/spaces
```

### Running a Single Test

Tests are defined using Zig's built-in `test` blocks. To run specific tests:

```bash
# Run all tests
zig build test

# Run tests from a specific file
zig test src/root.zig

# Run with filter (test names matching pattern)
zig test src/root.zig -test-filter "WorktreeManager"
```

### Version Info

```bash
just version                  # Show Zig compiler version
```

## Code Style Guidelines

### Module System & Imports

- **File extension**: `.zig`
- **Imports**: Use `@import()` at top of file
- **Standard library**: Always import as `const std = @import("std");`
- **Relative imports**: Use `@import("path/to/module.zig")`
- **Module root**: `src/root.zig` is the library entry point

```zig
const std = @import("std");
const WorktreeManager = @import("../lib/worktree.zig").WorktreeManager;
```

### Formatting

- Use `zig fmt` for formatting (standard Zig formatter)
- No manual formatting - let `zig fmt` handle it
- Check formatting with `zig fmt src/ --check`
- Indentation: 4 spaces (handled by formatter)

### Type Usage

- **Strict mode**: Zig is type-safe by default
- **Error unions**: Use `!Type` for functions that can fail
- **Optional types**: Use `?Type` for nullable values
- **No `anytype` abuse**: Use concrete types when possible

```zig
// Function that can fail
pub fn create(self: *Self, name: []const u8, branch: ?[]const u8) !Worktree {
    // ...
}

// Optional parameter
var branch: ?[]const u8 = null;
```

### Naming Conventions

| Category | Convention | Example |
|----------|-----------|---------|
| Files | `snake_case.zig` | `worktree.zig`, `command.zig` |
| Types (structs, enums, unions) | `PascalCase` | `Worktree`, `HookEvent`, `Command` |
| Functions | `camelCase` | `create()`, `getByName()`, `run()` |
| Constants | `camelCase` | `commands` array |
| Variables | `camelCase` | `worktree_path`, `repo_path` |
| Error types | `PascalCase` | `WorktreeNotFound`, `GitCommandFailed` |
| Enum fields | `snake_case` | `pre_create`, `post_enter` |
| Parameters | `camelCase` | `allocator`, `command_name` |

### Memory Management

Zig requires explicit memory management:

- **Allocators**: Pass `std.mem.Allocator` to functions that allocate
- **Deinitialization**: Always provide `deinit()` methods for structs
- **Cleanup**: Use `defer` for guaranteed cleanup

```zig
// Struct with owned memory
pub const WorktreeManager = struct {
    allocator: std.mem.Allocator,
    repo_path: []const u8,

    pub fn init(allocator: std.mem.Allocator, repo_path: []const u8) !Self {
        return Self{
            .allocator = allocator,
            .repo_path = try allocator.dupe(u8, repo_path),
        };
    }

    pub fn deinit(self: *Self) void {
        self.allocator.free(self.repo_path);
    }
};

// Usage with cleanup
var manager = try WorktreeManager.init(allocator, repo_path);
defer manager.deinit();
```

### Error Handling Patterns

- Use `try` to propagate errors
- Use `catch` to handle specific errors
- Define custom error sets
- Use descriptive error messages

```zig
// Propagate errors
const repo_path = try WorktreeManager.findRepoPath(allocator);
defer allocator.free(repo_path);

// Handle specific errors
if (std.fs.accessAbsolute(git_path, .{ .mode = .read_only })) |_| {
    // Success
} else |_| {
    // Handle error
    return error.GitRepoNotFound;
}

// Custom error types (implicit in error unions)
error.GitRepoNotFound, error.WorktreeNotFound, error.HookFailed
```

### Struct Definition Pattern

```zig
pub const Name = struct {
    field: Type,
    another_field: Type,

    const Self = @This();  // Self-referencing pattern

    pub fn init(allocator: std.mem.Allocator) !Self {
        return Self{
            .field = value,
        };
    }

    pub fn deinit(self: *Self) void {
        // Cleanup owned resources
    }

    pub fn method(self: *Self) !void {
        // Method implementation
    }
};
```

### Command Pattern (CLI)

Commands are registered in `src/commands/command.zig`:

```zig
const commands = [_]Command{
    .{
        .name = "list",
        .description = "List all worktrees",
        .run = @import("list.zig").run,
    },
    // ... more commands
};
```

Each command file exports a `run` function:

```zig
pub fn run(allocator: std.mem.Allocator, args: []const []const u8) !void {
    // Implementation
}
```

### Testing Conventions

- Use `test` blocks with descriptive names
- Use `std.testing.allocator` for test allocations
- Use `try std.testing.expect()` for assertions
- Return `error.SkipZigTest` to skip tests conditionally

```zig
test "WorktreeManager findRepoPath" {
    // Skip test if not in a git repo
    _ = std.fs.cwd().access(".git", .{ .mode = .read_only }) catch |err| {
        if (err == error.FileNotFound or err == error.AccessDenied) {
            return error.SkipZigTest;
        }
        return err;
    };

    const allocator = std.testing.allocator;
    const path = try WorktreeManager.findRepoPath(allocator);
    defer allocator.free(path);
    try std.testing.expect(path.len > 0);
}
```

### String Handling

- Slices: `[]const u8` for read-only strings
- Allocation: Use `allocator.dupe(u8, str)` to copy
- Formatting: Use `std.fmt.allocPrint(allocator, "{s}", .{arg})`
- Comparison: Use `std.mem.eql(u8, str1, str2)`

```zig
// String allocation
const worktree_path = try std.fmt.allocPrint(
    self.allocator,
    "{s}/.spaces/worktrees/{s}",
    .{ self.repo_path, name }
);

// String comparison
if (std.mem.eql(u8, cmd.name, name)) {
    return cmd;
}
```

## Architecture

### Project Structure

```
spaces/
├── src/
│   ├── main.zig              # CLI entry point
│   ├── root.zig              # Library exports + tests
│   ├── commands/             # CLI command implementations
│   │   ├── command.zig       # Command registry
│   │   ├── list.zig
│   │   ├── create.zig
│   │   ├── enter.zig
│   │   ├── remove.zig
│   │   ├── info.zig
│   │   └── hook.zig
│   └── lib/                  # Core library
│       ├── worktree.zig      # Git worktree management
│       └── hooks.zig         # Hook system
├── examples/
│   └── hooks/                # Example hook scripts
├── build.zig                 # Zig build configuration
├── build.zig.zon             # Zig package manifest
├── justfile                  # Build task runner
├── README.md                 # User documentation
├── QUICKSTART.md             # Quick start guide
└── spaces-enter.sh           # Shell integration script
```

### Entry Points

**CLI Entry**: `src/main.zig`
- Parses command-line arguments
- Routes to commands via `Command.get()`
- Handles errors and displays usage

**Library Entry**: `src/root.zig`
- Exports public API: `WorktreeManager`, `HookRunner`, `HookEvent`
- Contains library tests

### Key Abstractions

**WorktreeManager** (`src/lib/worktree.zig`)
- Manages git worktrees
- Methods: `list()`, `create()`, `remove()`, `getByName()`, `getWorktreePath()`, `enter()`
- Integrates with `HookRunner` for lifecycle hooks

**HookRunner** (`src/lib/hooks.zig`)
- Manages hook scripts in `.spaces/hooks/`
- Executes hooks before/after worktree operations
- Hook events: `pre_create`, `post_create`, `pre_enter`, `post_enter`, `pre_remove`, `post_remove`

**Command** (`src/commands/command.zig`)
- Registry pattern for CLI commands
- Command struct: name, description, run function pointer

### Important Patterns

1. **Command Registry**: All commands registered in `src/commands/command.zig`
2. **Hook Integration**: Worktree operations run hooks automatically
3. **Git Integration**: Uses `std.process.Child.run()` to execute git commands
4. **Path Resolution**: `findRepoPath()` searches up directory tree for `.git`
5. **Allocator Pattern**: All allocations use explicit allocator

## Development Workflow

### Adding a New Command

1. Create new file in `src/commands/` (e.g., `newcmd.zig`):
```zig
const std = @import("std");
const WorktreeManager = @import("../lib/worktree.zig").WorktreeManager;

pub fn run(allocator: std.mem.Allocator, args: []const []const u8) !void {
    // Implementation
}
```

2. Register in `src/commands/command.zig`:
```zig
.{
    .name = "newcmd",
    .description = "Description",
    .run = @import("newcmd.zig").run,
},
```

3. Update usage in `src/main.zig`

4. Test with `just test` and `just run newcmd`

### Adding Tests

Add test blocks directly in source files:

```zig
test "description" {
    const allocator = std.testing.allocator;
    // Test code
    try std.testing.expect(condition);
}
```

Tests run automatically with `just test`.

### Shell Integration

The tool provides shell integration via `spaces-enter.sh`:
- Wraps `spaces enter` for proper `cd` support
- Source in `~/.zshrc` or `~/.bashrc`
- Aliases: `se`, `sl`, `sc`, `sr`, `si`, `shk`

### Hooks

Hooks are executable scripts in `.spaces/hooks/`:
- Must be executable (`chmod +x`)
- Receive event context as JSON (TODO: implement serialization)
- Can be any executable script (bash, python, etc.)
- Fail silently if hook doesn't exist

## Hook Lifecycle

Hooks are automatically triggered during worktree operations:

| Command | Pre-hook | Post-hook |
|---------|----------|-----------|
| `create` | `pre-create` | `post-create` |
| `enter` | `pre-enter` | `post-enter` |
| `remove` | `pre-remove` | `post-remove` |

**Note**: Hook output is captured but not displayed to avoid clutter. Use output redirection in hook scripts if visibility is needed.

## Recent Fixes (2025-01-08)

### Hook System Issues Resolved

1. **Fixed `hookExists()` logic**: Previously returned `true` when hook didn't exist due to incorrect `== null` comparison. Now properly checks if file access succeeds.

2. **Added `enter()` method to `WorktreeManager`**: The `enter` command was missing lifecycle hooks. Added new `enter()` method that triggers `pre-enter` and `post-enter` hooks.

3. **Updated `enter.zig` command**: Now uses `manager.enter()` instead of `getWorktreePath()` to ensure hooks are triggered.

4. **All lifecycle hooks now working**: `pre_create`, `post_create`, `pre_enter`, `post_enter`, `pre_remove`, `post_remove` all execute correctly.

See `test-hooks.sh` for integration test that verifies all hooks execute.

## Dependencies

### External Dependencies

- **Zig stdlib**: Only dependency (standard library)
- **Git**: External process for worktree management
- **Just**: Task runner (optional, can use zig build directly)

### Minimum Zig Version

```zig
.minimum_zig_version = "0.15.2"
```

## Extension Points

### Adding Hook Events

1. Add event variant to `HookEvent` union in `src/lib/hooks.zig`
2. Add corresponding event struct (e.g., `PreNewEvent`)
3. Call hook in appropriate location in `WorktreeManager`

### Custom Commands

Follow the command pattern (see "Adding a New Command" above).

### Library Usage

The core library can be used independently:

```zig
const spaces = @import("spaces");

// Initialize manager
const manager = try spaces.WorktreeManager.init(allocator, repo_path);
defer manager.deinit();

// Use manager methods
const worktrees = try manager.list();
const worktree = try manager.create("feature-x", null);
try manager.remove("feature-x");
```

## Additional Notes

- **Agent-Friendly**: The `enter` command outputs only the path (no extra formatting) for easy parsing by AI agents
- **Worktree Storage**: Worktrees stored in `.spaces/worktrees/<name>`
- **Hook Storage**: Hooks stored in `.spaces/hooks/`
- **Path Output**: Uses direct stdout write for path output (agent-friendly)
- **Error Messages**: Use `std.debug.print()` for user-facing errors
- **Exit Codes**: Use `std.process.exit(1)` for errors in main
