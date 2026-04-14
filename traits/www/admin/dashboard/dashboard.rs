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
@media (max-width: 720px) {
  .page { padding: 18px 14px 32px; }
  .card { padding: 16px; }
  table { font-size: 11px; }
  th, td { padding: 6px 6px; }
}
"##;

const JS: &str = r##"
(function() {
var API = 'https://relay.slob.games/sync';
var token = '';
var usersData = [];
var gamesData = { external: [], internal: [] };
var currentTab = 'byOwner';

function esc(v) {
  var d = document.createElement('div');
  d.textContent = String(v == null ? '' : v);
  return d.innerHTML;
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
  var r = await fetch(API + path, { headers: { 'Authorization': 'Bearer ' + token } });
  var text = await r.text();
  var body = {};
  try { body = text ? JSON.parse(text) : {}; } catch (_) { body = {}; }
  if (!r.ok) return { ok: false, error: body.error || ('HTTP ' + r.status) };
  return body;
}

async function apiDelete(path) {
  var r = await fetch(API + path, { method: 'DELETE', headers: { 'Authorization': 'Bearer ' + token } });
  var text = await r.text();
  var body = {};
  try { body = text ? JSON.parse(text) : {}; } catch (_) { body = {}; }
  if (!r.ok) return { ok: false, error: body.error || ('HTTP ' + r.status) };
  if (typeof body.ok === 'undefined') body.ok = true;
  return body;
}

async function apiPut(path, body) {
  var r = await fetch(API + path, {
    method: 'PUT',
    headers: { 'Authorization': 'Bearer ' + token, 'Content-Type': 'application/json' },
    body: JSON.stringify(body)
  });
  return r.json();
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

  // Fetch users and games in parallel
  var p = await Promise.all([apiFetch('/admin/users'), apiFetch('/admin/games')]);
  usersData = Array.isArray(p[0]) ? p[0] : [];
  gamesData = p[1] && p[1].external ? p[1] : { external: [], internal: [] };

  renderUsers();
  renderGames();
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
  return res && res.result !== undefined ? res.result : res;
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
    var hash = decodeURIComponent(hashEnc || '');
    var content = '';
    var name = '';
    var version = '';

    var r = await fetch(API + '/game/' + encodeURIComponent(hash));
    var d = await r.json();
    if (!r.ok || !d.content) throw new Error(d.error || 'Could not load game');
    content = d.content;
    name = d.name || gameId || 'Game';
    version = d.version || '';

    await callTrait('sys.canvas', ['new', name, version]);
    await callTrait('sys.canvas', ['set', content]);
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
  var r = await fetch(API + '/internal/game/' + encodeURIComponent(gameId) + '/publish', {
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
  var h = '<table><tr><th>Name</th><th>Version</th><th>Size</th><th>Scope</th><th>Local ID</th><th>Relay ID</th><th>Status</th><th></th></tr>';
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
    var isSynced = !!syncGameId;
    var statusBadge = isSynced
      ? '<span class="badge-pub">synced</span>'
      : '<span class="badge-draft">local only</span>';
    var relayCell = isSynced
      ? ('<code title="' + esc(syncOwner) + '">' + esc(syncGameId.slice(0,10)) + '…</code>')
      : '<span style="opacity:0.3">—</span>';
    var idEnc = encodeURIComponent(id);
    h += '<tr>';
    h += '<td><strong>' + esc(name) + '</strong></td>';
    h += '<td>' + esc(version) + '</td>';
    h += '<td>' + size + '</td>';
    h += '<td><span class="badge-scope">' + esc(scope) + '</span></td>';
    h += '<td><code>' + esc(id.slice(0,10)) + '…</code></td>';
    h += '<td>' + relayCell + '</td>';
    h += '<td>' + statusBadge + '</td>';
    h += '<td class="actions">';
    h += '<button class="btn-sm accent" onclick="pvfsSyncToRelay(\'' + idEnc + '\')">Sync</button>';
    h += '</td>';
    h += '</tr>';
  }
  h += '</table>';
  el.innerHTML = h;
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
    var r = await fetch(API + '/internal/game/' + encodeURIComponent(gameId), {
      method: 'PUT',
      headers: { 'Authorization': 'Bearer ' + t, 'Content-Type': 'application/json' },
      body: JSON.stringify(body)
    });
    var d = await r.json().catch(function(){ return {}; });
    if (!r.ok) { alert(d.error || 'Sync failed (HTTP ' + r.status + ')'); return; }
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
load();
renderPvfsGames();
})();
"##;
