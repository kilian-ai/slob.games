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

## Game SDK (In-Iframe Bridge)

Every game loaded into the `#phone-viewport` iframe gets a **bridge `<script>`** injected into its `<head>`. The bridge runs inside the iframe and exposes `window.traits` — the game SDK.

**Source:** `traits/www/canvas/canvas.rs` — `const BRIDGE` template literal (~line 601).

**Injection:** `renderCanvas(content)` prepends BRIDGE into game HTML before setting `phoneViewport.srcdoc`:
- Content with `<head>` → inserted after opening `<head>` tag
- Content with `<html>` but no `<head>` → `<head>` wrapper created
- Bare fragment → full HTML skeleton generated with BRIDGE included

### `window.traits` API Reference

All methods are available to game code running inside the iframe. The bridge proxies calls to `window.parent._traitsSDK`.

```javascript
// ── General ──
window.traits.call(path, args)        // Call any trait by dot-path (e.g. 'sys.echo')
window.traits.list(namespace?)        // List traits (optionally filtered by namespace)
window.traits.info(path)              // Get trait metadata
window.traits.echo(text)              // Echo test

// ── Canvas ──
window.traits.canvas(action, content?) // Canvas operations (read/write scratchpad)

// ── Audio (WebAudio API) ──
window.traits.audio(action, ...args)  // See Audio section below

// ── High Score ──
window.traits.score(value?)           // Submit score (if value given) or read synced high score
                                       // Returns: number (current synced high score for this game)

// ── Pause/Resume Hooks (optional) ──
window.traits.onPause = function() {} // Called when game is paused by two-finger gesture
window.traits.onResume = function() {} // Called when game is resumed
```

### Audio API (`window.traits.audio`)

Games can generate sounds via the WebAudio API. The `sys.audio` trait returns action descriptors that the SDK JS bridge executes.

```javascript
// Tone — single oscillator note
traits.audio('tone', freq, duration, waveform, volume)
// freq: 20-20000 Hz (default 440)
// duration: 0.01-30 seconds (default 0.5)
// waveform: 'sine'|'square'|'sawtooth'|'triangle' (default 'sine')
// volume: 0.0-1.0 (default 0.3)

// Sequence — play notes in order
traits.audio('sequence', notes, tempo, volume)
// notes: array of {freq, dur, wave} objects
// tempo: 20-300 BPM (default 120)

// Drum pattern
traits.audio('drum', pattern, bpm, loops, volume)
// pattern: string like 'k..s..k.ks..s...' (k=kick, s=snare, h=hihat, .=rest)
// bpm: 20-300 (default 120), loops: 1-16 (default 2)

// Noise generator
traits.audio('noise', type, duration, volume)
// type: 'white'|'pink'|'brown' (default 'white')

// Chord — simultaneous frequencies
traits.audio('chord', freqs, duration, waveform, volume)
// freqs: array of Hz values

// Frequency sweep
traits.audio('sweep', startFreq, endFreq, duration, waveform, volume)

// Control
traits.audio('stop')     // Stop all audio
traits.audio('status')   // Check AudioContext state
```

**Note:** Most games implement their own WebAudio sound effects directly (creating `AudioContext`, `OscillatorNode`, etc.) rather than using `traits.audio()`. The bridge's pause engine automatically patches `AudioContext` to suspend/resume all contexts on pause/resume. Either approach works.

### High Score System (`window.traits.score`)

Cross-client high score synchronization via the relay WebSocket.

**Game-side usage:**
```javascript
// Report a score (call whenever score increases)
window.traits.score(newScore);

// Read the current synced high score for this game
var best = window.traits.score();

// Listen for real-time high score updates from other clients
window.addEventListener('message', function(e) {
    if (e.data && e.data.type === 'highscore-update') {
        // e.data.score = new high score from another player
    }
});
```

**Flow:**
1. Game calls `traits.score(val)` → bridge posts `{type:'canvas-score', score:N}` to parent
2. Parent computes SHA-256 hash of game content → `window.__activeGameHash` (first 16 hex chars)
3. Parent updates `window.__highScores[hash]` and sends `{type:'score', game_hash, score}` via sync WebSocket
4. Relay broadcasts `{type:'score-update', game_hash, score}` to all connected clients
5. Other clients receive and post `{type:'highscore-update', score}` into their game iframe

**Score variable patterns in existing games:**
- DOM-based: `<div id="score">0</div>` + `<div id="hs">best: 0</div>` (Snake)
- Canvas-rendered: `game.score` (Tetronix, Arcanoid, Pixel Runner), `score` global (Blast Zone)
- All games now have a universal score hook injected before `</body>` that uses MutationObserver on `#score` elements + polling `score`/`game.score` globals every 500ms

### Touch Controls

Games must implement their own touch controls. The bridge does NOT inject any touch abstraction — it only handles two-finger gestures (for carousel navigation) and forwards single-finger touches to the game unmodified.

**Common patterns in existing games:**

