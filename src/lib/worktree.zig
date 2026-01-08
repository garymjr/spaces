const std = @import("std");
const HookRunner = @import("hooks.zig").HookRunner;

pub const Worktree = struct {
    name: []const u8,
    path: []const u8,
    branch: []const u8,
    commit: []const u8,
    is_bare: bool = false,
    is_detached: bool = false,

    pub fn deinit(self: *const Worktree, allocator: std.mem.Allocator) void {
        allocator.free(self.name);
        allocator.free(self.path);
        allocator.free(self.branch);
        allocator.free(self.commit);
    }
};

pub const WorktreeManager = struct {
    allocator: std.mem.Allocator,
    repo_path: []const u8,
    hooks: HookRunner,

    const Self = @This();

    pub fn init(allocator: std.mem.Allocator, repo_path: []const u8) !Self {
        return Self{
            .allocator = allocator,
            .repo_path = try allocator.dupe(u8, repo_path),
            .hooks = try HookRunner.init(allocator, repo_path),
        };
    }

    pub fn deinit(self: *Self) void {
        self.allocator.free(self.repo_path);
        self.hooks.deinit();
    }

    /// Find the git repository path from current directory
    pub fn findRepoPath(allocator: std.mem.Allocator) ![]const u8 {
        var cwd = std.fs.cwd();
        var path_buf: [std.fs.max_path_bytes]u8 = undefined;
        const cwd_path = try cwd.realpath(".", &path_buf);

        var search_path = try allocator.dupe(u8, cwd_path);
        defer allocator.free(search_path);

        while (true) {
            // Check if .git exists
            const git_path = try std.fmt.allocPrint(allocator, "{s}/.git", .{search_path});
            defer allocator.free(git_path);

            if (std.fs.accessAbsolute(git_path, .{ .mode = .read_only })) |_| {
                return try allocator.dupe(u8, search_path);
            } else |_| {
                // Go up one directory
                const last_slash = std.mem.lastIndexOfScalar(u8, search_path, '/');
                if (last_slash == null or last_slash.? == 0) {
                    return error.GitRepoNotFound;
                }
                search_path = search_path[0..last_slash.?];
            }
        }
    }

    /// List all worktrees in the repository
    pub fn list(self: *Self) ![]Worktree {
        const git_worktree_list = try self.runGit(&.{"worktree", "list", "--porcelain"});
        defer self.allocator.free(git_worktree_list);

        var worktrees = std.ArrayList(Worktree).empty;
        defer {
            for (worktrees.items) |*w| w.deinit(self.allocator);
            worktrees.deinit(self.allocator);
        }

        var lines = std.mem.splitScalar(u8, git_worktree_list, '\n');
        var current: ?*Worktree = null;

        while (lines.next()) |line| {
            if (line.len == 0) {
                current = null;
                continue;
            }

            if (std.mem.startsWith(u8, line, "worktree ")) {
                const path = line["worktree ".len..];
                try worktrees.append(self.allocator, .{
                    .name = "",
                    .path = try self.allocator.dupe(u8, path),
                    .branch = "",
                    .commit = "",
                });
                current = &worktrees.items[worktrees.items.len - 1];
            } else if (std.mem.startsWith(u8, line, "branch ")) {
                if (current) |w| {
                    var branch = line["branch ".len..];
                    // Strip refs/heads/ if present
                    if (std.mem.startsWith(u8, branch, "refs/heads/")) {
                        branch = branch["refs/heads/".len..];
                    }
                    w.branch = try self.allocator.dupe(u8, branch);
                }
            } else if (std.mem.startsWith(u8, line, "HEAD ")) {
                if (current) |w| {
                    w.commit = try self.allocator.dupe(u8, line["HEAD ".len..]);
                }
            } else if (std.mem.eql(u8, line, "detached")) {
                if (current) |w| {
                    w.is_detached = true;
                }
            }
        }

        // Extract name from path for worktrees
        for (worktrees.items) |*w| {
            const last_slash = std.mem.lastIndexOfScalar(u8, w.path, '/');
            if (last_slash) |idx| {
                w.name = try self.allocator.dupe(u8, w.path[idx + 1 ..]);
            }
        }

        return worktrees.toOwnedSlice(self.allocator);
    }

    /// Create a new worktree
    /// If create_new_branch is true, creates a new branch (branch name defaults to worktree name)
    /// If create_new_branch is false, checks out the existing branch specified in `branch`
    ///
    /// Examples:
    ///   create("feat-x", null, true)  -> git worktree add .spaces/worktrees/feat-x -b feat-x
    ///   create("wt-1", "main", false) -> git worktree add .spaces/worktrees/wt-1 main
    ///   create("wt-1", "feat-y", true) -> git worktree add .spaces/worktrees/wt-1 -b feat-y
    pub fn create(self: *Self, name: []const u8, branch: ?[]const u8, create_new_branch: bool) !Worktree {
        // Run pre-create hook
        try self.hooks.run("pre-create", .{ .name = name, .branch = branch });

        const worktree_path = try std.fmt.allocPrint(self.allocator, "{s}/.spaces/worktrees/{s}", .{ self.repo_path, name });

        // Create the worktree
        var args = std.ArrayList([]const u8).empty;
        defer args.deinit(self.allocator);

        try args.append(self.allocator, "worktree");
        try args.append(self.allocator, "add");
        try args.append(self.allocator, worktree_path);

        if (create_new_branch) {
            // Create new branch: git worktree add <path> -b <branch>
            const branch_name = branch orelse name;
            try args.append(self.allocator, "-b");
            try args.append(self.allocator, branch_name);
        } else {
            // Checkout existing branch: git worktree add <path> <branch>
            if (branch) |b| {
                try args.append(self.allocator, b);
            } else {
                return error.BranchRequired;
            }
        }

        const result = try self.runGit(args.items);
        defer self.allocator.free(result);

        const new_worktree = try self.getByName(name);

        // Run post-create hook
        try self.hooks.run("post-create", .{ .name = name, .worktree = new_worktree });

        return new_worktree;
    }

    /// Remove a worktree
    pub fn remove(self: *Self, name: []const u8) !void {
        const worktree = try self.getByName(name);

        // Run pre-remove hook
        try self.hooks.run("pre-remove", .{ .name = name, .worktree = worktree });

        const result = try self.runGit(&.{ "worktree", "remove", worktree.path });
        defer self.allocator.free(result);

        // Run post-remove hook
        try self.hooks.run("post-remove", .{ .name = name });
    }

    /// Get worktree by name
    pub fn getByName(self: *Self, name: []const u8) !Worktree {
        const worktrees = try self.list();
        defer {
            for (worktrees) |*w| w.deinit(self.allocator);
            self.allocator.free(worktrees);
        }

        for (worktrees) |w| {
            if (std.mem.eql(u8, w.name, name)) {
                return Worktree{
                    .name = try self.allocator.dupe(u8, w.name),
                    .path = try self.allocator.dupe(u8, w.path),
                    .branch = try self.allocator.dupe(u8, w.branch),
                    .commit = try self.allocator.dupe(u8, w.commit),
                    .is_bare = w.is_bare,
                    .is_detached = w.is_detached,
                };
            }
        }

        return error.WorktreeNotFound;
    }

    /// Get the path to a worktree
    pub fn getWorktreePath(self: *Self, name: []const u8) ![]const u8 {
        const worktree = try self.getByName(name);
        defer worktree.deinit(self.allocator);
        return try self.allocator.dupe(u8, worktree.path);
    }

    /// Enter a worktree (run hooks and return path)
    pub fn enter(self: *Self, name: []const u8) ![]const u8 {
        const worktree = try self.getByName(name);

        // Run pre-enter hook
        try self.hooks.run("pre-enter", .{ .name = name, .worktree = worktree });

        // Run post-enter hook
        try self.hooks.run("post-enter", .{ .name = name, .worktree = worktree });

        return try self.allocator.dupe(u8, worktree.path);
    }

    /// Run a git command and return stdout
    fn runGit(self: *Self, args: []const []const u8) ![]const u8 {
        const full_args = try self.allocator.alloc([]const u8, args.len + 3);
        defer self.allocator.free(full_args);

        full_args[0] = "git";
        full_args[1] = "-C";
        full_args[2] = self.repo_path;
        std.mem.copyForwards([]const u8, full_args[3..], args);

        const result = try std.process.Child.run(.{
            .allocator = self.allocator,
            .argv = full_args,
        });

        defer {
            self.allocator.free(result.stdout);
            self.allocator.free(result.stderr);
        }

        if (result.term.Exited != 0) {
            std.debug.print("git failed: {s}\n", .{result.stderr});
            return error.GitCommandFailed;
        }

        return self.allocator.dupe(u8, result.stdout);
    }
};
