use maud::{html, DOCTYPE, PreEscaped};
use serde_json::Value;

pub fn spa(_args: &[Value]) -> Value {
    let markup = html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { "slob.games — Settings" }
                style { (PreEscaped(CSS)) }
            }
            body {
                div.page {
                    section.hero.card {
                        p.eyebrow { "settings" }
                        h1 { "slob.games" }
                        p.subtitle {
                            "Manage your games, secrets, and runtime."
                        }
                        div.badges {
                            span.badge id="platformBadge" { "detecting..." }
                            span.badge id="runtimeBadge" { "wasm" }
                            span.badge id="versionBadge" { "—" }
                        }
                    }

                    // Kernel info (enriched)
                    section.card id="kernelCard" data-trait="sys.list" data-handler="refreshStats" data-interval="30000" {
                        h2 { "Kernel" }
                        div.kernel-grid {
                            div.kstat {
                                span.kstat-value id="traitCount" { "—" }
                                span.kstat-label { "traits" }
                            }
                            div.kstat {
                                span.kstat-value id="namespaceCount" { "—" }
                                span.kstat-label { "namespaces" }
                            }
                            div.kstat {
                                span.kstat-value id="callableCount" { "—" }
                                span.kstat-label { "callable" }
                            }
                            div.kstat {
                                span.kstat-value id="buildVersion" { "—" }
                                span.kstat-label { "version" }
                            }
                        }
                        table.stats style="margin-top:14px;" {
                            tr { td { "Runtime" } td id="runtimeMode" { "—" } }
                            tr { td { "Dispatch" } td id="dispatchPath" { "—" } }
                            tr { td { "Storage" } td id="storageMode" { "—" } }
                        }
                    }

                    // Games list
                    section.card {
                        h2 { "Games" }
                        p.note { "Your games from the relay. Log in to see your collection." }
                      p.note id="gamesSummary" { "—" }
                        div id="gamesList" { p.muted { "Loading games…" } }
                    }

                    // Secrets & Environment
                    div.grid {
                      section.card {
                        h2 { "Account" }
                        p.note { "Register/login for a user token used by relay sync and private internal rooms." }
                        div.form-row {
                          input id="authUsername" type="text" placeholder="Username";
                          input id="authPassword" type="password" placeholder="Password";
                        }
                        div.form-row {
                          input id="authEmail" type="email" placeholder="Email (for register)";
                          button.primary onclick="registerUser()" { "Register" }
                          button onclick="loginUser()" { "Login" }
                        }
                        p.inline-status id="authStatus" {}
                      }

                      section.card {
                        h2 { "Models" }
                        p.note { "Voice and canvas LLM model preferences. Stored in this browser." }
                        div.form-row {
                          label.select-label for="voiceModelSelect" { "Voice Model" }
                          select id="voiceModelSelect" onchange="saveModelPref('SLOB_VOICE_MODEL', this.value)" {
                            option value="gpt-realtime-mini-2025-12-15" { "realtime-mini (default)" }
                            option value="gpt-4o-realtime-preview" { "gpt-4o-realtime (quality)" }
                            option value="gpt-4o-mini-realtime-preview" { "gpt-4o-mini-realtime" }
                          }
                        }
                        div.form-row {
                          label.select-label for="voiceNameSelect" { "Voice" }
                          select id="voiceNameSelect" onchange="saveModelPref('SLOB_VOICE_NAME', this.value)" {
                            option value="shimmer" { "Shimmer" }
                            option value="alloy" { "Alloy" }
                            option value="ash" { "Ash" }
                            option value="ballad" { "Ballad" }
                            option value="coral" { "Coral" }
                            option value="echo" { "Echo" }
                            option value="sage" { "Sage" }
                            option value="verse" { "Verse" }
                          }
                        }
                        div.form-row {
                          label.select-label for="canvasModelSelect" { "Canvas LLM" }
                          select id="canvasModelSelect" onchange="saveModelPref('SLOB_CANVAS_MODEL', this.value)" {
                            option value="gpt-4.1" { "gpt-4.1 (default)" }
                            option value="gpt-4.1-mini" { "gpt-4.1-mini (fast)" }
                            option value="gpt-4.1-nano" { "gpt-4.1-nano (budget)" }
                            option value="gpt-5.4" { "gpt-5.4" }
                            option value="gpt-5.3-codex" { "gpt-5.3-codex" }
                            option value="gpt-5.3-mini" { "gpt-5.3-mini" }
                            option value="gpt-4o" { "gpt-4o" }
                            option value="gpt-4o-mini" { "gpt-4o-mini" }
                            option value="o3" { "o3 (reasoning)" }
                          }
                        }
                        p.inline-status id="modelStatus" {}
                      }

                        section.card {
                            h2 { "Secrets" }
                            p.note {
                            "Local secrets in " code { "localStorage" } ". Account secrets sync from relay on login."
                            }
                            table id="secretTable" {
                                tr { td colspan="3" { "Loading…" } }
                            }
                            div.form-row {
                            input id="secretKey" type="text" placeholder="Secret ID (e.g. SLOB_USER_TOKEN)";
                                input id="secretValue" type="password" placeholder="Secret value";
                                button.primary onclick="saveSecret()" { "Store" }
                            }
                            p.inline-status id="secretStatus" {}
                        }

                        section.card {
                            h2 { "Environment" }
                            p.note {
                            "Stored in this browser under " code { "localStorage" } ". Not encrypted. Set " code { "SLOB_USERNAME" } " for game identity."
                            }
                            table id="envTable" {
                                tr { td colspan="4" { "Loading…" } }
                            }
                            div.form-row {
                            input id="envKey" type="text" placeholder="Variable name (e.g. SLOB_USERNAME)";
                                input id="envValue" type="text" placeholder="Variable value";
                                button.primary onclick="saveEnvVar()" { "Store" }
                            }
                            p.inline-status id="envStatus" {}
                        }
                    }
                }

                script { (PreEscaped(JS)) }
            }
        }
    };

    Value::String(markup.into_string())
}

