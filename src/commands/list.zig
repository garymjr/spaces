const std = @import("std");
const WorktreeManager = @import("../lib/worktree.zig").WorktreeManager;

pub fn run(allocator: std.mem.Allocator, args: []const []const u8) !void {
    _ = args;

    const repo_path = try WorktreeManager.findRepoPath(allocator);
    defer allocator.free(repo_path);

    var manager = try WorktreeManager.init(allocator, repo_path);
    defer manager.deinit();

    const worktrees = try manager.list();
    defer {
        for (worktrees) |*w| w.deinit(allocator);
        allocator.free(worktrees);
    }

    if (worktrees.len == 0) {
        std.debug.print("No worktrees found.\n", .{});
        return;
    }

    std.debug.print("{s:<30} {s:<30} {s}\n", .{ "Name", "Branch", "Commit" });
    std.debug.print("{s:<30} {s:<30} {s}\n", .{ "----", "------", "------" });

    for (worktrees) |w| {
        const short_commit = if (w.commit.len > 8) w.commit[0..8] else w.commit;
        std.debug.print("{s:<30} {s:<30} {s}\n", .{ w.name, w.branch, short_commit });
    }
}
