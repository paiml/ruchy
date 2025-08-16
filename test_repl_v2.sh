#!/bin/bash
echo "Testing ReplV2 with variable persistence..."
cat << 'INPUT' | timeout 5 ./target/release/ruchy
let x = 10
x
let y = 20
x + y
[1, 2, 3]
"Hello, " + "World"
:quit
INPUT
