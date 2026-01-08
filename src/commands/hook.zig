const std = @import("std");
const WorktreeManager = @import("../lib/worktree.zig").WorktreeManager;
const HookRunner = @import("../lib/hooks.zig").HookRunner;

pub fn run(allocator: std.mem.Allocator, args: []const []const u8) !void {
    if (args.len < 1) {
        std.debug.print("Usage: spaces hook <name> <event>\n", .{});
        return error.MissingArgument;
    }

    if (std.mem.eql(u8, args[0], "list")) {
        return listHooks(allocator);
    }

    if (args.len < 2) {
        std.debug.print("Usage: spaces hook <name> <event>\n", .{});
        return error.MissingArgument;
    }

    const name = args[0];
    const event = args[1];

    const repo_path = try WorktreeManager.findRepoPath(allocator);
    defer allocator.free(repo_path);

    var manager = try WorktreeManager.init(allocator, repo_path);
    defer manager.deinit();

    std.debug.print("Running hook '{s}' for worktree '{s}'...\n", .{ event, name });

    try manager.hooks.run(event, .{ .name = name });

    std.debug.print("✓ Hook completed successfully!\n", .{});
}

fn listHooks(allocator: std.mem.Allocator) !void {
    const repo_path = try WorktreeManager.findRepoPath(allocator);
    defer allocator.free(repo_path);

    var hooks = try HookRunner.init(allocator, repo_path);
    defer hooks.deinit();

    const hook_list = try hooks.listHooks();
    defer {
        for (hook_list) |h| allocator.free(h);
        allocator.free(hook_list);
    }

    std.debug.print("Available hooks:\n", .{});

    if (hook_list.len == 0) {
        std.debug.print("  (none)\n", .{});
        return;
    }

    for (hook_list) |hook| {
        std.debug.print("  {s}\n", .{hook});
    }
}
