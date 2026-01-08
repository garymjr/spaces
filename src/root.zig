//! spaces - Git worktree manager for AI agents
const std = @import("std");

pub const WorktreeManager = @import("lib/worktree.zig").WorktreeManager;
pub const HookRunner = @import("lib/hooks.zig").HookRunner;
pub const HookEvent = @import("lib/hooks.zig").HookEvent;

test "WorktreeManager findRepoPath" {
    // Only run this test if we're in a git repo
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
