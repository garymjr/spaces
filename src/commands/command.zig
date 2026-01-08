const std = @import("std");

pub const Command = struct {
    name: []const u8,
    description: []const u8,
    run: *const fn (allocator: std.mem.Allocator, args: []const []const u8) anyerror!void,

    pub fn get(name: []const u8) ?*const Command {
        for (&commands) |*cmd| {
            if (std.mem.eql(u8, cmd.name, name)) {
                return cmd;
            }
        }
        return null;
    }

    pub fn list() []const Command {
        return commands;
    }
};

const commands = [_]Command{
    .{
        .name = "list",
        .description = "List all worktrees",
        .run = @import("list.zig").run,
    },
    .{
        .name = "create",
        .description = "Create a new worktree",
        .run = @import("create.zig").run,
    },
    .{
        .name = "enter",
        .description = "Enter a worktree (cd)",
        .run = @import("enter.zig").run,
    },
    .{
        .name = "remove",
        .description = "Remove a worktree",
        .run = @import("remove.zig").run,
    },
    .{
        .name = "info",
        .description = "Show worktree details",
        .run = @import("info.zig").run,
    },
    .{
        .name = "hook",
        .description = "Run a hook manually",
        .run = @import("hook.zig").run,
    },
};
