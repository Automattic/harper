// Errorz
const std = @import("std");

pub fn main() !void {
    try std.fs.File.stdout().writeAll("hello world!\n");
}
