const std = @import("std");
const Command = @import("commands/command.zig").Command;

pub fn main() !void {
    const allocator = std.heap.page_allocator;
    const args = try std.process.argsAlloc(allocator);
    defer std.process.argsFree(allocator, args);

    // Check for global help flags
    if (args.len < 2 or isHelpFlag(args[1])) {
        printUsage();
        return;
    }

    const command_name = args[1];
    const command_args = args[2..];

    // Check if first arg after command is help
    if (command_args.len > 0 and isHelpFlag(command_args[0])) {
        const command = Command.get(command_name) orelse {
            std.debug.print("Unknown command: {s}\n\n", .{command_name});
            printUsage();
            std.process.exit(1);
        };
        printCommandHelp(command);
        return;
    }

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

fn isHelpFlag(arg: []const u8) bool {
    return std.mem.eql(u8, arg, "--help") or std.mem.eql(u8, arg, "-h");
}

fn printUsage() void {
    std.debug.print(
        \\spaces - Git worktree manager for AI agents
        \\
        \\USAGE:
        \\  spaces <command> [args...]
        \\  spaces [flags]
        \\
        \\FLAGS:
        \\  -h, --help    Show this help message
        \\
        \\COMMANDS:
        \\  list                        List all worktrees
        \\  create <name> [branch|-b]   Create a new worktree
        \\  enter <name>                Enter a worktree (cd)
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
        \\  spaces create feat-auth           # Create new branch 'feat-auth'
        \\  spaces create test-main main      # Create worktree from existing 'main' branch
        \\  spaces create wt-1 -b feat-x      # Create worktree with new branch 'feat-x'
        \\  spaces enter feat-auth
        \\  spaces list
        \\  spaces create --help
        \\
    , .{});
}

fn printCommandHelp(command: *const Command) void {
    std.debug.print(
        \\spaces - {s}
        \\
        \\USAGE:
        \\  spaces {s}
        \\
        \\DESCRIPTION:
        \\  {s}
        \\
    , .{ command.description, command.usage, command.description });
}
