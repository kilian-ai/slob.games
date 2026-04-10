# slob.games — Agent Instructions

> **Fork of traits.build** — WASM-only browser runtime. No native binary, no server.
> Static SPA hosted on GitHub Pages.
>
> - **Repository:** https://github.com/kilian-ai/slob.games
> - **Homepage:** https://slob.games/

---

## Project Overview

**slob.games** is a WASM-only fork of traits.build. The native Rust binary, Fly.io backend, and all server-side infrastructure have been removed. The trait kernel compiles exclusively to `wasm32-unknown-unknown` and runs entirely in the browser.

- **WASM kernel** — runs in the browser, compiled via `wasm-pack`, ~40 WASM-compiled traits
- **Static SPA** — `index.html` with hash-based routing, served from GitHub Pages
- **Dispatch**: WASM local only (no helper, no relay, no server REST tier)
- **Default page**: Canvas (`/` → `www.canvas`)
- **No native binary, no CLI, no MCP server, no Fly.io**

---

## Key Differences from traits.build

| Aspect | traits.build | slob.games |
|--------|-------------|------------|
| Binary | `./target/release/traits` | **None** |
| `Cargo.toml` | Full workspace (~10 members) | `logic` + `wasm` only |
| `build.rs` | Root-level codegen for native dispatch | **Deleted** |
| `src/main.rs` | Binary entry point | **Deleted** |
| `build.sh` | cargo + wasm-pack + static gen | **wasm-pack + static gen only** |
| Fly.io | Optional backend | **Not used** |
| CI deploy | Builds WASM on CI | **Ships pre-built `index.html`** |
| Default route | `www.traits.build` (homepage) | `www.canvas` |
| Domain | `www.traits.build` | `slob.games` |
| DNS | Any | **Cloudflare, proxy DISABLED (grey cloud)** |

---

## Directory Structure

