# m(a)yday

## Prerequisites

- [Rust](https://rustup.rs/)
- [Node.js](https://nodejs.org/) 22+
- [pnpm](https://pnpm.io/) 9+

## Dev

```bash
pnpm install

# start everything (vite + server + tauri window)
pnpm dev:desktop

# or web-only in the browser (no tauri window)
cargo run          # rust server on :3001
pnpm dev:web       # vite frontend on :5173
```

In dev mode, Tauri loads the frontend from Vite's dev server (`:5173`) with hot reload.
The desktop app uses Tauri IPC commands instead of the HTTP server -- the `myday-server` crate
is used as a library for its service layer, not as a running web server.

## Build

```bash
pnpm install
pnpm --filter @myday/desktop build
```

The `.dmg` will be in `apps/desktop/src-tauri/target/release/bundle/dmg/`.

## CI/CD

- **CI** (`ci.yml`) -- runs on PRs and pushes to `main`. Checks Rust compilation and builds the desktop app without bundling.
- **Release** (`release.yml`) -- manual trigger via `workflow_dispatch`. Builds signed and notarized `.dmg` files for both `aarch64` and `x86_64`, uploads artifacts, and optionally creates a draft GitHub Release when a version tag is provided.

### Required secrets for release builds

| Secret | Description |
|--------|-------------|
| `APPLE_CERTIFICATE` | Base64-encoded `.p12` of the Developer ID Application cert |
| `APPLE_CERTIFICATE_PASSWORD` | Password for the `.p12` export |
| `APPLE_SIGNING_IDENTITY` | Certificate SHA-1 hash or name |
| `APPLE_API_KEY` | App Store Connect API key ID |
| `APPLE_API_ISSUER` | App Store Connect issuer ID |
| `APPLE_API_KEY_CONTENT` | Contents of the `.p8` API key file |

## License

Apache-2.0 -- see [LICENSE](LICENSE) for details.

Copyright 2026 Elena Gantner
