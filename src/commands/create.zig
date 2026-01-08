const std = @import("std");
const WorktreeManager = @import("../lib/worktree.zig").WorktreeManager;

pub fn run(allocator: std.mem.Allocator, args: []const []const u8) !void {
    if (args.len < 1) {
        std.debug.print("Usage: spaces create <name> [-b <branch>]\n", .{});
        return error.MissingArgument;
    }

    const name = args[0];
    var branch: ?[]const u8 = null;

    // Parse optional -b flag
    if (args.len >= 3 and std.mem.eql(u8, args[1], "-b")) {
        branch = args[2];
    }

    const repo_path = try WorktreeManager.findRepoPath(allocator);
    defer allocator.free(repo_path);

    var manager = try WorktreeManager.init(allocator, repo_path);
    defer manager.deinit();

    std.debug.print("Creating worktree '{s}'...\n", .{name});

    const worktree = manager.create(name, branch) catch |err| {
        std.debug.print("Failed to create worktree: {}\n", .{err});
        return err;
    };
    defer worktree.deinit(allocator);

    std.debug.print("✓ Worktree created successfully!\n", .{});
    std.debug.print("  Path: {s}\n", .{worktree.path});
    std.debug.print("  Branch: {s}\n", .{worktree.branch});
    std.debug.print("  Enter with: spaces enter {s}\n", .{name});
}