```
slob.games/
├── build.sh              # WASM-only pipeline: wasm-pack + runtime gen + standalone HTML
├── Cargo.toml            # Workspace: only traits/kernel/logic + traits/kernel/wasm
├── Cargo.lock
├── index.html            # DEPLOYED SPA (pre-built standalone, commit this after build.sh)
├── CNAME                 # GitHub Pages: slob.games
├── Dockerfile            # Present but unused (no native binary)
├── fly.toml              # Present but unused (no server)
├── traits.toml           # Present but unused at runtime (WASM reads no config file)
├── scripts/              # Build helpers (some unused, kept for reference)
├── traits/
│   ├── kernel/           # 5 modules (Layer 2 native modules removed)
│   │   ├── call/         # Inter-trait dispatch (wasm = true)
│   │   ├── cli/          # Portable CLI processor (wasm_callable = false)
│   │   ├── logic/        # Shared library: registry, types, platform abstraction
│   │   ├── types/        # TraitValue, TraitType, type coercion (wasm = true)
│   │   └── wasm/         # WASM browser kernel (wasm-pack build target)
│   ├── sys/              # System traits (only wasm=true ones are compiled in)
│   │   ├── audio/        # Audio playback
│   │   ├── bindings/     # Runtime interface binding management
│   │   ├── call/         # Outbound HTTP/REST API calls
│   │   ├── canvas/       # Persistent canvas / scratchpad
│   │   ├── chat/         # Chat session management
│   │   ├── chat_delete/  # Delete chat sessions
│   │   ├── chat_learnings/  # Chat learning extraction
│   │   ├── chat_protocols/  # Chat protocol reader
│   │   ├── chat_workspaces/ # VS Code workspace scanner
│   │   ├── checksum/     # SHA-256 hashing (WASM build, no dylib)
│   │   ├── cli/          # CLI bootstrap + wasm/ sub-trait
│   │   ├── config/       # Persistent key-value config
│   │   ├── docs/         # Documentation generation
│   │   ├── dylib_loader/ # Runtime cdylib loading (native-only, not compiled in WASM)
│   │   ├── echo/         # Echo test trait
│   │   ├── list/         # List all traits
│   │   ├── llm/          # Unified LLM inference router
│   │   ├── mcp/          # MCP server (native-only, not compiled in WASM)
│   │   ├── openapi/      # OpenAPI spec generation
│   │   ├── ps/           # Background task list
│   │   ├── registry/     # Registry read API
│   │   ├── release/      # Release pipeline (native-only)
│   │   ├── reload/       # Registry hot-reload
│   │   ├── secrets/      # AES-256-GCM encrypted secrets store
│   │   ├── serve/        # HTTP server (native-only, not compiled in WASM)
│   │   ├── snapshot/     # Snapshot trait version
│   │   ├── spa/          # SPA session control
│   │   ├── test_runner/  # .features.json test runner
│   │   ├── version/      # YYMMDD version string
│   │   ├── vfs/          # Virtual filesystem
│   │   └── voice/        # Voice I/O chat service
│   ├── www/              # Web/SPA traits
│   │   ├── admin/        # Admin dashboard (spa/ variant for browser-only)
│   │   ├── canvas/       # Canvas page (DEFAULT ROUTE)
│   │   ├── chat_logs/    # Chat history viewer
│   │   ├── docs/         # Documentation site
│   │   ├── homepage/     # Homepage (if present)
│   │   ├── llm/          # LLM UI
│   │   ├── llm_test/     # LLM inference tester
│   │   ├── local/        # Helper/install shell scripts
│   │   ├── playground/   # Interactive trait playground
│   │   ├── sdk/          # TypeScript SDK (traits.js — single source)
│   │   ├── seo/          # SEO traits
│   │   ├── splats/       # URL splat handling
│   │   ├── static/       # SPA shell source (index.html, wasm-runtime.js, etc.)
│   │   ├── terminal/     # xterm.js WASM terminal
│   │   ├── traits/       # www.traits.build homepage (dylib — present but adapted)
│   │   └── wasm/         # WASM internals page
│   └── llm/              # LLM provider traits
│       ├── agent/        # LLM agent loop
│       ├── prompt/       # llm/prompt interface + openai + webllm
│       ├── voice/        # TTS / STT
│       └── skills/       # (spotify, etc.)
└── .github/
    ├── agents/
    │   ├── slob.games.agent.md   # THIS FILE — slob.games-specific agent instructions
    │   └── traits.build.agent.md # Upstream reference (kept for context)
    └── workflows/
        ├── deploy-docs.yml       # Pages deploy (copies pre-built index.html)
        └── test.yml              # WASM build check only
```

**Removed from traits.build:**
- `src/main.rs`, `build.rs`, `sha256.rs`, `clippy.toml`, `trait_config.toml`
- `traits/browser/` (fetch, interact, screenshot — Playwright traits)
- `traits/kernel/config/`, `dispatcher/`, `globals/`, `main/`, `plugin_api/`, `registry/`
- `traits/sys/info/`, `traits/sys/shell/`
- `traits/sys/checksum/Cargo.toml` (was cdylib — checksum now compiled as builtin only)
- `.github/workflows/release.yml` (native binary release workflow)

---

## Build System

### build.sh (WASM-Only Pipeline)

```bash
bash build.sh   # the only build command needed
```

Steps:
1. **Version bump** — updates `traits/sys/version/version.trait.toml` (OS-portable: uses `sed -i ''` on macOS, `sed -i` on Linux)
2. **`wasm-pack build`** — compiles `traits/kernel/wasm/` → `traits/kernel/wasm/pkg/` (`traits_wasm.js` + `traits_wasm_bg.wasm`)
3. **`wasm-runtime.js`** — Python embeds WASM binary as base64 into IIFE-wrapped classic script
4. **`traits-worker.js`** — Python generates Web Worker variant with same WASM binary
5. **`terminal-runtime.js`** — strips ESM export from `terminal.js`, inlines CSS
6. **`sdk-runtime.js`** — IIFE-wraps `traits.js` ES module into classic script
7. **`index.standalone.html`** — inlines all 4 runtimes into `traits/www/static/index.html`
8. **`index.html`** — copies standalone → repo root (the GitHub Pages entry point)

> **No `cargo build --release`** — there is no native binary target in this fork.

