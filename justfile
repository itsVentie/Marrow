frontend := "apps/Marrow"

install:
    pnpm --dir {{frontend}} install --frozen-lockfile

dev:
    pnpm --dir {{frontend}} tauri dev

build:
    pnpm --dir {{frontend}} tauri build

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

check:
    cargo check --workspace

clippy:
    cargo clippy --workspace --all-targets -- -D warnings

test:
    cargo test --workspace

frontend-typecheck:
    pnpm --dir {{frontend}} exec tsc --noEmit

frontend-build:
    pnpm --dir {{frontend}} build

check-all: fmt-check check clippy frontend-typecheck

ci: fmt-check clippy test frontend-typecheck frontend-build

clean:
    cargo clean