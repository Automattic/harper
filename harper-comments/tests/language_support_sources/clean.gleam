//// Clean comments.

import gleam/io

/// Print a friendly message.
pub fn main() {
  // Keep the output simple.
  io.println("Errorz should be ignored inside strings")
}

/// Add two numbers.
pub fn add(left: Int, right: Int) -> Int {
  left + right
}