### wasm/build.rs (WASM Code Generation)

Located at `traits/kernel/wasm/build.rs`. Scans `traits/` for `.trait.toml` files with `wasm = true` and generates:
- `wasm_compiled_traits.rs` — module declarations + dispatch match table + `WASM_CALLABLE` const

This is the **only** build.rs in the project (root `build.rs` was deleted).

### Cargo.toml (Workspace)

```toml
[workspace]
members = [
  "traits/kernel/logic",
  "traits/kernel/wasm",
]
resolver = "2"
```

No binary members, no plugin_api, no cdylib members.

---

## SPA Route Table

```javascript
'/':            'www.canvas'       // DEFAULT — canvas scratchpad
'/playground':  'www.playground'   // Interactive trait playground
'/settings':    'www.admin.spa'    // Settings / admin (SPA mode)
'/terminal':    ... (terminal page)
```

**Note:** The default route is `www.canvas`, not `www.traits.build` as in the upstream.

---

## Deployment

### GitHub Pages (Only deploy target)

- **Domain:** `slob.games`
- **CNAME file:** `slob.games` (at repo root)
- **Source:** `index.html` at repo root — **pre-built locally and committed**
- **Routing:** Hash-based (`#/playground`, `#/settings`, etc.) — `isLocal = true` always in standalone
- **CI workflow:** `.github/workflows/deploy-docs.yml` — just copies pre-built `index.html` + CNAME to `_site/`, no compilation on CI

**Deploy workflow:**
```bash
bash build.sh                          # 1. Build WASM + generate index.html locally
git add -A && git commit -m "..."      # 2. Commit (includes index.html + pkg changes)
git push                               # 3. Push → GitHub Actions deploys in ~10s
```

**CI does NOT run wasm-pack** — the `traits/kernel/wasm/pkg/` directory is gitignored, but `index.html` (which has the WASM binary inlined as base64) is committed. CI simply ships what's already in the repo.

### DNS (Cloudflare)

Domain is managed through Cloudflare. **Critical:** GitHub Pages requires the A records to be set to **DNS only (grey cloud)** — the orange proxy cloud MUST be disabled or GitHub cannot verify domain ownership.

Required DNS records:
| Type | Name | Content | Proxy |
|------|------|---------|-------|
| A | `@` | `185.199.108.153` | DNS only |
| A | `@` | `185.199.109.153` | DNS only |
| A | `@` | `185.199.110.153` | DNS only |
| A | `@` | `185.199.111.153` | DNS only |
| CNAME | `www` | `kilian-ai.github.io` | DNS only |

Verify: `dig slob.games A +short` — must show `185.199.x.x` IPs, not Cloudflare proxy IPs.

### No Fly.io

There is no server backend. `fly.toml` and `Dockerfile` are present in the repo but unused. Do not deploy to Fly.io for this project.

---

## Dispatch Flow

slob.games uses **WASM-only dispatch**:

```
Browser call → WASM kernel (traits_wasm_bg.wasm)
```

No helper probe, no relay, no server REST tier. All ~40 compiled traits run in the browser via the embedded WASM binary.

The SDK (`traits.js` / `window._traitsSDK`) still has the cascade logic inherited from traits.build, but without a helper or server, only WASM dispatch is active.

---

## Kernel Architecture (WASM-Only)

Only **Layer 0** and **Layer 1** kernel modules remain:

```
Layer 0: Shared Library
  kernel.logic    — types, registry model, platform abstraction (Cargo workspace member)
  kernel.wasm     — WASM browser kernel (wasm-pack compilation target)

Layer 1: Portable Traits (wasm = true)
  kernel.call     — cross-trait dispatch by dot-path
  kernel.cli      — portable CLI processor (wasm_callable = false)
  kernel.types    — type system introspection
```

**Layer 2 (native infrastructure) has been removed entirely:**
- `kernel.config`, `kernel.dispatcher`, `kernel.globals`, `kernel.main`, `kernel.plugin_api`, `kernel.registry` — all deleted

**No dylib traits** — `plugin_api` crate is gone. All traits are compiled as builtins into the WASM kernel. The `sys.checksum` dir no longer has a `Cargo.toml` (no cdylib build).

