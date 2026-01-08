const std = @import("std");

pub const HookEvent = union(enum) {
    pre_create: PreCreateEvent,
    post_create: PostCreateEvent,
    pre_enter: PreEnterEvent,
    post_enter: PostEnterEvent,
    pre_remove: PreRemoveEvent,
    post_remove: PostRemoveEvent,

    pub const PreCreateEvent = struct {
        name: []const u8,
        branch: ?[]const u8,
    };

    pub const PostCreateEvent = struct {
        name: []const u8,
        worktree: WorktreeInfo,
    };

    pub const PreEnterEvent = struct {
        name: []const u8,
        worktree: WorktreeInfo,
    };

    pub const PostEnterEvent = struct {
        name: []const u8,
        worktree: WorktreeInfo,
    };

    pub const PreRemoveEvent = struct {
        name: []const u8,
        worktree: WorktreeInfo,
    };

    pub const PostRemoveEvent = struct {
        name: []const u8,
    };

    pub const WorktreeInfo = struct {
        path: []const u8,
        branch: []const u8,
        commit: []const u8,
        is_detached: bool,
    };
};

pub const HookRunner = struct {
    allocator: std.mem.Allocator,
    hooks_dir: []const u8,

    const Self = @This();

    pub fn init(allocator: std.mem.Allocator, repo_path: []const u8) !Self {
        const hooks_dir = try std.fmt.allocPrint(allocator, "{s}/.spaces/hooks", .{repo_path});
        return Self{
            .allocator = allocator,
            .hooks_dir = hooks_dir,
        };
    }

    pub fn deinit(self: *Self) void {
        self.allocator.free(self.hooks_dir);
    }

    /// Run a hook by event name
    pub fn run(self: *Self, event_name: []const u8, event_data: anytype) !void {
        const hook_path = try std.fmt.allocPrint(self.allocator, "{s}/{s}", .{ self.hooks_dir, event_name });
        defer self.allocator.free(hook_path);

        // Check if hook exists
        if (std.fs.accessAbsolute(hook_path, .{})) |_| {
            // Hook exists, run it
            try self.executeHook(hook_path, event_data);
        } else |_| {
            // Hook doesn't exist, that's fine
            return;
        }
    }

    fn executeHook(self: *Self, hook_path: []const u8, event_data: anytype) !void {
        _ = event_data; // TODO: Serialize event data and pass to hook

        const result = try std.process.Child.run(.{
            .allocator = self.allocator,
            .argv = &[_][]const u8{ hook_path },
            .cwd = self.hooks_dir,
        });

        defer {
            self.allocator.free(result.stdout);
            self.allocator.free(result.stderr);
        }

        if (result.term.Exited != 0) {
            std.debug.print("Hook {s} failed: {s}\n", .{ hook_path, result.stderr });
            return error.HookFailed;
        }
    }

    /// Check if a hook exists
    pub fn hookExists(self: *Self, event_name: []const u8) bool {
        const hook_path = std.fmt.allocPrint(self.allocator, "{s}/{s}", .{ self.hooks_dir, event_name }) catch return false;
        defer self.allocator.free(hook_path);

        return std.fs.accessAbsolute(hook_path, .{}) == null;
    }

    /// List all available hooks
    pub fn listHooks(self: *Self) ![][]const u8 {
        var dir = std.fs.openDirAbsolute(self.hooks_dir, .{}) catch {
            // Hooks directory doesn't exist, return empty list
            return self.allocator.alloc([]const u8, 0);
        };
        defer dir.close();

        var hooks = std.ArrayList([]const u8).empty;

        var iter = dir.iterate();
        while (try iter.next()) |entry| {
            if (entry.kind == .file) {
                const hook_name = try self.allocator.dupe(u8, entry.name);
                try hooks.append(self.allocator, hook_name);
            }
        }

        return hooks.toOwnedSlice(self.allocator);
    }
};