const CSS: &str = r##"
:root {
  --bg: #0a0a0f;
  --panel: #111118;
  --panel-2: #16161f;
  --line: #1e1e2e;
  --text: #e8e6e3;
  --muted: #5a6570;
  --accent: #00e0ff;
  --green: #00ff88;
  --warn: #f5b942;
  --danger: #ef6b73;
}

* { box-sizing: border-box; }
body {
  margin: 0;
  background:
    radial-gradient(circle at top left, rgba(0,224,255,0.05), transparent 28%),
    linear-gradient(180deg, #060610 0%, var(--bg) 100%);
  color: var(--text);
  font-family: system-ui, -apple-system, sans-serif;
}
.page {
  max-width: 1120px;
  margin: 0 auto;
  padding: 32px 20px 48px;
}
.grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
  gap: 18px;
  margin-bottom: 18px;
}
.card {
  background: linear-gradient(180deg, rgba(17,17,26,0.97), rgba(12,12,18,0.97));
  border: 1px solid rgba(0,224,255,0.07);
  border-radius: 14px;
  padding: 20px;
  box-shadow: 0 20px 48px rgba(0,0,0,0.3);
  margin-bottom: 18px;
}
.hero {
  position: relative;
  overflow: hidden;
}
.hero::after {
  content: "";
  position: absolute;
  inset: auto -40px -60px auto;
  width: 180px; height: 180px;
  border-radius: 999px;
  background: radial-gradient(circle, rgba(0,224,255,0.1), transparent 70%);
  pointer-events: none;
}
.eyebrow {
  margin: 0 0 8px;
  color: var(--accent);
  text-transform: uppercase;
  letter-spacing: 0.16em;
  font-size: 11px;
  font-family: 'Courier New', Menlo, monospace;
}
h1 {
  margin: 0;
  font-family: 'Courier New', Menlo, monospace;
  font-size: clamp(34px, 5vw, 56px);
  line-height: 0.96;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}
