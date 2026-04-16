// ═══════════════════════════════════════════
// ── Shared WASM-powered Terminal ──
// Thin display layer: all line editing, history,
// tab completion, and interactive mode live in the
// WASM kernel (kernel/cli CliSession).
// JS just pipes xterm.js data ↔ wasm.cli_input().
// ═══════════════════════════════════════════

const _sharedDefaults = (typeof window !== 'undefined' && window.TerminalShared && window.TerminalShared.defaults)
    ? window.TerminalShared.defaults
    : null;
const _sharedAdapters = (typeof window !== 'undefined' && window.TerminalSharedAdapters)
    ? window.TerminalSharedAdapters
    : null;

const _sentinels = _sharedDefaults?.sentinels || {};
const CLEAR_SENTINEL = _sentinels.clear || '\x1b[CLEAR]';
const REST_RE = new RegExp(`${(_sentinels.restOpen || '\\x1b\\[REST\\]').replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}([\\s\\S]*?)${(_sentinels.restClose || '\\x1b\\[/REST\\]').replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}`);
const WEBLLM_RE = new RegExp(`${(_sentinels.webllmOpen || '\\x1b\\[WEBLLM\\]').replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}([\\s\\S]*?)${(_sentinels.webllmClose || '\\x1b\\[/WEBLLM\\]').replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}`);
const LUA_RE = new RegExp(`${(_sentinels.luaOpen || '\x1b[LUA]').replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}([\\s\\S]*?)${(_sentinels.luaClose || '\x1b[/LUA]').replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}`);
const VOICE_RE = new RegExp(`${(_sentinels.voiceOpen || '\\x1b\\[VOICE\\]').replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}([\\s\\S]*?)${(_sentinels.voiceClose || '\\x1b\\[/VOICE\\]').replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}`);
// Source of truth: kernel/cli/cli.rs PROMPT constant. Must stay in sync.
const PROMPT = _sharedDefaults?.prompt || '\x1b[32mtraits \x1b[0m';

const _keys = _sharedDefaults?.storageKeys || {};
const LS_SCROLLBACK = _keys.scrollback || 'traits.terminal.scrollback';
const LS_HISTORY    = _keys.history || 'traits.terminal.history';
const LS_PVFS       = _keys.pvfs || 'traits.pvfs';
const LS_VFS_LEGACY = _keys.legacyVfs || 'traits.terminal.vfs';

let Terminal, FitAddon, WebLinksAddon, SerializeAddon;

/**
 * Create and mount a WASM-powered terminal.
 * @param {HTMLElement} mountEl  — element to mount xterm.js into (e.g. #xterm)
 * @param {object} opts
 * @param {HTMLElement} [opts.header]    — clickable header for collapse/expand
 * @param {HTMLElement} [opts.container] — container to toggle .collapsed on
 * @param {HTMLElement} [opts.toggleBtn] — button whose text changes on collapse
 * @param {HTMLElement} [opts.statusEl]  — element for WASM status badge
 * @param {function}    [opts.onReady]   — called with { wasm, traitCount, wasmCount } when ready
 * @returns {Promise<{term, fitAddon, wasm}>}
 */
