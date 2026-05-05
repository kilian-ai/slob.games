use maud::{html, DOCTYPE, PreEscaped};
use serde_json::Value;

pub fn dashboard(_args: &[Value]) -> Value {
    let markup = html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { "slob.games — Admin" }
                style { (PreEscaped(CSS)) }
            }
            body {
                div.page {
                    section.hero.card {
                        p.eyebrow { "admin dashboard" }
                        h1 { "slob.games" }
                        p.subtitle { "Users, games, and system overview." }
                    }

                    section.card {
                        h2 { "Users" }
                        p.note id="usersStatus" { "Loading…" }
                        div id="usersTable" {}
                    }

                    section.card {
                        h2 { "Games" }
                        div.tab-bar {
                            button.tab-btn.active onclick="switchTab('byOwner')" id="tabByOwner" { "By Owner" }
                            button.tab-btn onclick="switchTab('byName')" id="tabByName" { "By Name" }
                        }
                        p.note id="gamesStatus" { "Loading\u{2026}" }
                        p.note id="gamesLegend" { "Public = scope \"external\" AND published = true." }
                        div id="gamesTable" {}
                    }

                    section.card id="pvfsCard" {
                        h2 { "PVFS Games" }
                        p.note id="pvfsStatus" { "Reading local storage\u{2026}" }
                        div id="pvfsTable" {}
                    }

                    section.card id="githubCard" {
                        h2 { "GitHub Catalog" }
                        p.note id="githubStatus" { "Loading published games\u{2026}" }
                        p.note { "Games and sprites stored in the GitHub repo. Disable hides from the carousel; delete removes the JSON, sprite folder, and index entry." }
                        div id="githubTable" {}
                    }

                    section.card id="adminTerminalCard" {
                      h2 { "Traits Terminal" }
                      p.note {
                        "Shared docs/api terminal mounted inside Admin. Use it to call Rig traits directly. "
                        code { "openai_api_key" }
                        " must be present in browser secrets for the OpenAI-backed examples."
                      }
                      div.terminal-examples {
                        div.example-row {
                          div.example-meta {
                            strong { "Rig Providers" }
                            span { "List the Rig-backed WASM shims available in this build." }
                          }
                          div.example-actions {
                            button.btn-sm onclick="copyAdminTerminalCommand('call llm.rig.providers')" { "Copy" }
                            button.btn-sm.accent onclick="runAdminTerminalCommand('call llm.rig.providers')" { "Run" }
                          }
                        }
                        pre.cmd-box { "call llm.rig.providers" }

                        div.example-row {
                          div.example-meta {
                            strong { "Rig Embed" }
                            span { "Run the embedding shim through the OpenAI embeddings endpoint." }
                          }
                          div.example-actions {
                            button.btn-sm onclick="copyAdminTerminalCommand('call llm.rig.openai.embed \"slob games is wasm-first\" \"text-embedding-3-small\"')" { "Copy" }
                            button.btn-sm.accent onclick="runAdminTerminalCommand('call llm.rig.openai.embed \"slob games is wasm-first\" \"text-embedding-3-small\"')" { "Run" }
                          }
                        }
                        pre.cmd-box { "call llm.rig.openai.embed \"slob games is wasm-first\" \"text-embedding-3-small\"" }

                        div.example-row {
                          div.example-meta {
                            strong { "Rig Agent" }
                            span { "Build a concise Rig OpenAI agent with README context and execute it." }
                          }
                          div.example-actions {
                            button.btn-sm onclick="copyAdminTerminalCommand('call llm.rig.openai.agent \"Summarize slob.games in one sentence.\" \"gpt-4o-mini\" \"You are concise.\" \"README.md\"')" { "Copy" }
                            button.btn-sm.accent onclick="runAdminTerminalCommand('call llm.rig.openai.agent \"Summarize slob.games in one sentence.\" \"gpt-4o-mini\" \"You are concise.\" \"README.md\"')" { "Run" }
                          }
                        }
                        pre.cmd-box { "call llm.rig.openai.agent \"Summarize slob.games in one sentence.\" \"gpt-4o-mini\" \"You are concise.\" \"README.md\"" }
                      }
                      div.terminal-wrap id="adminTermWrap" style="display:none" {
                        div.terminal-header id="adminTermHeader" {
                          button.terminal-toggle id="adminBtnToggleTerm" { "▼ Terminal" }
                          span.terminal-hint { "WASM-powered traits CLI inside Admin" }
                          span.terminal-status id="adminTermStatus" {}
                        }
                        div.terminal-container id="adminTermContainer" {
                          div.xterm-mount id="adminXterm" {}
                        }
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
  --line: #1e1e2e;
  --text: #e8e6e3;
  --muted: #5a6570;
  --accent: #00e0ff;
  --green: #00ff88;
  --danger: #ef6b73;
}
* { box-sizing: border-box; }
body {
  margin: 0;
  background: linear-gradient(180deg, #060610 0%, var(--bg) 100%);
  color: var(--text);
  font-family: system-ui, -apple-system, sans-serif;
}
.page { max-width: 1120px; margin: 0 auto; padding: 32px 20px 48px; }
.card {
  background: linear-gradient(180deg, rgba(17,17,26,0.97), rgba(12,12,18,0.97));
  border: 1px solid rgba(0,224,255,0.07);
  border-radius: 14px; padding: 20px;
  box-shadow: 0 20px 48px rgba(0,0,0,0.3);
  margin-bottom: 18px;
}
.hero { position: relative; overflow: hidden; }
.eyebrow {
  margin: 0 0 8px; color: var(--accent);
  text-transform: uppercase; letter-spacing: 0.16em;
  font-size: 11px; font-family: 'Courier New', Menlo, monospace;
}
h1 {
  margin: 0; font-family: 'Courier New', Menlo, monospace;
  font-size: clamp(34px, 5vw, 56px); line-height: 0.96;
  text-transform: uppercase; letter-spacing: 0.04em;
}
h2 {
  margin: 0 0 12px; font-family: 'Courier New', Menlo, monospace;
  font-size: 18px; text-transform: uppercase;
  letter-spacing: 0.06em; color: var(--accent);
}
.subtitle { margin: 14px 0 0; color: var(--muted); line-height: 1.6; }
.note { color: var(--muted); font-size: 14px; }
a { color: var(--accent); }
code {
  font-family: 'Courier New', Menlo, monospace; font-size: 13px;
  color: #66f0ff; background: rgba(0,224,255,0.05);
  padding: 3px 6px; border-radius: 8px;
}
table {
  width: 100%; border-collapse: collapse;
  font-size: 13px; font-family: 'Courier New', Menlo, monospace;
}
th {
  text-align: left; padding: 8px 10px;
  border-bottom: 2px solid rgba(0,224,255,0.15);
  color: var(--accent); font-size: 11px;
  text-transform: uppercase; letter-spacing: 0.06em;
}
td {
  padding: 8px 10px;
  border-bottom: 1px solid rgba(30,30,46,0.5);
  vertical-align: top;
}
tr:hover td { background: rgba(0,224,255,0.03); }
.badge-admin {
  background: rgba(0,224,255,0.15); color: var(--accent);
  padding: 2px 8px; border-radius: 4px; font-size: 10px;
  text-transform: uppercase; letter-spacing: 0.05em;
}
.badge-user {
  background: rgba(90,101,112,0.15); color: var(--muted);
  padding: 2px 8px; border-radius: 4px; font-size: 10px;
  text-transform: uppercase; letter-spacing: 0.05em;
}
.badge-pub {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  background: rgba(0,255,136,0.12);
  color: var(--green);
  border: 1px solid rgba(0,255,136,0.2);
  padding: 2px 8px;
  border-radius: 999px;
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
.badge-draft {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  background: rgba(239,107,115,0.12);
  color: var(--danger);
  border: 1px solid rgba(239,107,115,0.2);
  padding: 2px 8px;
  border-radius: 999px;
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
.badge-public {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  background: rgba(0,224,255,0.1);
  color: var(--accent);
  border: 1px solid rgba(0,224,255,0.22);
  padding: 2px 8px;
  border-radius: 999px;
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
.badge-private {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  background: rgba(90,101,112,0.18);
  color: #9aa6b2;
  border: 1px solid rgba(90,101,112,0.35);
  padding: 2px 8px;
  border-radius: 999px;
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
.badge-scope {
  display: inline-flex;
  align-items: center;
  background: rgba(90,101,112,0.1);
  color: #9aa6b2;
  border: 1px solid rgba(90,101,112,0.25);
  padding: 2px 8px;
  border-radius: 999px;
  font-size: 10px;
  text-transform: lowercase;
  letter-spacing: 0.02em;
}
.group-header {
  padding: 12px 10px 6px;
  font-family: 'Courier New', Menlo, monospace;
  font-size: 14px; font-weight: 700;
  color: var(--accent);
  border-bottom: 1px solid rgba(0,224,255,0.12);
}
.tab-bar { display: flex; gap: 8px; margin-bottom: 14px; }
.tab-btn {
  padding: 8px 14px; border-radius: 8px;
  border: 1px solid var(--line); background: rgba(10,10,16,0.92);
  color: var(--muted); cursor: pointer;
  font-family: 'Courier New', Menlo, monospace; font-size: 12px;
  text-transform: uppercase; letter-spacing: 0.04em;
}
.tab-btn:hover { border-color: #3d5b6c; }
.tab-btn.active {
  background: rgba(0,224,255,0.12); border-color: rgba(0,224,255,0.25); color: var(--accent);
}
.denied {
  padding: 48px; text-align: center; color: var(--muted);
  font-family: 'Courier New', Menlo, monospace;
}
.btn-sm {
  padding: 4px 10px; border-radius: 6px; font-size: 10px;
  border: 1px solid var(--line); background: rgba(10,10,16,0.92);
  color: var(--muted); cursor: pointer; margin-left: 4px;
  font-family: 'Courier New', Menlo, monospace;
  text-transform: uppercase; letter-spacing: 0.02em;
}
.btn-sm:hover { border-color: #3d5b6c; color: var(--text); }
.btn-sm.danger { border-color: rgba(239,107,115,0.26); color: var(--danger); }
.btn-sm.danger:hover { background: rgba(107,37,48,0.3); }
.btn-sm.accent { border-color: rgba(0,224,255,0.25); color: var(--accent); }
.btn-sm.accent:hover { background: rgba(0,224,255,0.12); }
.actions { white-space: nowrap; text-align: right; }
.modal-bg {
  position: fixed; inset: 0; background: rgba(0,0,0,0.7);
  display: flex; align-items: center; justify-content: center; z-index: 999;
}
.modal {
  background: var(--panel); border: 1px solid rgba(0,224,255,0.15);
  border-radius: 14px; padding: 24px; min-width: 320px; max-width: 440px;
}
.modal h3 {
  margin: 0 0 16px; color: var(--accent);
  font-family: 'Courier New', Menlo, monospace; font-size: 15px;
  text-transform: uppercase;
}
.modal label {
  display: block; font-size: 11px; color: var(--muted);
  text-transform: uppercase; letter-spacing: 0.06em;
  margin: 12px 0 4px; font-family: 'Courier New', Menlo, monospace;
}
.modal input, .modal select {
  width: 100%; padding: 10px 12px; border-radius: 8px;
  border: 1px solid var(--line); background: rgba(10,10,16,0.92);
  color: var(--text); font: inherit; font-size: 14px;
}
.modal-actions { display: flex; gap: 8px; margin-top: 18px; justify-content: flex-end; }
.modal-actions button {
  padding: 8px 16px; border-radius: 8px; border: 1px solid var(--line);
  background: rgba(10,10,16,0.92); color: var(--muted); cursor: pointer;
  font-family: 'Courier New', Menlo, monospace; font-size: 12px;
}
.modal-actions button.primary {
  background: rgba(0,224,255,0.15); border-color: rgba(0,224,255,0.25); color: var(--accent);
}
.modal-actions button:hover { border-color: #3d5b6c; }
.rev-pill {
  display: inline-block;
  padding: 2px 8px;
  border: 1px solid #2f3a47;
  border-radius: 999px;
  color: var(--text);
  background: #111520;
  font-size: 11px;
  cursor: help;
}
.rev-pill:hover { border-color: #4a5d75; }
.rev-hover {
  position: fixed;
  z-index: 12000;
  width: min(320px, calc(100vw - 24px));
  max-height: 240px;
  overflow: auto;
  border: 1px solid #2f3a47;
  background: #0b1017;
  box-shadow: 0 12px 30px rgba(0,0,0,0.45);
  border-radius: 10px;
  padding: 10px;
  font-size: 12px;
  display: none;
}
.rev-hover .title { font-weight: 700; margin-bottom: 8px; color: var(--text); }
.rev-hover .empty { color: var(--muted); }
.rev-hover .row {
  border-top: 1px solid #1c2330;
  padding: 6px 0;
}
.rev-hover .row:first-of-type { border-top: 0; }
.rev-hover .meta { color: var(--muted); font-size: 11px; }
.rev-row-btn {
  display: block;
  width: 100%;
  text-align: left;
  border: 1px solid transparent;
  background: transparent;
  color: inherit;
  padding: 6px 6px;
  border-radius: 8px;
  cursor: pointer;
}
.rev-row-btn:hover {
  border-color: #33445a;
  background: rgba(0,224,255,0.06);
}
.terminal-examples {
  display: grid;
  gap: 10px;
  margin-bottom: 16px;
}
.example-row {
  display: flex;
  gap: 12px;
  align-items: center;
  justify-content: space-between;
}
.example-meta {
  display: grid;
  gap: 4px;
}
.example-meta strong {
  color: var(--text);
  font-size: 13px;
}
.example-meta span {
  color: var(--muted);
  font-size: 12px;
}
.example-actions {
  flex: 0 0 auto;
  white-space: nowrap;
}
.cmd-box {
  margin: 0;
  padding: 10px 12px;
  border-radius: 10px;
  border: 1px solid rgba(0,224,255,0.12);
  background: rgba(6,10,18,0.95);
  color: #8ff3ff;
  white-space: pre-wrap;
  word-break: break-word;
  font-size: 12px;
  line-height: 1.5;
}
#adminTermWrap {
  margin-top: 18px;
  border: 1px solid rgba(0,224,255,0.08);
  border-radius: 12px;
  overflow: hidden;
  background: rgba(8,12,20,0.92);
}
@media (max-width: 720px) {
  .page { padding: 18px 14px 32px; }
  .card { padding: 16px; }
  table { font-size: 11px; }
  th, td { padding: 6px 6px; }
  .example-row { display: block; }
  .example-actions { margin-top: 8px; }
}
"##;

const JS: &str = r##"
(function() {
var API_CANDIDATES = ['https://relay.slob.games/sync', 'https://relay.traits.build/sync'];
var API = API_CANDIDATES[0];
var token = '';
var currentUsername = '';
var usersData = [];
var gamesData = { external: [], internal: [] };
var currentTab = 'byOwner';
var adminTerminalInstance = null;
var pendingAdminTerminalCommand = '';

function _decodeB64Url(s) {
  var t = String(s || '').replace(/-/g, '+').replace(/_/g, '/');
  while (t.length % 4) t += '=';
  return atob(t);
}

function tokenRelaySyncBase() {
  var t = getToken();
  if (!t) return '';
  var parts = String(t).split('.');
  for (var i = 0; i < parts.length; i++) {
    try {
      var obj = JSON.parse(_decodeB64Url(parts[i]));
      if (obj && typeof obj.relay === 'string' && obj.relay) {
        return String(obj.relay).replace(/\/+$/, '') + '/sync';
      }
    } catch (_) {}
  }
  return '';
}

function apiOrigin(syncBase) {
  return String(syncBase || '').replace(/\/sync\/?$/, '');
}

async function selectApiBase(forceProbe) {
  if (!forceProbe && API) return API;
  var tokenBase = tokenRelaySyncBase();
  var ordered = [];
  if (tokenBase) ordered.push(tokenBase);
  for (var i = 0; i < API_CANDIDATES.length; i++) {
    if (ordered.indexOf(API_CANDIDATES[i]) < 0) ordered.push(API_CANDIDATES[i]);
  }
  for (var j = 0; j < ordered.length; j++) {
    var cand = ordered[j];
    try {
      var hr = await fetch(apiOrigin(cand) + '/health', { method: 'GET' });
      if (hr.ok) {
        API = cand;
        return API;
      }
    } catch (_) {}
  }
  API = API_CANDIDATES[0];
  return API;
}

async function relayFetch(path, opts) {
  await selectApiBase(false);
  var lastErr = null;
  var tokenBase = tokenRelaySyncBase();
  var isAuthScoped = /^\/(internal|admin|auth|github|github-mgr)\//.test(String(path || ''));
  var first = tokenBase || API;
  var ordered = [];
  if (isAuthScoped && tokenBase) {
    ordered = [tokenBase];
  } else {
    ordered = [first];
    for (var i = 0; i < API_CANDIDATES.length; i++) {
      if (API_CANDIDATES[i] !== first) ordered.push(API_CANDIDATES[i]);
    }
  }

  for (var j = 0; j < ordered.length; j++) {
    var base = ordered[j];
    try {
      var res = await fetch(base + path, opts || {});
      // Retry on upstream/server failure or path mismatch at current endpoint.
      if ((res.status >= 500 || res.status === 404) && j < ordered.length - 1) {
        lastErr = new Error('HTTP ' + res.status);
        continue;
      }
      API = base;
      return res;
    } catch (e) {
      lastErr = e;
      if (j < ordered.length - 1) continue;
    }
  }
  throw (lastErr || new Error('Failed to fetch relay endpoint'));
}

function esc(v) {
  var d = document.createElement('div');
  d.textContent = String(v == null ? '' : v);
  return d.innerHTML;
}

function slugify(s) {
  return String(s || '')
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '') || 'untitled';
}

function getToken() {
  try { return (localStorage.getItem('traits.secret.SLOB_USER_TOKEN') || '').trim(); } catch(_) { return ''; }
}

function ago(iso) {
  if (!iso) return '—';
  var d = new Date(iso);
  var now = Date.now();
  var s = Math.floor((now - d.getTime()) / 1000);
  if (s < 60) return s + 's ago';
  if (s < 3600) return Math.floor(s/60) + 'm ago';
  if (s < 86400) return Math.floor(s/3600) + 'h ago';
  return Math.floor(s/86400) + 'd ago';
}

async function apiFetch(path) {
  var r;
  try {
    r = await relayFetch(path, { headers: { 'Authorization': 'Bearer ' + token } });
  } catch (e) {
    return { ok: false, error: 'Failed to fetch relay' };
  }
  var text = await r.text();
  var body = {};
  try { body = text ? JSON.parse(text) : {}; } catch (_) { body = {}; }
  if (!r.ok) return { ok: false, error: body.error || ('HTTP ' + r.status) };
  return body;
}

async function apiDelete(path) {
  var r;
  try {
    r = await relayFetch(path, { method: 'DELETE', headers: { 'Authorization': 'Bearer ' + token } });
  } catch (e) {
    return { ok: false, error: 'Failed to fetch relay' };
  }
  var text = await r.text();
  var body = {};
  try { body = text ? JSON.parse(text) : {}; } catch (_) { body = {}; }
  if (!r.ok) return { ok: false, error: body.error || ('HTTP ' + r.status) };
  if (typeof body.ok === 'undefined') body.ok = true;
  return body;
}

async function apiPut(path, body) {
  var r;
  try {
    r = await relayFetch(path, {
      method: 'PUT',
      headers: { 'Authorization': 'Bearer ' + token, 'Content-Type': 'application/json' },
      body: JSON.stringify(body)
    });
  } catch (e) {
    return { ok: false, error: 'Failed to fetch relay' };
  }
  var text = await r.text();
  var parsed = {};
  try { parsed = text ? JSON.parse(text) : {}; } catch (_) { parsed = {}; }
  if (!r.ok) return { ok: false, error: parsed.error || ('HTTP ' + r.status) };
  return parsed;
}

async function apiPatch(path, body) {
  var r;
  try {
    r = await relayFetch(path, {
      method: 'PATCH',
      headers: { 'Authorization': 'Bearer ' + token, 'Content-Type': 'application/json' },
      body: JSON.stringify(body || {})
    });
  } catch (e) {
    return { ok: false, error: 'Failed to fetch relay' };
  }
  var text = await r.text();
  var parsed = {};
  try { parsed = text ? JSON.parse(text) : {}; } catch (_) { parsed = {}; }
  if (!r.ok) return { ok: false, error: parsed.error || ('HTTP ' + r.status) };
  return parsed;
}

// ── GitHub catalog: published games + sprites stored in the repo ──
var __ghGames = [];
var __ghSpritesCache = {}; // owner/gameId -> {files, expanded}

async function refreshGithubCatalog() {
  var status = document.getElementById('githubStatus');
  var el = document.getElementById('githubTable');
  if (!status || !el) return;
  status.textContent = 'Loading published games\u2026';
  try {
    var r = await apiFetch('/github/games');
    if (r && r.ok === false) {
      status.textContent = 'Failed: ' + (r.error || 'unknown');
      el.innerHTML = '';
      return;
    }
    __ghGames = (r && Array.isArray(r.games)) ? r.games : [];
    renderGithubCatalog();
  } catch (e) {
    status.textContent = 'Error: ' + (e && e.message || e);
  }
}

function renderGithubCatalog() {
  var status = document.getElementById('githubStatus');
  var el = document.getElementById('githubTable');
  if (!status || !el) return;
  if (!__ghGames.length) {
    status.textContent = 'No games published to GitHub yet.';
    el.innerHTML = '';
    return;
  }
  var enabled = __ghGames.filter(function(g){ return g.published !== false; }).length;
  var disabled = __ghGames.length - enabled;
  status.textContent = __ghGames.length + ' game' + (__ghGames.length === 1 ? '' : 's') + ' \u2022 ' + enabled + ' enabled \u2022 ' + disabled + ' disabled';

  var h = '<table><tr><th>Name</th><th>Owner</th><th>Hash</th><th>Size</th><th>Sprites</th><th>Updated</th><th>Status</th><th></th></tr>';
  for (var i = 0; i < __ghGames.length; i++) {
    var g = __ghGames[i];
    var ownerEnc = encodeURIComponent(g.owner || '');
    var idEnc = encodeURIComponent(g.game_id || '');
    var key = (g.owner || '') + '/' + (g.game_id || '');
    var keyEsc = key.replace(/'/g, "\\'");
    var disabled = g.published === false;
    var spriteCount = (typeof g.sprite_count === 'number') ? g.sprite_count : null;
    var spritesLabel = (spriteCount === null) ? '<button class="btn-sm" onclick="toggleGithubSprites(\'' + keyEsc + '\')">show</button>'
      : (spriteCount + ' <button class="btn-sm" onclick="toggleGithubSprites(\'' + keyEsc + '\')">view</button>');
    var rawUrl = 'https://raw.githubusercontent.com/'
      + ((window.GITHUB_REPO_HINT || '') || 'OWNER/REPO')
      + '/main/games/' + (g.owner || '') + '/' + (g.game_id || '') + '.json';
    h += '<tr id="ghrow-' + esc(key) + '">';
    h += '<td><strong>' + esc(g.name || '') + '</strong>'
      + (g.version ? '<br><span class="badge-scope">' + esc(g.version) + '</span>' : '') + '</td>';
    h += '<td>' + esc(g.owner || '') + '</td>';
    h += '<td><code title="' + esc(g.content_hash || '') + '">' + esc(String(g.content_hash || '').slice(0, 8)) + '</code></td>';
    h += '<td>' + (g.size ? (Math.round(g.size / 1024) + ' KB') : '\u2014') + '</td>';
    h += '<td>' + spritesLabel + '</td>';
    h += '<td title="' + esc(g.updated || '') + '">' + ago(g.updated || '') + '</td>';
    h += '<td>' + (disabled ? '<span class="badge-draft">disabled</span>' : '<span class="badge-pub">enabled</span>') + '</td>';
    h += '<td class="actions">';
    h += '<a class="btn-sm" href="' + esc(rawUrl) + '" target="_blank" rel="noopener">JSON</a>';
    h += '<button class="btn-sm" onclick="renameGithubGame(\'' + keyEsc + '\')">Rename</button>';
    if (disabled) {
      h += '<button class="btn-sm accent" onclick="enableGithubGame(\'' + keyEsc + '\')">Enable</button>';
    } else {
      h += '<button class="btn-sm" onclick="disableGithubGame(\'' + keyEsc + '\')">Disable</button>';
    }
    h += '<button class="btn-sm danger" onclick="deleteGithubGame(\'' + keyEsc + '\')">Delete</button>';
    h += '</td>';
    h += '</tr>';
    h += '<tr id="ghsprites-' + esc(key) + '" style="display:none"><td colspan="8" style="background:rgba(255,255,255,0.02)"><div id="ghsprites-body-' + esc(key) + '" style="padding:8px 12px;font-size:12px;color:#888">\u2014</div></td></tr>';
  }
  h += '</table>';
  el.innerHTML = h;
}

async function toggleGithubSprites(key) {
  var parts = String(key || '').split('/');
  if (parts.length !== 2) return;
  var owner = parts[0], gameId = parts[1];
  var row = document.getElementById('ghsprites-' + key);
  var body = document.getElementById('ghsprites-body-' + key);
  if (!row || !body) return;
  var visible = row.style.display !== 'none';
  if (visible) { row.style.display = 'none'; return; }
  row.style.display = '';
  body.textContent = 'Loading sprites\u2026';
  var r;
  try {
    r = await apiFetch('/github/games/' + encodeURIComponent(owner) + '/' + encodeURIComponent(gameId) + '/sprites');
  } catch (e) {
    body.textContent = 'Failed to load sprites';
    return;
  }
  if (r && r.ok === false) { body.textContent = 'Error: ' + (r.error || 'failed'); return; }
  var files = ((r && r.files) || []).filter(function(f){ return f.path !== (gameId + '.json'); });
  if (!files.length) { body.textContent = 'No sprite files for this game.'; return; }
  var h = '<div style="display:grid;grid-template-columns:repeat(auto-fill,minmax(220px,1fr));gap:6px">';
  for (var i = 0; i < files.length; i++) {
    var f = files[i];
    h += '<div style="display:flex;align-items:center;gap:8px;padding:6px;border:1px solid rgba(255,255,255,0.06);border-radius:4px">';
    var isImage = /\.(png|jpe?g|gif|webp|svg)$/i.test(f.path);
    if (isImage && f.download_url) {
      h += '<img src="' + esc(f.download_url) + '" style="width:32px;height:32px;object-fit:contain;background:#000;border-radius:2px" loading="lazy">';
    } else {
      h += '<div style="width:32px;height:32px;display:flex;align-items:center;justify-content:center;background:#222;border-radius:2px;font-size:10px;color:#888">file</div>';
    }
    h += '<div style="flex:1;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap"><div title="' + esc(f.path) + '" style="font-size:11px">' + esc(f.path) + '</div>';
    h += '<div style="font-size:10px;color:#888">' + (f.size ? Math.round(f.size / 1024) + ' KB' : '\u2014') + '</div></div>';
    if (f.download_url) {
      h += '<a class="btn-sm" href="' + esc(f.download_url) + '" target="_blank" rel="noopener">DL</a>';
    }
    h += '</div>';
  }
  h += '</div>';
  body.innerHTML = h;
}

async function disableGithubGame(key) {
  var parts = String(key || '').split('/');
  if (parts.length !== 2) return;
  if (!confirm('Disable "' + key + '"? It will be hidden from the carousel until re-enabled.')) return;
  var r = await apiPatch('/github-mgr/games/' + encodeURIComponent(parts[0]) + '/' + encodeURIComponent(parts[1]), { published: false });
  if (!r.ok) { alert('Disable failed: ' + (r.error || 'unknown')); return; }
  await refreshGithubCatalog();
}

async function enableGithubGame(key) {
  var parts = String(key || '').split('/');
  if (parts.length !== 2) return;
  var r = await apiPatch('/github-mgr/games/' + encodeURIComponent(parts[0]) + '/' + encodeURIComponent(parts[1]), { published: true });
  if (!r.ok) { alert('Enable failed: ' + (r.error || 'unknown')); return; }
  await refreshGithubCatalog();
}

async function renameGithubGame(key) {
  var parts = String(key || '').split('/');
  if (parts.length !== 2) return;
  var current = '';
  for (var i = 0; i < __ghGames.length; i++) {
    if ((__ghGames[i].owner + '/' + __ghGames[i].game_id) === key) { current = __ghGames[i].name || ''; break; }
  }
  // Inline rename row instead of prompt() (some browsers suppress prompt dialogs)
  var row = document.getElementById('ghrow-' + key);
  if (!row) return;
  var existing = document.getElementById('ghrename-' + key);
  if (existing) { existing.remove(); return; }
  var tr = document.createElement('tr');
  tr.id = 'ghrename-' + key;
  var td = document.createElement('td');
  td.colSpan = 8;
  td.style.cssText = 'background:rgba(255,255,255,0.04);padding:10px';
  td.innerHTML = 'Rename <strong>' + esc(key) + '</strong>: '
    + '<input type="text" id="ghrename-input-' + esc(key) + '" style="width:280px;padding:4px;background:#111;color:#eee;border:1px solid #333;border-radius:3px" />'
    + ' <button class="btn-sm accent" id="ghrename-save-' + esc(key) + '">Save</button>'
    + ' <button class="btn-sm" id="ghrename-cancel-' + esc(key) + '">Cancel</button>';
  row.parentNode.insertBefore(tr, row.nextSibling);
  tr.appendChild(td);
  var input = document.getElementById('ghrename-input-' + key);
  input.value = current;
  input.focus();
  input.select();
  document.getElementById('ghrename-cancel-' + key).addEventListener('click', function(){ tr.remove(); });
  async function doSave() {
    var next = String(input.value || '').trim();
    if (!next || next === current) { tr.remove(); return; }
    var btn = document.getElementById('ghrename-save-' + key);
    if (btn) { btn.disabled = true; btn.textContent = 'Saving\u2026'; }
    var r = await apiPatch('/github-mgr/games/' + encodeURIComponent(parts[0]) + '/' + encodeURIComponent(parts[1]), { name: next });
    if (!r || r.ok === false) {
      alert('Rename failed: ' + ((r && r.error) || 'unknown'));
      if (btn) { btn.disabled = false; btn.textContent = 'Save'; }
      return;
    }
    tr.remove();
    await refreshGithubCatalog();
  }
  document.getElementById('ghrename-save-' + key).addEventListener('click', doSave);
  input.addEventListener('keydown', function(ev){
    if (ev.key === 'Enter') { ev.preventDefault(); doSave(); }
    else if (ev.key === 'Escape') { tr.remove(); }
  });
}

async function deleteGithubGame(key) {
  var parts = String(key || '').split('/');
  if (parts.length !== 2) return;
  if (!confirm('Permanently delete "' + key + '" from GitHub?\n\nThis removes the game JSON, all sprite files, and the index entry. This cannot be undone.')) return;
  var r = await apiDelete('/github-mgr/games/' + encodeURIComponent(parts[0]) + '/' + encodeURIComponent(parts[1]));
  if (!r.ok) { alert('Delete failed: ' + (r.error || 'unknown')); return; }
  await refreshGithubCatalog();
}

async function load() {
  token = getToken();
  if (!token) {
    document.querySelector('.page').innerHTML = '<div class="denied"><h2>Access Denied</h2><p>Log in via Settings first.</p></div>';
    return;
  }

  // Check role via /auth/me
  var me = await apiFetch('/auth/me');
  if (!me.ok || me.role !== 'admin') {
    document.querySelector('.page').innerHTML = '<div class="denied"><h2>Access Denied</h2><p>Admin role required. Your role: ' + esc(me.role || me.error || 'unknown') + '</p></div>';
    return;
  }
  currentUsername = String(me.username || '').trim().toLowerCase();

  // Fetch users and games in parallel
  var p = await Promise.all([apiFetch('/admin/users'), apiFetch('/admin/games')]);
  usersData = Array.isArray(p[0]) ? p[0] : [];
  gamesData = p[1] && p[1].external ? p[1] : { external: [], internal: [] };

  renderUsers();
  renderGames();
  renderPvfsGames();
}

function renderUsers() {
  var el = document.getElementById('usersTable');
  var status = document.getElementById('usersStatus');
  if (!usersData.length) { status.textContent = 'No users found.'; el.innerHTML = ''; return; }
  status.textContent = usersData.length + ' registered user' + (usersData.length === 1 ? '' : 's');

  var h = '<table><tr><th>Username</th><th>Email</th><th>Role</th><th>Created</th><th>Last Login</th><th></th></tr>';
  for (var i = 0; i < usersData.length; i++) {
    var u = usersData[i];
    var roleBadge = u.role === 'admin'
      ? '<span class="badge-admin">admin</span>'
      : '<span class="badge-user">user</span>';
    var uEnc = encodeURIComponent(u.username);
    h += '<tr>';
    h += '<td><strong>' + esc(u.username) + '</strong></td>';
    h += '<td>' + esc(u.email) + '</td>';
    h += '<td>' + roleBadge + '</td>';
    h += '<td title="' + esc(u.created) + '">' + ago(u.created) + '</td>';
    h += '<td title="' + esc(u.last_login) + '">' + (u.last_login ? ago(u.last_login) : '—') + '</td>';
    h += '<td class="actions">';
    h += '<button class="btn-sm" onclick="manageSecrets(\'' + uEnc + '\')">Secrets</button>';
    h += '<button class="btn-sm accent" onclick="editUser(\'' + uEnc + '\')">Edit</button>';
    h += '<button class="btn-sm danger" onclick="deleteUser(\'' + uEnc + '\')">Del</button>';
    h += '</td>';
    h += '</tr>';
  }
  h += '</table>';
  el.innerHTML = h;
}

function renderGames() {
  var el = document.getElementById('gamesTable');
  var status = document.getElementById('gamesStatus');
  var legend = document.getElementById('gamesLegend');

  // All games in a unified list
  var all = [];
  for (var i = 0; i < gamesData.external.length; i++) {
    var g = gamesData.external[i];
    var isPublished = (g.published === undefined) ? true : !!g.published;
    var scope = String(g.scope || 'external');
    var isPublic = (scope === 'external') && isPublished;
    all.push({
      owner: g.owner || 'public',
      game_id: g.game_id || '',
      name: g.name,
      size: g.size,
      updated: g.updated,
      fullHash: g.content_hash || '',
      hash: (g.content_hash || '').slice(0, 8),
      version: g.version || '',
      forked: !!g.forked_from_hash,
      highscore: g.highscore || 0,
      highscore_player: g.highscore_player || '',
      published: isPublished,
      publicVisible: isPublic,
      scope: scope,
    });
  }

  var pubCount = all.filter(function(gm){ return gm.published; }).length;
  var publicCount = all.filter(function(gm){ return gm.publicVisible; }).length;
  status.textContent = all.length + ' game' + (all.length === 1 ? '' : 's') + ' • ' + pubCount + ' published • ' + publicCount + ' public';
  if (legend) {
    legend.textContent = 'Public = scope "external" AND published = true. Draft/private rows stay in user catalog only.';
  }

  if (!all.length) { el.innerHTML = '<p class="note">No games found.</p>'; return; }

  function publishBadge(gm) {
    return gm.published ? '<span class="badge-pub">published</span>' : '<span class="badge-draft">draft</span>';
  }

  function visibilityBadge(gm) {
    return gm.publicVisible ? '<span class="badge-public">public</span>' : '<span class="badge-private">private</span>';
  }

  function scopeBadge(gm) {
    return '<span class="badge-scope">' + esc(gm.scope || 'external') + '</span>';
  }

  if (currentTab === 'byOwner') {
    // Group by owner
    var byOwner = {};
    for (var k = 0; k < all.length; k++) {
      var o = all[k].owner;
      if (!byOwner[o]) byOwner[o] = [];
      byOwner[o].push(all[k]);
    }
    var owners = Object.keys(byOwner).sort();
    var h = '';
    for (var oi = 0; oi < owners.length; oi++) {
      var ow = owners[oi];
      var gs = byOwner[ow];
      h += '<div class="group-header">' + esc(ow) + ' (' + gs.length + ')</div>';
      h += '<table><tr><th>Identity</th><th>Name</th><th>Publish</th><th>Visibility</th><th>Scope</th><th>Version</th><th>HS</th><th>Size</th><th>Updated</th><th></th></tr>';
      for (var gi = 0; gi < gs.length; gi++) {
        var gm = gs[gi];
        var identity = esc(gm.owner + '/' + gm.game_id);
        var gh = encodeURIComponent(gm.fullHash);
        h += '<tr><td><code>' + identity + '</code></td><td><span style="cursor:pointer;color:var(--accent);text-decoration:underline" onclick="playAdminGame(\'' + encodeURIComponent(gm.owner) + '\',\'' + encodeURIComponent(gm.game_id || '') + '\',\'' + gh + '\')">' + esc(gm.name) + '</span></td>';
        h += '<td>' + publishBadge(gm) + '</td>';
        h += '<td>' + visibilityBadge(gm) + '</td>';
        h += '<td>' + scopeBadge(gm) + '</td>';
        h += '<td>' + esc(gm.version || '—') + '</td>';
        h += '<td>' + (gm.highscore ? '<span title="' + esc(gm.highscore_player || '') + '">' + gm.highscore + '</span>' : '<span style="opacity:0.3">—</span>') + '</td>';
        h += '<td>' + formatSize(gm.size) + '</td>';
        h += '<td title="' + esc(gm.updated) + '">' + ago(gm.updated) + '</td>';
        h += '<td class="actions">';
        h += '<button class="btn-sm ' + (gm.published ? '' : 'accent') + '" onclick="toggleAdminPublish(\'' + encodeURIComponent(gm.owner) + '\',\'' + encodeURIComponent(gm.game_id) + '\',\'' + gh + '\',' + (gm.published ? 'true' : 'false') + ')">' + (gm.published ? 'Unpublish' : 'Publish') + '</button>';
        h += '<button class="btn-sm accent" onclick="assignGame(\'' + gh + '\',\'' + encodeURIComponent(gm.owner) + '\')">' + 'Assign</button>';
        h += '<button class="btn-sm danger" onclick="deleteGame(\'' + gh + '\')">' + 'Del</button>';
        h += '</td></tr>';
      }
      h += '</table>';
    }
    el.innerHTML = h;
  } else {
    // Raw list sorted by name only — no grouping/deduping of same-name games.
    var byNameList = all.slice().sort(function(a, b) {
      var an = String(a.name || 'untitled').toLowerCase();
      var bn = String(b.name || 'untitled').toLowerCase();
      if (an !== bn) return an.localeCompare(bn);
      return String(b.updated || '').localeCompare(String(a.updated || ''));
    });
    var h2 = '';
    h2 += '<table><tr><th>Owner/ID</th><th>Name</th><th>Publish</th><th>Visibility</th><th>Scope</th><th>Version</th><th>HS</th><th>Size</th><th>Updated</th><th></th></tr>';
    for (var ni = 0; ni < byNameList.length; ni++) {
      var gm2 = byNameList[ni];
      var gh2 = encodeURIComponent(gm2.fullHash);
      h2 += '<tr><td><code>' + esc(gm2.owner + '/' + gm2.game_id) + '</code></td>';
      h2 += '<td><span style="cursor:pointer;color:var(--accent);text-decoration:underline" onclick="playAdminGame(\'' + encodeURIComponent(gm2.owner) + '\',\'' + encodeURIComponent(gm2.game_id || '') + '\',\'' + gh2 + '\')">' + esc(gm2.name) + '</span></td>';
      h2 += '<td>' + publishBadge(gm2) + '</td>';
      h2 += '<td>' + visibilityBadge(gm2) + '</td>';
      h2 += '<td>' + scopeBadge(gm2) + '</td>';
      h2 += '<td>' + esc(gm2.version || '—') + '</td>';
      h2 += '<td>' + (gm2.highscore ? '<span title="' + esc(gm2.highscore_player || '') + '">' + gm2.highscore + '</span>' : '<span style="opacity:0.3">—</span>') + '</td>';
      h2 += '<td>' + formatSize(gm2.size) + '</td>';
      h2 += '<td title="' + esc(gm2.updated) + '">' + ago(gm2.updated) + '</td>';
      h2 += '<td class="actions">';
      h2 += '<button class="btn-sm ' + (gm2.published ? '' : 'accent') + '" onclick="toggleAdminPublish(\'' + encodeURIComponent(gm2.owner) + '\',\'' + encodeURIComponent(gm2.game_id) + '\',\'' + gh2 + '\',' + (gm2.published ? 'true' : 'false') + ')">' + (gm2.published ? 'Unpublish' : 'Publish') + '</button>';
      h2 += '<button class="btn-sm accent" onclick="assignGame(\'' + gh2 + '\',\'' + encodeURIComponent(gm2.owner) + '\')">' + 'Assign</button>';
      h2 += '<button class="btn-sm danger" onclick="deleteGame(\'' + gh2 + '\')">' + 'Del</button>';
      h2 += '</td></tr>';
    }
    h2 += '</table>';
    el.innerHTML = h2;
  }
}

async function callTrait(path, args) {
  var sdk = window._traitsSDK;
  if (!sdk) throw new Error('SDK not ready');
  var res = await sdk.call(path, args || []);
  var out = res && res.result !== undefined ? res.result : res;
  if (!res || res.ok === false || !out || out.ok === false) {
    throw new Error((out && out.error) || (res && res.error) || ('Trait call failed: ' + path));
  }
  return out;
}

function goCanvasRoute() {
  if (location.protocol === 'file:') {
    sessionStorage.setItem('traits.shell.route', '/');
    location.hash = '#/';
  } else {
    history.pushState({ route: '/' }, '', '/');
  }
  window.dispatchEvent(new PopStateEvent('popstate', { state: { route: '/' } }));
}

async function playAdminGame(ownerEnc, gameIdEnc, hashEnc) {
  try {
    var owner = decodeURIComponent(ownerEnc || '');
    var gameId = decodeURIComponent(gameIdEnc || '');
    var hash = String(decodeURIComponent(hashEnc || '')).toLowerCase();
    var content = '';
    var name = '';
    var version = '';

    var r = await relayFetch('/game/' + encodeURIComponent(hash));
    var d = await r.json();
    if (!r.ok || !d.content) throw new Error(d.error || 'Could not load game');
    content = d.content;
    name = d.name || gameId || 'Game';
    version = d.version || '';

    var idSeed = (hash || (owner + '-' + gameId))
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, '')
      .slice(0, 16);
    if (!idSeed) idSeed = String(Date.now());
    var localId = 's-' + idSeed;
    var syncGameId = (String(gameId || '').trim().toLowerCase() || idSeed);

    await callTrait('sys.canvas', [
      'load_game',
      localId,
      name,
      version,
      content,
      'external',
      owner || 'community',
      syncGameId,
      hash
    ]);
    goCanvasRoute();
  } catch (e) {
    alert((e && e.message) ? e.message : 'Failed to play game');
  }
}

function formatSize(b) {
  if (!b) return '—';
  if (b < 1024) return b + ' B';
  if (b < 1024 * 1024) return (b / 1024).toFixed(1) + ' KB';
  return (b / (1024 * 1024)).toFixed(1) + ' MB';
}

function switchTab(tab) {
  currentTab = tab;
  document.querySelectorAll('.tab-btn').forEach(function(b) { b.classList.remove('active'); });
  document.getElementById(tab === 'byOwner' ? 'tabByOwner' : 'tabByName').classList.add('active');
  renderGames();
}

// ── Modal helpers ──
function showModal(html) {
  var bg = document.createElement('div');
  bg.className = 'modal-bg';
  bg.onclick = function(e) { if (e.target === bg) bg.remove(); };
  bg.innerHTML = '<div class="modal">' + html + '</div>';
  document.body.appendChild(bg);
  return bg;
}

function closeModal() {
  var m = document.querySelector('.modal-bg');
  if (m) m.remove();
}

function copyAdminTerminalCommand(cmd) {
  var text = String(cmd || '');
  if (!text) return;
  if (navigator.clipboard && navigator.clipboard.writeText) {
    navigator.clipboard.writeText(text).catch(function(){});
  }
}

function focusAdminTerminal() {
  if (!adminTerminalInstance || !adminTerminalInstance.term) return;
  var container = document.getElementById('adminTermContainer');
  var toggleBtn = document.getElementById('adminBtnToggleTerm');
  if (container && container.classList.contains('collapsed')) {
    container.classList.remove('collapsed');
    if (toggleBtn) toggleBtn.textContent = '▼ Terminal';
  }
  try {
    if (adminTerminalInstance.fitAddon && adminTerminalInstance.fitAddon.fit) adminTerminalInstance.fitAddon.fit();
  } catch (_) {}
  adminTerminalInstance.term.focus();
}

function pasteAdminTerminalCommand(cmd, execute) {
  var text = String(cmd || '');
  if (!text) return;
  if (!adminTerminalInstance || !adminTerminalInstance.term || typeof adminTerminalInstance.term.paste !== 'function') {
    pendingAdminTerminalCommand = text + (execute ? '\r' : '');
    return;
  }
  focusAdminTerminal();
  adminTerminalInstance.term.paste(text + (execute ? '\r' : ''));
}

function runAdminTerminalCommand(cmd) {
  pasteAdminTerminalCommand(cmd, true);
}

async function initAdminTerminal() {
  var wrap = document.getElementById('adminTermWrap');
  var mount = document.getElementById('adminXterm');
  if (!wrap || !mount) return;

  var createTerminal = window.createTerminal;
  if (!createTerminal) {
    var paths = ['/static/www/terminal/terminal.js', '../terminal/terminal.js'];
    for (var i = 0; i < paths.length; i++) {
      try {
        var mod = await import(paths[i]);
        createTerminal = mod.createTerminal;
        break;
      } catch (_) {}
    }
  }
  if (!createTerminal) return;

  wrap.style.display = '';
  adminTerminalInstance = await createTerminal(mount, {
    header: document.getElementById('adminTermHeader'),
    container: document.getElementById('adminTermContainer'),
    toggleBtn: document.getElementById('adminBtnToggleTerm'),
    statusEl: document.getElementById('adminTermStatus')
  });
  window.adminTerminalInstance = adminTerminalInstance;

  if (pendingAdminTerminalCommand) {
    var queued = pendingAdminTerminalCommand;
    pendingAdminTerminalCommand = '';
    setTimeout(function() {
      pasteAdminTerminalCommand(queued.replace(/\r$/, ''), /\r$/.test(queued));
    }, 120);
  }
}

// ── User actions ──
async function deleteUser(usernameEnc) {
  var username = decodeURIComponent(usernameEnc);
  if (!confirm('Delete user "' + username + '"? This cannot be undone.')) return;
  var r = await apiDelete('/admin/users/' + usernameEnc);
  if (r.ok) {
    usersData = usersData.filter(function(u) { return u.username !== username; });
    renderUsers();
  } else {
    alert(r.error || 'Delete failed');
  }
}

function editUser(usernameEnc) {
  var username = decodeURIComponent(usernameEnc);
  var user = null;
  for (var i = 0; i < usersData.length; i++) {
    if (usersData[i].username === username) { user = usersData[i]; break; }
  }
  if (!user) return;
  var h = '<h3>Edit ' + esc(username) + '</h3>';
  h += '<label>Email</label><input id="modalEmail" value="' + esc(user.email) + '">';
  h += '<label>Role</label><select id="modalRole">';
  h += '<option value="user"' + (user.role === 'user' ? ' selected' : '') + '>user</option>';
  h += '<option value="admin"' + (user.role === 'admin' ? ' selected' : '') + '>admin</option>';
  h += '</select>';
  h += '<label>New Password <small>(leave blank to keep current)</small></label>';
  h += '<input id="modalPassword" type="password" placeholder="new password">';
  h += '<div class="modal-actions"><button onclick="closeModal()">Cancel</button>';
  h += '<button class="primary" onclick="submitEditUser(\'' + usernameEnc + '\')">Save</button></div>';
  showModal(h);
}

async function submitEditUser(usernameEnc) {
  var email = document.getElementById('modalEmail').value.trim();
  var role = document.getElementById('modalRole').value;
  var pw = document.getElementById('modalPassword').value;
  var body = { email: email, role: role };
  if (pw) body.password = pw;
  var r = await apiPut('/admin/users/' + usernameEnc, body);
  closeModal();
  if (r.ok) {
    var username = decodeURIComponent(usernameEnc);
    for (var i = 0; i < usersData.length; i++) {
      if (usersData[i].username === username) {
        if (email) usersData[i].email = email;
        if (role) usersData[i].role = role;
        break;
      }
    }
    // If admin changed their own role, persist locally so SPA shell picks it up immediately
    try {
      var me = (localStorage.getItem('traits.env.SLOB_USERNAME') || '').trim();
      if (role && me && me === username) {
        localStorage.setItem('traits.env.SLOB_USER_ROLE', role);
      }
    } catch(_) {}
    renderUsers();
  } else {
    alert(r.error || 'Update failed');
  }
}

// ── Game actions ──
async function toggleAdminPublish(ownerEnc, gameIdEnc, hashEnc, currentPublished) {
  var gameId = decodeURIComponent(gameIdEnc);
  var owner = decodeURIComponent(ownerEnc);
  var hash = decodeURIComponent(hashEnc);
  var nextPublished = !currentPublished;
  var r = await relayFetch('/internal/game/' + encodeURIComponent(gameId) + '/publish', {
    method: 'PATCH',
    headers: { 'Authorization': 'Bearer ' + token, 'Content-Type': 'application/json', 'X-Game-Owner': owner },
    body: JSON.stringify({ published: nextPublished })
  });
  var d = await r.json().catch(function(){ return {}; });
  if (!r.ok) { alert(d.error || 'Toggle failed'); return; }
  // Update local data and re-render
  var all = gamesData.external || [];
  for (var i = 0; i < all.length; i++) {
    if (String(all[i].content_hash || '') === hash) {
      all[i].published = nextPublished ? 1 : 0;
      break;
    }
  }
  renderGames();
}

async function deleteGame(hashEnc) {
  var hash = decodeURIComponent(hashEnc);
  if (!confirm('Delete game #' + hash.slice(0,8) + '? This cannot be undone.')) return;

  // Optimistic removal immediately — delete is very likely to succeed.
  gamesData.external = (gamesData.external || []).filter(function(g) {
    return String(g.content_hash || '') !== hash;
  });
  gamesData.internal = (gamesData.internal || []).filter(function(g) {
    return String(g.content_hash || '') !== hash;
  });
  renderGames();

  // Fire server-side delete; only restore + alert on a real server error.
  try {
    var r = await apiDelete('/admin/games/' + hashEnc);
    if (!r.ok) {
      alert(r.error || 'Delete failed on server — refresh to see current state');
      try { var gp = await apiFetch('/admin/games'); if (gp && gp.external) { gamesData = gp; renderGames(); } } catch(_) {}
    }
  } catch (_) {
    // Network error (e.g. "Failed to fetch") — optimistic removal is already reflected;
    // the delete almost certainly went through, so we silently swallow this.
  }
}

function assignGame(hashEnc, currentOwnerEnc) {
  var hash = decodeURIComponent(hashEnc);
  var currentOwner = decodeURIComponent(currentOwnerEnc);
  var h = '<h3>Assign Game #' + esc(hash.slice(0,8)) + '</h3>';
  h += '<label>Current Owner</label><input disabled value="' + esc(currentOwner) + '">';
  h += '<label>New Owner</label><input id="modalNewOwner" value="' + esc(currentOwner) + '" placeholder="username">';
  h += '<div class="modal-actions"><button onclick="closeModal()">Cancel</button>';
  h += '<button class="primary" onclick="submitAssignGame(\'' + hashEnc + '\')">Assign</button></div>';
  showModal(h);
}

async function submitAssignGame(hashEnc) {
  var newOwner = document.getElementById('modalNewOwner').value.trim();
  if (!newOwner) { alert('Owner required'); return; }
  var r = await apiPut('/admin/games/' + hashEnc + '/assign', { owner: newOwner });
  closeModal();
  if (r.ok) {
    // Reload games to reflect change
    var gp = await apiFetch('/admin/games');
    gamesData = gp && gp.external ? gp : { external: [], internal: [] };
    renderGames();
  } else {
    alert(r.error || 'Assign failed');
  }
}

async function manageSecrets(usernameEnc) {
  var username = decodeURIComponent(usernameEnc);
  var r = await apiFetch('/admin/users/' + usernameEnc + '/secrets');
  var secrets = Array.isArray(r) ? r : [];
  var h = '<h3>Secrets \u2014 ' + esc(username) + '</h3>';
  if (secrets.length) {
    h += '<table><tr><th>Key</th><th>Updated</th><th></th></tr>';
    for (var i = 0; i < secrets.length; i++) {
      var s = secrets[i];
      var kEnc = encodeURIComponent(s.key);
      h += '<tr><td><code>' + esc(s.key) + '</code></td>';
      h += '<td>' + ago(s.updated) + '</td>';
      h += '<td class="actions"><button class="btn-sm danger" onclick="deleteUserSecret(\'' + usernameEnc + '\',\'' + kEnc + '\')">Del</button></td>';
      h += '</tr>';
    }
    h += '</table>';
  } else {
    h += '<p class="note">No secrets stored for this user.</p>';
  }
  h += '<label>Key</label><input id="modalSecretKey" placeholder="e.g. OPENAI_API_KEY">';
  h += '<label>Value</label><input id="modalSecretValue" type="password" placeholder="secret value">';
  h += '<div class="modal-actions">';
  h += '<button onclick="closeModal()">Close</button>';
  h += '<button class="primary" onclick="addUserSecret(\'' + usernameEnc + '\')">Add Secret</button>';
  h += '</div>';
  showModal(h);
}

async function addUserSecret(usernameEnc) {
  var key = (document.getElementById('modalSecretKey').value || '').trim();
  var value = document.getElementById('modalSecretValue').value || '';
  if (!key || !value) { alert('Key and value required'); return; }
  var r = await apiPut('/admin/users/' + usernameEnc + '/secrets/' + encodeURIComponent(key), { value: value });
  if (r.ok) {
    closeModal();
    manageSecrets(usernameEnc);
  } else {
    alert(r.error || 'Failed to add secret');
  }
}

async function deleteUserSecret(usernameEnc, keyEnc) {
  var key = decodeURIComponent(keyEnc);
  if (!confirm('Delete secret "' + key + '"?')) return;
  var r = await apiDelete('/admin/users/' + usernameEnc + '/secrets/' + keyEnc);
  if (r.ok) {
    closeModal();
    manageSecrets(usernameEnc);
  } else {
    alert(r.error || 'Failed to delete secret');
  }
}

// ── PVFS Games (local browser storage) ──
function readPvfsGames() {
  try {
    var raw = localStorage.getItem('traits.pvfs');
    if (!raw) return {};
    var pvfs = JSON.parse(raw);
    var gamesRaw = pvfs['canvas/games.json'];
    if (!gamesRaw) return {};
    var data = typeof gamesRaw === 'string' ? JSON.parse(gamesRaw) : gamesRaw;
    return data.games || {};
  } catch(_) { return {}; }
}

function readPvfsRevisionIndex() {
  try {
    var raw = localStorage.getItem('traits.pvfs');
    if (!raw) return {};
    var pvfs = JSON.parse(raw);
    var idxRaw = pvfs['canvas/revisions/index.json'];
    if (!idxRaw) return {};
    var idx = typeof idxRaw === 'string' ? JSON.parse(idxRaw) : idxRaw;
    return (idx && typeof idx === 'object') ? idx : {};
  } catch(_) { return {}; }
}

function pvfsRevisionKeyForGame(localId, g) {
  var owner = String((g && (g._sync_owner || g.owner)) || 'local').trim().toLowerCase() || 'local';
  var gid = String((g && (g._sync_game_id || g.game_id)) || slugify((g && g.name) || localId || 'untitled')).trim().toLowerCase();
  return owner + '/' + gid;
}

function pvfsRevisionCandidates(localId, g) {
  var keys = {};
  function addKey(k) {
    k = String(k || '').trim().toLowerCase();
    if (!k || k.indexOf('/') <= 0) return;
    keys[k] = true;
  }
  addKey(pvfsRevisionKeyForGame(localId, g));

  var ownerCandidates = { local: true, public: true };
  var o1 = String((g && g._sync_owner) || '').trim().toLowerCase();
  var o2 = String((g && g.owner) || '').trim().toLowerCase();
  if (o1) ownerCandidates[o1] = true;
  if (o2) ownerCandidates[o2] = true;
  if (currentUsername) ownerCandidates[currentUsername] = true;

  var gidCandidates = {};
  var gid1 = String((g && g._sync_game_id) || '').trim().toLowerCase();
  var gid2 = String((g && g.game_id) || '').trim().toLowerCase();
  var gid3 = slugify((g && g.name) || localId || 'untitled');
  var gid4 = slugify(localId || '');
  if (gid1) gidCandidates[gid1] = true;
  if (gid2) gidCandidates[gid2] = true;
  if (gid3) gidCandidates[gid3] = true;
  if (gid4) gidCandidates[gid4] = true;

  var owners = Object.keys(ownerCandidates);
  var gids = Object.keys(gidCandidates);
  for (var i = 0; i < owners.length; i++) {
    for (var j = 0; j < gids.length; j++) {
      addKey(owners[i] + '/' + gids[j]);
    }
  }
  return Object.keys(keys);
}

function pvfsRevisionsForGame(localId, g) {
  var idx = readPvfsRevisionIndex();
  var keys = pvfsRevisionCandidates(localId, g);
  var byId = {};
  var out = [];
  for (var i = 0; i < keys.length; i++) {
    var arr = (idx && idx[keys[i]] && Array.isArray(idx[keys[i]])) ? idx[keys[i]] : [];
    for (var j = arr.length - 1; j >= 0; j--) {
      var meta = arr[j] || {};
      var rid = String(meta.id || '').trim();
      if (!rid || byId[rid]) continue;
      byId[rid] = true;
      out.push({
        id: rid,
        version: String(meta.version || ''),
        created: String(meta.created || ''),
        length: Number(meta.length || 0),
        path: String(meta.path || ''),
        key: String(keys[i] || '')
      });
    }
  }
  out.sort(function(a, b) {
    if (b.created !== a.created) return String(b.created).localeCompare(String(a.created));
    return String(b.id).localeCompare(String(a.id));
  });
  return out;
}

function pvfsReadRevisionSnapshot(localId, revId) {
  try {
    var games = readPvfsGames();
    var g = games[localId];
    if (!g) return null;
    var revs = pvfsRevisionsForGame(localId, g);
    var meta = null;
    for (var i = 0; i < revs.length; i++) {
      if (String(revs[i].id || '') === String(revId || '')) { meta = revs[i]; break; }
    }
    if (!meta) return null;
    var raw = localStorage.getItem('traits.pvfs') || '{}';
    var pvfs = JSON.parse(raw);
    var path = String(meta.path || ('canvas/revisions/' + revId + '.json'));
    var snapRaw = pvfs[path];
    if (!snapRaw && path !== ('canvas/revisions/' + revId + '.json')) {
      path = 'canvas/revisions/' + revId + '.json';
      snapRaw = pvfs[path];
    }
    var snap = snapRaw ? (typeof snapRaw === 'string' ? JSON.parse(snapRaw) : snapRaw) : null;
    return {
      localId: localId,
      game: g,
      meta: meta,
      path: path,
      snapshot: snap
    };
  } catch (_) {
    return null;
  }
}

function downloadTextFile(filename, text, mime) {
  var blob = new Blob([String(text || '')], { type: mime || 'text/plain;charset=utf-8' });
  var url = URL.createObjectURL(blob);
  var a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  a.remove();
  URL.revokeObjectURL(url);
}

function pvfsDownloadRevisionHtml(localIdEnc, revIdEnc) {
  var localId = decodeURIComponent(localIdEnc || '');
  var revId = decodeURIComponent(revIdEnc || '');
  var data = pvfsReadRevisionSnapshot(localId, revId);
  if (!data || !data.snapshot) { alert('Revision snapshot not found in PVFS'); return; }
  var html = String(data.snapshot.content || '');
  if (!html) { alert('Revision has no HTML content'); return; }
  var gameName = slugify((data.game && data.game.name) || localId || 'game');
  downloadTextFile(gameName + '-' + revId + '.html', html, 'text/html;charset=utf-8');
}

function pvfsDownloadRevisionJson(localIdEnc, revIdEnc) {
  var localId = decodeURIComponent(localIdEnc || '');
  var revId = decodeURIComponent(revIdEnc || '');
  var data = pvfsReadRevisionSnapshot(localId, revId);
  if (!data || !data.snapshot) { alert('Revision snapshot not found in PVFS'); return; }
  var gameName = slugify((data.game && data.game.name) || localId || 'game');
  downloadTextFile(gameName + '-' + revId + '.json', JSON.stringify(data.snapshot, null, 2), 'application/json;charset=utf-8');
}

function pvfsDeleteRevision(localIdEnc, revIdEnc) {
  var localId = decodeURIComponent(localIdEnc || '');
  var revId = decodeURIComponent(revIdEnc || '');
  var games = readPvfsGames();
  var g = games[localId];
  if (!g) { alert('Game not found in PVFS'); return; }
  if (!confirm('Delete revision "' + revId + '" for "' + (g.name || localId) + '"?')) return;
  try {
    var raw = localStorage.getItem('traits.pvfs') || '{}';
    var pvfs = JSON.parse(raw);
    var idxRaw = pvfs['canvas/revisions/index.json'];
    var idx = idxRaw ? (typeof idxRaw === 'string' ? JSON.parse(idxRaw) : idxRaw) : {};
    if (!idx || typeof idx !== 'object') idx = {};

    var removedPath = '';
    var keys = Object.keys(idx);
    for (var i = 0; i < keys.length; i++) {
      var k = keys[i];
      var arr = Array.isArray(idx[k]) ? idx[k] : [];
      var keep = [];
      for (var j = 0; j < arr.length; j++) {
        var row = arr[j] || {};
        if (String(row.id || '') === revId) {
          if (!removedPath) removedPath = String(row.path || '');
          continue;
        }
        keep.push(row);
      }
      if (keep.length) idx[k] = keep;
      else delete idx[k];
    }

    if (!removedPath) removedPath = 'canvas/revisions/' + revId + '.json';
    delete pvfs[removedPath];
    if (removedPath !== ('canvas/revisions/' + revId + '.json')) {
      delete pvfs['canvas/revisions/' + revId + '.json'];
    }
    pvfs['canvas/revisions/index.json'] = JSON.stringify(idx);
    localStorage.setItem('traits.pvfs', JSON.stringify(pvfs));

    closeModal();
    renderPvfsGames();
  } catch (_) {
    alert('Failed to delete revision');
  }
}

function pvfsDeleteGame(localIdEnc) {
  var localId = decodeURIComponent(localIdEnc || '');
  var games = readPvfsGames();
  var g = games[localId];
  if (!g) { alert('Game not found in PVFS'); return; }
  if (!confirm('Delete PVFS game "' + (g.name || localId) + '" and its local revisions?')) return;

  try {
    var raw = localStorage.getItem('traits.pvfs') || '{}';
    var pvfs = JSON.parse(raw);
    var gamesRaw = pvfs['canvas/games.json'];
    var col = gamesRaw ? (typeof gamesRaw === 'string' ? JSON.parse(gamesRaw) : gamesRaw) : { active: null, games: {} };
    if (!col.games || typeof col.games !== 'object') col.games = {};

    var revs = pvfsRevisionsForGame(localId, g);
    var idxRaw = pvfs['canvas/revisions/index.json'];
    var idx = idxRaw ? (typeof idxRaw === 'string' ? JSON.parse(idxRaw) : idxRaw) : {};
    if (!idx || typeof idx !== 'object') idx = {};

    var revIdSet = {};
    for (var ri = 0; ri < revs.length; ri++) {
      var rr = revs[ri] || {};
      var rid = String(rr.id || '');
      if (!rid) continue;
      revIdSet[rid] = true;
      if (rr.path) delete pvfs[String(rr.path)];
      delete pvfs['canvas/revisions/' + rid + '.json'];
    }

    var idxKeys = Object.keys(idx);
    for (var i = 0; i < idxKeys.length; i++) {
      var k = idxKeys[i];
      var arr = Array.isArray(idx[k]) ? idx[k] : [];
      var keep = [];
      for (var j = 0; j < arr.length; j++) {
        var row = arr[j] || {};
        var rid2 = String(row.id || '');
        if (rid2 && revIdSet[rid2]) continue;
        keep.push(row);
      }
      if (keep.length) idx[k] = keep;
      else delete idx[k];
    }

    delete col.games[localId];
    if (String(col.active || '') === localId) {
      var remainIds = Object.keys(col.games);
      col.active = remainIds.length ? remainIds[0] : null;
      if (col.active && col.games[col.active] && typeof col.games[col.active].content === 'string') {
        pvfs['canvas/app.html'] = col.games[col.active].content;
      } else {
        delete pvfs['canvas/app.html'];
      }
    }

    pvfs['canvas/games.json'] = JSON.stringify(col);
    pvfs['canvas/revisions/index.json'] = JSON.stringify(idx);
    localStorage.setItem('traits.pvfs', JSON.stringify(pvfs));
    renderPvfsGames();
  } catch (_) {
    alert('Failed to delete PVFS game');
  }
}

function pvfsOpenRevision(localIdEnc, revIdEnc) {
  hidePvfsRevisionHover(true);
  var localId = decodeURIComponent(localIdEnc || '');
  var revId = decodeURIComponent(revIdEnc || '');
  var data = pvfsReadRevisionSnapshot(localId, revId);
  if (!data) { alert('Revision not found'); return; }
  var snap = data.snapshot || {};
  var content = String(snap.content || '');
  var resources = (snap.resources && typeof snap.resources === 'object') ? snap.resources : {};
  var body = '';
  body += '<h3>Revision: ' + esc((data.game && data.game.name) || localId) + '</h3>';
  body += '<p class="note"><code>' + esc(revId) + '</code> • ' + esc(data.meta.created || 'unknown time') + ' • ' + formatSize(data.meta.length || content.length || 0) + '</p>';
  body += '<p class="note">Key: <code>' + esc(data.meta.key || '') + '</code><br>Path: <code>' + esc(data.path || '') + '</code><br>Resources: ' + Object.keys(resources).length + '</p>';
  body += '<pre style="max-height:36vh;overflow:auto;background:#0a0a0f;border:1px solid var(--line);padding:10px;border-radius:8px;white-space:pre-wrap;">' + esc(content ? content.slice(0, 1600) : '(no content)') + '</pre>';
  body += '<div class="modal-actions">';
  body += '<button onclick="closeModal()">Close</button>';
  body += '<button class="btn-sm danger" onclick="pvfsDeleteRevision(\'' + encodeURIComponent(localId) + '\',\'' + encodeURIComponent(revId) + '\')">Delete Revision</button>';
  body += '<button onclick="pvfsDownloadRevisionJson(\'' + encodeURIComponent(localId) + '\',\'' + encodeURIComponent(revId) + '\')">Download JSON</button>';
  body += '<button class="primary" onclick="pvfsDownloadRevisionHtml(\'' + encodeURIComponent(localId) + '\',\'' + encodeURIComponent(revId) + '\')">Download HTML</button>';
  body += '</div>';
  showModal(body);
}

var __pvfsRevHideTimer = 0;

function ensurePvfsRevisionHover() {
  var tip = document.getElementById('pvfsRevHover');
  if (tip) return tip;
  tip = document.createElement('div');
  tip.id = 'pvfsRevHover';
  tip.className = 'rev-hover';
  tip.onmouseenter = function() {
    if (__pvfsRevHideTimer) {
      clearTimeout(__pvfsRevHideTimer);
      __pvfsRevHideTimer = 0;
    }
  };
  tip.onmouseleave = function() {
    hidePvfsRevisionHover();
  };
  document.body.appendChild(tip);
  return tip;
}

function positionPvfsRevisionHover(ev) {
  var tip = ensurePvfsRevisionHover();
  var x = (ev && typeof ev.clientX === 'number') ? ev.clientX : 0;
  var y = (ev && typeof ev.clientY === 'number') ? ev.clientY : 0;
  var pad = 12;
  var maxX = Math.max(8, window.innerWidth - tip.offsetWidth - 8);
  var maxY = Math.max(8, window.innerHeight - tip.offsetHeight - 8);
  var left = Math.min(maxX, Math.max(8, x + pad));
  var top = Math.min(maxY, Math.max(8, y + pad));
  tip.style.left = left + 'px';
  tip.style.top = top + 'px';
}

function hidePvfsRevisionHover(immediate) {
  var tip = document.getElementById('pvfsRevHover');
  if (!tip) return;
  if (__pvfsRevHideTimer) {
    clearTimeout(__pvfsRevHideTimer);
    __pvfsRevHideTimer = 0;
  }
  if (immediate) {
    tip.style.display = 'none';
    return;
  }
  __pvfsRevHideTimer = setTimeout(function() {
    var t = document.getElementById('pvfsRevHover');
    if (t) t.style.display = 'none';
    __pvfsRevHideTimer = 0;
  }, 140);
}

function showPvfsRevisionHover(ev, localIdEnc) {
  var localId = decodeURIComponent(localIdEnc || '');
  var games = readPvfsGames();
  var g = games[localId];
  if (!g) return;
  var revs = pvfsRevisionsForGame(localId, g);
  var tip = ensurePvfsRevisionHover();
  var html = '<div class="title">' + esc(g.name || localId) + ' revisions (' + revs.length + ')</div>';
  if (!revs.length) {
    html += '<div class="empty">No revisions found for this game key yet.</div>';
  } else {
    var max = Math.min(8, revs.length);
    for (var i = 0; i < max; i++) {
      var r = revs[i];
      html += '<div class="row">';
      var revEnc = encodeURIComponent(r.id || '');
      html += '<button class="rev-row-btn" onclick="pvfsOpenRevision(\'' + localIdEnc + '\',\'' + revEnc + '\')">';
      html += '<div><strong>' + esc(r.version || 'v?') + '</strong> <span style="opacity:0.85">' + esc(r.id.slice(0, 18)) + '…</span></div>';
      html += '<div class="meta">' + esc(r.created || 'unknown time') + ' • ' + formatSize(r.length || 0) + '</div>';
      html += '</button>';
      html += '</div>';
    }
    if (revs.length > max) html += '<div class="meta" style="padding-top:6px;">+' + (revs.length - max) + ' more</div>';
  }
  tip.innerHTML = html;
  tip.style.display = 'block';
  positionPvfsRevisionHover(ev);
}

function movePvfsRevisionHover(ev) {
  var tip = document.getElementById('pvfsRevHover');
  if (!tip || tip.style.display === 'none') return;
  positionPvfsRevisionHover(ev);
}

function pvfsContentMeta(g) {
  var content = typeof g.content === 'string' ? g.content : '';
  var hasInline = !!content;
  var isHtml = /<\s*html|<\s*body|<\s*canvas|<\s*script/i.test(content);
  var refCount = 0;
  try {
    var refs = [];
    content.replace(/(?:src|href)\s*=\s*["']([^"']+)["']/gi, function(_, u){ refs.push(u); return _; });
    content.replace(/url\(([^)]+)\)/gi, function(_, u){ refs.push(String(u||'').trim().replace(/^['"]|['"]$/g,'')); return _; });
    var clean = {};
    for (var i = 0; i < refs.length; i++) {
      var r = String(refs[i] || '').trim();
      if (!r) continue;
      if (/^(data:|javascript:|https?:)/i.test(r)) continue;
      clean[r] = true;
    }
    refCount = Object.keys(clean).length;
  } catch (_) {}
  return {
    hasInline: hasInline,
    isHtml: isHtml,
    bytes: content.length,
    refCount: refCount
  };
}

function renderPvfsGames() {
  var el = document.getElementById('pvfsTable');
  var status = document.getElementById('pvfsStatus');
  if (!el) return;
  var games = readPvfsGames();
  var ids = Object.keys(games);
  if (!ids.length) {
    status.textContent = 'No PVFS games in local storage.';
    el.innerHTML = '';
    return;
  }
  status.textContent = ids.length + ' local game' + (ids.length === 1 ? '' : 's') + ' in PVFS';

  var relayIndex = {};
  var allRelay = [];
  if (Array.isArray(gamesData.external)) allRelay = allRelay.concat(gamesData.external);
  if (Array.isArray(gamesData.internal)) allRelay = allRelay.concat(gamesData.internal);
  for (var ri = 0; ri < allRelay.length; ri++) {
    var rg = allRelay[ri] || {};
    var ro = String(rg.owner || '').trim().toLowerCase();
    var rgid = String(rg.game_id || '').trim().toLowerCase();
    if (ro && rgid) relayIndex[ro + '/' + rgid] = true;
  }

  var h = '<table><tr><th>Name</th><th>Version</th><th>Size</th><th>Scope</th><th>Local ID</th><th>Relay ID</th><th>Status</th><th>Storage</th><th>Content</th><th>Revisions</th><th></th></tr>';
  for (var i = 0; i < ids.length; i++) {
    var id = ids[i];
    var g = games[id];
    var name = g.name || id;
    var version = g.version || '—';
    var size = formatSize((g.content || '').length);
    var scope = g.scope || 'external';
    var syncOwner = g._sync_owner || '';
    var syncGameId = g._sync_game_id || '';
    var syncHash = g._sync_hash || '';
    var meta = pvfsContentMeta(g);
    var ownerNorm = String(syncOwner || currentUsername || '').trim().toLowerCase();
    var gameNorm = String(syncGameId || '').trim().toLowerCase();
    var syncKey = (ownerNorm && gameNorm) ? (ownerNorm + '/' + gameNorm) : '';
    var ownerMismatch = !!(syncGameId && syncOwner && currentUsername && String(syncOwner).trim().toLowerCase() !== currentUsername);
    var linkedHere = !!(syncKey && relayIndex[syncKey]);
    var isSynced = !!(syncGameId && !ownerMismatch && (linkedHere || !currentUsername));

    var statusBadge = '<span class="badge-draft">local only</span>';
    if (isSynced) {
      statusBadge = '<span class="badge-pub">synced</span>';
    } else if (syncGameId && ownerMismatch) {
      statusBadge = '<span class="badge-draft">linked to other owner</span>';
    } else if (syncGameId && !linkedHere) {
      statusBadge = '<span class="badge-draft">stale relay link</span>';
    }

    var relayCell = syncGameId
      ? ('<code title="' + esc(syncOwner || currentUsername || 'unknown') + '">' + esc(syncGameId.slice(0,10)) + '…</code>')
      : '<span style="opacity:0.3">—</span>';
    var storageCell = meta.hasInline
      ? '<span class="badge-pub">inline payload</span>'
      : '<span class="badge-draft">no content</span>';
    var contentCell = meta.hasInline
      ? ('<span>' + (meta.isHtml ? 'html' : 'text') + '</span><br><span style="opacity:0.7">' + formatSize(meta.bytes) + ', refs: ' + meta.refCount + '</span>')
      : '<span style="opacity:0.5">—</span>';
    var idEnc = encodeURIComponent(id);
    h += '<tr>';
    h += '<td><strong>' + esc(name) + '</strong></td>';
    h += '<td>' + esc(version) + '</td>';
    h += '<td>' + size + '</td>';
    h += '<td><span class="badge-scope">' + esc(scope) + '</span></td>';
    h += '<td><code>' + esc(id.slice(0,10)) + '…</code></td>';
    h += '<td>' + relayCell + '</td>';
    h += '<td>' + statusBadge + '</td>';
    h += '<td>' + storageCell + '</td>';
    h += '<td>' + contentCell + '</td>';
    var revs = pvfsRevisionsForGame(id, g);
    var idHover = encodeURIComponent(id);
    h += '<td><span class="rev-pill" onmouseenter="showPvfsRevisionHover(event,\'' + idHover + '\')" onmousemove="movePvfsRevisionHover(event)" onmouseleave="hidePvfsRevisionHover()">' + revs.length + ' rev' + (revs.length === 1 ? '' : 's') + '</span></td>';
    h += '<td class="actions">';
    h += '<button class="btn-sm" onclick="pvfsInspect(\'' + idEnc + '\')">View</button>';
    h += '<button class="btn-sm" onclick="pvfsExtract(\'' + idEnc + '\')">Extract HTML</button>';
    h += '<button class="btn-sm" onclick="pvfsCleanUpload(\'' + idEnc + '\')">Clean Upload</button>';
    h += '<button class="btn-sm accent" onclick="pvfsSyncToRelay(\'' + idEnc + '\')">Sync</button>';
    h += '<button class="btn-sm danger" onclick="pvfsDeleteGame(\'' + idEnc + '\')">Delete</button>';
    h += '</td>';
    h += '</tr>';
  }
  h += '</table>';
  el.innerHTML = h;
}

function pvfsInspect(localIdEnc) {
  var localId = decodeURIComponent(localIdEnc);
  var games = readPvfsGames();
  var g = games[localId];
  if (!g) { alert('Game not found in PVFS'); return; }
  var content = typeof g.content === 'string' ? g.content : '';
  var clone = {};
  for (var k in g) {
    if (!Object.prototype.hasOwnProperty.call(g, k)) continue;
    if (k === 'content') continue;
    clone[k] = g[k];
  }
  clone.content_bytes = content.length;
  clone.content_preview = content ? content.slice(0, 800) : '';
  var raw = JSON.stringify(clone, null, 2);
  var body = '<h3>PVFS Game: ' + esc(g.name || localId) + '</h3>'
    + '<p class="note">The full game HTML/code is stored inline in <code>canvas/games.json</code> under <code>content</code>.</p>'
    + '<pre style="max-height:50vh;overflow:auto;background:#0a0a0f;border:1px solid var(--line);padding:10px;border-radius:8px;white-space:pre-wrap;">'
    + esc(raw)
    + '</pre>'
    + '<div class="modal-actions"><button onclick="closeModal()">Close</button></div>';
  showModal(body);
}

function pvfsExtract(localIdEnc) {
  var localId = decodeURIComponent(localIdEnc);
  var games = readPvfsGames();
  var g = games[localId];
  if (!g) { alert('Game not found in PVFS'); return; }
  var content = typeof g.content === 'string' ? g.content : '';
  if (!content) { alert('No inline content to extract'); return; }
  var fileBase = slugify(g.name || localId);
  var blob = new Blob([content], { type: 'text/html;charset=utf-8' });
  var url = URL.createObjectURL(blob);
  var a = document.createElement('a');
  a.href = url;
  a.download = fileBase + '.html';
  document.body.appendChild(a);
  a.click();
  a.remove();
  URL.revokeObjectURL(url);
}

async function pvfsCleanUpload(localIdEnc) {
  var localId = decodeURIComponent(localIdEnc);
  var games = readPvfsGames();
  var g = games[localId];
  if (!g) { alert('Game not found in PVFS'); return; }
  var t = getToken();
  if (!t) { alert('Log in first (Settings) to upload'); return; }

  var content = typeof g.content === 'string' ? g.content : '';
  if (!content.trim()) { alert('Game has no content'); return; }

  var gameId = slugify(g.name || localId);
  if (!confirm('Clean upload "' + (g.name || localId) + '" as game_id "' + gameId + '"?')) return;

  var body = {
    name: g.name || 'Untitled',
    content: content,
    scope: 'internal',
    version: g.version || ''
  };

  try {
    var r = await relayFetch('/internal/game/' + encodeURIComponent(gameId), {
      method: 'PUT',
      headers: { 'Authorization': 'Bearer ' + t, 'Content-Type': 'application/json' },
      body: JSON.stringify(body)
    });
    var d = await r.json().catch(function(){ return {}; });
    if (!r.ok) {
      var msg = d.error || ('Upload failed (HTTP ' + r.status + ')');
      if (r.status >= 500) msg = 'Relay unavailable (HTTP ' + r.status + '). Please retry.';
      alert(msg);
      return;
    }

    // Rewrite local metadata with clean current-owner linkage.
    try {
      var me = await apiFetch('/auth/me');
      var owner = (me && me.ok && me.username) ? String(me.username).trim().toLowerCase() : '';
      var raw = localStorage.getItem('traits.pvfs');
      var pvfs = raw ? JSON.parse(raw) : {};
      var gamesRaw = pvfs['canvas/games.json'];
      var data = gamesRaw ? (typeof gamesRaw === 'string' ? JSON.parse(gamesRaw) : gamesRaw) : { games: {} };
      if (!data.games) data.games = {};
      if (data.games[localId]) {
        data.games[localId]._sync_game_id = d.game_id || gameId;
        data.games[localId]._sync_hash = d.content_hash || d.checksum || '';
        data.games[localId]._sync_owner = d.owner || owner || '';
        data.games[localId].scope = 'internal';
        data.games[localId]._scope = 'internal';
        data.games[localId].owner = d.owner || owner || '';
      }
      pvfs['canvas/games.json'] = JSON.stringify(data);
      localStorage.setItem('traits.pvfs', JSON.stringify(pvfs));
    } catch (_) {}

    await load();
    renderPvfsGames();
  } catch (e) {
    alert((e && e.message) ? e.message : 'Upload failed');
  }
}

async function pvfsSyncToRelay(localIdEnc) {
  var localId = decodeURIComponent(localIdEnc);
  var games = readPvfsGames();
  var g = games[localId];
  if (!g) { alert('Game not found in PVFS'); return; }
  var t = getToken();
  if (!t) { alert('Log in first (Settings) to sync to relay'); return; }
  var gameId = g._sync_game_id || localId;
  var body = {
    name: g.name || 'Untitled',
    content: g.content || '',
    scope: g.scope || 'external',
    version: g.version || ''
  };
  try {
    var r = await relayFetch('/internal/game/' + encodeURIComponent(gameId), {
      method: 'PUT',
      headers: { 'Authorization': 'Bearer ' + t, 'Content-Type': 'application/json' },
      body: JSON.stringify(body)
    });
    var d = await r.json().catch(function(){ return {}; });
    if (!r.ok) {
      var msg = d.error || ('Sync failed (HTTP ' + r.status + ')');
      if (r.status >= 500) msg = 'Relay unavailable (HTTP ' + r.status + '). Please retry.';
      alert(msg);
      return;
    }
    // Update PVFS sync metadata
    try {
      var raw = localStorage.getItem('traits.pvfs');
      var pvfs = raw ? JSON.parse(raw) : {};
      var gamesData2 = pvfs['canvas/games.json'];
      var data = gamesData2 ? (typeof gamesData2 === 'string' ? JSON.parse(gamesData2) : gamesData2) : { games: {} };
      if (!data.games) data.games = {};
      if (data.games[localId]) {
        data.games[localId]._sync_game_id = d.game_id || gameId;
        data.games[localId]._sync_hash = d.content_hash || '';
        data.games[localId]._sync_owner = d.owner || '';
      }
      pvfs['canvas/games.json'] = JSON.stringify(data);
      localStorage.setItem('traits.pvfs', JSON.stringify(pvfs));
    } catch(_) {}
    renderPvfsGames();
  } catch(e) {
    alert((e && e.message) ? e.message : 'Sync failed');
  }
}

window.switchTab = switchTab;
window.deleteUser = deleteUser;
window.editUser = editUser;
window.submitEditUser = submitEditUser;
window.deleteGame = deleteGame;
window.toggleAdminPublish = toggleAdminPublish;
window.playAdminGame = playAdminGame;
window.assignGame = assignGame;
window.submitAssignGame = submitAssignGame;
window.manageSecrets = manageSecrets;
window.addUserSecret = addUserSecret;
window.deleteUserSecret = deleteUserSecret;
window.closeModal = closeModal;
window.pvfsSyncToRelay = pvfsSyncToRelay;
window.pvfsInspect = pvfsInspect;
window.pvfsExtract = pvfsExtract;
window.pvfsCleanUpload = pvfsCleanUpload;
window.pvfsDeleteGame = pvfsDeleteGame;
window.pvfsDeleteRevision = pvfsDeleteRevision;
window.pvfsOpenRevision = pvfsOpenRevision;
window.pvfsDownloadRevisionHtml = pvfsDownloadRevisionHtml;
window.pvfsDownloadRevisionJson = pvfsDownloadRevisionJson;
window.showPvfsRevisionHover = showPvfsRevisionHover;
window.movePvfsRevisionHover = movePvfsRevisionHover;
window.hidePvfsRevisionHover = hidePvfsRevisionHover;
window.copyAdminTerminalCommand = copyAdminTerminalCommand;
window.runAdminTerminalCommand = runAdminTerminalCommand;
window.refreshGithubCatalog = refreshGithubCatalog;
window.toggleGithubSprites = toggleGithubSprites;
window.disableGithubGame = disableGithubGame;
window.enableGithubGame = enableGithubGame;
window.renameGithubGame = renameGithubGame;
window.deleteGithubGame = deleteGithubGame;
load();
renderPvfsGames();
refreshGithubCatalog();
initAdminTerminal();
})();
"##;
