#!/bin/bash
# Test REPL function persistence

echo "Testing REPL function persistence..."

# Test function definition and call
echo -e 'fun add(a: i64, b: i64) -> i64 { a + b }\nadd(5, 3)\n:quit' | target/debug/ruchy

echo "Done!"