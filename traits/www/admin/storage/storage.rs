use maud::{html, DOCTYPE, PreEscaped};
use serde_json::Value;

pub fn storage(_args: &[Value]) -> Value {
    let markup = html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { "slob.games — Storage Inspector" }
                style { (PreEscaped(CSS)) }
            }
            body {
                div.page {
                    section.hero.card {
                        p.eyebrow { "storage inspector" }
                        h1 { "localStorage" }
                        p.subtitle { "Disk usage, games, sprites, and quota." }
                        div.badges {
                            span.badge id="quotaBadge" { "—" }
                            span.badge id="usedBadge" { "—" }
                            span.badge id="freeBadge" { "—" }
                            a.badge style="cursor:pointer;text-decoration:none" onclick="location.hash='#/settings'" { "← Settings" }
                        }
                    }

                    // Quota bar
                    section.card id="quotaCard" {
                        h2 { "Quota" }
                        div.quota-bar-container {
                            div.quota-bar id="quotaBar" {}
                        }
                        div.quota-labels {
                            span id="quotaUsedLabel" { "—" }
                            span id="quotaFreeLabel" { "—" }
                        }
                    }

                    // Top keys
                    section.card {
                        h2 { "All Keys" }
                        table id="keysTable" {
                            tr { td colspan="3" { "Loading…" } }
                        }
                    }

                    // VFS Breakdown
                    section.card {
                        h2 { "VFS Breakdown" }
                        div.kernel-grid id="vfsStats" {
                            div.kstat {
                                span.kstat-value id="pvfsSize" { "—" }
                                span.kstat-label { "pvfs total" }
                            }
                            div.kstat {
                                span.kstat-value id="gamesSize" { "—" }
                                span.kstat-label { "games" }
                            }
                            div.kstat {
                                span.kstat-value id="spritesSize" { "—" }
                                span.kstat-label { "sprites" }
                            }
                            div.kstat {
                                span.kstat-value id="otherSize" { "—" }
                                span.kstat-label { "other" }
                            }
                        }
                    }

                    // Games
                    section.card {
                        h2 { "Games" }
                        p.note id="gamesCount" { "Loading…" }
                        table id="gamesTable" {
                            tr { td colspan="5" { "Loading…" } }
                        }
                    }

                    // Sprites
                    section.card {
                        h2 { "Sprites & Resources" }
                        p.note id="spritesCount" { "Loading…" }
                        table id="spritesTable" {
                            tr { td colspan="2" { "Loading…" } }
                        }
                    }

                    // Other VFS
                    section.card {
                        h2 { "Other VFS Files" }
                        table id="otherTable" {
                            tr { td colspan="2" { "Loading…" } }
                        }
                    }
                }

                // Game preview modal
                div.modal-overlay id="gameModal" style="display:none" {
                    div.modal {
                        div.modal-header {
                            span.modal-title id="modalTitle" { "" }
                            div.modal-actions {
                                button.primary id="modalUnhide" onclick="unhideGame()" { "Unhide → Games" }
                            button.primary id="modalMoveIdb" style="display:none" onclick="movePreviewToIndexedDb()" { "Move → IndexedDB" }
                            button.danger id="modalDelete" style="display:none" onclick="deletePreviewItem()" { "Delete" }
                                button.danger onclick="closeModal()" { "Close" }
                            }
                        }
                        iframe id="modalFrame" {}
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
.card {
  background: linear-gradient(180deg, rgba(17,17,26,0.97), rgba(12,12,18,0.97));
  border: 1px solid rgba(0,224,255,0.07);
  border-radius: 14px;
  padding: 20px;
  box-shadow: 0 20px 48px rgba(0,0,0,0.3);
  margin-bottom: 18px;
}
.hero {
  position: relative; overflow: hidden;
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
  margin: 0 0 8px; color: var(--accent);
  text-transform: uppercase; letter-spacing: 0.16em; font-size: 11px;
  font-family: 'Courier New', Menlo, monospace;
}
h1 {
  margin: 0; font-family: 'Courier New', Menlo, monospace;
  font-size: clamp(34px, 5vw, 56px); line-height: 0.96;
  text-transform: uppercase; letter-spacing: 0.04em;
}
h2 {
  margin: 0 0 12px; font-family: 'Courier New', Menlo, monospace;
  font-size: 18px; text-transform: uppercase; letter-spacing: 0.06em;
  color: var(--accent);
}
.subtitle { margin: 14px 0 0; color: var(--muted); line-height: 1.6; }
.badges { display: flex; gap: 8px; flex-wrap: wrap; margin-top: 16px; }
.badge {
  border: 1px solid rgba(0,224,255,0.18); color: var(--accent);
  border-radius: 999px; padding: 6px 10px; font-size: 12px;
  letter-spacing: 0.04em; font-family: 'Courier New', Menlo, monospace;
}
.note { color: var(--muted); line-height: 1.5; font-size: 14px; margin-bottom: 12px; }
table { width: 100%; border-collapse: collapse; }
td {
  padding: 10px 0; border-bottom: 1px solid rgba(30,30,46,0.65);
  vertical-align: top; font-size: 14px;
}
td:first-child { color: var(--muted); }
code {
  font-family: 'Courier New', Menlo, monospace; font-size: 13px;
  color: #66f0ff; background: rgba(0,224,255,0.05);
  padding: 3px 6px; border-radius: 8px;
}
button {
  border-radius: 10px; border: 1px solid var(--line);
  background: rgba(10,10,16,0.92); color: var(--text);
  font: inherit; padding: 8px 14px; cursor: pointer;
}
button:hover { border-color: #3d5b6c; }
button.primary {
  background: linear-gradient(180deg, rgba(0,224,255,0.18), rgba(0,180,220,0.12));
  border-color: rgba(0,224,255,0.25); color: #00e0ff;
}
button.danger {
  background: linear-gradient(180deg, #6b2530, #531b24);
  border-color: rgba(239,107,115,0.26); color: #ef6b73;
}
button.sm { padding: 5px 10px; font-size: 12px; }
a.game-link {
  color: #66f0ff; cursor: pointer; text-decoration: none;
}
a.game-link:hover { text-decoration: underline; }

/* Quota bar */
.quota-bar-container {
  width: 100%; height: 24px; border-radius: 12px;
  background: rgba(30,30,46,0.5); overflow: hidden;
  border: 1px solid rgba(0,224,255,0.08);
}
.quota-bar {
  height: 100%; border-radius: 12px 0 0 12px; width: 0%;
  background: linear-gradient(90deg, rgba(0,224,255,0.3), rgba(0,224,255,0.5));
  transition: width 0.6s ease;
}
.quota-labels {
  display: flex; justify-content: space-between; margin-top: 8px;
  font-size: 13px; color: var(--muted);
  font-family: 'Courier New', Menlo, monospace;
}

/* Kernel stats grid */
.kernel-grid {
  display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px;
}
.kstat {
  display: flex; flex-direction: column; align-items: center;
  padding: 14px 8px; border-radius: 10px;
  background: rgba(0,224,255,0.03); border: 1px solid rgba(0,224,255,0.08);
}
.kstat-value {
  font-family: 'Courier New', Menlo, monospace;
  font-size: 22px; font-weight: 700; color: var(--accent); line-height: 1;
}
.kstat-label {
  font-size: 11px; color: var(--muted);
  text-transform: uppercase; letter-spacing: 0.08em; margin-top: 6px;
}

/* Active marker */
.active-tag {
  display: inline-block; background: rgba(0,255,136,0.12);
  color: var(--green); border-radius: 6px; padding: 2px 6px;
  font-size: 11px; font-family: 'Courier New', Menlo, monospace;
  margin-left: 6px;
}
.ext-tag {
  display: inline-block; background: rgba(245,185,66,0.12);
  color: var(--warn); border-radius: 6px; padding: 2px 6px;
  font-size: 11px; font-family: 'Courier New', Menlo, monospace;
  margin-left: 6px;
}
.hash-tag {
  color: var(--muted); font-size: 12px;
  font-family: 'Courier New', Menlo, monospace;
}

/* Modal */
.modal-overlay {
  position: fixed; inset: 0; z-index: 9999;
  background: rgba(0,0,0,0.85);
  display: flex; align-items: center; justify-content: center;
}
.modal {
  width: 90vw; max-width: 420px;
  height: 85vh; height: 85dvh;
  max-height: 800px;
  background: var(--panel); border: 1px solid rgba(0,224,255,0.12);
  border-radius: 16px; display: flex; flex-direction: column;
  overflow: hidden; box-shadow: 0 40px 80px rgba(0,0,0,0.5);
}
.modal-header {
  display: flex; align-items: center; justify-content: space-between;
  padding: 14px 16px; border-bottom: 1px solid var(--line);
  flex-shrink: 0;
}
.modal-title {
  font-family: 'Courier New', Menlo, monospace;
  font-size: 15px; color: var(--accent);
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  flex: 1; margin-right: 12px;
}
.modal-actions { display: flex; gap: 8px; flex-shrink: 0; }
#modalFrame {
  flex: 1; border: none; width: 100%; background: #000;
  border-radius: 0 0 16px 16px;
}
.status-msg {
  color: var(--green); font-size: 12px; margin-left: 8px;
  font-family: 'Courier New', Menlo, monospace;
}

@media (max-width: 720px) {
  .page { padding: 18px 14px 32px; }
  .card { padding: 16px; border-radius: 12px; }
  .kernel-grid { grid-template-columns: repeat(2, 1fr); }
  .modal-overlay { background: #000; }
  .modal {
    width: 100vw; max-width: none;
    height: 100vh; height: 100dvh;
    max-height: none;
    border-radius: 0; border: none;
  }
  .modal-header { padding: 10px 14px; }
  .modal-actions { flex-wrap: wrap; }
  #modalFrame { border-radius: 0; }
}
"##;

const JS: &str = r##"
(function() {
var QUOTA = 5 * 1024 * 1024; // 5 MB standard localStorage quota
var _gameSortColumn = 'active'; // '' for active/size default, 'name', 'size'
var _gameSortAsc = false; // false = descending, true = ascending

function fmtSize(n) {
  n = n || 0;
  if (n >= 1024 * 1024) return (n / 1024 / 1024).toFixed(1) + ' MB';
  if (n >= 1024) return (n / 1024).toFixed(1) + ' KB';
  return n + ' B';
}

function esc(v) {
  var d = document.createElement('div');
  d.textContent = String(v == null ? '' : v);
  return d.innerHTML;
}

function byId(id) { return document.getElementById(id); }

function browserVfs() {
  return (typeof window !== 'undefined' && window.__traitsBrowserVfs) ? window.__traitsBrowserVfs : null;
}

async function listIndexedDbEntries() {
  var vfs = browserVfs();
  if (!vfs || typeof vfs.listIndexedDbEntries !== 'function') return [];
  try { return await vfs.listIndexedDbEntries(''); } catch(_) { return []; }
}

async function movePathToIndexedDb(path) {
  var vfs = browserVfs();
  if (!vfs || typeof vfs.moveToIndexedDb !== 'function') throw new Error('IndexedDB VFS unavailable');
  return vfs.moveToIndexedDb(path);
}

async function deletePathEverywhere(path) {
  var vfs = browserVfs();
  if (vfs && typeof vfs.deletePath === 'function') {
    return vfs.deletePath(path);
  }
  var deleted = false;
  try {
    var pvfs = JSON.parse(localStorage.getItem('traits.pvfs') || '{}');
    if (Object.prototype.hasOwnProperty.call(pvfs, path)) {
      delete pvfs[path];
      deleted = true;
    }
    if (pvfs.files && typeof pvfs.files === 'object' && Object.prototype.hasOwnProperty.call(pvfs.files, path)) {
      delete pvfs.files[path];
      deleted = true;
    }
    if (deleted) localStorage.setItem('traits.pvfs', JSON.stringify(pvfs));
  } catch(_) {}
  return { deleted: deleted };
}

// ══════════════════════════════════════════════════════════════
// Parse all localStorage data
// ══════════════════════════════════════════════════════════════
async function inspect() {
  var totalUsed = 0;
  var keys = [];
  for (var i = 0; i < localStorage.length; i++) {
    var k = localStorage.key(i);
    if (!k) continue;
    var val = localStorage.getItem(k) || '';
    var bytes = (k.length + val.length) * 2;
    totalUsed += bytes;
    keys.push({ key: k, size: bytes });
  }
  keys.sort(function(a, b) { return b.size - a.size; });

  var pvfsRaw = localStorage.getItem('traits.pvfs') || '{}';
  var pvfsBytes = (('traits.pvfs').length + pvfsRaw.length) * 2;
  var files;
  try { files = JSON.parse(pvfsRaw); } catch(_) { files = {}; }
  var flat = files.files ? {} : files;
  if (files.files) {
    for (var fk in files.files) {
      if (!files.files.hasOwnProperty(fk)) continue;
      var fv = files.files[fk];
      flat[fk] = (typeof fv === 'object' && fv.content !== undefined) ? fv.content : fv;
    }
  }

  // Parse games
  var games = [];
  var gamesTotal = 0;
  var activeId = null;
  var gamesJson = flat['canvas/games.json'];
  if (gamesJson) {
    try {
      var col = JSON.parse(gamesJson);
      activeId = col.active || null;
      var gamesMap = col.games || {};
      for (var id in gamesMap) {
        if (!gamesMap.hasOwnProperty(id)) continue;
        var g = gamesMap[id];
        var content = g.content || '';
        var sz = content.length * 2;
        gamesTotal += sz;
        games.push({
          id: id,
          name: g.name || id,
          scope: g.scope || g._scope || 'internal',
          size: sz,
          hash: (g._sync_hash || g.checksum || '').slice(0, 8),
          active: col.active === id,
          content: content,
          owner: g._sync_owner || g.owner || 'local',
          game_id: g._sync_game_id || g.game_id || '',
          created: g.created || '',
          updated: g.updated || '',
        });
      }
      games.sort(function(a, b) {
        if (a.active !== b.active) return a.active ? -1 : 1;
        return b.size - a.size;
      });
    } catch(_) {}
  }

  // Sprites and other VFS
  var sprites = [];
  var spritesTotal = 0;
  var otherVfs = [];
  var otherTotal = 0;
  var seenPaths = {};
  for (var path in flat) {
    if (!flat.hasOwnProperty(path) || path === 'canvas/games.json') continue;
    var val = flat[path];
    var sz = (typeof val === 'string' ? val.length : JSON.stringify(val).length) * 2;
    var isSprite = /^canvas\/sprites\/|\.png$|\.svg$|\.gif$|\.jpg$|\.webp$/i.test(path);
    if (isSprite) {
      spritesTotal += sz;
      sprites.push({ path: path, size: sz, content: typeof val === 'string' ? val : JSON.stringify(val), storage: 'localStorage' });
    } else {
      otherTotal += sz;
      otherVfs.push({ path: path, size: sz, content: typeof val === 'string' ? val : JSON.stringify(val), storage: 'localStorage' });
    }
    seenPaths[path] = true;
  }

  var idbEntries = await listIndexedDbEntries();
  for (var j = 0; j < idbEntries.length; j++) {
    var entry = idbEntries[j];
    if (!entry || !entry.path || seenPaths[entry.path] || entry.path === 'canvas/games.json') continue;
    var idbContent = typeof entry.content === 'string' ? entry.content : JSON.stringify(entry.content);
    var idbSize = idbContent.length * 2;
    var idbSprite = /^canvas\/sprites\/|\.png$|\.svg$|\.gif$|\.jpg$|\.webp$/i.test(entry.path);
    if (idbSprite) {
      spritesTotal += idbSize;
      sprites.push({ path: entry.path, size: idbSize, content: idbContent, storage: 'indexeddb' });
    } else {
      otherTotal += idbSize;
      otherVfs.push({ path: entry.path, size: idbSize, content: idbContent, storage: 'indexeddb' });
    }
  }
  sprites.sort(function(a, b) { return b.size - a.size; });
  otherVfs.sort(function(a, b) { return b.size - a.size; });

  return {
    totalUsed: totalUsed,
    quota: QUOTA,
    keys: keys,
    pvfsBytes: pvfsBytes,
    games: games,
    gamesTotal: gamesTotal,
    sprites: sprites,
    spritesTotal: spritesTotal,
    otherVfs: otherVfs,
    otherTotal: otherTotal,
    activeId: activeId,
  };
}

// ══════════════════════════════════════════════════════════════
// Render functions
// ══════════════════════════════════════════════════════════════
var _data = null;

function storageBadge(storage) {
  var s = String(storage || 'localStorage');
  return '<span class="hash-tag">' + (s === 'indexeddb' ? 'idb' : 'local') + '</span>';
}

async function render() {
  _data = await inspect();
  var d = _data;
  var pct = d.quota > 0 ? Math.round(d.totalUsed / d.quota * 100) : 0;

  byId('quotaBadge').textContent = pct + '% used';
  byId('usedBadge').textContent = fmtSize(d.totalUsed) + ' used';
  byId('freeBadge').textContent = fmtSize(Math.max(0, d.quota - d.totalUsed)) + ' free';

  // Quota bar
  var barPct = Math.min(pct, 100);
  byId('quotaBar').style.width = barPct + '%';
  if (pct > 90) byId('quotaBar').style.background = 'linear-gradient(90deg, rgba(239,107,115,0.4), rgba(239,107,115,0.6))';
  else if (pct > 70) byId('quotaBar').style.background = 'linear-gradient(90deg, rgba(245,185,66,0.3), rgba(245,185,66,0.5))';

  byId('quotaUsedLabel').textContent = fmtSize(d.totalUsed) + ' / ' + fmtSize(d.quota) + ' (' + pct + '%)';
  byId('quotaFreeLabel').textContent = fmtSize(Math.max(0, d.quota - d.totalUsed)) + ' free';

  // VFS stats
  byId('pvfsSize').textContent = fmtSize(d.pvfsBytes);
  byId('gamesSize').textContent = fmtSize(d.gamesTotal);
  byId('spritesSize').textContent = fmtSize(d.spritesTotal);
  byId('otherSize').textContent = fmtSize(d.otherTotal);

  // Keys table
  var kt = '';
  for (var i = 0; i < d.keys.length; i++) {
    var k = d.keys[i];
    var kpct = d.totalUsed > 0 ? Math.round(k.size / d.totalUsed * 100) : 0;
    kt += '<tr><td><code>' + esc(k.key) + '</code></td>'
        + '<td style="text-align:right;font-family:Courier New,monospace;font-size:13px">' + fmtSize(k.size) + '</td>'
        + '<td style="text-align:right;color:var(--muted);font-size:12px">' + kpct + '%</td></tr>';
  }
  byId('keysTable').innerHTML = kt || '<tr><td>(empty)</td></tr>';

  // Games table
  byId('gamesCount').textContent = d.games.length + ' games, ' + fmtSize(d.gamesTotal) + ' total';
  
  // Apply sort if a custom sort column is set
  if (_gameSortColumn === 'name') {
    d.games.sort(function(a, b) {
      var aName = (a.name || '').toLowerCase();
      var bName = (b.name || '').toLowerCase();
      return _gameSortAsc ? (aName > bName ? 1 : -1) : (aName < bName ? 1 : -1);
    });
  } else if (_gameSortColumn === 'size') {
    d.games.sort(function(a, b) {
      return _gameSortAsc ? (a.size - b.size) : (b.size - a.size);
    });
  }
  
  var gt = '<tr style="color:var(--muted);font-size:11px;text-transform:uppercase;letter-spacing:0.06em">'
         + '<td style="cursor:pointer;user-select:none" onclick="sortGamesBy(\'name\')">Name' + (_gameSortColumn === 'name' ? (_gameSortAsc ? ' ▲' : ' ▼') : '') + '</td>'
         + '<td style="cursor:pointer;user-select:none" onclick="sortGamesBy(\'size\')">Size' + (_gameSortColumn === 'size' ? (_gameSortAsc ? ' ▲' : ' ▼') : '') + '</td>'
         + '<td>Scope</td><td>Hash</td><td></td></tr>';
  for (var i = 0; i < d.games.length; i++) {
    var g = d.games[i];
    var idx = i;
    var tags = '';
    if (g.active) tags += '<span class="active-tag">active</span>';
    if (g.scope === 'external') tags += '<span class="ext-tag">ext</span>';
    gt += '<tr>'
        + '<td><a class="game-link" onclick="previewGame(' + idx + ')">' + esc(g.name) + '</a>' + tags + '</td>'
        + '<td style="font-family:Courier New,monospace;font-size:13px">' + fmtSize(g.size) + '</td>'
        + '<td style="font-size:13px">' + esc(g.scope) + '</td>'
        + '<td><span class="hash-tag">' + esc(g.hash) + '</span></td>'
        + '<td style="text-align:right;white-space:nowrap"><button class="sm" onclick="renameGame(\'' + esc(g.id).replace(/'/g, "\\'") + '\')">' + '\u270F</button> <button class="sm danger" onclick="deleteGame(\'' + esc(g.id).replace(/'/g, "\\'") + '\')">' + 'Del</button></td>'
        + '</tr>';
  }
  byId('gamesTable').innerHTML = gt;

  // Sprites table
  byId('spritesCount').textContent = d.sprites.length + ' files, ' + fmtSize(d.spritesTotal) + ' total';
  var st = '<tr style="color:var(--muted);font-size:11px;text-transform:uppercase;letter-spacing:0.06em">'
         + '<td>Path</td><td>Size</td><td></td></tr>';
  for (var i = 0; i < d.sprites.length; i++) {
    var sp = d.sprites[i];
    st += '<tr>'
      + '<td><a class="game-link" onclick="previewSprite(' + i + ')"><code>' + esc(sp.path) + '</code></a> ' + storageBadge(sp.storage) + '</td>'
        + '<td style="text-align:right;font-family:Courier New,monospace;font-size:13px">' + fmtSize(sp.size) + '</td>'
        + '<td style="text-align:right"><button class="sm danger" onclick="deleteVfs(\'' + esc(sp.path).replace(/'/g, "\\'") + '\')">Del</button></td>'
        + '</tr>';
  }
  byId('spritesTable').innerHTML = st || '<tr><td>(none)</td></tr>';

  // Other VFS
  var ot = '<tr style="color:var(--muted);font-size:11px;text-transform:uppercase;letter-spacing:0.06em">'
         + '<td>Path</td><td>Size</td><td></td></tr>';
  for (var i = 0; i < d.otherVfs.length; i++) {
    var f = d.otherVfs[i];
    ot += '<tr>'
      + '<td><a class="game-link" onclick="previewVfs(' + i + ')">' + esc(f.path) + '</a> ' + storageBadge(f.storage) + '</td>'
        + '<td style="font-family:Courier New,monospace;font-size:13px">' + fmtSize(f.size) + '</td>'
        + '<td style="text-align:right"><button class="sm danger" onclick="deleteVfs(\'' + esc(f.path).replace(/'/g, "\\'") + '\')">' + 'Del</button></td>'
        + '</tr>';
  }
  byId('otherTable').innerHTML = d.otherVfs.length ? ot : '<tr><td>(none)</td></tr>';
}

function sortGamesBy(column) {
  if (_gameSortColumn === column) {
    _gameSortAsc = !_gameSortAsc;
  } else {
    _gameSortColumn = column;
    _gameSortAsc = false;
  }
  render();
}

// ══════════════════════════════════════════════════════════════
// Preview modal
// ══════════════════════════════════════════════════════════════
var _previewIdx = -1;
var _previewMode = 'game'; // 'game' or 'vfs'

function guessMime(path) {
  var p = String(path || '').toLowerCase();
  if (/\.png$/i.test(p)) return 'image/png';
  if (/\.jpg$/i.test(p) || /\.jpeg$/i.test(p)) return 'image/jpeg';
  if (/\.gif$/i.test(p)) return 'image/gif';
  if (/\.webp$/i.test(p)) return 'image/webp';
  if (/\.svg$/i.test(p)) return 'image/svg+xml';
  return 'application/octet-stream';
}

function spritePreviewDoc(path, content) {
  var text = String(content || '');
  var src = '';
  if (/^data:image\//i.test(text)) {
    src = text;
  } else if (/^\s*<svg[\s>]/i.test(text)) {
    var svgBase64 = btoa(unescape(encodeURIComponent(text)));
    src = 'data:image/svg+xml;base64,' + svgBase64;
  } else if (/^[A-Za-z0-9+/=\r\n]+$/.test(text.trim()) && text.trim().length > 24) {
    src = 'data:' + guessMime(path) + ';base64,' + text.replace(/\s+/g, '');
  }
  if (src) {
    return '<!DOCTYPE html><html><head><style>'
      + 'html,body{margin:0;height:100%;background:#0a0a0f;display:flex;align-items:center;justify-content:center}'
      + 'img{max-width:100%;max-height:100%;object-fit:contain;background:transparent}'
      + '</style></head><body><img src="' + src + '" alt="sprite" /></body></html>';
  }
  return '<!DOCTYPE html><html><head><style>'
    + 'body{margin:0;padding:16px;background:#0a0a0f;color:#e8e6e3;'
    + 'font-family:Courier New,monospace;font-size:13px;white-space:pre-wrap;word-break:break-all}'
    + '</style></head><body>' + esc(text) + '</body></html>';
}

function previewGame(idx) {
  if (!_data || !_data.games[idx]) return;
  _previewIdx = idx;
  _previewMode = 'game';
  var g = _data.games[idx];
  byId('modalTitle').textContent = g.name;
  byId('modalFrame').srcdoc = g.content;
  byId('modalUnhide').textContent = 'Unhide \u2192 Games';
  byId('modalUnhide').style.display = '';
  byId('modalMoveIdb').style.display = 'none';
  byId('modalDelete').style.display = 'none';
  byId('gameModal').style.display = 'flex';
}

function previewSprite(idx) {
  if (!_data || !_data.sprites[idx]) return;
  _previewIdx = idx;
  _previewMode = 'sprite';
  var sp = _data.sprites[idx];
  byId('modalTitle').textContent = sp.path;
  byId('modalFrame').srcdoc = spritePreviewDoc(sp.path, sp.content);
  byId('modalUnhide').style.display = 'none';
  byId('modalMoveIdb').style.display = sp.storage === 'indexeddb' ? 'none' : '';
  byId('modalDelete').style.display = '';
  byId('gameModal').style.display = 'flex';
}

function previewVfs(idx) {
  if (!_data || !_data.otherVfs[idx]) return;
  _previewIdx = idx;
  _previewMode = 'vfs';
  var f = _data.otherVfs[idx];
  byId('modalTitle').textContent = f.path;
  var content = f.content;
  // Try to extract HTML from JSON wrapper (e.g. {"content":"<html>..."} or game entries)
  if (/^\s*\{/.test(content)) {
    try {
      var parsed = JSON.parse(content);
      if (typeof parsed.content === 'string') content = parsed.content;
      else if (typeof parsed.html === 'string') content = parsed.html;
    } catch(_) {}
  }
  // If content looks like HTML, render directly; otherwise wrap in pre
  var isHtml = /^\s*<(!doctype|html|head|body|div|script)/i.test(content);
  if (isHtml) {
    byId('modalFrame').srcdoc = content;
  } else {
    var wrapped = '<!DOCTYPE html><html><head><style>'
      + 'body{margin:0;padding:16px;background:#0a0a0f;color:#e8e6e3;'
      + 'font-family:Courier New,monospace;font-size:13px;white-space:pre-wrap;word-break:break-all}'
      + '</style></head><body>' + esc(content) + '</body></html>';
    byId('modalFrame').srcdoc = wrapped;
  }
  byId('modalUnhide').textContent = 'Unhide \u2192 Games';
  byId('modalUnhide').style.display = '';
  byId('modalMoveIdb').style.display = 'none';
  byId('modalDelete').style.display = '';
  byId('gameModal').style.display = 'flex';
}

async function movePreviewToIndexedDb() {
  if (_previewIdx < 0 || !_data || _previewMode !== 'sprite' || !_data.sprites[_previewIdx]) return;
  var sp = _data.sprites[_previewIdx];
  try {
    await movePathToIndexedDb(sp.path);
    var btn = byId('modalMoveIdb');
    btn.textContent = 'Moved!';
    btn.style.color = 'var(--green)';
    btn.style.borderColor = 'rgba(0,255,136,0.25)';
    setTimeout(function() {
      btn.textContent = 'Move → IndexedDB';
      btn.style.color = '';
      btn.style.borderColor = '';
    }, 2000);
    closeModal();
    render();
  } catch (e) {
    alert('Failed to move: ' + (e.message || e));
  }
}

function deletePreviewItem() {
  if (_previewIdx < 0 || !_data) return;
  if (_previewMode === 'sprite') {
    if (!_data.sprites[_previewIdx]) return;
    var sp = _data.sprites[_previewIdx];
    deleteVfs(sp.path);
    closeModal();
    return;
  }
  if (_previewMode === 'vfs') {
    if (!_data.otherVfs[_previewIdx]) return;
    var f = _data.otherVfs[_previewIdx];
    deleteVfs(f.path);
    closeModal();
  }
}

function closeModal() {
  byId('gameModal').style.display = 'none';
  byId('modalFrame').srcdoc = '';
  byId('modalUnhide').style.display = '';
  byId('modalMoveIdb').style.display = 'none';
  byId('modalDelete').style.display = 'none';
  _previewIdx = -1;
}

// Close modal on overlay click (not modal body)
byId('gameModal').addEventListener('click', function(e) {
  if (e.target === byId('gameModal')) closeModal();
});

// Close on Escape
document.addEventListener('keydown', function(e) {
  if (e.key === 'Escape' && byId('gameModal').style.display !== 'none') closeModal();
});

// ══════════════════════════════════════════════════════════════
// Unhide: save game to games.json as visible internal game
// ══════════════════════════════════════════════════════════════
function unhideGame() {
  if (_previewIdx < 0 || !_data) return;
  if (_previewMode === 'vfs') return unhideVfs();
  if (!_data.games[_previewIdx]) return;
  var g = _data.games[_previewIdx];

  try {
    var pvfs = JSON.parse(localStorage.getItem('traits.pvfs') || '{}');
    var raw = pvfs['canvas/games.json'];
    if (!raw && pvfs.files && pvfs.files['canvas/games.json'])
      raw = String((pvfs.files['canvas/games.json'] || {}).content || '');
    var col = raw ? JSON.parse(raw) : { active: null, games: {} };
    if (!col.games) col.games = {};

    // Generate a new ID if the game isn't already in the collection,
    // or reuse existing ID and update scope to internal
    var targetId = g.id;
    if (!col.games[targetId]) {
      targetId = 'g-' + Date.now();
    }

    col.games[targetId] = {
      name: g.name,
      content: g.content,
      scope: 'internal',
      _scope: 'internal',
      owner: g.owner || 'local',
      _sync_owner: g.owner || 'local',
      game_id: g.game_id || '',
      _sync_game_id: g.game_id || '',
      checksum: g.hash || '',
      _sync_hash: g.hash || '',
      created: g.created || new Date().toISOString(),
      updated: new Date().toISOString(),
      version: '',
    };

    // Set as active
    col.active = targetId;

    var json = JSON.stringify(col);
    pvfs['canvas/games.json'] = json;
    // Also update files format if present
    if (pvfs.files && typeof pvfs.files === 'object') {
      var ts = Date.now();
      var prev = pvfs.files['canvas/games.json'] || {};
      pvfs.files['canvas/games.json'] = {
        content: json,
        created: typeof prev.created === 'number' ? prev.created : ts,
        modified: ts,
      };
    }
    // Also write canvas/app.html so canvas renders this game
    pvfs['canvas/app.html'] = g.content;
    if (pvfs.files && typeof pvfs.files === 'object') {
      var tsA = Date.now();
      var prevA = pvfs.files['canvas/app.html'] || {};
      pvfs.files['canvas/app.html'] = {
        content: g.content,
        created: typeof prevA.created === 'number' ? prevA.created : tsA,
        modified: tsA,
      };
    }

    localStorage.setItem('traits.pvfs', JSON.stringify(pvfs));

    // Visual feedback
    var btn = byId('modalUnhide');
    btn.textContent = 'Saved!';
    btn.style.color = 'var(--green)';
    btn.style.borderColor = 'rgba(0,255,136,0.25)';
    setTimeout(function() {
      btn.textContent = 'Unhide → Games';
      btn.style.color = '';
      btn.style.borderColor = '';
    }, 2000);

    // Refresh data
    render();
  } catch (e) {
    alert('Failed to save: ' + (e.message || e));
  }
}

// ══════════════════════════════════════════════════════════════
// Unhide VFS file: save content as a game in games.json
// ══════════════════════════════════════════════════════════════
function unhideVfs() {
  if (_previewIdx < 0 || !_data || !_data.otherVfs[_previewIdx]) return;
  var f = _data.otherVfs[_previewIdx];
  var content = f.content;
  // Extract HTML from JSON wrapper if present
  if (/^\s*\{/.test(content)) {
    try {
      var parsed = JSON.parse(content);
      if (typeof parsed.content === 'string') content = parsed.content;
      else if (typeof parsed.html === 'string') content = parsed.html;
    } catch(_) {}
  }
  // Derive a name from the path (last segment, strip extension)
  var name = f.path.replace(/^.*\//, '').replace(/\.[^.]+$/, '') || f.path;

  try {
    var pvfs = JSON.parse(localStorage.getItem('traits.pvfs') || '{}');
    var raw = pvfs['canvas/games.json'];
    if (!raw && pvfs.files && pvfs.files['canvas/games.json'])
      raw = String((pvfs.files['canvas/games.json'] || {}).content || '');
    var col = raw ? JSON.parse(raw) : { active: null, games: {} };
    if (!col.games) col.games = {};

    var targetId = 'g-' + Date.now();
    col.games[targetId] = {
      name: name,
      content: content,
      scope: 'internal',
      _scope: 'internal',
      owner: 'local',
      _sync_owner: 'local',
      game_id: '',
      _sync_game_id: '',
      checksum: '',
      _sync_hash: '',
      created: new Date().toISOString(),
      updated: new Date().toISOString(),
      version: '',
    };
    col.active = targetId;

    var json = JSON.stringify(col);
    pvfs['canvas/games.json'] = json;
    if (pvfs.files && typeof pvfs.files === 'object') {
      var ts = Date.now();
      var prev = pvfs.files['canvas/games.json'] || {};
      pvfs.files['canvas/games.json'] = {
        content: json,
        created: typeof prev.created === 'number' ? prev.created : ts,
        modified: ts,
      };
    }
    pvfs['canvas/app.html'] = content;
    if (pvfs.files && typeof pvfs.files === 'object') {
      var tsA = Date.now();
      var prevA = pvfs.files['canvas/app.html'] || {};
      pvfs.files['canvas/app.html'] = {
        content: content,
        created: typeof prevA.created === 'number' ? prevA.created : tsA,
        modified: tsA,
      };
    }
    localStorage.setItem('traits.pvfs', JSON.stringify(pvfs));

    var btn = byId('modalUnhide');
    btn.textContent = 'Saved!';
    btn.style.color = 'var(--green)';
    btn.style.borderColor = 'rgba(0,255,136,0.25)';
    setTimeout(function() {
      btn.textContent = 'Unhide \u2192 Games';
      btn.style.color = '';
      btn.style.borderColor = '';
    }, 2000);
    render();
  } catch (e) {
    alert('Failed to save: ' + (e.message || e));
  }
}

// ══════════════════════════════════════════════════════════════
// Delete VFS file from pvfs
// ══════════════════════════════════════════════════════════════
function deleteVfs(path) {
  if (!confirm('Delete VFS file "' + path + '"?')) return;
  try {
    deletePathEverywhere(path).then(function() { render(); }).catch(function(e) {
      alert('Failed to delete: ' + (e.message || e));
    });
    return;
  } catch(e) {
    alert('Failed to delete: ' + (e.message || e));
  }
}

// ══════════════════════════════════════════════════════════════
// Rename game in collection
// ══════════════════════════════════════════════════════════════
function renameGame(id) {
  var pvfs = JSON.parse(localStorage.getItem('traits.pvfs') || '{}');
  var raw = pvfs['canvas/games.json'];
  if (!raw && pvfs.files && pvfs.files['canvas/games.json'])
    raw = String((pvfs.files['canvas/games.json'] || {}).content || '');
  var col = raw ? JSON.parse(raw) : { active: null, games: {} };
  if (!col.games || !col.games[id]) return;
  var oldName = col.games[id].name || id;
  var newName = prompt('Rename game:', oldName);
  if (!newName || newName === oldName) return;
  try {
    col.games[id].name = newName;
    col.games[id].updated = new Date().toISOString();
    var json = JSON.stringify(col);
    pvfs['canvas/games.json'] = json;
    if (pvfs.files && typeof pvfs.files === 'object') {
      var ts = Date.now();
      var prev = pvfs.files['canvas/games.json'] || {};
      pvfs.files['canvas/games.json'] = {
        content: json,
        created: typeof prev.created === 'number' ? prev.created : ts,
        modified: ts,
      };
    }
    localStorage.setItem('traits.pvfs', JSON.stringify(pvfs));
  } catch(e) {
    alert('Failed to rename: ' + (e.message || e));
  }
  render();
}

// ══════════════════════════════════════════════════════════════
// Delete game from collection
// ══════════════════════════════════════════════════════════════
function deleteGame(id) {
  if (!confirm('Delete game "' + id + '" from localStorage?')) return;
  try {
    var pvfs = JSON.parse(localStorage.getItem('traits.pvfs') || '{}');
    var raw = pvfs['canvas/games.json'];
    if (!raw && pvfs.files && pvfs.files['canvas/games.json'])
      raw = String((pvfs.files['canvas/games.json'] || {}).content || '');
    var col = raw ? JSON.parse(raw) : { active: null, games: {} };
    if (col.games && col.games[id]) {
      delete col.games[id];
      if (col.active === id) {
        var remaining = Object.keys(col.games);
        col.active = remaining.length > 0 ? remaining[0] : null;
      }
      var json = JSON.stringify(col);
      pvfs['canvas/games.json'] = json;
      if (pvfs.files && typeof pvfs.files === 'object') {
        var ts = Date.now();
        var prev = pvfs.files['canvas/games.json'] || {};
        pvfs.files['canvas/games.json'] = {
          content: json,
          created: typeof prev.created === 'number' ? prev.created : ts,
          modified: ts,
        };
      }
      localStorage.setItem('traits.pvfs', JSON.stringify(pvfs));
    }
  } catch(e) {
    alert('Failed to delete: ' + (e.message || e));
  }
  render();
}

// ══════════════════════════════════════════════════════════════
// Init
// ══════════════════════════════════════════════════════════════
render();

// Expose to onclick handlers
window.previewGame = previewGame;
window.previewSprite = previewSprite;
window.previewVfs = previewVfs;
window.closeModal = closeModal;
window.movePreviewToIndexedDb = movePreviewToIndexedDb;
window.unhideGame = unhideGame;
window.unhideVfs = unhideVfs;
window.deletePreviewItem = deletePreviewItem;
window.renameGame = renameGame;
window.deleteGame = deleteGame;
window.deleteVfs = deleteVfs;
window.sortGamesBy = sortGamesBy;

})();
"##;