---

## What IS and ISN'T in this project

**Present:**
| Component | Details |
|-----------|---------|
| WASM kernel | Browser runtime, ~40 traits compiled to wasm32 |
| Static SPA | `index.html` — self-contained, all JS/WASM inlined |
| Canvas page | Default route, persistent scratchpad |
| Playground | Interactive trait testing |
| LLM integration | `llm.prompt.webllm` (in-browser WebLLM), `llm.prompt.openai` (remote) |
| Voice I/O | `sys.voice`, `llm.voice.speak`, `llm.voice.listen` |
| Terminal UI | `www.terminal` — xterm.js + WASM CLI |
| Secrets store | `sys.secrets` — AES-256-GCM encrypted, in-browser only |
| SDK | `traits.js` → `sdk-runtime.js` (IIFE classic script) |

**Not present (removed from traits.build):**
| Component | Reason |
|-----------|--------|
| Native binary | WASM-only fork |
| CLI (`traits` command) | No binary |
| MCP server | Requires native binary |
| HTTP server / Fly.io backend | No server |
| Dylib plugins | `plugin_api` removed |
| Browser automation traits | `traits/browser/` removed |
| `sys.info`, `sys.shell` | Native-only, removed |
| GitHub Actions WASM build | CI ships pre-built artifact |
| `release.yml` workflow | No native binary releases |

---

## Platform Abstraction Layer

Unchanged from traits.build. Path: `traits/kernel/logic/src/platform/`

WASM initialization (`kernel/wasm/src/lib.rs` → `init()`):
- `dispatch` → `wasm_traits::dispatch`
- `registry_*` → `get_registry()` (WasmRegistry)
- `config_get` → returns default (no config file in browser)
- `secret_get` → `wasm_secrets::get_secret` (localStorage-backed)
- `make_vfs` → `make_wasm_vfs` (embedded trait TOMLs)
- `background_tasks` → `wasm_background_tasks`

---

## Type System

```rust
TraitType: int, float, string, bool, bytes, null, any, handle, list<T>, map<K,V>, T?
TraitValue: Null, Bool, Int(i64), Float(f64), String, List, Map, Bytes
```

---

## Trait .trait.toml Template

```toml
[trait]
description = "Short description"
version = "v260408"
author = "system"
tags = ["namespace", "category"]

[signature]
params = [
  { name = "param1", type = "string", description = "desc", required = true },
]

[signature.returns]
type = "string"
description = "Return description"

[implementation]
language = "rust"
source = "builtin"
entry = "function_name"
wasm = true              # REQUIRED for traits to be accessible in the browser

[requires]
dep = "namespace/interface"

[bindings]
dep = "namespace.concrete_trait"
```

**Important:** Every trait you want accessible in the browser MUST have `wasm = true` in `[implementation]`. Traits without it exist in the registry (visible in `list`) but cannot be called.

---

## Canvas Bridge Script

Every game loaded into the `#phone-viewport` iframe gets a **bridge `<script>`** injected into its `<head>`. This bridge runs inside the iframe's execution context and provides the game-to-parent communication layer.

**Location:** `traits/www/canvas/canvas.rs` — the `const BRIDGE` template literal (~line 600).

**Injection mechanism:** The `renderCanvas(content)` function prepends BRIDGE into the game HTML before setting `phoneViewport.srcdoc`:
- If the content has `<head>`, BRIDGE is inserted after the opening `<head>` tag.
- If the content has `<html>` but no `<head>`, a `<head>` wrapper is created for BRIDGE.
- If the content is a bare fragment, a full HTML skeleton is generated with BRIDGE included.

### What the bridge provides:

1. **`window.traits` SDK** — Proxy object that delegates to `window.parent._traitsSDK`:
   ```javascript
   window.traits.call(path, args)    // call any trait
   window.traits.list(namespace?)    // sys.list
   window.traits.info(path)          // sys.info
   window.traits.echo(text)          // sys.echo
   window.traits.canvas(action, content?) // sys.canvas
   window.traits.audio(action, ...rest)   // sys.audio
   ```

