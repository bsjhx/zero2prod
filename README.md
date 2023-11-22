# cargo watch
Run check - test - run loop:
```shell
cargo watch -x check -x test -x run
```

Code coverage
```shell
cargo install cargo-tarpaulin
cargo tarpaulin --ignore-test
```

Linting
```shell
rustup component add clippy
cargo clippy
cargo clippy -- -D warnings
```

Formatting
```shell
rustup component add rustfmt 
cargo fmt
cargo fmt -- --check
```

Security Vulnerabilities
```shell
cargo install cargo-audit
cargo audit
```

Removing unused dependencies
```shell
cargo install cargo-udeps
cargo +nightly udeps
```

When you want to see all logs coming out of a certain test case to debug it you can run
```shell
# We are using the `bunyan` CLI to prettify the outputted logs
# The original `bunyan` requires NPM, but you can install a Rust-port with
# `cargo install bunyan`
TEST_LOG=true cargo test health_check_works | bunyan
```
