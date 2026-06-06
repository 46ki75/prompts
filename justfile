# Build the static site into ./dist from prompts under ./prompts
build:
    cargo run --release --package builder -- prompts

fmt-check:
    cargo fmt --all -- --check

lint:
    cargo clippy --workspace --all-targets -- -D warnings

lint-md:
    pnpm lint

test:
    cargo test --workspace

# Live tests — hit the real GitHub Pages distribution. Not part of `ci`.
test-live:
    cargo test --workspace -- --ignored

# Instrumented unit / hermetic test run (no report yet)
test-cov:
    cargo llvm-cov --no-report --workspace

# AI-friendly: per-file table (drop 100% files) + uncovered line numbers
coverage: test-cov
    cargo llvm-cov report --show-missing-lines --color=always 2>&1 | grep -v " 100.00%"

# Local HTML drilldown
coverage-html: test-cov
    cargo llvm-cov report --html --open

# CI / Codecov upload
coverage-ci: test-cov
    cargo llvm-cov report --lcov --output-path lcov.info

ci: fmt-check lint test

ci-live: fmt-check lint test test-live
