default:
    @just --list

fmt:
    CARGO_HOME="${CARGO_HOME:-$PWD/.cargo}" cargo fmt

clippy:
    CARGO_HOME="${CARGO_HOME:-$PWD/.cargo}" cargo clippy -- -D warnings

test:
    CARGO_HOME="${CARGO_HOME:-$PWD/.cargo}" cargo test

check: fmt clippy test
