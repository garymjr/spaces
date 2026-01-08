const std = @import("std");
const WorktreeManager = @import("../lib/worktree.zig").WorktreeManager;

pub fn run(allocator: std.mem.Allocator, args: []const []const u8) !void {
    if (args.len < 1) {
        std.debug.print("Usage: spaces create <name> [existing-branch | -b <new-branch>]\n", .{});
        std.debug.print("\nExamples:\n", .{});
        std.debug.print("  spaces create feat-auth           # Create new branch 'feat-auth'\n", .{});
        std.debug.print("  spaces create test-main main      # Create worktree from existing 'main' branch\n", .{});
        std.debug.print("  spaces create wt-1 -b feat-x      # Create worktree 'wt-1' with new branch 'feat-x'\n", .{});
        return error.MissingArgument;
    }

    const name = args[0];
    var branch: ?[]const u8 = null;
    var create_new_branch = true;

    // Parse optional argument
    if (args.len >= 2) {
        if (std.mem.eql(u8, args[1], "-b")) {
            // -b <branch>: create new branch with this name
            if (args.len < 3) {
                std.debug.print("Error: -b requires a branch name\n", .{});
                return error.MissingArgument;
            }
            branch = args[2];
            create_new_branch = true;
        } else {
            // <existing-branch>: checkout existing branch
            branch = args[1];
            create_new_branch = false;
        }
    }

    const repo_path = try WorktreeManager.findRepoPath(allocator);
    defer allocator.free(repo_path);

    var manager = try WorktreeManager.init(allocator, repo_path);
    defer manager.deinit();

    std.debug.print("Creating worktree '{s}'...\n", .{name});

    if (create_new_branch) {
        std.debug.print("  Creating new branch '{s}'...\n", .{branch orelse name});
    } else {
        std.debug.print("  Checking out existing branch '{s}'...\n", .{branch.?});
    }

    const worktree = manager.create(name, branch, create_new_branch) catch |err| {
        std.debug.print("Failed to create worktree: {}\n", .{err});
        return err;
    };
    defer worktree.deinit(allocator);

    std.debug.print("✓ Worktree created successfully!\n", .{});
    std.debug.print("  Path: {s}\n", .{worktree.path});
    std.debug.print("  Branch: {s}\n", .{worktree.branch});
    std.debug.print("  Enter with: spaces enter {s}\n", .{name});
}