| Pattern | Games | Implementation |
|---------|-------|----------------|
| Swipe direction | Snake, Tetronix | `touchstart`/`touchend` → compute dx/dy → map to direction |
| Touch zones (left/center/right) | Pixel Runner | Left 1/3 = left, Right 1/3 = right, Center = jump |
| Touch drag position | Blast Zone, BrickStorm, Arcanoid | `touchstart`/`touchmove` → set paddle x from touch x |
| Touch position (aim) | ShooterX | Touch y ≥ 70% = move left/right, else = aim direction |
| D-pad buttons | Snake (built-in default) | HTML buttons with `click` handlers |

**Multi-touch gotcha:** When using touch zones, track touches by `touch.identifier` and only clear the zone for that specific finger on `touchend`/`touchcancel`. Clearing ALL state on any `touchend` breaks simultaneous left+jump. See Pixel Runner's fixed implementation for reference.

**Two-finger is reserved:** The bridge intercepts all two-finger events for carousel navigation. Games should only rely on single-finger touch.

### Pause Engine

The bridge monkey-patches `requestAnimationFrame`, `setTimeout`, `setInterval` (and their cancel counterparts) plus `AudioContext`. On pause:
- All new rAF/timer callbacks are queued instead of scheduled
- All active intervals are cleared and stashed
- All AudioContexts are suspended
- `window.traits.onPause()` is called if defined

On resume: queued callbacks are flushed, intervals restarted, AudioContexts resumed, `window.traits.onResume()` called.

**Games are generically frozen without cooperation** — no game code changes needed for basic pause. Custom hooks are optional (e.g., show pause screen, save state).

### Console Capture

The bridge wraps `console.log/warn/error` to forward messages to the parent:
```javascript
window.parent.postMessage({type:'canvas-console', level:'log'|'warn'|'error', message:string}, '*')
```
Parent stores last 50 entries in `window.__canvasGameLogs` for voice agent context. Uncaught errors (`window.error` events) are also captured.

### postMessage Protocol

**iframe → parent:**

| `type` | Fields | Purpose |
|--------|--------|---------|
| `canvas-console` | `level`, `message` | Forward game console output |
| `canvas-score` | `score` | Report game score to sync system |
| `canvas-two-finger-start` | `x`, `y` | Two-finger touchstart midpoint |
| `canvas-two-finger-move` | `x`, `y` | Two-finger touchmove midpoint |
| `canvas-two-finger-end` | — | All fingers lifted after two-finger gesture |

**parent → iframe:**

| `type` | Fields | Purpose |
|--------|--------|---------|
| `canvas-pause` | — | Freeze game execution (rAF + timers + audio) |
| `canvas-resume` | — | Resume game execution, flush queued callbacks |
| `highscore-update` | `score` | New high score from another client |

### Mobile Carousel Gestures (parent-side)

- **Two-finger contact** → immediately pauses game (sends `canvas-pause`), shows chrome
- **Two-finger drag** → translates `#phone-viewport` via CSS `translateX`, shows peeking game labels
- **Release after drag >30% screen width** → animate off-screen, switch game, animate in, auto-resume
- **Release after small drag** → snap back, game paused, chrome visible
- **Two-finger tap (no drag)** → pause + show chrome. Single tap while paused → resume + hide chrome

### Extending the Bridge

- Edit `const BRIDGE` in `canvas.rs` (~line 601)
- Use `window.parent.postMessage({type:'canvas-*', ...}, '*')` for new iframe→parent messages
- Add corresponding `window.addEventListener('message', ...)` in the parent script section
- **Keep ES5-compatible** (`var`, `function`, no arrow functions)
- Must be a self-executing IIFE wrapped in `<script>...</script>`
- Escape closing tag as `<\/script>` (lives inside a JS template literal)

---

## Game Sync & REST API

Games are synced between all connected clients via a **Cloudflare Durable Object** (`GameRoom`). The relay also exposes a REST API for reading, editing, and deleting games programmatically.

**Relay:** `relay/src/index.js` — Cloudflare Worker at `https://relay.slob.games`
**Deploy:** `cd relay && npx wrangler deploy`

### REST Endpoints

All endpoints are under `https://relay.slob.games/sync/`. CORS allows all origins.

```bash
# List all games (name, hash, size, updated)
curl -s https://relay.slob.games/sync/games | python3 -m json.tool

# Get full HTML content of one game
curl -s https://relay.slob.games/sync/game/<HASH>

# Update a game (content + optional rename) — returns new hash
curl -X PUT https://relay.slob.games/sync/game/<HASH> \
  -H 'Content-Type: application/json' \
  -d '{"name": "Game Name", "content": "<html>...</html>"}'

# Delete a game
curl -X DELETE https://relay.slob.games/sync/game/<HASH>

# List all high scores
curl -s https://relay.slob.games/sync/scores
```

### REST Response Formats

**GET /sync/games:**
```json
[{"content_hash":"42ac8829...","name":"Pixel Runner","size":28288,"updated":"2026-04-09T..."}]
```

**GET /sync/game/:hash:**
```json
{"content_hash":"42ac8829...","name":"Pixel Runner","content":"<!DOCTYPE html>...","updated":"..."}
```