export async function createTerminal(mountEl, opts = {}) {
    // ── Load xterm.js ──
    try {
        const xtermMod = await import('https://cdn.jsdelivr.net/npm/@xterm/xterm@5/+esm');
        Terminal = xtermMod.Terminal;
        const fitMod = await import('https://cdn.jsdelivr.net/npm/@xterm/addon-fit@0.10/+esm');
        FitAddon = fitMod.FitAddon;
        const linksMod = await import('https://cdn.jsdelivr.net/npm/@xterm/addon-web-links@0.11/+esm');
        WebLinksAddon = linksMod.WebLinksAddon;
        const serMod = await import('https://cdn.jsdelivr.net/npm/@xterm/addon-serialize@0.13/+esm');
        SerializeAddon = serMod.SerializeAddon;
    } catch (e) {
        mountEl.innerHTML = `<div style="padding:1rem;color:#f85149">Failed to load terminal: ${e.message}</div>`;
        throw e;
    }

    const term = new Terminal({
        cursorBlink: true,
        cursorStyle: 'block',
        fontSize: 13,
        fontFamily: "'SF Mono', 'Fira Code', 'Cascadia Code', 'Menlo', monospace",
        lineHeight: 1.3,
        scrollback: 5000,
        // Ensure macOS Option key emits Meta/Alt escape sequences (
        // so word-wise navigation like Opt+Arrow can be parsed by the CLI).
        macOptionIsMeta: true,
        theme: {
            background: '#0d1117',
            foreground: '#c9d1d9',
            cursor: '#c9d1d9',
            cursorAccent: '#0d1117',
            selectionBackground: '#264f78',
            selectionForeground: '#ffffff',
            black: '#484f58',   red: '#f85149',   green: '#3fb950',
            yellow: '#d29922',  blue: '#58a6ff',  magenta: '#bc8cff',
            cyan: '#76e3ea',    white: '#c9d1d9',
            brightBlack: '#6e7681',  brightRed: '#ffa198',   brightGreen: '#56d364',
            brightYellow: '#e3b341', brightBlue: '#79c0ff',  brightMagenta: '#d2a8ff',
            brightCyan: '#b3f0ff',   brightWhite: '#f0f6fc',
        },
    });

    const fitAddon = new FitAddon();
    const serializeAddon = SerializeAddon ? new SerializeAddon() : null;
    term.loadAddon(fitAddon);
    if (serializeAddon) term.loadAddon(serializeAddon);
    term.loadAddon(new WebLinksAddon());
    term.open(mountEl);
    // Keep focus/navigation keys inside terminal input stream.
    term.attachCustomKeyEventHandler((ev) => {
        const key = ev.key || '';
        const isTab = key === 'Tab';
        const isWordArrow = (ev.altKey || ev.metaKey) && (key === 'ArrowLeft' || key === 'ArrowRight');
        if (isTab || isWordArrow) {
            ev.preventDefault();
            ev.stopPropagation();
            return true;
        }
        return true;
    });
    fitAddon.fit();

    // ── Persist scrollback + history to localStorage ──
    let wasm = null;
    let backgroundCall = null;
    let activeSdk = window._traitsSDK || null;

    const persistence = _sharedAdapters?.createPersistenceAdapter
        ? _sharedAdapters.createPersistenceAdapter({
            keys: {
                scrollback: LS_SCROLLBACK,
                history: LS_HISTORY,
                pvfs: LS_PVFS,
                legacyVfs: LS_VFS_LEGACY,
            },
            serializeAddon,
            getBackgroundCall: () => backgroundCall,
        })
        : null;
    const saveState = persistence?.saveState || (() => {
        if (serializeAddon) {
            try { localStorage.setItem(LS_SCROLLBACK, serializeAddon.serialize()); } catch (_) {}
        }
    });
    if (persistence?.attachAutoSaveHandlers) {
        persistence.attachAutoSaveHandlers();
    } else {
        window.addEventListener('pagehide', saveState);
        window.addEventListener('hashchange', saveState);
        document.addEventListener('visibilitychange', () => {
            if (document.visibilityState === 'hidden') saveState();
        });
    }

    // ── Collapse/expand ──
    if (opts.header && opts.container) {
        opts.header.addEventListener('click', () => {
            opts.container.classList.toggle('collapsed');
            if (opts.toggleBtn) {
                opts.toggleBtn.textContent = opts.container.classList.contains('collapsed')
                    ? '▶ Terminal' : '▼ Terminal';
            }
            if (!opts.container.classList.contains('collapsed')) {
                setTimeout(() => { fitAddon.fit(); term.focus(); }, 50);
            }
        });
    }
    if (opts.container) {
        new ResizeObserver(() => {
            if (!opts.container.classList.contains('collapsed')) fitAddon.fit();
        }).observe(opts.container);
    }

    // ── Status ──
    const setStatus = (text, cls) => {
        if (opts.statusEl) {
            opts.statusEl.textContent = text;
            opts.statusEl.className = 'terminal-status ' + (cls || '');
        }
    };
    setStatus('loading WASM…', 'loading');

    // ── Load background runtime (preferred: SDK adapter; fallback: direct WASM) ──
    try {
        if (_sharedAdapters?.initTraitsTransport) {
            const init = await _sharedAdapters.initTraitsTransport({
                activeSdk,
                setStatus,
                onReady: opts.onReady,
            });
            activeSdk = init.activeSdk || activeSdk;
            backgroundCall = init.backgroundCall || backgroundCall;
            wasm = init.wasm || wasm;
        } else {
            if (activeSdk && typeof activeSdk.backgroundCall === 'function') {
                await activeSdk.initWorkerPool();
                backgroundCall = (cmd, payload = {}) => activeSdk.backgroundCall(cmd, payload);
                const status = activeSdk.status || {};
                setStatus('WASM worker', 'ready');
                if (window.TraitsWasm && window.TraitsWasm.register_task) {
                    try { window.TraitsWasm.register_task('terminal', 'Terminal', 'service', Date.now(), 'xterm.js CLI session'); } catch(e) {}
                }
                if (opts.onReady) opts.onReady({ wasm: null, traitCount: status.traits || 0, wasmCount: status.callable || 0, background: true });
            } else {
                if (window.TraitsWasm && window.TraitsWasm.cli_input) {
                    wasm = window.TraitsWasm;
                    const count = wasm.is_registered ? JSON.parse(wasm.callable_traits()).length : 0;
                    setStatus('WASM (SPA)', 'ready');
                    if (opts.onReady) opts.onReady({ wasm, traitCount: 0, wasmCount: count, background: false });
                } else {
                    const wasmJsUrl = '/wasm/traits_wasm.js';
                    const wasmBinUrl = '/wasm/traits_wasm_bg.wasm';
                    const mod = await import(wasmJsUrl);
                    await mod.default(wasmBinUrl);
                    const initResult = JSON.parse(mod.init());
                    wasm = mod;
                    const count = initResult.traits_registered || 0;
                    const wasmCount = initResult.wasm_callable || 0;
                    setStatus(`${count} traits (${wasmCount} WASM)`, 'ready');
                    if (opts.onReady) opts.onReady({ wasm, traitCount: count, wasmCount, background: false });
                }

                if (window.Traits) {
                    activeSdk = new window.Traits({
                        useWasm: false,
                        useHelper: false,
                        server: '',
                    });
                    activeSdk.attachWasm(wasm);
                    activeSdk.setBackgroundBinding('sdk.background.direct');
                    backgroundCall = (cmd, payload = {}) => activeSdk.backgroundCall(cmd, payload, { impl: 'sdk.background.direct' });
                }
                if (wasm && wasm.register_task) {
                    try { wasm.register_task('terminal', 'Terminal', 'service', Date.now(), 'xterm.js CLI session'); } catch(e) {}
                }
            }
        }
    } catch (e) {
        setStatus('background failed', 'error');
        console.error('Background runtime load failed:', e);
    }

    // ── Input → WASM session → output (with REST fallback) ──
    let restPending = false;

    const writeText = (text) => {
        const s = String(text || '');
        if (s) term.write(s.replace(/\n/g, '\r\n'));
    };

    const writeLine = (text = '') => {
        writeText(text);
        term.write('\r\n');
    };

    const parseLuaJson = (raw) => {
        const text = String(raw || '').trim();
        if (!text) return {};
        try { return JSON.parse(text); } catch (_) { return text; }
    };

    const unwrapResult = (res) => {
        if (res && typeof res === 'object' && Object.prototype.hasOwnProperty.call(res, 'result')) {
            return res.result;
        }
        return res;
    };

    const resolveLuaPath = (name) => {
        const trimmed = String(name || '').trim();
        if (!trimmed) return '';
        if (trimmed === 'relay-monitor') return 'tools/relay-monitor.lua';
        if (trimmed === 'storage-inspector') return 'tools/storage-inspector.lua';
        return trimmed.replace(/^\.\//, '');
    };

    const parseRelayMonitorToken = (args) => {
        for (let i = 0; i < args.length; i++) {
            if (args[i] === '--token') return args[i + 1] || '';
            if (args[i] && args[i].startsWith('--token=')) return args[i].slice('--token='.length);
        }
        try {
            return (localStorage.getItem('traits.secret.SLOB_USER_TOKEN') || '').trim();
        } catch (_) {
            return '';
        }
    };

    const readVfsText = async (path) => {
        if (!activeSdk) throw new Error('SDK unavailable');
        const res = await activeSdk.call('sys.vfs', ['read', path], { force: 'wasm' });
        const data = unwrapResult(res);
        if (!res?.ok || !data?.ok) {
            throw new Error(data?.error || res?.error || `failed to read ${path}`);
        }
        return String(data.content || '');
    };

    const runLuaCode = async (code, input) => {
        if (!activeSdk) return { ok: false, error: 'SDK unavailable' };
        const res = await activeSdk.call('sys.lua', [String(code || ''), input || {}], { force: 'wasm' });
        if (!res?.ok) return { ok: false, error: res?.error || 'sys.lua call failed' };
        return unwrapResult(res) || { ok: false, error: 'empty lua result' };
    };

    const printLuaOutcome = (outcome) => {
        if (!outcome || typeof outcome !== 'object') {
            writeLine('\x1b[31mLua error: invalid result\x1b[0m');
            return;
        }
        const stdout = Array.isArray(outcome.stdout) ? outcome.stdout : [];
        const stderr = Array.isArray(outcome.stderr) ? outcome.stderr : [];
        stdout.forEach((line) => writeLine(String(line)));
        stderr.forEach((line) => writeLine(`\x1b[31m${String(line)}\x1b[0m`));
        if (outcome.ok === false) {
            writeLine(`\x1b[31mLua error: ${String(outcome.error || 'unknown error')}\x1b[0m`);
            return;
        }
        if (!stdout.length && outcome.result !== undefined && outcome.result !== null && outcome.result !== '') {
            const text = typeof outcome.result === 'string'
                ? outcome.result
                : JSON.stringify(outcome.result, null, 2);
            writeLine(text);
        }
    };

    const fetchJson = async (url, headers) => {
        const res = await fetch(url, { headers: headers || {} });
        if (!res.ok) throw new Error(`${res.status} ${res.statusText}`.trim());
        return res.json();
    };

    const fetchText = async (url, headers) => {
        const res = await fetch(url, { headers: headers || {} });
        if (!res.ok) throw new Error(`${res.status} ${res.statusText}`.trim());
        return res.text();
    };

    const runLuaRelayMonitor = async (args) => {
        const token = parseRelayMonitorToken(args);
        const headers = token ? { Authorization: `Bearer ${token}` } : {};
        const input = {
            relay: 'https://relay.slob.games',
            token_provided: !!token,
        };

        try {
            input.games = await fetchJson('https://relay.slob.games/sync/games');
        } catch (e) {
            input.games = [];
            input.games_error = e && e.message ? e.message : String(e);
        }

        try {
            input.scores = await fetchJson('https://relay.slob.games/sync/scores');
        } catch (e) {
            input.scores = [];
            input.scores_error = e && e.message ? e.message : String(e);
        }

        try {
            input.manifest = await fetchText('https://relay.slob.games/sync/games.toml');
        } catch (e) {
            input.manifest = '';
            input.manifest_error = e && e.message ? e.message : String(e);
        }

        if (token) {
            try {
                input.my_games = await fetchJson('https://relay.slob.games/sync/internal/games', headers);
            } catch (e) {
                input.my_games = [];
                input.my_games_error = e && e.message ? e.message : String(e);
            }
        }

        const script = await readVfsText('tools/relay-monitor.lua');
        return runLuaCode(script, input);
    };

    const runLuaStorageInspector = async () => {
        const input = { keys: [], games: [], sprites: [], other_vfs: [] };
        try {
            // Estimate quota (most browsers: 5MB for localStorage)
            input.quota_bytes = 5 * 1024 * 1024;

            // Enumerate all localStorage keys and sizes
            let totalUsed = 0;
            const keyList = [];
            for (let i = 0; i < localStorage.length; i++) {
                const k = localStorage.key(i);
                if (!k) continue;
                const val = localStorage.getItem(k) || '';
                // JS strings are UTF-16, each char = 2 bytes in storage
                const bytes = (k.length + val.length) * 2;
                totalUsed += bytes;
                keyList.push({ key: k, size: bytes });
            }
            keyList.sort((a, b) => b.size - a.size);
            input.keys = keyList;
            input.used_bytes = totalUsed;

            // Parse traits.pvfs
            const pvfsRaw = localStorage.getItem('traits.pvfs') || '{}';
            input.pvfs_bytes = (('traits.pvfs').length + pvfsRaw.length) * 2;

            const files = JSON.parse(pvfsRaw);
            // Detect WASM Rust {files:{}} format vs flat format
            const flat = files.files ? Object.fromEntries(
                Object.entries(files.files).map(([k, v]) => [k, typeof v === 'object' && v.content ? v.content : v])
            ) : files;

            // Parse games collection
            let gamesTotal = 0;
            const gamesJson = flat['canvas/games.json'];
            if (gamesJson) {
                const col = JSON.parse(gamesJson);
                const gamesMap = col.games || {};
                for (const [id, g] of Object.entries(gamesMap)) {
                    const content = g.content || '';
                    const sz = content.length * 2;
                    gamesTotal += sz;
                    input.games.push({
                        id: id,
                        name: g.name || id,
                        scope: g.scope || g._scope || 'internal',
                        size: sz,
                        hash: g._sync_hash || g.checksum || '',
                        active: col.active === id,
                    });
                }
                // Sort: active first, then by size desc
                input.games.sort((a, b) => {
                    if (a.active !== b.active) return a.active ? -1 : 1;
                    return b.size - a.size;
                });
            }
            input.games_total = gamesTotal;

            // Categorize other VFS entries
            let spritesTotal = 0;
            for (const [path, val] of Object.entries(flat)) {
                if (path === 'canvas/games.json') continue;
                const sz = (typeof val === 'string' ? val.length : JSON.stringify(val).length) * 2;
                const isSprite = /^canvas\/sprites\/|\.png$|\.svg$|\.gif$|\.jpg$|\.webp$/i.test(path);
                if (isSprite) {
                    spritesTotal += sz;
                    input.sprites.push({ path: path, size: sz });
                } else {
                    input.other_vfs.push({ path: path, size: sz });
                }
            }
            input.sprites.sort((a, b) => b.size - a.size);
            input.other_vfs.sort((a, b) => b.size - a.size);
            input.sprites_total = spritesTotal;
        } catch (e) {
            input.error = e && e.message ? e.message : String(e);
        }

        const script = await readVfsText('tools/storage-inspector.lua');
        return runLuaCode(script, input);
    };

    // ── WebLLM progress — show model loading status inline ──
    window.addEventListener('webllm-progress', (e) => {
        if (restPending && e.detail) {
            term.write(`\r\x1b[K\x1b[90m⏳ ${e.detail}\x1b[0m`);
        }
    });

    let ioChain = Promise.resolve();
    term.onData(data => {
        if (!backgroundCall || restPending) return;
        ioChain = ioChain.then(async () => {
            const inputRes = await backgroundCall('cli_input', { data });
            if (!inputRes?.ok) {
                term.write(`\x1b[31mCLI error: ${inputRes?.error || 'unknown'}\x1b[0m\r\n`);
                term.write(PROMPT);
                return;
            }
            const output = inputRes.result || '';
            if (!output) return;

            // Check for REST dispatch sentinel
            const restMatch = output.match(REST_RE);
            if (restMatch) {
            // Write visible part (loading message) without the sentinel
                const visible = output.replace(REST_RE, '');
                if (visible) term.write(visible);

            // Parse dispatch info and call via SDK cascade (WASM → helper → REST)
            // Supports @target routing: sentinel JSON may contain "t" field (rest/relay/helper/wasm)
            // Chat mode: "rp" = return prompt (instead of PROMPT), "sid" = session ID for VFS storage
                try {
                    const { p, a, t, rp, sid, stream: useStream } = JSON.parse(restMatch[1]);
                    const returnPrompt = rp || PROMPT;
                    restPending = true;
                    const callOpts = t ? { force: t } : {};
                    if (useStream) callOpts.stream = true;

                    // Helper: store assistant response in WASM VFS for chat history
                    const storeChatResponse = (text) => {
                        if (!sid || !backgroundCall) return;
                        const vfsKey = `chat/${sid}.json`;
                        backgroundCall('vfs_read', { path: vfsKey }).then(res => {
                            let msgs = [];
                            try { if (res?.ok && res.result) msgs = JSON.parse(res.result); } catch (_) {}
                            msgs.push({ role: 'assistant', content: text });
                            backgroundCall('vfs_write', { path: vfsKey, content: JSON.stringify(msgs) });
                        }).catch(() => {});
                    };

                    if (activeSdk) {
                        activeSdk.call(p, a, callOpts).then(async res => {
                        // Streaming path: consume async generator, write tokens to terminal
                        if (res.ok && res.stream) {
                            let streamStarted = false;
                            let fullText = '';
                            try {
                                for await (const chunk of res.stream) {
                                    const text = typeof chunk === 'string' ? chunk : (chunk?.result ?? JSON.stringify(chunk));
                                    if (!streamStarted) {
                                        term.write('\r\x1b[K'); // Clear "thinking…" line
                                        streamStarted = true;
                                    }
                                    term.write(text.replace(/\n/g, '\r\n'));
                                    fullText += text;
                                }
                            } catch (e) {
                                term.write(`\r\n\x1b[31mStream error: ${e.message}\x1b[0m\r\n`);
                            }
                            if (!streamStarted) term.write('\r\x1b[K'); // Clear "thinking…" if no chunks
                            if (fullText && !fullText.endsWith('\n')) term.write('\r\n');
                            storeChatResponse(fullText);
                            term.write(returnPrompt);
                            return;
                        }
                        // Non-streaming path (fallback)
                        term.write('\r\x1b[K'); // Clear progress line
                        if (res.ok && res.result !== undefined) {
                            // Try WASM CLI formatter first, fall back to JSON
                            let text = '';
                            const resultJson = typeof res.result === 'string'
                                ? JSON.stringify(res.result)
                                : JSON.stringify(res.result);
                            const fmt = await backgroundCall('cli_format_rest_result', {
                                path: p,
                                args_json: JSON.stringify(a),
                                result_json: resultJson,
                            });
                            if (fmt?.ok) {
                                text = fmt.result || '';
                            }
                            if (!text) {
                                text = typeof res.result === 'string'
                                    ? res.result
                                    : JSON.stringify(res.result, null, 2);
                            }
                            term.write(text.replace(/\n/g, '\r\n'));
                            if (!text.endsWith('\n')) term.write('\r\n');
                            storeChatResponse(text);
                        } else if (res.error) {
                            // Try WASM formatter with null result (local fallback)
                            let fallback = '';
                            const fmt = await backgroundCall('cli_format_rest_result', {
                                path: p,
                                args_json: JSON.stringify(a),
                                result_json: 'null',
                            });
                            if (fmt?.ok) {
                                fallback = fmt.result || '';
                            }
                            if (fallback) {
                                term.write(fallback.replace(/\n/g, '\r\n'));
                                if (!fallback.endsWith('\n')) term.write('\r\n');
                            } else {
                                term.write(`\x1b[31mError: ${res.error}\x1b[0m\r\n`);
                            }
                        }
                        term.write(returnPrompt);
                        }).catch(e => {
                            term.write(`\x1b[31mDispatch error: ${e.message}\x1b[0m\r\n`);
                            term.write(returnPrompt);
                        }).finally(() => { restPending = false; requestAnimationFrame(saveState); });
                    } else {
                    // Last-resort REST fallback (SDK unavailable)
                        const restPath = p.replace(/\./g, '/');
                        fetch(`/traits/${restPath}`, {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ args: a }),
                    })
                    .then(r => r.json())
                    .then(data => {
                        if (data.result !== undefined) {
                            let text = '';
                            if (wasm && wasm.cli_format_rest_result) {
                                text = wasm.cli_format_rest_result(p, JSON.stringify(a),
                                    JSON.stringify(data.result));
                            }
                            if (!text) {
                                text = typeof data.result === 'string'
                                    ? data.result
                                    : JSON.stringify(data.result, null, 2);
                            }
                            term.write(text.replace(/\n/g, '\r\n'));
                            if (!text.endsWith('\n')) term.write('\r\n');
                            storeChatResponse(text);
                        } else if (data.error) {
                            term.write(`\x1b[31mError: ${data.error}\x1b[0m\r\n`);
                        }
                        term.write(returnPrompt);
                    })
                    .catch(e => {
                        term.write(`\x1b[31mREST error: ${e.message}\x1b[0m\r\n`);
                        term.write(returnPrompt);
                    })
                    .finally(() => { restPending = false; requestAnimationFrame(saveState); });
                    }
                } catch (e) {
                    term.write(`\x1b[31mREST parse error: ${e.message}\x1b[0m\r\n`);
                    term.write(PROMPT);
                    restPending = false;
                    requestAnimationFrame(saveState);
                }
                return;
            }

            // Check for WebLLM dispatch sentinel
            const webllmMatch = output.match(WEBLLM_RE);
            if (webllmMatch && activeSdk) {
                const visible = output.replace(WEBLLM_RE, '');
                if (visible) term.write(visible);
                try {
                    const { prompt, model } = JSON.parse(webllmMatch[1]);
                    restPending = true;
                    let streamStarted = false;
                    const onToken = (text) => {
                        if (!streamStarted) {
                            term.write('\r\x1b[K'); // Clear progress line on first token
                            streamStarted = true;
                        }
                        term.write(text.replace(/\n/g, '\r\n'));
                    };
                    activeSdk._callWebLLM(prompt, model, onToken).then(res => {
                        if (streamStarted) {
                            // Streaming completed — just add newline + prompt
                            if (res.ok) {
                                const text = typeof res.result === 'string' ? res.result : '';
                                if (!text.endsWith('\n')) term.write('\r\n');
                            } else if (res.error) {
                                term.write(`\r\n\x1b[31mWebLLM: ${res.error}\x1b[0m\r\n`);
                            }
                        } else {
                            // No tokens streamed (non-streaming fallback or empty result)
                            term.write('\r\x1b[K');
                            if (res.ok && res.result !== undefined) {
                                const text = typeof res.result === 'string'
                                    ? res.result : JSON.stringify(res.result, null, 2);
                                term.write(text.replace(/\n/g, '\r\n'));
                                if (!text.endsWith('\n')) term.write('\r\n');
                            } else if (res.error) {
                                term.write(`\x1b[31mWebLLM: ${res.error}\x1b[0m\r\n`);
                            } else {
                                term.write('\x1b[33mWebLLM returned empty result\x1b[0m\r\n');
                            }
                        }
                        term.write(PROMPT);
                    }).catch(e => {
                        console.error('[terminal] WebLLM dispatch error:', e);
                        term.write(`\r\x1b[K\x1b[31mWebLLM error: ${e.message || e}\x1b[0m\r\n`);
                        term.write(PROMPT);
                    }).finally(() => { restPending = false; requestAnimationFrame(saveState); });
                } catch (e) {
                    term.write(`\x1b[31mWebLLM parse error: ${e.message}\x1b[0m\r\n`);
                    term.write(PROMPT);
                    restPending = false;
                    requestAnimationFrame(saveState);
                }
                return;
            }

            const luaMatch = output.match(LUA_RE);
            if (luaMatch) {
                const visible = output.replace(LUA_RE, '');
                if (visible) term.write(visible);
                try {
                    const payload = parseLuaJson(luaMatch[1]);
                    restPending = true;
                    let outcome;
                    const payloadObj = payload && typeof payload === 'object' ? payload : {};
                    const path = resolveLuaPath(payloadObj.path || '');
                    const isRelayMonitor = payloadObj.command === 'relay-monitor' || path === 'tools/relay-monitor.lua';
                    const isStorageInspector = payloadObj.command === 'storage-inspector' || path === 'tools/storage-inspector.lua';
                    if (isRelayMonitor) {
                        outcome = await runLuaRelayMonitor(Array.isArray(payloadObj.args) ? payloadObj.args : []);
                    } else if (isStorageInspector) {
                        outcome = await runLuaStorageInspector();
                    } else {
                        if (!path) throw new Error('missing lua script path');
                        const script = await readVfsText(path);
                        outcome = await runLuaCode(script, payloadObj.input || {});
                    }
                    printLuaOutcome(outcome);
                    term.write(PROMPT);
                } catch (e) {
                    term.write(`\x1b[31mLua error: ${e && e.message ? e.message : String(e)}\x1b[0m\r\n`);
                    term.write(PROMPT);
                } finally {
                    restPending = false;
                    requestAnimationFrame(saveState);
                }
                return;
            }

            // Check for Voice dispatch sentinel
            const voiceMatch = output.match(VOICE_RE);
            if (voiceMatch) {
                const visible = output.replace(VOICE_RE, '');
                if (visible) term.write(visible);
                try {
                    const { v: voiceName, m: model, a: agent, s: sessionId, rp: returnPrompt, local: localFlag, voxtral: voxtralFlag } = JSON.parse(voiceMatch[1]);
                    restPending = true;

                    // Check if helper is connected (required for native voice with sox)
                    const helperConnected = activeSdk && (activeSdk.helperConnected || activeSdk.helperUrl);
                    
                    // Check for browser voice support (WebAudio + getUserMedia)
                    const browserVoiceSupported = typeof navigator !== 'undefined' && 
                        navigator.mediaDevices && navigator.mediaDevices.getUserMedia;

                    // Check WebGPU support for local voice
                    const webgpuAvailable = typeof navigator !== 'undefined' && !!navigator.gpu;

                    if (!helperConnected && !browserVoiceSupported) {
                        term.write(`\r\n\x1b[33mVoice requires either:\x1b[0m\r\n`);
                        term.write(`  1. A local helper (native sox) — run traits CLI\r\n`);
                        term.write(`  2. Browser voice support — use Chrome/Edge/Safari\r\n`);
                        term.write(returnPrompt);
                        restPending = false;
                        requestAnimationFrame(saveState);
                        return;
                    }

                    // Detect whether to use local voice (WebGPU STT+LLM+TTS) or cloud voice (OpenAI Realtime)
                    // Priority:
                    //   1. Explicit local:true flag in sentinel → always local
                    //   2. User preference localStorage['traits.voice.mode'] = 'realtime' + API key → cloud
                    //   3. User preference localStorage['traits.voice.mode'] = 'local' → local
                    //   4. Auto-fallback: no API key + WebGPU available → local
                    let useLocalVoice = !!localFlag;
                    let useVoxtralVoice = !!voxtralFlag;
                    let hasApiKey = false;
                    try {
                        const settingsKey = (localStorage.getItem('traits.secret.OPENAI_API_KEY') || '').trim();
                        const legacyKey = (localStorage.getItem('traits.voice.api_key') || '').trim();
                        hasApiKey = !!(settingsKey || legacyKey);
                    } catch(_) {}

                    if (!useLocalVoice && !useVoxtralVoice) {
                        // Check stored voice mode preference
                        const storedMode = (localStorage.getItem('traits.voice.mode') || '').trim();
                        if (storedMode === 'realtime' && hasApiKey) {
                            useLocalVoice = false; // explicitly cloud
                        } else if (storedMode === 'local') {
                            useLocalVoice = true;
                        } else if (storedMode === 'local-realtime') {
                            useVoxtralVoice = true;
                        } else if (!helperConnected && browserVoiceSupported && webgpuAvailable && !hasApiKey) {
                            useLocalVoice = true; // auto-fallback
                        }
                    }

                    // ── Voxtral local-realtime mode (Voxtral ONNX STT → LLM → Kokoro TTS) ──
                    if (useVoxtralVoice && browserVoiceSupported) {
                        term.write(`\x1b[90mStarting Voxtral local-realtime voice…\x1b[0m\r\n`);
                        term.write(`\x1b[90mFirst run downloads ~1.5 GB Voxtral model + ~92 MB TTS.\x1b[0m\r\n`);

                        activeSdk.startVoxtralVoice({
                            voice: voiceName || 'af_heart',
                            instructions: agent
                                ? `You are the "${agent}" agent on traits.build. Keep responses very short (1-2 sentences). Be conversational.`
                                : undefined,
                            onTranscript: (text) => {
                                term.write(`\r\n\x1b[92m🎤 ${text}\x1b[0m\r\n`);
                            },
                            onResponse: (text) => {
                                term.write(`\x1b[96m💬 ${text}\x1b[0m\r\n`);
                            },
                            onToolCall: (name, args) => {
                                term.write(`\x1b[93m⚡ ${name.replace(/_/g, '.')}\x1b[0m\r\n`);
                            },
                            onToolResult: (name, resultStr) => {
                                if (name === 'sys_echo') {
                                    try {
                                        const r = JSON.parse(resultStr);
                                        const text = r.text || r.result?.text || '';
                                        if (text) term.write(`\x1b[97m📋 ${text}\x1b[0m\r\n`);
                                    } catch(_) {}
                                }
                            },
                            onProgress: (text) => {
                                if (text) term.write(`\r\x1b[K\x1b[90m⏳ ${text}\x1b[0m`);
                            },
                            onError: (msg) => {
                                term.write(`\r\n\x1b[31mVoxtral voice error: ${msg}\x1b[0m\r\n`);
                            },
                        }).then(result => {
                            if (result.ok) {
                                const toolMsg = result.tools ? `, ${result.tools} tools` : '';
                                term.write(`\r\x1b[K\x1b[90mVoxtral voice active! Speak to start${toolMsg}. Press Esc to stop.\x1b[0m\r\n`);
                                const onVoiceStopped = (e) => {
                                    if (e.detail && e.detail.type === 'stopped') {
                                        window.removeEventListener('voice-event', onVoiceStopped);
                                        term.write(`\r\n\x1b[90mVoxtral voice session ended.\x1b[0m\r\n`);
                                        term.write(returnPrompt);
                                        restPending = false;
                                        requestAnimationFrame(saveState);
                                    }
                                };
                                window.addEventListener('voice-event', onVoiceStopped);
                                const stopHandler = (data) => {
                                    if (data === '\x1b' || data === '\x03') {
                                        activeSdk.stopVoxtralVoice().then(() => {
                                            window.removeEventListener('voice-event', onVoiceStopped);
                                            term.write(`\r\n\x1b[90mVoxtral voice stopped.\x1b[0m\r\n`);
                                            term.write(returnPrompt);
                                            restPending = false;
                                            requestAnimationFrame(saveState);
                                        });
                                        term.offData(stopHandler);
                                    }
                                };
                                term.onData(stopHandler);
                            } else {
                                term.write(`\r\n\x1b[31mVoxtral voice error: ${result.error}\x1b[0m\r\n`);
                                term.write(returnPrompt);
                                restPending = false;
                                requestAnimationFrame(saveState);
                            }
                        });
                        return;
                    }

                    // ── Local voice mode (WebGPU: Whisper STT → LLM → Kokoro TTS) ──
                    if (useLocalVoice && browserVoiceSupported && webgpuAvailable) {
                        term.write(`\x1b[90mStarting local voice…\x1b[0m\r\n`);
                        term.write(`\x1b[90mFirst run downloads ~250 MB of AI models.\x1b[0m\r\n`);

                        activeSdk.startLocalVoice({
                            voice: voiceName || 'af_heart',
                            language: 'en',
                            instructions: agent
                                ? `You are the "${agent}" agent on traits.build. Keep responses very short (1-2 sentences). Be conversational.`
                                : undefined,
                            onTranscript: (text) => {
                                term.write(`\r\n\x1b[92m🎤 ${text}\x1b[0m\r\n`);
                            },
                            onResponse: (text) => {
                                term.write(`\x1b[96m💬 ${text}\x1b[0m\r\n`);
                            },
                            onToolCall: (name, args) => {
                                term.write(`\x1b[93m⚡ ${name.replace(/_/g, '.')}\x1b[0m\r\n`);
                            },
                            onToolResult: (name, resultStr) => {
                                // sys.echo: display the echoed text prominently
                                if (name === 'sys_echo') {
                                    try {
                                        const r = JSON.parse(resultStr);
                                        const text = r.text || r.result?.text || '';
                                        if (text) term.write(`\x1b[97m📋 ${text}\x1b[0m\r\n`);
                                    } catch(_) {}
                                }
                                // sys.canvas: show brief confirmation in terminal
                                if (name === 'sys_canvas') {
                                    try {
                                        const r = JSON.parse(resultStr);
                                        const act = r.action || r.result?.action || '';
                                        if (act) term.write(`\x1b[96m🎨 canvas ${act} (${r.length || r.result?.length || 0} bytes)\x1b[0m\r\n`);
                                    } catch(_) {}
                                }
                            },
                            onProgress: (text) => {
                                if (text) term.write(`\r\x1b[K\x1b[90m⏳ ${text}\x1b[0m`);
                            },
                            onError: (msg) => {
                                term.write(`\r\n\x1b[31mLocal voice error: ${msg}\x1b[0m\r\n`);
                            },
                        }).then(result => {
                            if (result.ok) {
                                const toolMsg = result.tools ? `, ${result.tools} tools` : '';
                                term.write(`\r\x1b[K\x1b[90mLocal voice active! Speak to start${toolMsg}. Press Esc to stop.\x1b[0m\r\n`);
                                // Listen for voice-event 'stopped'
                                const onVoiceStopped = (e) => {
                                    if (e.detail && e.detail.type === 'stopped') {
                                        window.removeEventListener('voice-event', onVoiceStopped);
                                        term.write(`\r\n\x1b[90mLocal voice session ended.\x1b[0m\r\n`);
                                        term.write(returnPrompt);
                                        restPending = false;
                                        requestAnimationFrame(saveState);
                                    }
                                };
                                window.addEventListener('voice-event', onVoiceStopped);
                                // Esc key handler
                                const stopHandler = (data) => {
                                    if (data === '\x1b' || data === '\x03') {
                                        activeSdk.stopLocalVoice().then(() => {
                                            window.removeEventListener('voice-event', onVoiceStopped);
                                            term.write(`\r\n\x1b[90mLocal voice stopped.\x1b[0m\r\n`);
                                            term.write(returnPrompt);
                                            restPending = false;
                                            requestAnimationFrame(saveState);
                                        });
                                        term.offData(stopHandler);
                                    }
                                };
                                term.onData(stopHandler);
                            } else {
                                term.write(`\r\n\x1b[31mLocal voice error: ${result.error}\x1b[0m\r\n`);
                                term.write(returnPrompt);
                                restPending = false;
                                requestAnimationFrame(saveState);
                            }
                        });
                        return;
                    }

                    // ── Cloud voice mode (OpenAI Realtime via WebRTC) ──
                    if (!helperConnected && browserVoiceSupported) {
                        term.write(`\x1b[90mStarting browser voice with ${voiceName}…\x1b[0m\r\n`);
                        activeSdk.startVoice({
                            voice: voiceName,
                            model: model || 'gpt-realtime-mini-2025-12-15',
                            agent: agent || '',
                            onTranscript: (text) => {
                                term.write(`\r\n\x1b[92m🎤 ${text}\x1b[0m\r\n`);
                            },
                            onResponse: (text) => {
                                term.write(`\x1b[96m💬 ${text}\x1b[0m\r\n`);
                            },
                            onToolCall: (name, args) => {
                                term.write(`\x1b[93m⚡ ${name.replace(/_/g, '.')}\x1b[0m\r\n`);
                            },
                            onToolResult: (name, resultStr) => {
                                // sys.echo: display the echoed text prominently
                                if (name === 'sys_echo') {
                                    try {
                                        const r = JSON.parse(resultStr);
                                        const text = r.text || r.result?.text || '';
                                        if (text) term.write(`\x1b[97m📋 ${text}\x1b[0m\r\n`);
                                    } catch(_) {}
                                }
                                // sys.canvas: show brief confirmation in terminal
                                if (name === 'sys_canvas') {
                                    try {
                                        const r = JSON.parse(resultStr);
                                        const act = r.action || r.result?.action || '';
                                        if (act) term.write(`\x1b[96m🎨 canvas ${act} (${r.length || r.result?.length || 0} bytes)\x1b[0m\r\n`);
                                    } catch(_) {}
                                }
                            },
                            onError: (msg) => {
                                term.write(`\x1b[31mVoice error: ${msg}\x1b[0m\r\n`);
                            },
                        }).then(result => {
                            if (result.ok) {
                                const toolMsg = result.tools ? `, ${result.tools} tools` : '';
                                term.write(`\x1b[90mVoice active! Speak to start conversation${toolMsg}. Press Esc to stop.\x1b[0m\r\n`);
                                // Listen for voice-event 'stopped' (model quit or disconnect)
                                const onVoiceStopped = (e) => {
                                    if (e.detail && e.detail.type === 'stopped') {
                                        window.removeEventListener('voice-event', onVoiceStopped);
                                        term.write(`\r\n\x1b[90mVoice session ended.\x1b[0m\r\n`);
                                        term.write(returnPrompt);
                                        restPending = false;
                                        requestAnimationFrame(saveState);
                                    }
                                };
                                window.addEventListener('voice-event', onVoiceStopped);
                                // Setup Esc key handler to stop voice
                                const stopVoiceHandler = (data) => {
                                    // Esc = \x1b (alone, not followed by [ which is an arrow key)
                                    if (data === '\x1b' || data === '\x03') {
                                        activeSdk.stopVoice().then(() => {
                                            window.removeEventListener('voice-event', onVoiceStopped);
                                            term.write(`\r\n\x1b[90mVoice stopped.\x1b[0m\r\n`);
                                            term.write(returnPrompt);
                                            restPending = false;
                                            requestAnimationFrame(saveState);
                                        });
                                        term.offData(stopVoiceHandler);
                                    }
                                };
                                term.onData(stopVoiceHandler);
                            } else {
                                term.write(`\x1b[31mVoice error: ${result.error}\x1b[0m\r\n`);
                                term.write(returnPrompt);
                                restPending = false;
                                requestAnimationFrame(saveState);
                            }
                        });
                        return;
                    }

                    // Helper is connected - dispatch native voice call
                    term.write(`\x1b[90mStarting voice with ${voiceName}…\x1b[0m\r\n`);
                    const args = [voiceName, model || 'gpt-realtime-mini-2025-12-15', agent || '', sessionId || ''];
                    activeSdk.call('sys.voice', args).then(res => {
                        term.write('\r\x1b[K');
                        if (res.ok && res.result !== undefined) {
                            const text = typeof res.result === 'string' ? res.result : JSON.stringify(res.result, null, 2);
                            term.write(text.replace(/\n/g, '\r\n'));
                            if (!text.endsWith('\n')) term.write('\r\n');
                        } else if (res.error) {
                            term.write(`\x1b[31mVoice error: ${res.error}\x1b[0m\r\n`);
                        }
                        term.write(returnPrompt);
                    }).catch(e => {
                        term.write(`\x1b[31mVoice dispatch error: ${e.message}\x1b[0m\r\n`);
                        term.write(returnPrompt);
                    }).finally(() => { restPending = false; requestAnimationFrame(saveState); });
                } catch (e) {
                    term.write(`\x1b[31mVoice parse error: ${e.message}\x1b[0m\r\n`);
                    term.write(PROMPT);
                    restPending = false;
                    requestAnimationFrame(saveState);
                }
                return;
            }

            if (output.includes(CLEAR_SENTINEL)) {
                term.clear();
                const rest = output.replaceAll(CLEAR_SENTINEL, '');
                if (rest) term.write(rest);
                try { localStorage.removeItem(LS_SCROLLBACK); } catch (_) {}
            } else {
                term.write(output);
                // Save after a command completes (output contains newline from Enter)
                if (data.includes('\r') || data.includes('\n')) requestAnimationFrame(saveState);
            }
        }).catch(e => {
            term.write(`\x1b[31mTerminal IO error: ${e.message}\x1b[0m\r\n`);
            term.write(PROMPT);
        });
    });

    // ── External terminal input (sys.spa "terminal" action) ──
    window.addEventListener('traits-terminal-input', (e) => {
        const text = e.detail?.data;
        if (!text || !backgroundCall || restPending) return;
        ioChain = ioChain.then(async () => {
            const inputRes = await backgroundCall('cli_input', { data: text });
            if (!inputRes?.ok) return;
            const output = inputRes.result || '';
            if (output) term.write(output);
        }).catch(e => {
            console.error('[terminal] external input error:', e);
        });
    });

    // ── Restore history + VFS into WASM session ──
    if (persistence?.restoreSession) {
        await persistence.restoreSession();
    } else {
        const savedHistory = localStorage.getItem(LS_HISTORY);
        if (savedHistory && backgroundCall) {
            try { await backgroundCall('cli_set_history', { history_json: savedHistory }); } catch (_) {}
        }
        let savedVfs = localStorage.getItem(LS_PVFS);
        if (!savedVfs) {
            savedVfs = localStorage.getItem(LS_VFS_LEGACY);
            if (savedVfs) {
                try { localStorage.setItem(LS_PVFS, savedVfs); } catch (_) {}
            }
        }
        if (savedVfs && backgroundCall) {
            try { await backgroundCall('pvfs_load', { json: savedVfs }); } catch (_) {}
        }
    }

    // ── Restore scrollback or show welcome ──
    const savedScrollback = localStorage.getItem(LS_SCROLLBACK);
    if (savedScrollback) {
        term.write(savedScrollback);
    } else if (backgroundCall) {
        const welcome = await backgroundCall('cli_welcome');
        if (welcome?.ok && welcome.result) {
            term.write(welcome.result);
        } else {
            term.writeln('\x1b[33mWASM kernel not loaded — commands unavailable\x1b[0m');
        }
    } else {
        term.writeln('\x1b[33mWASM kernel not loaded — commands unavailable\x1b[0m');
    }

    return { term, fitAddon, wasm };
}
