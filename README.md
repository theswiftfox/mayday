# m(a)yday

## Dev

```bash
pnpm install
# start everything (web + server)
pnpm dev
# or individually
cargo run              # rust server on :3001
pnpm dev:web           # vite frontend on :5173
```

## Build Desktop (Tauri)

```bash
pnpm install
pnpm --filter @myday/desktop build
```

The `.dmg` / `.msi` / `.AppImage` will be in `apps/desktop/src-tauri/target/release/bundle/`.
