use serde_json::Value;
use maud::{html, DOCTYPE, PreEscaped};

pub fn playground(_args: &[Value]) -> Value {
    let markup = html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { "slob.games — Build" }
                style { (PreEscaped(CSS)) }
            }
            body {
                // Header with game selector
                div.build-header {
                    h1 { span.accent { "slob" } ".games " span.muted { "build" } }
                    div.header-actions {
                        select #game-select { option value="" { "— select game —" } }
                    }
                }

                // Background CTA
                div.cta-bg {
                    div.cta-text {
                        "Register to build & modify games on slob.games"
                    }
                    div.cta-sub {
                        "Sign up for a builder account to create, remix, and publish games."
                    }
                }

                // Chat box (embedded, always visible)
                div #build-chat {
                    div.bcm-header {
                        span.bcm-title { "💬  Build Chat" }
                    }
                    div.bcm-log #bcmLog {
                        div.vcm-msg.system {
                            "Start a voice session or type below to chat with the build agent."
                        }
                    }
                    div.bcm-input-row {
                        input #bcmInput type="text" placeholder="Type to build agent…" {}
                        button #bcmSend { "↑" }
                    }
                }

                // FAB
                div #build-fab {
                    button.fab-btn #fabToggle { "+" }
                    div.fab-menu #fabMenu {
                        button #fabVoice {
                            span.fab-icon { "🎤" }
                            span #fabVoiceLabel { "Start Voice" }
                        }
                        button #fabNew {
                            span.fab-icon { "✨" }
                            span { "New Game" }
                        }
                        button #fabSplats {
                            span.fab-icon { "🔮" }
                            span { "Splat Viewer" }
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
:root { --bg: #0a0a0f; --fg: #e0e0e0; --accent: #00e0ff; --border: #1a1a2e; }
html, body { margin:0; padding:0; background:var(--bg); color:var(--fg); font-family:'Courier New',Menlo,monospace; overflow:hidden; height:100%; }

.build-header {
    display:flex; align-items:center; justify-content:space-between;
    padding:10px 20px; border-bottom:1px solid rgba(0,224,255,0.1);
    background:rgba(8,8,14,0.95);
}
.build-header h1 { font-size:13px; font-weight:700; text-transform:uppercase; letter-spacing:0.06em; color:#8899a6; margin:0; }
.build-header .accent { color:var(--accent); text-shadow:0 0 8px rgba(0,224,255,0.4); }
.build-header .muted { color:#556; }
.header-actions { display:flex; gap:8px; }
#game-select {
    background:#0c0c16; color:#8899a6;
    border:1px solid #1e1e2e; border-radius:4px;
    padding:4px 8px; font-size:11px;
    cursor:pointer; outline:none; max-width:180px;
    font-family:'Courier New',Menlo,monospace;
    text-transform:uppercase; letter-spacing:0.02em;
    -webkit-appearance:none; appearance:none;
    background-image:url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='6'%3E%3Cpath d='M0 0l5 6 5-6z' fill='%23556'/%3E%3C/svg%3E");
    background-repeat:no-repeat; background-position:right 8px center;
    padding-right:22px;
}
#game-select:hover { border-color:var(--accent); }
#game-select:focus { border-color:var(--accent); box-shadow:0 0 6px rgba(0,224,255,0.15); }
#game-select option { background:#0c0c16; color:#8899a6; }

/* Background CTA */
.cta-bg {
    position:absolute; top:50%; left:50%;
    transform:translate(-50%,-50%);
    text-align:center; pointer-events:none;
    z-index:1;
}
.cta-text {
    font-size:clamp(24px,4vw,42px);
    font-weight:700; text-transform:uppercase;
    letter-spacing:0.06em;
    color:rgba(0,224,255,0.06);
    line-height:1.2;
    max-width:600px;
}
.cta-sub {
    margin-top:16px;
    font-size:clamp(12px,1.8vw,16px);
    color:rgba(138,153,166,0.15);
    letter-spacing:0.02em;
    max-width:500px;
    margin-left:auto; margin-right:auto;
}

/* Chat box (embedded, always visible) */
#build-chat {
    position:fixed;
    bottom:74px; right:20px;
    width:360px; height:480px;
    background:rgba(10,10,18,0.97);
    border:1px solid rgba(0,224,255,0.2);
    border-radius:12px; z-index:100;
    box-shadow:0 8px 32px rgba(0,0,0,0.6), 0 0 20px rgba(0,224,255,0.05);
    backdrop-filter:blur(16px);
    display:flex; flex-direction:column;
}
.bcm-header {
    display:flex; justify-content:space-between; align-items:center;
    padding:10px 14px; border-bottom:1px solid rgba(255,255,255,0.07);
    flex-shrink:0;
}
.bcm-title { color:#00e0ff; font-size:13px; font-weight:600; letter-spacing:0.04em; text-transform:uppercase; }
.bcm-log {
    flex:1; overflow-y:auto; padding:10px 12px;
    display:flex; flex-direction:column; gap:5px;
}
.bcm-log::-webkit-scrollbar { width:4px; }
.bcm-log::-webkit-scrollbar-thumb { background:#333; border-radius:2px; }
.vcm-msg {
    padding:5px 10px; border-radius:8px; font-size:12px;
    line-height:1.5; max-width:95%; word-break:break-word;
}
.vcm-msg.user { background:rgba(0,224,255,0.1); color:#66f0ff; align-self:flex-end; }
.vcm-msg.assistant { background:rgba(30,30,38,0.9); color:#e0e0e0; align-self:flex-start; border:1px solid rgba(255,255,255,0.06); }
.vcm-msg.tool { background:rgba(0,200,100,0.08); color:#4ade80; font-family:monospace; font-size:11px; align-self:flex-start; }
.vcm-msg.tool-result { background:rgba(56,189,248,0.07); color:#7dd3fc; font-family:monospace; font-size:11px; align-self:flex-start; }
.vcm-msg.system { color:#555; font-size:11px; font-style:italic; align-self:center; }
.bcm-input-row {
    display:flex; gap:6px; padding:8px 10px;
    border-top:1px solid rgba(255,255,255,0.07); flex-shrink:0;
}
#bcmInput {
    flex:1; background:rgba(255,255,255,0.06); border:1px solid #333;
    border-radius:6px; color:#eee; padding:6px 10px; font-size:13px; outline:none;
    font-family:'Courier New',Menlo,monospace;
}
#bcmInput:focus { border-color:rgba(0,224,255,0.4); }
#bcmSend {
    background:rgba(0,224,255,0.12); border:1px solid rgba(0,224,255,0.3);
    border-radius:6px; color:#00e0ff; padding:6px 12px;
    cursor:pointer; font-size:16px; transition:background 0.15s;
}
#bcmSend:hover { background:rgba(0,224,255,0.25); }

/* FAB */
#build-fab {
    position:fixed; bottom:20px; right:20px; z-index:9990;
}
#build-fab .fab-btn {
    width:44px; height:44px; border-radius:50%;
    background:rgba(0,224,255,0.06); border:1px solid rgba(0,224,255,0.25);
    color:#00e0ff; font-size:22px; cursor:pointer;
    display:flex; align-items:center; justify-content:center;
    backdrop-filter:blur(8px); transition:all 0.2s;
    box-shadow:0 0 12px rgba(0,224,255,0.12);
    font-family:'Courier New',monospace;
}
#build-fab .fab-btn:hover { background:rgba(0,224,255,0.12); transform:scale(1.08); box-shadow:0 0 20px rgba(0,224,255,0.25); }
#build-fab .fab-btn.open { transform:rotate(45deg); }
#build-fab .fab-menu {
    display:none; position:absolute; bottom:52px; right:0;
    background:rgba(10,10,18,0.97); border:1px solid rgba(0,224,255,0.1);
    border-radius:8px; padding:4px 0; min-width:170px;
    backdrop-filter:blur(12px); box-shadow:0 4px 24px rgba(0,0,0,0.6);
}
#build-fab .fab-menu.show { display:block; }
#build-fab .fab-menu button {
    display:flex; align-items:center; gap:8px; width:100%;
    padding:8px 14px; border:none; background:none;
    color:#8899a6; font-size:12px; cursor:pointer; text-align:left;
    text-transform:uppercase; letter-spacing:0.02em;
    font-family:'Courier New',Menlo,monospace;
}
#build-fab .fab-menu button:hover { background:rgba(0,224,255,0.06); color:#00e0ff; }
#build-fab .fab-menu button .fab-icon { width:18px; text-align:center; flex-shrink:0; }

@media (max-width:768px) {
    #build-chat {
        left:10px; right:10px; bottom:68px;
        width:auto; height:calc(100vh - 140px);
    }
    .cta-bg { display:none; }
}
"##;

const JS: &str = r##"
(function() {
    var sdk = function() { return window._traitsSDK; };

    // ── Game selector dropdown ──
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

    function populateGameSelect() {
        var gs = document.getElementById('game-select');
        if (!gs) return;
        var col = readGamesCollection();
        gs.innerHTML = '<option value="">— select game —</option>';
        var gObj = col.games || {};
        for (var id in gObj) {
            if (!gObj.hasOwnProperty(id)) continue;
            var g = gObj[id];
            var opt = document.createElement('option');
            opt.value = id;
            opt.textContent = g.name || 'untitled';
            if (id === col.active) opt.selected = true;
            gs.appendChild(opt);
        }
        gs.addEventListener('change', function() {
            var v = gs.value;
            if (!v) return;
            var s = sdk();
            if (s) s.call('sys.canvas', ['activate', v]);
        });
    }
    populateGameSelect();

    // ── FAB ──
    var fabToggle = document.getElementById('fabToggle');
    var fabMenu = document.getElementById('fabMenu');
    fabToggle.addEventListener('click', function(e) {
        e.stopPropagation();
        fabMenu.classList.toggle('show');
        fabToggle.classList.toggle('open');
    });
    document.addEventListener('click', function() {
        fabMenu.classList.remove('show');
        fabToggle.classList.remove('open');
    });

    // FAB voice toggle
    var voiceActive = false;
    var fabVoice = document.getElementById('fabVoice');
    var fabVoiceLabel = document.getElementById('fabVoiceLabel');
    function updateVoiceBtn(active) {
        voiceActive = active;
        if (fabVoiceLabel) fabVoiceLabel.textContent = active ? 'Stop Voice' : 'Start Voice';
    }
    if (fabVoice) {
        fabVoice.addEventListener('click', function() {
            var s = sdk();
            if (!s) return;
            fabMenu.classList.remove('show');
            fabToggle.classList.remove('open');
            if (voiceActive) { s.stopVoice(); updateVoiceBtn(false); }
            else { s.startVoice(); updateVoiceBtn(true); }
        });
    }
    // FAB New Game
    var fabNew = document.getElementById('fabNew');
    if (fabNew) {
        fabNew.addEventListener('click', function() {
            fabMenu.classList.remove('show');
            fabToggle.classList.remove('open');
            var s = sdk();
            if (s) s.call('sys.canvas', ['new']);
        });
    }
    // FAB Splats
    var fabSplats = document.getElementById('fabSplats');
    if (fabSplats) {
        fabSplats.addEventListener('click', function() {
            fabMenu.classList.remove('show');
            fabToggle.classList.remove('open');
            var route = '/wasm';
            if (location.protocol === 'file:') {
                sessionStorage.setItem('traits.shell.route', route);
                location.hash = '#' + route;
            } else {
                history.pushState({ route: route }, '', route);
            }
            window.dispatchEvent(new PopStateEvent('popstate', { state: { route: route } }));
        });
    }

    // ── Long-press / dblclick FAB to toggle chat focus ──
    (function() {
        var _pressTimer = null;
        fabToggle.addEventListener('pointerdown', function() {
            _pressTimer = setTimeout(function() {
                _pressTimer = null;
                fabMenu.classList.remove('show');
                fabToggle.classList.remove('open');
                document.getElementById('bcmInput').focus();
            }, 500);
        });
        fabToggle.addEventListener('pointerup', function() { if (_pressTimer) clearTimeout(_pressTimer); });
        fabToggle.addEventListener('pointercancel', function() { if (_pressTimer) clearTimeout(_pressTimer); });
        fabToggle.addEventListener('dblclick', function(e) {
            e.stopPropagation();
            fabMenu.classList.remove('show');
            fabToggle.classList.remove('open');
            document.getElementById('bcmInput').focus();
        });
    })();

    // ── Chat box ──
    var bcmLog = document.getElementById('bcmLog');
    var bcmInput = document.getElementById('bcmInput');
    var bcmSendBtn = document.getElementById('bcmSend');

    function bcmAppend(role, text) {
        if (!text) return;
        var el = document.createElement('div');
        el.className = 'vcm-msg ' + role;
        el.textContent = text;
        bcmLog.appendChild(el);
        bcmLog.scrollTop = bcmLog.scrollHeight;
    }

    function bcmSendText() {
        var text = bcmInput.value.trim();
        if (!text) return;
        var s = sdk();
        if (!s || !s.sendVoiceText) {
            bcmAppend('system', '\u26A0 Voice not active — start voice first');
            return;
        }
        var ok = s.sendVoiceText(text);
        if (ok) {
            bcmAppend('user', text);
            bcmInput.value = '';
        } else {
            bcmAppend('system', '\u26A0 Voice not connected');
        }
    }
    bcmSendBtn.addEventListener('click', bcmSendText);
    bcmInput.addEventListener('keydown', function(e) { if (e.key === 'Enter') bcmSendText(); });

    // Voice events → chat log
    window.addEventListener('voice-event', function(e) {
        var d = e.detail;
        switch (d.type) {
            case 'started':
                updateVoiceBtn(true);
                bcmAppend('system', '🎤 Voice session started');
                break;
            case 'stopped':
            case 'disconnected':
                updateVoiceBtn(false);
                bcmAppend('system', '⏹ Voice session ended');
                break;
            case 'transcript':
                bcmAppend('user', d.text);
                break;
            case 'response':
                bcmAppend('assistant', d.text);
                break;
            case 'tool_call': {
                var args = d.arguments || '';
                try { args = JSON.stringify(JSON.parse(args), null, 0).slice(0, 140); } catch(_) { args = args.slice(0, 100); }
                bcmAppend('tool', '\u26A1 ' + d.name + '(' + args + ')');
                break;
            }
            case 'tool_result': {
                var preview = (d.result || '').slice(0, 140);
                bcmAppend('tool-result', '\u2713 ' + d.name + ': ' + preview);
                break;
            }
            case 'error':
                bcmAppend('system', '\u26A0 ' + d.message);
                break;
        }
    });

    // Re-populate games list on changes
    window.addEventListener('traits-canvas-projects-changed', populateGameSelect);
})();
"##;

