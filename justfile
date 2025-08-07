# Justfile for Orama Core Client Rust
# Install just: https://github.com/casey/just

# Default recipe
default:
    @just --list

# Build the project
build:
    cargo build

# Build in release mode
release:
    cargo build --release

# Run all tests
test:
    cargo test

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# Run integration tests (requires environment setup)
test-integration:
    TEST_INTEGRATION=1 cargo test --test integration_tests

# Run a specific test
test-one TEST:
    cargo test {{TEST}} -- --nocapture

# Run clippy linting
lint:
    cargo clippy -- -D warnings

# Format code
fmt:
    cargo fmt

# Check formatting without changing files
fmt-check:
    cargo fmt -- --check

# Generate documentation
docs:
    cargo doc --no-deps --document-private-items --open

# Clean build artifacts
clean:
    cargo clean

# Check for security vulnerabilities
audit:
    cargo audit

# Run all quality checks
check: fmt-check lint test
    @echo "All checks passed!"

# Run examples
example EXAMPLE:
    cargo run --example {{EXAMPLE}}

# Run all examples
examples:
    @echo "Running basic_search example..."
    @cargo check --example basic_search
    @echo "Running document_management example..."
    @cargo check --example document_management
    @echo "Running ai_session example..."
    @cargo check --example ai_session
    @echo "Running server_app example..."
    @cargo check --example server_app
    @echo "All examples compile successfully!"

# Watch for changes and run tests
watch:
    cargo watch -x test

# Watch for changes and run a specific test
watch-test TEST:
    cargo watch -x "test {{TEST}}"

# Benchmark (if you add benchmarks later)
bench:
    cargo bench

# Install development dependencies
dev-setup:
    cargo install cargo-watch cargo-audit just

# Publish to crates.io (dry run)
publish-dry:
    cargo publish --dry-run

# Publish to crates.io
publish:
    cargo publish

# Update dependencies
update:
    cargo update

# Show outdated dependencies
outdated:
    cargo outdated

# Generate code coverage report (requires cargo-tarpaulin)
coverage:
    cargo tarpaulin --out Html --output-dir coverage

# Setup for development
setup: dev-setup
    @echo "Development environment setup complete!"
    @echo "Available commands:"
    @just --list