2. **`document.querySelector` patch** — Strips `#phone-viewport` and `#canvas-container` prefixes from selectors, so code written for the outer document works inside the iframe.

3. **Console capture** — Wraps `console.log/warn/error` to forward messages to the parent via `postMessage`:
   ```javascript
   window.parent.postMessage({type:'canvas-console', level:'log'|'warn'|'error', message:string}, '*')
   ```
   The parent stores these in `window.__canvasGameLogs` for voice agent context.

4. **Uncaught error capture** — Listens for `window.error` events and forwards them as `canvas-console` error messages.

5. **Two-finger tap (mobile chrome toggle)** — Listens for `touchstart` with 2+ touches and posts:
   ```javascript
   window.parent.postMessage({type:'canvas-toggle-chrome'}, '*')
   ```
   The parent uses this to show/hide the shell-nav and FAB on mobile without intercepting single taps.

### postMessage protocol (iframe → parent):

| `type` | Fields | Purpose |
|--------|--------|---------|
| `canvas-console` | `level`, `message` | Forward game console output to parent |
| `canvas-toggle-chrome` | — | Two-finger tap: toggle mobile UI chrome |

### Extending the bridge:

- Edit the `const BRIDGE` template literal in `canvas.rs`.
- Use `window.parent.postMessage({type:'canvas-*', ...}, '*')` for new iframe→parent messages.
- Add the corresponding `window.addEventListener('message', ...)` handler in the parent script section of `canvas.rs`.
- Keep the bridge ES5-compatible (use `var`, `function`, no arrow functions) — some games may set strict CSP or run in older WebView contexts.
- The bridge must be a **self-executing IIFE** wrapped in `<script>...</script>` tags.
- The closing tag must be escaped as `<\/script>` since it lives inside a JS template literal.

---

## Conventions

- **All trait code is Rust.** Frontend JS is in `terminal.js`, `traits.js`, etc. served as static assets.
- **No native-only patterns.** Do not add `#[cfg(not(target_arch = "wasm32"))]` paths that call native APIs — all code must compile for `wasm32-unknown-unknown`.
- **WASM-compatible imports only.** No `std::fs`, `std::net`, `tokio`, `actix`, `libloading`, `std::process`. Use `wasm_bindgen`, `js_sys`, `web_sys` for browser APIs.
- **`source = "builtin"`** for all traits — there are no dylib traits in this fork.
- **Trait files live in `traits/{namespace}/{name}/`** — each directory has `name.trait.toml` + `name.rs` + `name.features.json`
- **wasm/build.rs auto-discovers** everything — no manual module registration needed.
- **After modifying traits:** run `bash build.sh` and commit the resulting `index.html`.
- **GitHub Pages deploy:** `git push origin main` — CI copies the committed `index.html` to Pages in ~10s.
- **Canvas is the default page** — route `/` maps to `www.canvas`.
- **No CLI output formatting** (no `.cli.rs` companion files needed — WASM CLI only).

---

## AGENT RULES

- **Always run `bash build.sh`** after making changes to Rust traits or JS/SDK files.
- **Always commit `index.html`** after running `build.sh` — this is the deployed artifact.
- **Always run `git add -A`** to capture version bumps in `.trait.toml` files.
- **Always push to `main`** to deploy — CI takes ~10s, no manual steps.
- **Never add native-only code** — all Rust must compile to `wasm32-unknown-unknown`.
- **Never add a root `build.rs`** — the only build.rs is in `traits/kernel/wasm/`.
- **Never add new Cargo workspace members** beyond `kernel/logic` and `kernel/wasm`.
- **Always create `features.json`** for any new trait you add.
- **Always set `wasm = true`** in `[implementation]` for any trait that should be callable in the browser.
- **Keep DNS proxy disabled** on Cloudflare A records — orange cloud breaks GitHub Pages cert issuance.
- **Update this file** (`.github/agents/slob.games.agent.md`) when project structure, conventions, or deployment process changes.
- **Do not deploy to Fly.io** — slob.games is static-only.
- **Store memory files** in `.github/memories/`.