**PUT /sync/game/:hash:**
```json
{"ok":true,"old_hash":"f52da07a...","content_hash":"42ac8829...","name":"Pixel Runner","size":28288}
```
Note: PUT computes a new SHA-256 hash from the updated content. The old hash becomes invalid. Broadcasts the update to all connected WebSocket clients immediately.

**DELETE /sync/game/:hash:**
```json
{"ok":true,"deleted":"71e0d5ff..."}
```

### Agent Workflow: Editing Games via REST

To programmatically view and modify synced games:

```bash
# 1. List games to find the hash
curl -s https://relay.slob.games/sync/games | python3 -c "
import sys,json
for g in json.loads(sys.stdin.read()):
    print(f'{g[\"content_hash\"]} {g[\"size\"]:>6}B  {g[\"name\"]}')
"

# 2. Download a game
curl -s https://relay.slob.games/sync/game/HASH | python3 -c "
import sys,json; print(json.loads(sys.stdin.read())['content'])" > /tmp/game.html

# 3. Edit the HTML (search/replace, inject code, fix bugs)

# 4. Push the modified game back
python3 -c "
import urllib.request, ssl, json
html = open('/tmp/game.html').read()
data = json.dumps({'name': 'Game Name', 'content': html}).encode()
req = urllib.request.Request(
    'https://relay.slob.games/sync/game/HASH',
    data=data,
    headers={'Content-Type': 'application/json', 'User-Agent': 'Mozilla/5.0'},
    method='PUT')
with urllib.request.urlopen(req, context=ssl.create_default_context()) as r:
    print(json.loads(r.read()))
"
```

**Important:** Python's default `User-Agent` (`Python-urllib/3.x`) is blocked by Cloudflare. Always set `User-Agent: Mozilla/5.0` when using Python `urllib`. Alternatively, use `curl` via subprocess.

### Batch Processing Games

For bulk operations (fix controls, inject score hooks, rename, delete stubs), write a Python script:

```python
import urllib.request, ssl, json
API = 'https://relay.slob.games/sync'
UA = {'User-Agent': 'Mozilla/5.0'}
ctx = ssl.create_default_context()

def api_get(path):
    req = urllib.request.Request(f'{API}{path}', headers=UA)
    with urllib.request.urlopen(req, context=ctx) as r:
        return json.loads(r.read())

def api_put(path, data):
    body = json.dumps(data).encode()
    req = urllib.request.Request(f'{API}{path}', data=body,
        headers={**UA, 'Content-Type': 'application/json'}, method='PUT')
    with urllib.request.urlopen(req, context=ctx) as r:
        return json.loads(r.read())

def api_delete(path):
    req = urllib.request.Request(f'{API}{path}', headers=UA, method='DELETE')
    with urllib.request.urlopen(req, context=ctx) as r:
        return json.loads(r.read())

# List all games
games = api_get('/games')

# Get game content
data = api_get(f'/game/{hash}')
html = data['content']

# Modify and push back
html = html.replace('old', 'new')
result = api_put(f'/game/{hash}', {'name': 'New Name', 'content': html})
# result['content_hash'] = new hash (old hash is now invalid)
```

### WebSocket Game Sync Protocol

Clients connect via `wss://relay.slob.games/sync` for real-time game sync. The parent `canvas.rs` manages this connection.

**Server → Client messages:**

| `type` | Fields | Purpose |
|--------|--------|---------|
| `sync` | `games[]` | Full game catalog or incremental update. Each entry: `{content_hash, name, content, updated}` |
| `scores` | `scores[]` | Initial high score catalog. Each: `{game_hash, score, player, updated}` |
| `score-update` | `game_hash`, `score` | Real-time high score broadcast |

**Client → Server messages:**

| `type` | Fields | Purpose |
|--------|--------|---------|
| `sync` | `games[]` | Push local games to server |
| `score` | `game_hash`, `score` | Report a new high score |

### Current Game Inventory (as of April 2026)

8 games, all with score integration + touch controls:

| Hash (first 8) | Name | Size | Controls | Notes |
|----------------|------|------|----------|-------|
| `f3861a4b` | Snake Classic | ~7KB | D-pad buttons + keyboard + swipe | Simple grid snake |
| `d1bd2198` | Snake | ~26KB | D-pad buttons + keyboard + canvas swipe | Feature-rich snake |
| `fef61d05` | Tetronix | ~29KB | Keyboard + swipe gestures | Tetris clone with powerups |
| `420c8ded` | ShooterX | ~27KB | Keyboard + touch position | Space shooter |
| `ebb3564c` | Blast Zone! | ~29KB | Keyboard + touch drag paddle | Breakout with levels/powerups |
| `8b4fa1da` | BrickStorm DX | ~28KB | Keyboard + touch drag paddle | Breakout variant |
| `d58e092b` | Arcanoid Ultra | ~23KB | Keyboard + touch drag paddle | Arcanoid clone |
| `42ac8829` | Pixel Runner | ~28KB | Keyboard + touch zones (L/J/R) | Platformer runner |

**Note:** Game hashes change whenever content is updated (hash = SHA-256 of content, first 16 hex chars). Always use `GET /sync/games` to get current hashes before editing.

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
