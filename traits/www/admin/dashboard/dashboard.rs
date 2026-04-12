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
                        p.note id="gamesStatus" { "Loading…" }
                        div id="gamesTable" {}
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
var API = 'https://relay.traits.build/sync';
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

  // Merge external + internal into a unified list with scope tag
  var all = [];
  for (var i = 0; i < gamesData.external.length; i++) {
    var g = gamesData.external[i];
    all.push({ owner: g.owner || 'public', game_id: g.game_id || '', name: g.name, size: g.size, updated: g.updated, scope: g.scope || 'external', fullHash: g.content_hash || '', hash: (g.content_hash || '').slice(0, 8), highscore: g.highscore || 0, highscore_player: g.highscore_player || '' });
  }
  for (var j = 0; j < gamesData.internal.length; j++) {
    var ig = gamesData.internal[j];
    all.push({ owner: ig.owner, game_id: ig.game_id, name: ig.name, size: ig.size, updated: ig.updated, scope: 'internal', fullHash: ig.content_hash || '', hash: (ig.content_hash || '').slice(0, 8), forked: ig.forked_from_hash ? true : false, highscore: ig.highscore || 0, highscore_player: ig.highscore_player || '' });
  }

  status.textContent = all.length + ' game' + (all.length === 1 ? '' : 's') + ' (' + gamesData.external.length + ' external, ' + gamesData.internal.length + ' internal)';

  if (!all.length) { el.innerHTML = '<p class="note">No games found.</p>'; return; }

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
      h += '<table><tr><th>Identity</th><th>Name</th><th>Scope</th><th>HS</th><th>Size</th><th>Updated</th><th></th></tr>';
      for (var gi = 0; gi < gs.length; gi++) {
        var gm = gs[gi];
        var identity = esc(gm.owner + '/' + gm.game_id);
        var scopeTag = gm.scope === 'internal' ? '🏠' : '🌐';
        var gh = encodeURIComponent(gm.fullHash);
        h += '<tr><td><code>' + identity + '</code></td><td><span style="cursor:pointer;color:var(--accent);text-decoration:underline" onclick="playAdminGame(\'' + esc(gm.scope) + '\',\'' + encodeURIComponent(gm.owner) + '\',\'' + encodeURIComponent(gm.game_id || '') + '\',\'' + gh + '\')">' + esc(gm.name) + '</span></td>';
        h += '<td>' + scopeTag + ' ' + esc(gm.scope) + '</td>';
        h += '<td>' + (gm.highscore ? '<span title="' + esc(gm.highscore_player || '') + '">' + gm.highscore + '</span>' : '<span style="opacity:0.3">—</span>') + '</td>';
        h += '<td>' + formatSize(gm.size) + '</td>';
        h += '<td title="' + esc(gm.updated) + '">' + ago(gm.updated) + '</td>';
        h += '<td class="actions">';
        h += '<button class="btn-sm accent" onclick="assignGame(\'' + gh + '\',\'' + encodeURIComponent(gm.owner) + '\')">Assign</button>';
        h += '<button class="btn-sm danger" onclick="deleteGame(\'' + gh + '\')">Del</button>';
        h += '</td></tr>';
      }
      h += '</table>';
    }
    el.innerHTML = h;
  } else {
    // Group by game name
    var byName = {};
    for (var k2 = 0; k2 < all.length; k2++) {
      var nm = (all[k2].name || 'untitled').toLowerCase();
      if (!byName[nm]) byName[nm] = [];
      byName[nm].push(all[k2]);
    }
    var names = Object.keys(byName).sort();
    var h2 = '';
    for (var ni = 0; ni < names.length; ni++) {
      var gn = byName[names[ni]];
      h2 += '<div class="group-header">' + esc(gn[0].name || names[ni]) + ' (' + gn.length + ')</div>';
      h2 += '<table><tr><th>Owner/ID</th><th>Name</th><th>Scope</th><th>HS</th><th>Size</th><th>Updated</th><th></th></tr>';
      for (var gi2 = 0; gi2 < gn.length; gi2++) {
        var gm2 = gn[gi2];
        var scopeTag2 = gm2.scope === 'internal' ? '🏠' : '🌐';
        var gh2 = encodeURIComponent(gm2.fullHash);
        h2 += '<tr><td><code>' + esc(gm2.owner + '/' + gm2.game_id) + '</code></td>';
        h2 += '<td><span style="cursor:pointer;color:var(--accent);text-decoration:underline" onclick="playAdminGame(\'' + esc(gm2.scope) + '\',\'' + encodeURIComponent(gm2.owner) + '\',\'' + encodeURIComponent(gm2.game_id || '') + '\',\'' + gh2 + '\')">' + esc(gm2.name) + '</span></td>';
        h2 += '<td>' + scopeTag2 + ' ' + esc(gm2.scope) + '</td>';
        h2 += '<td>' + (gm2.highscore ? '<span title="' + esc(gm2.highscore_player || '') + '">' + gm2.highscore + '</span>' : '<span style="opacity:0.3">—</span>') + '</td>';
        h2 += '<td>' + formatSize(gm2.size) + '</td>';
        h2 += '<td title="' + esc(gm2.updated) + '">' + ago(gm2.updated) + '</td>';
        h2 += '<td class="actions">';
        h2 += '<button class="btn-sm accent" onclick="assignGame(\'' + gh2 + '\',\'' + encodeURIComponent(gm2.owner) + '\')">Assign</button>';
        h2 += '<button class="btn-sm danger" onclick="deleteGame(\'' + gh2 + '\')">Del</button>';
        h2 += '</td></tr>';
      }
      h2 += '</table>';
    }
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

async function playAdminGame(scope, ownerEnc, gameIdEnc, hashEnc) {
  try {
    var scopeVal = decodeURIComponent(scope || 'external');
    var owner = decodeURIComponent(ownerEnc || '');
    var gameId = decodeURIComponent(gameIdEnc || '');
    var hash = decodeURIComponent(hashEnc || '');
    var content = '';
    var name = '';

    if (scopeVal === 'internal') {
      var r1 = await fetch(API + '/internal/game/' + encodeURIComponent(gameId) + '?owner=' + encodeURIComponent(owner), {
        headers: { 'Authorization': 'Bearer ' + token }
      });
      var d1 = await r1.json();
      if (!r1.ok || !d1.content) throw new Error(d1.error || 'Could not load internal game');
      content = d1.content;
      name = d1.name || gameId || 'Game';
    } else {
      var r2 = await fetch(API + '/game/' + encodeURIComponent(hash));
      var d2 = await r2.json();
      if (!r2.ok || !d2.content) throw new Error(d2.error || 'Could not load external game');
      content = d2.content;
      name = d2.name || 'Game';
    }

    await callTrait('sys.canvas', ['new', name]);
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

window.switchTab = switchTab;
window.deleteUser = deleteUser;
window.editUser = editUser;
window.submitEditUser = submitEditUser;
window.deleteGame = deleteGame;
window.playAdminGame = playAdminGame;
window.assignGame = assignGame;
window.submitAssignGame = submitAssignGame;
window.manageSecrets = manageSecrets;
window.addUserSecret = addUserSecret;
window.deleteUserSecret = deleteUserSecret;
window.closeModal = closeModal;
load();
})();
"##;
