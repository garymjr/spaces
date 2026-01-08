const std = @import("std");
const WorktreeManager = @import("../lib/worktree.zig").WorktreeManager;

pub fn run(allocator: std.mem.Allocator, args: []const []const u8) !void {
    if (args.len < 1) {
        std.debug.print("Usage: spaces remove <name>\n", .{});
        return error.MissingArgument;
    }

    const name = args[0];

    const repo_path = try WorktreeManager.findRepoPath(allocator);
    defer allocator.free(repo_path);

    var manager = try WorktreeManager.init(allocator, repo_path);
    defer manager.deinit();

    std.debug.print("Removing worktree '{s}'...\n", .{name});

    try manager.remove(name);

    std.debug.print("✓ Worktree removed successfully!\n", .{});
}
