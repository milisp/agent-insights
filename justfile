dev:
  cargo run --manifest-path apps/backend/Cargo.toml &
  cd apps/frontend && bun run dev
