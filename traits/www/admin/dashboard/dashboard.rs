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

  var h = '<table><tr><th>Username</th><th>Email</th><th>Role</th><th>Created</th><th>Last Login</th></tr>';
  for (var i = 0; i < usersData.length; i++) {
    var u = usersData[i];
    var roleBadge = u.role === 'admin'
      ? '<span class="badge-admin">admin</span>'
      : '<span class="badge-user">user</span>';
    h += '<tr>';
    h += '<td><strong>' + esc(u.username) + '</strong></td>';
    h += '<td>' + esc(u.email) + '</td>';
    h += '<td>' + roleBadge + '</td>';
    h += '<td title="' + esc(u.created) + '">' + ago(u.created) + '</td>';
    h += '<td title="' + esc(u.last_login) + '">' + (u.last_login ? ago(u.last_login) : '—') + '</td>';
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
    all.push({ owner: g.owner || 'public', game_id: g.game_id || '', name: g.name, size: g.size, updated: g.updated, scope: g.scope || 'external', hash: (g.content_hash || '').slice(0, 8) });
  }
  for (var j = 0; j < gamesData.internal.length; j++) {
    var ig = gamesData.internal[j];
    all.push({ owner: ig.owner, game_id: ig.game_id, name: ig.name, size: ig.size, updated: ig.updated, scope: 'internal', hash: (ig.content_hash || '').slice(0, 8), forked: ig.forked_from_hash ? true : false });
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
      h += '<table><tr><th>Identity</th><th>Name</th><th>Scope</th><th>Size</th><th>Updated</th></tr>';
      for (var gi = 0; gi < gs.length; gi++) {
        var gm = gs[gi];
        var identity = esc(gm.owner + '/' + gm.game_id);
        var scopeTag = gm.scope === 'internal' ? '🏠' : '🌐';
        h += '<tr><td><code>' + identity + '</code></td><td>' + esc(gm.name) + '</td>';
        h += '<td>' + scopeTag + ' ' + esc(gm.scope) + '</td>';
        h += '<td>' + formatSize(gm.size) + '</td>';
        h += '<td title="' + esc(gm.updated) + '">' + ago(gm.updated) + '</td></tr>';
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
      h2 += '<table><tr><th>Owner/ID</th><th>Scope</th><th>Size</th><th>Updated</th></tr>';
      for (var gi2 = 0; gi2 < gn.length; gi2++) {
        var gm2 = gn[gi2];
        var scopeTag2 = gm2.scope === 'internal' ? '🏠' : '🌐';
        h2 += '<tr><td><code>' + esc(gm2.owner + '/' + gm2.game_id) + '</code></td>';
        h2 += '<td>' + scopeTag2 + ' ' + esc(gm2.scope) + '</td>';
        h2 += '<td>' + formatSize(gm2.size) + '</td>';
        h2 += '<td title="' + esc(gm2.updated) + '">' + ago(gm2.updated) + '</td></tr>';
      }
      h2 += '</table>';
    }
    el.innerHTML = h2;
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

window.switchTab = switchTab;
load();
})();
"##;
