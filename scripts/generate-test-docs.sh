#!/usr/bin/env bash
set -euo pipefail

echo "Generating rustdoc JSON for test crates..."

# Generate for test-visibility
echo "  - test-visibility"
output=$(cargo +nightly rustdoc -p test-visibility -- -Zunstable-options --output-format json 2>&1)
echo "$output" | grep -v "^warning:" || true

# Generate for test-reexports
echo "  - test-reexports"
output=$(cargo +nightly rustdoc -p test-reexports -- -Zunstable-options --output-format json 2>&1)
echo "$output" | grep -v "^warning:" || true

echo "Rustdoc JSON generation complete"
