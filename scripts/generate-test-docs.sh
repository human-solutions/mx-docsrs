#!/usr/bin/env bash
set -euo pipefail

echo "Generating rustdoc JSON for test crates..."

# Generate for test-visibility
echo "  - test-visibility"
cargo +nightly rustdoc -p test-visibility -- -Zunstable-options --output-format json 2>&1 | grep -v "^warning:" || true

# Generate for test-reexports
echo "  - test-reexports"
cargo +nightly rustdoc -p test-reexports -- -Zunstable-options --output-format json 2>&1 | grep -v "^warning:" || true

echo "Rustdoc JSON generation complete"