h2 {
  margin: 0 0 12px;
  font-family: 'Courier New', Menlo, monospace;
  font-size: 18px;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--accent);
}
.subtitle {
  margin: 14px 0 0;
  max-width: 760px;
  color: var(--muted);
  line-height: 1.6;
}
.badges {
  display: flex; gap: 8px; flex-wrap: wrap;
  margin-top: 16px;
}
.badge {
  border: 1px solid rgba(0,224,255,0.18);
  color: var(--accent);
  border-radius: 999px;
  padding: 6px 10px;
  font-size: 12px;
  letter-spacing: 0.04em;
  font-family: 'Courier New', Menlo, monospace;
}
.notice, .note, .inline-status {
  color: var(--muted);
  line-height: 1.5; font-size: 14px;
}
table {
  width: 100%;
  border-collapse: collapse;
}
td {
  padding: 10px 0;
  border-bottom: 1px solid rgba(30,30,46,0.65);
  vertical-align: top;
}
td:first-child {
  width: 34%;
  color: var(--muted);
}
.form-row {
  display: flex; flex-wrap: wrap; gap: 10px;
  margin-top: 16px;
}
input, select, button {
  border-radius: 10px;
  border: 1px solid var(--line);
  background: rgba(10,10,16,0.92);
  color: var(--text);
  font: inherit;
}
input {
  min-width: 180px; flex: 1 1 220px;
  padding: 12px 14px;
}
select {
  min-width: 180px; flex: 1 1 220px;
  padding: 12px 14px;
  appearance: none;
  -webkit-appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='8'%3E%3Cpath d='M1 1l5 5 5-5' stroke='%235a6570' stroke-width='1.5' fill='none'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 14px center;
}
.select-label {
  display: block;
  width: 100%;
  font-size: 12px;
  color: var(--muted);
  text-transform: uppercase;
  letter-spacing: 0.06em;
  margin-bottom: -6px;
  font-family: 'Courier New', Menlo, monospace;
}
button {
  padding: 10px 14px;
  cursor: pointer;
}
button:hover { border-color: #3d5b6c; }
button.primary {
  background: linear-gradient(180deg, rgba(0,224,255,0.18), rgba(0,180,220,0.12));
  border-color: rgba(0,224,255,0.25);
  color: #00e0ff;
}
button.danger {
  background: linear-gradient(180deg, #6b2530, #531b24);
  border-color: rgba(239,107,115,0.26);
  color: #ef6b73;
}
code {
  font-family: 'Courier New', Menlo, monospace;
  font-size: 13px;
  color: #66f0ff;
  background: rgba(0,224,255,0.05);
  padding: 3px 6px;
  border-radius: 8px;
}
a { color: #00e0ff; }

/* Kernel stats grid */
.kernel-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
}
.kstat {
  display: flex; flex-direction: column; align-items: center;
  padding: 14px 8px;
  border-radius: 10px;
  background: rgba(0,224,255,0.03);
  border: 1px solid rgba(0,224,255,0.08);
}
.kstat-value {
  font-family: 'Courier New', Menlo, monospace;
  font-size: 26px; font-weight: 700;
  color: var(--accent);
  line-height: 1;
}
.kstat-label {
  font-size: 11px;
  color: var(--muted);
  text-transform: uppercase;
  letter-spacing: 0.08em;
  margin-top: 6px;
}

/* Games list */
.game-row {
  display: flex; align-items: center; gap: 12px;
  padding: 12px 0;
  border-bottom: 1px solid rgba(30,30,46,0.5);
}
.game-row:last-child { border-bottom: none; }
.game-info { flex: 1; min-width: 0; }
.game-name {
  font-family: 'Courier New', Menlo, monospace;
  font-size: 14px; font-weight: 700;
  color: var(--text);
  white-space: nowrap;
  overflow: hidden; text-overflow: ellipsis;
}
.game-meta {
  font-size: 11px; color: var(--muted);
  margin-top: 2px;
  display: flex; gap: 12px; flex-wrap: wrap;
}
.game-actions {
  display: flex; gap: 6px; flex-shrink: 0;
}
.game-actions button {
  padding: 5px 10px; font-size: 11px;
  border-radius: 6px;
  text-transform: uppercase;
  letter-spacing: 0.02em;
  font-family: 'Courier New', Menlo, monospace;
}
.btn-play {
  background: rgba(0,255,136,0.1);
  border-color: rgba(0,255,136,0.25);
  color: #00ff88;
}
.btn-play:hover { background: rgba(0,255,136,0.2); }
.btn-build {
  background: rgba(0,224,255,0.1);
  border-color: rgba(0,224,255,0.2);
  color: #00e0ff;
}
.btn-build:hover { background: rgba(0,224,255,0.2); }
.no-games {
  padding: 24px; text-align: center;
  color: var(--muted);
  font-family: 'Courier New', Menlo, monospace;
  font-size: 13px;
}

@media (max-width: 720px) {
  .page { padding: 18px 14px 32px; }
  .card { padding: 16px; border-radius: 12px; }
  .kernel-grid { grid-template-columns: repeat(2, 1fr); }
  td:first-child { width: auto; }
  .game-row { flex-wrap: wrap; }
  .game-actions { width: 100%; justify-content: flex-end; }
}
"##;

const JS: &str = r##"
(function() {
// ═══════════════════════════════════════════════════════════════
// Platform detection (badge only)
// ═══════════════════════════════════════════════════════════════
(function setupCard() {
  var ua = navigator.userAgent || '';
  var p = navigator.platform || '';
  var label = 'Unknown';
  if (/iPhone|iPad|iPod/.test(ua) || (/Mac/.test(p) && 'ontouchend' in document)) {
    label = 'iPhone / iPad';
  } else if (/Android/.test(ua)) {
    label = 'Android';
  } else if (/Mac/.test(p) || /Mac/.test(ua)) {
    label = 'macOS';
  } else if (/Win/.test(p) || /Win/.test(ua)) {
    label = 'Windows';
  } else if (/Linux/.test(p) || /Linux/.test(ua)) {
    label = 'Linux';
  } else if (/CrOS/.test(ua)) {
    label = 'ChromeOS';
  }
  var badge = document.getElementById('platformBadge');
  if (badge) badge.textContent = label;
})();

// ═══════════════════════════════════════════════════════════════
// Storage helpers
// ═══════════════════════════════════════════════════════════════
var SECRET_PFX = 'traits.secret.';
var ENV_PFX = 'traits.env.';
var memoryStorage = (function() {
  var store = {};
  var keys = [];
  return {
    get length() { return keys.length; },
    key: function(i) { return keys[i] || null; },
    getItem: function(k) { return store.hasOwnProperty(k) ? store[k] : null; },
    setItem: function(k, v) { if (!store.hasOwnProperty(k)) keys.push(k); store[k] = String(v); },
    removeItem: function(k) { if (store.hasOwnProperty(k)) { delete store[k]; keys = keys.filter(function(x){return x!==k;}); } }
  };
})();

function resolveStorage() {
  try {
    if (typeof window !== 'undefined' && window.localStorage) {
      var probe = '__traits_spa_probe__';
      window.localStorage.setItem(probe, '1');
      window.localStorage.removeItem(probe);
      return { backend: window.localStorage, persistent: true };
    }
  } catch(_) {}
  return { backend: memoryStorage, persistent: false };
}

var storageState = resolveStorage();
var storage = storageState.backend;

function esc(v) {
  var d = document.createElement('div');
  d.textContent = String(v == null ? '' : v);
  return d.innerHTML;
}
function byId(id) { return document.getElementById(id); }

function callTrait(path, args) {
  if (window._traitsSDK) {
    return window._traitsSDK.call(path, args || []);
  }
  return Promise.resolve({ ok: false, error: 'unavailable' });
}

function storageEntries(prefix) {
  var out = [];
  for (var i = 0; i < storage.length; i++) {
    var key = storage.key(i);
    if (key && key.indexOf(prefix) === 0) {
      out.push({ key: key.slice(prefix.length), value: storage.getItem(key) || '' });
    }
  }
  out.sort(function(a, b) { return a.key.localeCompare(b.key); });
  return out;
}

function previewValue(v) {
  if (!v) return '<span style="color:#555">(empty)</span>';
  if (v.length <= 48) return '<span style="color:#ccc">' + esc(v) + '</span>';
  return '<span style="color:#ccc">' + esc(v.slice(0, 28)) + ' … ' + esc(v.slice(-12)) + '</span>';
}

// ═══════════════════════════════════════════════════════════════
// Kernel stats (enriched from dispatch info)
// ═══════════════════════════════════════════════════════════════
TC.on('refreshStats', function(el, traits, meta) {
  var ns = {};
  var callable = 0;
  for (var i = 0; i < traits.length; i++) {
    var parts = (traits[i].path || '').split('.');
    if (parts[0]) ns[parts[0]] = true;
    if (traits[i].callable !== false) callable++;
  }
  var nsCount = Object.keys(ns).length;

  callTrait('sys.version', []).then(function(ver) {
    var version = ver.ok
      ? (ver.result.version || ver.result.date || JSON.stringify(ver.result))
      : '—';

    byId('traitCount').textContent = String(traits.length);
    byId('namespaceCount').textContent = String(nsCount);
    byId('callableCount').textContent = String(callable);
    byId('buildVersion').textContent = String(version || '—');
    byId('versionBadge').textContent = 'v' + String(version || '—');

    var dispatch = meta.dispatch || 'wasm';
    byId('runtimeMode').textContent = dispatch;
    byId('runtimeBadge').textContent = dispatch;

    var sdk = window._traitsSDK;
    var tiers = [];
    if (sdk) {
      var s = sdk.status;
      if (s.wasm) tiers.push('WASM (' + callable + ')');
      if (s.helper) tiers.push('Helper');
      if (s.relay) tiers.push('Relay');
    }
    byId('dispatchPath').textContent = tiers.length ? tiers.join(' → ') : dispatch;
    byId('storageMode').textContent = storageState.persistent ? 'localStorage (persistent)' : 'memory (session only)';
  });
});

// ═══════════════════════════════════════════════════════════════
// Games list — relay-backed with local fallback
// ═══════════════════════════════════════════════════════════════
function getAuthToken() {
  try { return localStorage.getItem('traits.secret.SLOB_USER_TOKEN') || ''; } catch(_) { return ''; }
}
function authHeaders() {
  var h = { 'Content-Type': 'application/json' };
  var t = getAuthToken();
  if (t) h['Authorization'] = 'Bearer ' + t;
  return h;
}

var _relayGames = null; // cached relay response

async function fetchRelayGames() {
  var token = getAuthToken();
  var internal = [];
  var external = [];
  try {
    var r1 = await fetch(relayApiBase() + '/games');
    if (r1.ok) external = await r1.json();
  } catch(_) {}
  if (token) {
    try {
      var r2 = await fetch('https://relay.traits.build/sync/internal/games', {
        headers: authHeaders()
      });
      if (r2.ok) internal = await r2.json();
    } catch(_) {}
  }
  _relayGames = { internal: internal, external: external };
  return _relayGames;
}

function readGamesCollection() {
  try {
    var raw = localStorage.getItem('traits.pvfs');
    if (!raw) return { active: null, games: {} };
    var files = JSON.parse(raw);
    var json = files['canvas/games.json'];
    if (!json) return { active: null, games: {} };
    return JSON.parse(json);
  } catch(_) { return { active: null, games: {} }; }
}

function slugify(s) {
  return String(s || '')
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '') || 'untitled';
}

function upsertInternalRelayGameToLocal(owner, gameId, data) {
  try {
    var raw = localStorage.getItem('traits.pvfs') || '{}';
    var files = JSON.parse(raw);
    var col = files['canvas/games.json']
      ? JSON.parse(files['canvas/games.json'])
      : { active: null, games: {} };

    var o = owner || data.owner || 'user';
    var gid = gameId || data.game_id || slugify(data.name || 'untitled');
    var identityKey = (o + '|' + gid).toLowerCase();
    var targetId = 'ri-' + slugify(o + '-' + gid).slice(0, 44);

    // Reuse an existing entry for this relay internal identity if present.
    var existingGame = null;
    for (var id in (col.games || {})) {
      if (!col.games.hasOwnProperty(id)) continue;
      var g = col.games[id] || {};
      var k = ((g._sync_owner || g.owner || '') + '|' + (g._sync_game_id || g.game_id || '')).toLowerCase();
      if (k && k === identityKey) {
        targetId = id;
        existingGame = g;
        break;
      }
    }

    // CRITICAL: If local game exists with content that differs from relay version,
    // preserve the local content (user may have edited it locally).
    // Only sync name/metadata from relay, not content.
    var localContent = (existingGame && existingGame.content) || '';
    var contentToUse = localContent && localContent !== data.content ? localContent : (data.content || '');

    col.games[targetId] = {
      name: data.name || gid,
      content: contentToUse,
      scope: 'internal',
      _scope: 'internal',
      owner: o,
      game_id: gid,
      _sync_owner: o,
      _sync_game_id: gid,
      _sync_hash: data.content_hash || data.checksum || '',
      version: data.version || 'v1',
      published: !!data.published,
      checksum: data.checksum || data.content_hash || '',
      created: (existingGame && existingGame.created) || data.updated || new Date().toISOString(),
      updated: (existingGame && contentToUse === localContent) ? (existingGame.updated || data.updated) : (data.updated || new Date().toISOString())
    };

    col.active = targetId;
    files['canvas/games.json'] = JSON.stringify(col);
    files['canvas/app.html'] = data.content || '';
    localStorage.setItem('traits.pvfs', JSON.stringify(files));
    return targetId;
  } catch(_) {
    return null;
  }
}

function formatSize(bytes) {
  if (bytes < 1024) return bytes + ' B';
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
}

function ago(ts) {
  if (!ts) return '—';
  var d = new Date(ts);
  var now = Date.now();
  var s = Math.floor((now - d.getTime()) / 1000);
  if (s < 60) return 'just now';
  if (s < 3600) return Math.floor(s/60) + 'm ago';
  if (s < 86400) return Math.floor(s/3600) + 'h ago';
  if (s < 2592000) return Math.floor(s/86400) + 'd ago';
  return d.toLocaleDateString();
}

async function renderGames() {
  var el = byId('gamesList');
  var summary = byId('gamesSummary');
  if (!el) return;

  // Try relay first
  var data = _relayGames;
  if (!data) {
    try { data = await fetchRelayGames(); } catch(_) {}
  }

  var hasRelay = data && (data.internal.length > 0 || data.external.length > 0);

  if (hasRelay) {
    renderRelayGames(data, el, summary);
  } else {
    renderLocalGames(el, summary);
  }
}

function renderRelayGames(data, el, summary) {
  var internal = (data.internal || []).slice().sort(function(a, b) {
    return String(a.name || '').localeCompare(String(b.name || ''));
  });
  var external = (data.external || []).slice().sort(function(a, b) {
    return String(a.name || '').localeCompare(String(b.name || ''));
  });
  if (summary) {
    summary.textContent = 'Internal: ' + internal.length + ' · External: ' + external.length + ' (relay)';
  }
  if (!internal.length && !external.length) {
    el.innerHTML = '<div class="no-games">No games on relay yet.</div>';
    return;
  }
  var html = '';
  if (internal.length) {
    html += '<div style="margin-bottom:6px;color:var(--accent);font-size:11px;text-transform:uppercase;letter-spacing:.1em">Internal</div>';
    for (var i = 0; i < internal.length; i++) {
      var g = internal[i];
      var name = esc(g.name || 'untitled');
      var size = formatSize(g.size || 0);
      var identity = esc((g.owner || '?') + '/' + (g.game_id || '?'));
      var hash = (g.content_hash || '').slice(0, 8);
      var ver = esc(g.version || '—');
      var pub = !!g.published;
      html += '<div class="game-row">';
      html += '<div class="game-info">';
      html += '<div class="game-name" style="cursor:pointer" onclick="playRelayGame(\'' + esc(g.owner) + '\',\'' + esc(g.game_id) + '\')">' + name + '</div>';
      html += '<div class="game-meta">';
      html += '<span>' + identity + '</span>';
      html += '<span>v' + ver + '</span>';
      html += '<span>' + size + '</span>';
      html += '<span>' + ago(g.updated) + '</span>';
      html += '<span>' + (pub ? 'published' : 'private') + '</span>';
      html += '<span style="opacity:0.5">#' + hash + '</span>';
      html += '</div></div>';
      html += '<div class="game-actions">';
      html += '<button class="btn-play" onclick="playRelayGame(\'' + esc(g.owner) + '\',\'' + esc(g.game_id) + '\')">Play</button>';
      html += '<button onclick="togglePublishInternal(\'' + esc(g.owner) + '\',\'' + esc(g.game_id) + '\',' + (pub ? 'false' : 'true') + ')">' + (pub ? 'Published' : 'Publish') + '</button>';
      html += '<button class="danger" onclick="deleteRelayGame(\'' + esc(g.owner) + '\',\'' + esc(g.game_id) + '\',\'' + name.replace(/'/g, "\\'") + '\')">Del</button>';
      html += '</div></div>';
    }
  }
  if (external.length) {
    html += '<div style="margin:12px 0 6px;color:var(--muted);font-size:11px;text-transform:uppercase;letter-spacing:.1em">External (public)</div>';
    for (var j = 0; j < external.length; j++) {
      var ge = external[j];
      var ename = esc(ge.name || 'untitled');
      var esize = formatSize(ge.size || 0);
      var ehash = (ge.content_hash || '').slice(0, 8);
      var ever = esc(ge.version || '—');
      html += '<div class="game-row">';
      html += '<div class="game-info">';
      html += '<div class="game-name" style="cursor:pointer" onclick="playExternalGame(\'' + esc(ge.content_hash) + '\')">' + ename + '</div>';
      html += '<div class="game-meta">';
      html += '<span>v' + ever + '</span>';
      html += '<span>' + esize + '</span>';
      html += '<span>' + ago(ge.updated) + '</span>';
      html += '<span style="opacity:0.5">#' + ehash + '</span>';
      html += '</div></div>';
      html += '<div class="game-actions">';
      html += '<button class="btn-play" onclick="playExternalGame(\'' + esc(ge.content_hash) + '\')">Play</button>';
      html += '</div></div>';
    }
  }
  el.innerHTML = html;
}

async function togglePublishInternal(owner, gameId, publish) {
  function _gamesMsg(msg) {
    var gs = byId('gamesSummary');
    if (gs) gs.textContent = msg;
  }
  var me = (storage.getItem(ENV_PFX + 'SLOB_USERNAME') || '').trim();
  if (!me || String(me).toLowerCase() !== String(owner || '').toLowerCase()) {
    _gamesMsg('Only the owner can change publish state.');
    return;
  }
  try {
    var path = 'https://relay.traits.build/sync/internal/game/' + encodeURIComponent(gameId) + '/publish';
    var body = null;
    if (publish) {
      try {
        var ver = await callTrait('sys.version', ['hhmmss']);
        var v = (ver && (ver.version || (ver.result && ver.result.version))) || '';
        if (v) body = { version: v };
      } catch (_) {}
    }
    var r = await fetch(path, {
      method: publish ? 'PUT' : 'DELETE',
      headers: authHeaders(),
      body: body ? JSON.stringify(body) : undefined
    });
    var data = await r.json().catch(function(){ return {}; });
    if (!r.ok) {
      _gamesMsg(data.error || 'Publish toggle failed');
      return;
    }
    _relayGames = null;
    await renderGames();
    _gamesMsg(publish ? 'Game published.' : 'Game set to private.');
  } catch (_) {
    _gamesMsg('Publish toggle request failed');
  }
}

function renderLocalGames(el, summary) {
  var col = readGamesCollection();
  var games = [];
  var gObj = col.games || {};
  var total = 0;
  for (var id in gObj) {
    if (gObj.hasOwnProperty(id)) {
      total++;
      games.push([id, gObj[id] || {}]);
    }
  }
  if (summary) {
    summary.textContent = total + ' game(s) in local cache (offline mode)';
  }
  if (!games.length) {
    el.innerHTML = '<div class="no-games">No games cached locally. Connect to relay and log in.</div>';
    return;
  }
  // No dedup — show all raw entries sorted by name so nothing is hidden
  var unique = games.slice().sort(function(a, b) { return ((a[1] && a[1].name) || '').localeCompare((b[1] && b[1].name) || ''); });
  var html = '';
  for (var i = 0; i < unique.length; i++) {
    var gid = unique[i][0];
    var g = unique[i][1] || {};
    var name = esc(g.name || 'untitled');
    var size = formatSize((g.content || '').length);
    var scope = g.scope || 'local';
    var ver = esc(g.version || '—');
    var active = gid === col.active ? ' <span style="color:#00ff88;font-size:10px;">● ACTIVE</span>' : '';
    html += '<div class="game-row">';
    html += '<div class="game-info">';
    html += '<div class="game-name">' + name + active + '</div>';
    html += '<div class="game-meta">';
    html += '<span>' + scope + '</span>';
    html += '<span>v' + ver + '</span>';
    html += '<span>' + size + '</span>';
    html += '<span>' + ago(g.updated) + '</span>';
    html += '<span style="opacity:0.4;font-size:10px;font-family:monospace">' + esc(gid) + '</span>';
    html += '</div></div>';
    html += '<div class="game-actions">';
    html += '<button class="btn-play" onclick="playGame(\'' + esc(gid) + '\')">Play</button>';
    html += '<button class="danger" onclick="deleteGame(\'' + esc(gid) + '\',\'' + name.replace(/'/g, "\\'") + '\')">Del</button>';
    html += '</div></div>';
  }
  el.innerHTML = html;
}

function playGame(id) {
  callTrait('sys.canvas', ['activate', id]).then(function() {
    if (location.protocol === 'file:') {
      sessionStorage.setItem('traits.shell.route', '/');
      location.hash = '#/';
    } else {
      history.pushState({ route: '/' }, '', '/');
    }
    window.dispatchEvent(new PopStateEvent('popstate', { state: { route: '/' } }));
  });
}

function playRelayGame(owner, gameId) {
  fetch('https://relay.traits.build/sync/internal/game/' + encodeURIComponent(gameId) + '?owner=' + encodeURIComponent(owner), {
    headers: authHeaders()
  }).then(function(r) { return r.json(); }).then(function(data) {
    if (!data.content) { alert('Could not load game'); return; }
    var localId = upsertInternalRelayGameToLocal(owner, gameId, data);
    var isNewGame = !localId;
    var activate = localId
      ? callTrait('sys.canvas', ['activate', localId])
      : callTrait('sys.canvas', ['new', data.name || gameId, data.version || '']);
    activate.then(function() {
      // CRITICAL: Only call 'set' for NEW games. Existing games already have their content
      // from activate or upsert. Calling 'set' would overwrite locally-edited content.
      var finish = isNewGame
        ? callTrait('sys.canvas', ['set', data.content])
        : Promise.resolve();
      finish.then(function() {
        window.dispatchEvent(new CustomEvent('traits-canvas-projects-changed'));
        if (location.protocol === 'file:') {
          sessionStorage.setItem('traits.shell.route', '/');
          location.hash = '#/';
        } else {
          history.pushState({ route: '/' }, '', '/');
        }
        window.dispatchEvent(new PopStateEvent('popstate', { state: { route: '/' } }));
      });
    });
  }).catch(function() { alert('Failed to load game from relay'); });
}

function playExternalGame(hash) {
  fetch(relayApiBase() + '/game/' + encodeURIComponent(hash)).then(function(r) {
    return r.json();
  }).then(function(data) {
    if (!data.content) { alert('Could not load game'); return; }
    callTrait('sys.canvas', ['new', data.name || 'Game', data.version || '']).then(function() {
      callTrait('sys.canvas', ['set', data.content]).then(function() {
        if (location.protocol === 'file:') {
          sessionStorage.setItem('traits.shell.route', '/');
          location.hash = '#/';
        } else {
          history.pushState({ route: '/' }, '', '/');
        }
        window.dispatchEvent(new PopStateEvent('popstate', { state: { route: '/' } }));
      });
    });
  }).catch(function() { alert('Failed to load game from relay'); });
}

function buildGame(id) {
  callTrait('sys.canvas', ['activate', id]).then(function() {
    var route = '/build';
    if (location.protocol === 'file:') {
      sessionStorage.setItem('traits.shell.route', route);
      location.hash = '#' + route;
    } else {
      history.pushState({ route: route }, '', route);
    }
    window.dispatchEvent(new PopStateEvent('popstate', { state: { route: route } }));
  });
}

function deleteGame(id, name) {
  if (!confirm('Delete "' + name + '"? This cannot be undone.')) return;
  callTrait('sys.canvas', ['delete', id]).then(function() {
    renderGames();
  });
}

async function deleteRelayGame(owner, gameId, name) {
  if (!confirm('Delete "' + name + '" from relay? This cannot be undone.')) return;
  try {
    var r = await fetch('https://relay.traits.build/sync/internal/game/' + encodeURIComponent(gameId) + '?owner=' + encodeURIComponent(owner), {
      method: 'DELETE',
      headers: authHeaders()
    });
    if (r.ok) {
      _relayGames = null;
      await renderGames();
    } else {
      var data = null;
      try { data = await r.json(); } catch (_) {}
      alert(data.error || 'Delete failed');
    }
  } catch(e) { alert('Delete request failed'); }
}

// ═══════════════════════════════════════════════════════════════
// Secrets & Environment
// ═══════════════════════════════════════════════════════════════
async function syncRelaySecrets() {
  var token = getAuthToken();
  if (!token) return;
  try {
    var r = await fetch(relayApiBase() + '/auth/secrets', {
      headers: authHeaders()
    });
    if (!r.ok) return;
    var secrets = await r.json();
    if (!Array.isArray(secrets)) return;
    var count = 0;
    for (var i = 0; i < secrets.length; i++) {
      var s = secrets[i];
      if (s.key && s.value) {
        storage.setItem(SECRET_PFX + s.key, s.value);
        count++;
      }
    }
    renderSecrets();
    if (count > 0) setStatus('secretStatus', 'Synced ' + count + ' secret(s) from relay.');
  } catch(_) {}
}

function setStatus(elId, msg, isErr) {
  var el = byId(elId);
  if (!el) return;
  el.textContent = msg || '';
  el.style.color = isErr ? '#ef6b73' : '#5a6570';
  if (msg) setTimeout(function() { if (el.textContent === msg) el.textContent = ''; }, 4000);
}

function renderSecrets() {
  var rows = storageEntries(SECRET_PFX);
  var t = byId('secretTable');
  if (!t) return;
  if (!rows.length) { t.innerHTML = '<tr><td colspan="3">No secrets stored</td></tr>'; return; }
  t.innerHTML = rows.map(function(e) {
    var k = encodeURIComponent(e.key);
    return '<tr><td><code>' + esc(e.key) + '</code></td><td>******</td><td style="text-align:right"><button class="danger" onclick="deleteSecret(decodeURIComponent(\'' + k + '\'))">Del</button></td></tr>';
  }).join('');
}

function renderEnvVars() {
  var rows = storageEntries(ENV_PFX);
  var t = byId('envTable');
  if (!t) return;
  if (!rows.length) { t.innerHTML = '<tr><td colspan="4">No env vars stored</td></tr>'; return; }
  t.innerHTML = rows.map(function(e) {
    var k = encodeURIComponent(e.key);
    return '<tr><td><code>' + esc(e.key) + '</code></td><td>' + previewValue(e.value) + '</td><td>' + e.value.length + '</td><td style="text-align:right"><button class="danger" onclick="deleteEnvVar(decodeURIComponent(\'' + k + '\'))">Del</button></td></tr>';
  }).join('');
}

function saveSecret() {
  var k = byId('secretKey').value.trim();
  var v = byId('secretValue').value;
  if (!k || !v) { setStatus('secretStatus', 'Key and value required.', true); return; }
  storage.setItem(SECRET_PFX + k, v);
  byId('secretKey').value = '';
  byId('secretValue').value = '';
  renderSecrets();
  setStatus('secretStatus', 'Stored.');
}

function deleteSecret(k) {
  storage.removeItem(SECRET_PFX + k);
  renderSecrets();
  setStatus('secretStatus', 'Deleted ' + k + '.');
}

function saveEnvVar() {
  var k = byId('envKey').value.trim();
  var v = byId('envValue').value;
  if (!k) { setStatus('envStatus', 'Name required.', true); return; }
  storage.setItem(ENV_PFX + k, v);
  byId('envKey').value = '';
  byId('envValue').value = '';
  renderEnvVars();
  setStatus('envStatus', 'Stored.');
}

function deleteEnvVar(k) {
  storage.removeItem(ENV_PFX + k);
  renderEnvVars();
  setStatus('envStatus', 'Deleted ' + k + '.');
}

function relayApiBase() {
  return 'https://relay.traits.build/sync';
}

async function registerUser() {
  var username = (byId('authUsername').value || '').trim();
  var email = (byId('authEmail').value || '').trim();
  var password = byId('authPassword').value || '';
  if (!username || !email || !password) {
    setStatus('authStatus', 'Username, email, and password are required.', true);
    return;
  }
  try {
    var r = await fetch(relayApiBase() + '/auth/register', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username: username, email: email, password: password })
    });
    var data = await r.json();
    if (!r.ok || !data.ok) {
      setStatus('authStatus', data.error || 'Register failed.', true);
      return;
    }
    storage.setItem(ENV_PFX + 'SLOB_USERNAME', data.username);
    storage.setItem(SECRET_PFX + 'SLOB_USER_TOKEN', data.token);
    if (data.role) storage.setItem(ENV_PFX + 'SLOB_USER_ROLE', data.role);
    renderEnvVars();
    renderSecrets();
    syncRelaySecrets();
    setStatus('authStatus', 'Registered as ' + data.username + '. Token stored.', false);
  } catch (e) {
    setStatus('authStatus', 'Register request failed.', true);
  }
}

async function loginUser() {
  var username = (byId('authUsername').value || '').trim();
  var password = byId('authPassword').value || '';
  if (!username || !password) {
    setStatus('authStatus', 'Username and password are required.', true);
    return;
  }
  try {
    var r = await fetch(relayApiBase() + '/auth/login', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username: username, password: password })
    });
    var data = await r.json();
    if (!r.ok || !data.ok) {
      setStatus('authStatus', data.error || 'Login failed.', true);
      return;
    }
    storage.setItem(ENV_PFX + 'SLOB_USERNAME', data.username);
    storage.setItem(SECRET_PFX + 'SLOB_USER_TOKEN', data.token);
    if (data.role) storage.setItem(ENV_PFX + 'SLOB_USER_ROLE', data.role);
    renderEnvVars();
    renderSecrets();
    syncRelaySecrets();
    setStatus('authStatus', 'Logged in as ' + data.username + '. Token stored.', false);
  } catch (e) {
    setStatus('authStatus', 'Login request failed.', true);
  }
}

