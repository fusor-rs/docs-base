fusor := env_var_or_default("FUSOR_BIN", "fusor")

default:
    @just --list

check:
    cargo fmt --all -- --check
    cargo test --workspace --locked
    cargo clippy --workspace --all-targets --locked -- -D warnings
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked

msrv:
    cargo +1.85 test --workspace --locked

book:
    {{fusor}} build -p docs-base-example --debug --locked

dev:
    {{fusor}} dev -p docs-base-example

preview:
    {{fusor}} preview examples/book/dist --port 4173

browser: book
    node scripts/browser.mjs

setup-browser:
    rustup target add wasm32-unknown-unknown
    cargo install fusor-cli --version 0.1.4 --locked
    cargo install wasm-bindgen-cli --version 0.2.117 --locked
    npm ci
    npx playwright install chromium

packages:
    cargo package -p docs-base-build --locked
    cargo package -p docs-base --locked
