/// This is a comment with a mispelling: "teh quick brown fox jumps over the lazy dog".
/// Another comment: "The quikc brown fox jumps over the lazy dog".
pub fn hello() void {
    // This is a singel line comment with errors
    const msg = "Hello, world!";
    std.debug.print("{s}\n", .{msg});
}

fn calculate() i32 {
    // TODO: Fix the logik here
    return 42;
}