// ═══════════════════════════════════════════════════════════════
// Model preferences
// ═══════════════════════════════════════════════════════════════
function saveModelPref(key, value) {
  storage.setItem(ENV_PFX + key, value);
  setStatus('modelStatus', key + ' → ' + value);
}

function initModelDropdowns() {
  var pairs = [
    ['voiceModelSelect',  'SLOB_VOICE_MODEL'],
    ['voiceNameSelect',   'SLOB_VOICE_NAME'],
    ['canvasModelSelect', 'SLOB_CANVAS_MODEL'],
  ];
  for (var i = 0; i < pairs.length; i++) {
    var el = byId(pairs[i][0]);
    var val = storage.getItem(ENV_PFX + pairs[i][1]);
    if (el && val) {
      for (var j = 0; j < el.options.length; j++) {
        if (el.options[j].value === val) { el.selectedIndex = j; break; }
      }
    }
  }
}

// ═══════════════════════════════════════════════════════════════
// Init
// ═══════════════════════════════════════════════════════════════
try { renderSecrets(); renderEnvVars(); } catch(_) {}
renderGames();
initModelDropdowns();
syncRelaySecrets();
window.addEventListener('traits-canvas-projects-changed', function() { _relayGames = null; renderGames(); });

// Expose to onclick handlers
window.saveSecret = saveSecret;
window.deleteSecret = deleteSecret;
window.saveEnvVar = saveEnvVar;
window.deleteEnvVar = deleteEnvVar;
window.registerUser = registerUser;
window.loginUser = loginUser;
window.playGame = playGame;
window.playRelayGame = playRelayGame;
window.playExternalGame = playExternalGame;
window.buildGame = buildGame;
window.deleteGame = deleteGame;
window.deleteRelayGame = deleteRelayGame;
window.playGame = playGame;
window.buildGame = buildGame;
window.deleteGame = deleteGame;
window.saveModelPref = saveModelPref;

})();
"##;
