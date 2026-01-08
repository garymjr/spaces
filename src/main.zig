const std = @import("std");
const Command = @import("commands/command.zig").Command;

pub fn main() !void {
    const allocator = std.heap.page_allocator;
    const args = try std.process.argsAlloc(allocator);
    defer std.process.argsFree(allocator, args);

    if (args.len < 2) {
        printUsage();
        return;
    }

    const command_name = args[1];
    const command_args = args[2..];

    const command = Command.get(command_name) orelse {
        std.debug.print("Unknown command: {s}\n\n", .{command_name});
        printUsage();
        std.process.exit(1);
    };

    if (command.run(allocator, command_args)) |_| {} else |err| {
        std.debug.print("Error: {}\n", .{err});
        std.process.exit(1);
    }
}

fn printUsage() void {
    std.debug.print(
        \\spaces - Git worktree manager for AI agents
        \\
        \\USAGE:
        \\  spaces <command> [args...]
        \\
        \\COMMANDS:
        \\  list                  List all worktrees
        \\  create <name>         Create a new worktree
        \\  enter <name>          Enter a worktree (cd)
        \\  remove <name>         Remove a worktree
        \\  info <name>           Show worktree details
        \\  hook <name> <event>   Run a hook manually
        \\
        \\HOOK EVENTS:
        \\  pre-create, post-create
        \\  pre-enter, post-enter
        \\  pre-remove, post-remove
        \\
        \\EXAMPLES:
        \\  spaces create feature-x
        \\  spaces enter feature-x
        \\  spaces list
        \\
    , .{});
}
