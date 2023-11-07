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
