# Civis — Agent Commands

## Build
```bash
cargo build --release
```

## Test
```bash
cargo test --all
```

## Lint
```bash
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## Full CI
```bash
task ci
```
