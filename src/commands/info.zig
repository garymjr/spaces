const std = @import("std");
const WorktreeManager = @import("../lib/worktree.zig").WorktreeManager;

pub fn run(allocator: std.mem.Allocator, args: []const []const u8) !void {
    if (args.len < 1) {
        std.debug.print("Usage: spaces info <name>\n", .{});
        return error.MissingArgument;
    }

    const name = args[0];

    const repo_path = try WorktreeManager.findRepoPath(allocator);
    defer allocator.free(repo_path);

    var manager = try WorktreeManager.init(allocator, repo_path);
    defer manager.deinit();

    const worktree = try manager.getByName(name);
    defer worktree.deinit(allocator);

    std.debug.print("Worktree: {s}\n", .{worktree.name});
    std.debug.print("  Path: {s}\n", .{worktree.path});
    std.debug.print("  Branch: {s}\n", .{worktree.branch});
    std.debug.print("  Commit: {s}\n", .{worktree.commit});
    if (worktree.is_detached) {
        std.debug.print("  Status: detached HEAD\n", .{});
    } else {
        std.debug.print("  Status: on branch\n", .{});
    }
}
