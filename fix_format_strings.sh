#!/bin/bash

# Fix format string issues for clippy compliance

echo "Fixing format string issues..."

# Fix format!("{}", var) to format!("{var}")
find src/ -name "*.rs" -exec sed -i 's/format!("\([^"]*\){}", \([^)]*\))/format!("\1{\2}")/g' {} \;

# Fix format!("{:?}", var) to format!("{var:?}")
find src/ -name "*.rs" -exec sed -i 's/format!("\([^"]*\){:?}", \([^)]*\))/format!("\1{\2:?}")/g' {} \;

echo "Completed format string fixes."