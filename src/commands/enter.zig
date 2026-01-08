const std = @import("std");
const WorktreeManager = @import("../lib/worktree.zig").WorktreeManager;

pub fn run(allocator: std.mem.Allocator, args: []const []const u8) !void {
    if (args.len < 1) {
        std.debug.print("Usage: spaces enter <name>\n", .{});
        return error.MissingArgument;
    }

    const name = args[0];

    const repo_path = try WorktreeManager.findRepoPath(allocator);
    defer allocator.free(repo_path);

    var manager = try WorktreeManager.init(allocator, repo_path);
    defer manager.deinit();

    // Enter runs pre-enter and post-enter hooks
    const worktree_path = try manager.enter(name);
    defer allocator.free(worktree_path);

    // Print the path for the shell to use
    const stdout_file = std.fs.File.stdout();
    var buf: [4096]u8 = undefined;
    var writer = stdout_file.writer(&buf);
    try writer.interface.writeAll(worktree_path);
    try writer.interface.flush();
}
