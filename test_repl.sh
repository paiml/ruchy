#!/bin/bash
(echo "let x = 10"; sleep 0.1; echo "x"; sleep 0.1; echo ":quit") | ./target/release/ruchy 2>&1
