use serde_json::Value;
use maud::{html, DOCTYPE, PreEscaped};

pub fn canvas(_args: &[Value]) -> Value {
    let markup = html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { "slob.games — Canvas" }
                style {
                    (PreEscaped(r#"
                        :root { --bg: #0a0a0a; --fg: #e0e0e0; --accent: #00e0ff; --border: #222; }
                        html, body { margin: 0; padding: 0; background: var(--bg); color: var(--fg); font-family: system-ui, sans-serif; overflow: hidden; height: 100%; }
                        .canvas-header {
                            display: flex; align-items: center; justify-content: space-between;
                            padding: 12px 20px; border-bottom: 1px solid var(--border);
                            background: #111;
                        }
                        .canvas-header h1 { font-size: 16px; font-weight: 500; }
                        .canvas-header h1 .accent { color: var(--accent); }
                        .canvas-header .actions { display: flex; gap: 8px; }
                        .canvas-header button {
                            background: transparent; border: 1px solid var(--border);
                            color: var(--fg); padding: 4px 12px; border-radius: 4px;
                            cursor: pointer; font-size: 12px;
                        }
                        .canvas-header button:hover { border-color: var(--accent); color: var(--accent); }
                        .canvas-header button.save-btn { border-color: #2a5; color: #2a5; }
                        .canvas-header button.save-btn:hover { border-color: #3c8; color: #3c8; }

                        /* Game selector dropdown */
                        #game-select {
                            background: #181818; color: var(--fg);
                            border: 1px solid var(--border); border-radius: 4px;
                            padding: 4px 8px; font-size: 12px;
                            cursor: pointer; outline: none; max-width: 180px;
                            font-family: system-ui, sans-serif;
                            -webkit-appearance: none; appearance: none;
                            background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='6'%3E%3Cpath d='M0 0l5 6 5-6z' fill='%23888'/%3E%3C/svg%3E");
                            background-repeat: no-repeat; background-position: right 8px center;
                            padding-right: 22px;
                        }
                        #game-select:hover { border-color: var(--accent); }
                        #game-select:focus { border-color: var(--accent); }
                        #game-select option { background: #181818; color: var(--fg); }

                        #canvas-container {
                            width: 100%; height: calc(100vh - 49px);
                            padding: 0 20px; position: relative;
                            display: flex; justify-content: center; align-items: center;
                            overflow: hidden; box-sizing: border-box;
                        }
                        .canvas-empty {
                            display: flex; flex-direction: column; align-items: center;
                            justify-content: center; height: 60vh; color: #555;
                        }
                        .canvas-empty .icon { font-size: 48px; margin-bottom: 16px; opacity: 0.5; }
                        .canvas-empty p { font-size: 14px; }
                        .canvas-empty code { color: var(--accent); font-size: 13px; }

                        /* Phone frame */
                        #phone-frame {
                            display: none;
                            position: relative;
                            width: 430px;
                            background: #1a1a1c;
                            border-radius: 50px;
                            border: 2px solid #3a3a3c;
                            box-shadow: 0 0 0 1px #111, 0 30px 80px rgba(0,0,0,0.8), inset 0 0 0 1px #2a2a2c;
                            padding: 18px 16px 22px;
                            flex-shrink: 0;
                        }
                        #phone-frame.visible { display: block; }
                        .phone-notch {
                            width: 120px; height: 34px;
                            background: #1a1a1c;
                            border-radius: 0 0 22px 22px;
                            margin: 0 auto 10px;
                            position: relative; z-index: 2;
                            display: flex; align-items: center; justify-content: center;
                            gap: 8px;
                        }
                        .phone-notch .camera {
                            width: 12px; height: 12px; border-radius: 50%;
                            background: #111; border: 1px solid #2a2a2c;
                        }
                        .phone-notch .speaker {
                            width: 50px; height: 6px; border-radius: 3px;
                            background: #111; border: 1px solid #222;
                        }
                        #phone-viewport {
                            width: 398px;
                            height: 844px;
                            border-radius: 36px;
                            overflow: hidden;
                            background: #000;
                            position: relative;
                            border: none;
                            display: block;
                        }
                        .phone-home-bar {
                            width: 134px; height: 5px;
                            background: #555; border-radius: 3px;
                            margin: 10px auto 0;
                        }

                        /* FAB menu */
                        #canvas-fab {
                            position: fixed; bottom: 20px; right: 20px; z-index: 9990;
                        }
                        #canvas-fab .fab-btn {
                            width: 44px; height: 44px; border-radius: 50%;
                            background: rgba(124,92,252,0.15); border: 1px solid rgba(124,92,252,0.4);
                            color: #b8a4fc; font-size: 20px; cursor: pointer;
                            display: flex; align-items: center; justify-content: center;
                            backdrop-filter: blur(8px); transition: transform 0.2s, background 0.2s;
                        }
                        #canvas-fab .fab-btn:hover { background: rgba(124,92,252,0.25); transform: scale(1.08); }
                        #canvas-fab .fab-btn.open { transform: rotate(45deg); }
                        #canvas-fab .fab-menu {
                            display: none; position: absolute; bottom: 52px; right: 0;
                            background: rgba(20,20,25,0.95); border: 1px solid #333;
                            border-radius: 8px; padding: 4px 0; min-width: 160px;
                            backdrop-filter: blur(12px); box-shadow: 0 4px 20px rgba(0,0,0,0.5);
                        }
                        #canvas-fab .fab-menu.show { display: block; }
                        #canvas-fab .fab-menu button {
                            display: flex; align-items: center; gap: 8px; width: 100%;
                            padding: 8px 14px; border: none; background: none;
                            color: #ccc; font-size: 13px; cursor: pointer; text-align: left;
                        }
                        #canvas-fab .fab-menu button:hover { background: rgba(124,92,252,0.12); color: #fff; }
                        #canvas-fab .fab-menu button .fab-icon { width: 18px; text-align: center; flex-shrink: 0; }

                        /* Voice Chat Modal */
                        #voice-chat-modal {
                            display: none; position: fixed;
                            bottom: 74px; right: 20px;
                            width: 340px; height: 460px;
                            background: rgba(14,14,18,0.97);
                            border: 1px solid rgba(124,92,252,0.4);
                            border-radius: 12px; z-index: 9995;
                            box-shadow: 0 8px 32px rgba(0,0,0,0.6);
                            backdrop-filter: blur(16px);
                            flex-direction: column;
                        }
                        #voice-chat-modal.vcm-open { display: flex; }
                        .vcm-header {
                            display: flex; justify-content: space-between; align-items: center;
                            padding: 10px 14px; border-bottom: 1px solid rgba(255,255,255,0.07);
                            cursor: move; flex-shrink: 0;
                        }
                        .vcm-title { color: #b8a4fc; font-size: 13px; font-weight: 600; letter-spacing: 0.02em; }
                        .vcm-close {
                            background: none; border: none; color: #555; font-size: 18px;
                            cursor: pointer; padding: 0 2px; line-height: 1; transition: color 0.15s;
                        }
                        .vcm-close:hover { color: #fff; }
                        .vcm-log {
                            flex: 1; overflow-y: auto; padding: 10px 12px;
                            display: flex; flex-direction: column; gap: 5px;
                        }
                        .vcm-log::-webkit-scrollbar { width: 4px; }
                        .vcm-log::-webkit-scrollbar-thumb { background: #333; border-radius: 2px; }
                        .vcm-msg {
                            padding: 5px 10px; border-radius: 8px; font-size: 12px;
                            line-height: 1.5; max-width: 95%; word-break: break-word;
                        }
                        .vcm-msg.user { background: rgba(124,92,252,0.2); color: #d4c8ff; align-self: flex-end; }
                        .vcm-msg.assistant { background: rgba(30,30,38,0.9); color: #e0e0e0; align-self: flex-start; border: 1px solid rgba(255,255,255,0.06); }
                        .vcm-msg.tool { background: rgba(0,200,100,0.08); color: #4ade80; font-family: monospace; font-size: 11px; align-self: flex-start; }
                        .vcm-msg.tool-result { background: rgba(56,189,248,0.07); color: #7dd3fc; font-family: monospace; font-size: 11px; align-self: flex-start; }
                        .vcm-msg.system { color: #555; font-size: 11px; font-style: italic; align-self: center; }
                        .vcm-input-row {
                            display: flex; gap: 6px; padding: 8px 10px;
                            border-top: 1px solid rgba(255,255,255,0.07); flex-shrink: 0;
                        }
                        #vcmInput {
                            flex: 1; background: rgba(255,255,255,0.06); border: 1px solid #333;
                            border-radius: 6px; color: #eee; padding: 6px 10px; font-size: 13px; outline: none;
                        }
                        #vcmInput:focus { border-color: rgba(124,92,252,0.55); }
                        #vcmSend {
                            background: rgba(124,92,252,0.25); border: 1px solid rgba(124,92,252,0.45);
                            border-radius: 6px; color: #b8a4fc; padding: 6px 12px;
                            cursor: pointer; font-size: 16px; transition: background 0.15s;
                        }
                        #vcmSend:hover { background: rgba(124,92,252,0.45); }

                        /* P2P Share Modal */
                        #share-modal {
                            display: none; position: fixed;
                            bottom: 74px; right: 20px;
                            width: 320px;
                            background: rgba(14,14,18,0.97);
                            border: 1px solid rgba(124,92,252,0.4);
                            border-radius: 12px; z-index: 9996;
                            box-shadow: 0 8px 40px rgba(0,0,0,0.7);
                            backdrop-filter: blur(16px);
                            flex-direction: column;
                        }
                        #share-modal.sm-open { display: flex; }
                        .sm-header {
                            display: flex; justify-content: space-between; align-items: center;
                            padding: 10px 14px; border-bottom: 1px solid rgba(255,255,255,0.07);
                            cursor: move; flex-shrink: 0;
                        }
                        .sm-title { color: #b8a4fc; font-size: 13px; font-weight: 600; }
                        .sm-close {
                            background: none; border: none; color: #555; font-size: 18px;
                            cursor: pointer; padding: 0 2px; line-height: 1; transition: color 0.15s;
                        }
                        .sm-close:hover { color: #fff; }
                        .sm-body {
                            padding: 18px 16px 16px;
                            display: flex; flex-direction: column; align-items: center; gap: 14px;
                        }
                        .sm-label { color: #888; font-size: 12px; text-align: center; }
                        .sm-code-display {
                            display: flex; gap: 8px;
                        }
                        .sm-code-char {
                            width: 52px; height: 64px;
                            background: rgba(124,92,252,0.1);
                            border: 1px solid rgba(124,92,252,0.4);
                            border-radius: 8px; color: #b8a4fc;
                            font-size: 28px; font-weight: 700; font-family: monospace;
                            display: flex; align-items: center; justify-content: center;
                            letter-spacing: 0;
                        }
                        .sm-code-inputs { display: flex; gap: 8px; }
                        .sm-ci {
                            width: 52px; height: 64px; text-align: center;
                            background: rgba(255,255,255,0.06);
                            border: 1px solid rgba(124,92,252,0.35);
                            border-radius: 8px; color: #d4c8ff;
                            font-size: 26px; font-weight: 700; font-family: monospace;
                            outline: none; text-transform: uppercase; caret-color: #b8a4fc;
                        }
                        .sm-ci:focus { border-color: rgba(124,92,252,0.8); background: rgba(124,92,252,0.08); }
                        .sm-link-row {
                            display: flex; gap: 6px; width: 100%; align-items: center;
                        }
                        .sm-link {
                            flex: 1; font-size: 10px; color: #555; overflow: hidden;
                            text-overflow: ellipsis; white-space: nowrap; user-select: text;
                        }
                        .sm-btn {
                            background: rgba(124,92,252,0.2); border: 1px solid rgba(124,92,252,0.4);
                            border-radius: 6px; color: #b8a4fc; padding: 5px 12px;
                            font-size: 12px; cursor: pointer; flex-shrink: 0; transition: background 0.15s;
                        }
                        .sm-btn:hover { background: rgba(124,92,252,0.4); }
                        .sm-btn-primary {
                            background: rgba(124,92,252,0.3); border: 1px solid rgba(124,92,252,0.6);
                            border-radius: 8px; color: #d4c8ff; padding: 9px 28px;
                            font-size: 14px; cursor: pointer; transition: background 0.15s; width: 100%;
                        }
                        .sm-btn-primary:hover { background: rgba(124,92,252,0.5); }
                        .sm-status {
                            font-size: 12px; color: #888; text-align: center; min-height: 16px;
                        }
                        .sm-status.ok { color: #4ade80; }
                        .sm-status.err { color: #f87171; }
                        .sm-progress {
                            width: 100%; height: 3px; background: rgba(255,255,255,0.08);
                            border-radius: 2px; overflow: hidden;
                        }
                        .sm-progress-bar {
                            height: 100%; background: linear-gradient(90deg,#7c5cfc,#b8a4fc);
                            border-radius: 2px; transition: width 0.3s;
                        }

                    "#))
                }
            }
            body {
                div .canvas-header {
                    h1 { "slob.games " span .accent { "canvas" } }
                    div .actions {
                        select #game-select { option value="" disabled selected { "no games" } }
                        button #btnSave .save-btn { "Save" }
                        button #btnClear { "Clear" }
                        button #btnSource { "View Source" }
                    }
                }
                div #canvas-container {
                    div .canvas-empty #canvas-empty {
                        div .icon { "🎨" }
                        p { "Canvas is empty — use " code { "sys.canvas set \"<html>\"" } " or voice to draw." }
                    }
                    div #phone-frame {
                        div .phone-notch {
                            div .speaker {}
                            div .camera {}
                        }
                        iframe #phone-viewport sandbox="allow-scripts allow-same-origin allow-forms" {}
                        div .phone-home-bar {}
                    }
                }

                // FAB menu
                div #canvas-fab {
                    button .fab-btn #fabToggle { "+" }
                    div .fab-menu #fabMenu {
                        button #fabNew {
                            span .fab-icon { "✨" }
                            span { "New Canvas" }
                        }
                        button #fabVoice {
                            span .fab-icon { "🎤" }
                            span { "Start Voice" }
                        }
                        button #fabSplats {
                            span .fab-icon { "🔮" }
                            span { "Splat Viewer" }
                        }
                        button #fabShare {
                            span .fab-icon { "📤" }
                            span { "Share Project" }
                        }
                        button #fabReceive {
                            span .fab-icon { "📥" }
                            span { "Receive Project" }
                        }
                    }
                }

                // P2P project share modal
                div #share-modal {
                    div .sm-header {
                        span .sm-title #smTitle { "📤  Share Project" }
                        button .sm-close #smClose { "×" }
                    }
                    div .sm-body #smBody {}
                }

                // Voice chat floating modal
                div #voice-chat-modal {
                    div .vcm-header {
                        span .vcm-title { "💬  Voice Chat" }
                        button .vcm-close #vcmClose { "×" }
                    }
                    div .vcm-log #vcmLog {}
                    div .vcm-input-row {
                        input #vcmInput type="text" placeholder="Type to voice agent…" {}
                        button #vcmSend { "↑" }
                    }
                }

                script { (PreEscaped(r#"
                    (function() {
                        // ── Canvas SDK: thin global API for scripts injected into the canvas ──
                        const _sdk = () => window._traitsSDK;
                        window.traits = {
                            call: (path, args) => _sdk()?.call(path, args || []),
                            list: (ns) => _sdk()?.call('sys.list', ns ? [ns] : []),
                            info: (path) => _sdk()?.call('sys.info', [path]),
                            canvas: (action, content) => {
                                const args = content !== undefined ? [action, content] : [action];
                                return _sdk()?.call('sys.canvas', args);
                            },
                            echo: (text) => _sdk()?.call('sys.echo', [text]),
                            audio: (action, ...a) => _sdk()?.call('sys.audio', [action, ...a]),
                        };

                        // ── Built-in seed games ──────────────────────────────────────────────
                        const SNAKE_GAME_HTML = `<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8"><title>Snake</title><style>*{margin:0;padding:0;box-sizing:border-box}body{width:390px;height:844px;overflow:hidden;background:#0a0a0a;color:#e0e0e0;font-family:system-ui,sans-serif;user-select:none}#hdr{display:flex;align-items:center;justify-content:space-between;padding:18px 20px 10px}#hdr .title{font-size:22px;font-weight:700}.accent{color:#00ff88}#score{font-size:22px;font-weight:700;color:#00ff88}#hs{font-size:11px;color:#555;margin-top:2px}#cw{display:flex;justify-content:center;padding:0 20px}canvas{border-radius:8px;border:1px solid #1a1a1a}#ctrl{display:flex;flex-direction:column;align-items:center;gap:4px;padding:14px}#ctrl .row{display:flex;gap:4px}.btn{width:68px;height:52px;background:rgba(255,255,255,.06);border:1px solid rgba(255,255,255,.1);border-radius:12px;font-size:20px;display:flex;align-items:center;justify-content:center;cursor:pointer;-webkit-tap-highlight-color:transparent;transition:background .1s}.btn:active,.btn.pressed{background:rgba(0,255,136,.15);border-color:#00ff88}#overlay{position:absolute;inset:0;display:flex;flex-direction:column;align-items:center;justify-content:center;background:rgba(10,10,10,.92);text-align:center;z-index:20}#overlay.off{display:none}#overlay h2{font-size:30px;font-weight:700;margin-bottom:8px}#overlay p{color:#666;font-size:13px;margin-bottom:24px}#overlay button{background:rgba(0,255,136,.15);border:1px solid #00ff88;color:#00ff88;padding:12px 36px;border-radius:10px;font-size:15px;cursor:pointer;font-family:inherit}</style></head><body><div id="hdr"><div><div class="title"><span class="accent">Snake</span></div><div id="hs">best: 0</div></div><div id="score">0</div></div><div id="cw"><canvas id="c" width="350" height="560"></canvas></div><div id="ctrl"><div class="row"><div class="btn" id="bU">\u25b2</div></div><div class="row"><div class="btn" id="bL">\u25c4</div><div class="btn" style="visibility:hidden">\u25bc</div><div class="btn" id="bR">\u25ba</div></div><div class="row"><div class="btn" id="bD">\u25bc</div></div></div><div id="overlay"><h2 id="otitle">\uD83D\uDC0D Snake</h2><p id="otext">Tap buttons or use arrow keys</p><button id="obtn">Start</button></div><script>const cv=document.getElementById('c'),ctx=cv.getContext('2d'),C=25,R=40,CW=cv.width/C,CH=cv.height/R;let sn,dir,nd,fd,sc,hs=0,running=false,iv;const seEl=document.getElementById('score'),hsEl=document.getElementById('hs'),ov=document.getElementById('overlay'),ot=document.getElementById('otitle'),op=document.getElementById('otext'),ob=document.getElementById('obtn');function rn(n){return Math.floor(Math.random()*n)}function spawnFood(){let p;do{p={x:rn(C),y:rn(R)}}while(sn.some(s=>s.x===p.x&&s.y===p.y));fd=p}function init(){const mx=Math.floor(C/2),my=Math.floor(R/2);sn=[{x:mx,y:my},{x:mx-1,y:my},{x:mx-2,y:my}];dir={x:1,y:0};nd={x:1,y:0};sc=0;seEl.textContent=0;spawnFood()}function step(){dir=nd;const h={(x:(sn[0].x+dir.x+C)%C),(y:(sn[0].y+dir.y+R)%R)};if(sn.some(s=>s.x===h.x&&s.y===h.y)){over();return}sn.unshift(h);if(h.x===fd.x&&h.y===fd.y){sc++;seEl.textContent=sc;if(sc>hs){hs=sc;hsEl.textContent='best: '+hs}spawnFood()}else{sn.pop()}draw()}function draw(){ctx.fillStyle='#0a0a0a';ctx.fillRect(0,0,cv.width,cv.height);ctx.strokeStyle='#111';ctx.lineWidth=.5;for(let i=0;i<=C;i++){ctx.beginPath();ctx.moveTo(i*CW,0);ctx.lineTo(i*CW,cv.height);ctx.stroke()}for(let j=0;j<=R;j++){ctx.beginPath();ctx.moveTo(0,j*CH);ctx.lineTo(cv.width,j*CH);ctx.stroke()}ctx.shadowColor='#ff4444';ctx.shadowBlur=10;ctx.fillStyle='#ff4444';ctx.beginPath();ctx.arc(fd.x*CW+CW/2,fd.y*CH+CH/2,Math.min(CW,CH)*.38,0,Math.PI*2);ctx.fill();ctx.shadowBlur=0;sn.forEach((s,i)=>{const t=i/sn.length;ctx.fillStyle=i===0?'#00ff88':\`hsl(\${145-t*30},\${80-t*20}%,\${50-t*15}%)\`;const p=i===0?1:2;if(ctx.roundRect){ctx.beginPath();ctx.roundRect(s.x*CW+p,s.y*CH+p,CW-p*2,CH-p*2,i===0?4:3);ctx.fill()}else{ctx.fillRect(s.x*CW+p,s.y*CH+p,CW-p*2,CH-p*2)}})}function over(){clearInterval(iv);running=false;ot.textContent='\uD83D\uDC80 Game Over';op.textContent='Score: '+sc;ob.textContent='Play Again';ov.classList.remove('off')}function start(){ov.classList.add('off');init();clearInterval(iv);running=true;iv=setInterval(step,120)}ob.addEventListener('click',start);const DM={ArrowUp:{x:0,y:-1},ArrowDown:{x:0,y:1},ArrowLeft:{x:-1,y:0},ArrowRight:{x:1,y:0},w:{x:0,y:-1},s:{x:0,y:1},a:{x:-1,y:0},d:{x:1,y:0}};function tryDir(d){if(running&&!(d.x===-dir.x&&d.y===-dir.y))nd=d}function onKey(e){if(!running){if(e.key==='Enter')start();return}const d=DM[e.key];if(d){tryDir(d);e.preventDefault()}}window.addEventListener('keydown',onKey);document.addEventListener('keydown',onKey);document.getElementById('bU').addEventListener('click',()=>tryDir({x:0,y:-1}));document.getElementById('bD').addEventListener('click',()=>tryDir({x:0,y:1}));document.getElementById('bL').addEventListener('click',()=>tryDir({x:-1,y:0}));document.getElementById('bR').addEventListener('click',()=>tryDir({x:1,y:0}));let tx=0,ty=0;cv.addEventListener('touchstart',e=>{tx=e.touches[0].clientX;ty=e.touches[0].clientY},{passive:true});cv.addEventListener('touchend',e=>{const dx=e.changedTouches[0].clientX-tx,dy=e.changedTouches[0].clientY-ty;if(!running)return;if(Math.abs(dx)>Math.abs(dy)){tryDir(dx>0?{x:1,y:0}:{x:-1,y:0})}else{tryDir(dy>0?{x:0,y:1}:{x:0,y:-1})}});draw();<\/script></body></html>`;
                        // ────────────────────────────────────────────────────────────────────

                        const container = document.getElementById('canvas-container');
                        const empty = document.getElementById('canvas-empty');
                        let sourceMode = false;

                        // ── Games collection: single source of truth in VFS canvas/games.json ──
                        // VFS auto-syncs to localStorage['traits.pvfs'] on every write.

                        function readGamesCollection() {
                            try {
                                const raw = localStorage.getItem('traits.pvfs');
                                if (!raw) return { active: null, games: {} };
                                const files = JSON.parse(raw);
                                const json = files['canvas/games.json'];
                                if (!json) return { active: null, games: {} };
                                return JSON.parse(json);
                            } catch(_) { return { active: null, games: {} }; }
                        }

                        function getActiveGameContent() {
                            const col = readGamesCollection();
                            if (!col.active || !col.games[col.active]) return '';
                            return col.games[col.active].content || '';
                        }

                        function getGamesList() {
                            const col = readGamesCollection();
                            const list = [];
                            for (const [id, g] of Object.entries(col.games || {})) {
                                list.push({
                                    id, name: g.name || 'untitled',
                                    length: (g.content || '').length,
                                    active: id === col.active,
                                    updated: g.updated || ''
                                });
                            }
                            list.sort((a, b) => (b.updated || '').localeCompare(a.updated || ''));
                            return list;
                        }

                        const gameSelect = document.getElementById('game-select');

                        function renderProjectBar() {
                            const games = getGamesList();
                            const sel = gameSelect;
                            sel.innerHTML = '';
                            if (!games.length) {
                                sel.innerHTML = '<option value="" disabled selected>no games</option>';
                                return;
                            }
                            games.forEach(g => {
                                const opt = document.createElement('option');
                                opt.value = g.id;
                                opt.textContent = (g.name || 'untitled');
                                if (g.active) opt.selected = true;
                                sel.appendChild(opt);
                            });
                        }

                        gameSelect.addEventListener('change', (e) => {
                            const id = e.target.value;
                            if (id) activateGame(id);
                        });

                        async function activateGame(id) {
                            const sdk = window._traitsSDK;
                            if (!sdk) return;
                            const res = await sdk.call('sys.canvas', ['activate', id]);
                            const r = res?.result || res || {};
                            const content = r.content || '';
                            _currentContent = ''; // force re-render
                            renderCanvas(content);
                            renderProjectBar();
                        }

                        async function saveProject() {
                            const col = readGamesCollection();
                            if (!col.active || !col.games[col.active]) {
                                alert('Canvas is empty — nothing to save.');
                                return;
                            }
                            const current = col.games[col.active].name || '';
                            const name = prompt('Game name:', current === 'untitled' ? '' : current);
                            if (!name || !name.trim()) return;
                            const sdk = window._traitsSDK;
                            if (sdk) {
                                await sdk.call('sys.canvas', ['rename', name.trim()]);
                                renderProjectBar();
                            }
                        }

                        document.getElementById('btnSave').addEventListener('click', saveProject);
                        window.addEventListener('traits-canvas-projects-changed', renderProjectBar);
                        renderProjectBar();

                        const phoneFrame    = document.getElementById('phone-frame');
                        const phoneViewport = document.getElementById('phone-viewport');
                        let _currentContent = '';

                        // Click on the phone frame focuses the iframe for keyboard input
                        phoneFrame.addEventListener('click', () => {
                            try {
                                const iDoc = phoneViewport.contentDocument;
                                const target = iDoc && (iDoc.querySelector('canvas') || iDoc.body);
                                if (target) { target.focus(); } else { phoneViewport.focus(); }
                            } catch(_) { phoneViewport.focus(); }
                        });

                        // Forward game keys from parent into iframe
                        // Strategy: dispatch KeyboardEvent on iDoc (for event listeners),
                        // directly set iWin.keys[code] (for poll-based games),
                        // and call iWin.handleInput() if it exists (fixes games where handleInput isn't in the loop)
                        const GAME_KEYS = ['ArrowUp','ArrowDown','ArrowLeft','ArrowRight',' ','w','a','s','d','z','c','Shift'];
                        function forwardKey(e, type) {
                            if (!phoneFrame.classList.contains('visible')) return;
                            if (!GAME_KEYS.includes(e.key)) return;
                            e.preventDefault();
                            try {
                                const iDoc = phoneViewport.contentDocument;
                                const iWin = phoneViewport.contentWindow;
                                if (!iDoc || !iWin) return;
                                // 1. Dispatch event on iframe document (for document.onkeydown listeners)
                                iDoc.dispatchEvent(new KeyboardEvent(type, {
                                    key: e.key, code: e.code, keyCode: e.keyCode,
                                    which: e.which, shiftKey: e.shiftKey,
                                    bubbles: true, cancelable: true
                                }));
                                // 2. Directly set key state for poll-based games (keys[code] pattern)
                                if (iWin.keys && typeof iWin.keys === 'object') {
                                    iWin.keys[e.code || e.key] = (type === 'keydown');
                                }
                                // 3. Call handleInput() if game exposes it (fixes games where it's not in the loop)
                                if (type === 'keydown' && typeof iWin.handleInput === 'function') {
                                    try { iWin.handleInput(); } catch(_) {}
                                }
                            } catch(_) {}
                        }
                        document.addEventListener('keydown', (e) => forwardKey(e, 'keydown'));
                        document.addEventListener('keyup', (e) => forwardKey(e, 'keyup'));
                        // Also intercept keys when iframe itself has focus (prevent parent scroll)
                        phoneViewport.addEventListener('load', () => {
                            try {
                                const iDoc = phoneViewport.contentDocument;
                                if (iDoc) {
                                    iDoc.addEventListener('keydown', (e) => {
                                        if (GAME_KEYS.includes(e.key)) e.preventDefault();
                                    }, { passive: false });
                                }
                            } catch(_) {}
                        });

                        const BRIDGE = `<script>(function(){
                            var sdk = function(){ return window.parent._traitsSDK; };
                            window.traits = {
                                call:   function(p,a)   { return sdk() && sdk().call(p, a||[]); },
                                list:   function(ns)    { return sdk() && sdk().call('sys.list', ns?[ns]:[]); },
                                info:   function(p)     { return sdk() && sdk().call('sys.info', [p]); },
                                echo:   function(t)     { return sdk() && sdk().call('sys.echo', [t]); },
                                canvas: function(a,c)   { return sdk() && sdk().call('sys.canvas', c!==undefined?[a,c]:[a]); },
                                audio:  function(a)     {
                                    var r = Array.prototype.slice.call(arguments, 1);
                                    return sdk() && sdk().call('sys.audio', [a].concat(r));
                                },
                            };
                            var _qs = document.querySelector.bind(document);
                            document.querySelector = function(s) {
                                return _qs(s.replace(/^#phone-viewport\s+/,'').replace(/^#canvas-container\s+/,''));
                            };
                        })();<\/script>`;

                        function renderCanvas(content) {
                            if (!content) {
                                if (_currentContent === '') return;
                                _currentContent = '';
                                phoneFrame.classList.remove('visible');
                                container.appendChild(empty);
                                empty.style.display = 'flex';
                                return;
                            }
                            if (content === _currentContent) return;
                            _currentContent = content;
                            empty.style.display = 'none';
                            phoneFrame.classList.add('visible');
                            document.querySelectorAll('style[data-canvas]').forEach(s => s.remove());

                            let fullHtml = content.trim();
                            if (!/<html[\s>]/i.test(fullHtml)) {
                                fullHtml = '<!DOCTYPE html><html><head>' +
                                    '<meta charset="UTF-8">' +
                                    '<style>*{margin:0;padding:0;box-sizing:border-box}' +
                                    'html,body{width:390px;height:844px;overflow:hidden;background:#0a0a0a;color:#e0e0e0}' +
                                    'canvas{display:block}</style>' +
                                    BRIDGE + '</head><body>' + fullHtml + '</body></html>';
                            } else {
                                if (/<head\b[^>]*>/i.test(fullHtml)) {
                                    fullHtml = fullHtml.replace(/(<head\b[^>]*>)/i, '$1' + BRIDGE);
                                } else {
                                    fullHtml = fullHtml.replace(/(<html\b[^>]*>)/i, '$1<head>' + BRIDGE + '</head>');
                                }
                            }
                            phoneViewport.srcdoc = fullHtml;
                            // Focus iframe so keyboard events reach the game
                            phoneViewport.addEventListener('load', function _f() {
                                phoneViewport.removeEventListener('load', _f);
                                try {
                                    const iDoc = phoneViewport.contentDocument;
                                    const target = iDoc && (iDoc.querySelector('canvas') || iDoc.body);
                                    if (target) { target.setAttribute('tabindex', '0'); target.focus(); }
                                    else { phoneViewport.focus(); }
                                } catch(_) { phoneViewport.focus(); }
                            });
                        }

                        // Read canvas/app.html from VFS (backward compat for poller)
                        function readCanvasFromStorage() {
                            try {
                                const raw = localStorage.getItem('traits.pvfs');
                                if (!raw) return '';
                                const files = JSON.parse(raw);
                                return files['canvas/app.html'] || '';
                            } catch(_) { return ''; }
                        }

                        // Listen for live updates from voice/SDK
                        window.addEventListener('traits-canvas-update', (e) => {
                            const content = e.detail?.content;
                            if (content !== undefined) {
                                __lastContent = content;
                                renderCanvas(content);
                                renderProjectBar();
                            } else {
                                // Re-read from games.json via VFS
                                const active = getActiveGameContent();
                                if (active) { __lastContent = active; renderCanvas(active); }
                                else {
                                    const stored = readCanvasFromStorage();
                                    if (stored) { __lastContent = stored; renderCanvas(stored); }
                                }
                                renderProjectBar();
                            }
                        });

                        // Clear button
                        document.getElementById('btnClear').addEventListener('click', async () => {
                            const sdk = window._traitsSDK;
                            if (sdk) await sdk.call('sys.canvas', ['clear']);
                            renderCanvas('');
                        });

                        // View Source toggle
                        document.getElementById('btnSource').addEventListener('click', () => {
                            sourceMode = !sourceMode;
                            const btn = document.getElementById('btnSource');
                            if (sourceMode) {
                                phoneFrame.classList.add('visible');
                                const escaped = (_currentContent || '(empty)')
                                    .replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;');
                                phoneViewport.srcdoc = '<!DOCTYPE html><html><head><style>' +
                                    'body{margin:0;background:#0a0a0a}' +
                                    'pre{white-space:pre-wrap;word-break:break-all;color:#888;' +
                                    'font-size:12px;padding:16px;height:100vh;box-sizing:border-box;overflow:auto}' +
                                    '</style></head><body><pre>' + escaped + '</pre></body></html>';
                                btn.textContent = 'Live View';
                            } else {
                                btn.textContent = 'View Source';
                                renderCanvas(_currentContent);
                            }
                        });

                        // ── Auto-load on page init ──
                        // Migrate existing canvas/app.html to games.json if no games exist yet.
                        // Also reconcile orphaned content (agent wrote canvas/app.html but games.json wasn't updated).
                        let __lastContent = '';
                        const _initCol = readGamesCollection();
                        const _initHasGames = Object.keys(_initCol.games || {}).length > 0;
                        const _existingHtml = readCanvasFromStorage();
                        if (!_initHasGames && _existingHtml) {
                            // Bootstrap: call sys.canvas set to create first game entry
                            __lastContent = _existingHtml;
                            renderCanvas(_existingHtml);
                            (async () => {
                                let tries = 0;
                                while (!window._traitsSDK && tries < 40) {
                                    await new Promise(r => setTimeout(r, 250));
                                    tries++;
                                }
                                const sdk = window._traitsSDK;
                                if (sdk) {
                                    await sdk.call('sys.canvas', ['set', _existingHtml]);
                                    const _titleMatch = _existingHtml.match(/<title[^>]*>([^<]+)<\/title>/i);
                                    const _gameName = _titleMatch ? _titleMatch[1].trim() : 'untitled';
                                    await sdk.call('sys.canvas', ['rename', _gameName]);
                                    renderProjectBar();
                                }
                            })();
                        } else if (_initHasGames && _existingHtml) {
                            // Check for orphaned content: canvas/app.html differs from active game
                            const activeContent = _initCol.games[_initCol.active]?.content || '';
                            if (_existingHtml.length > 100 && _existingHtml !== activeContent) {
                                // Orphan detected — show it now, save as new game once SDK ready
                                __lastContent = _existingHtml;
                                renderCanvas(_existingHtml);
                                (async () => {
                                    let tries = 0;
                                    while (!window._traitsSDK && tries < 40) {
                                        await new Promise(r => setTimeout(r, 250));
                                        tries++;
                                    }
                                    const sdk = window._traitsSDK;
                                    if (sdk) {
                                        const _titleMatch = _existingHtml.match(/<title[^>]*>([^<]+)<\/title>/i);
                                        const _gameName = _titleMatch ? _titleMatch[1].trim() : 'untitled';
                                        await sdk.call('sys.canvas', ['new', _gameName]);
                                        await sdk.call('sys.canvas', ['set', _existingHtml]);
                                        renderProjectBar();
                                    }
                                })();
                            } else {
                                __lastContent = activeContent || _existingHtml;
                                if (__lastContent) renderCanvas(__lastContent);
                                // Fix name mismatch: if content <title> differs from game name, rename
                                (async () => {
                                    let tries = 0;
                                    while (!window._traitsSDK && tries < 40) {
                                        await new Promise(r => setTimeout(r, 250));
                                        tries++;
                                    }
                                    const sdk = window._traitsSDK;
                                    if (sdk && activeContent) {
                                        const contentTitle = (activeContent.match(/<title[^>]*>([^<]+)<\/title>/i) || [])[1]?.trim();
                                        const gameName = _initCol.games[_initCol.active]?.name;
                                        if (contentTitle && gameName && contentTitle.toLowerCase() !== gameName.toLowerCase()) {
                                            await sdk.call('sys.canvas', ['rename', contentTitle]);
                                            renderProjectBar();
                                        }
                                    }
                                })();
                            }
                        } else {
                            __lastContent = getActiveGameContent();
                            if (!__lastContent) __lastContent = readCanvasFromStorage();
                            if (__lastContent) renderCanvas(__lastContent);
                        }

                        // Also restore from WASM VFS once SDK is ready
                        (async () => {
                            try {
                                let tries = 0;
                                while (!window._traitsSDK && tries < 40) {
                                    await new Promise(r => setTimeout(r, 250));
                                    tries++;
                                }
                                const content = getActiveGameContent();
                                if (content && content !== __lastContent) {
                                    __lastContent = content;
                                    renderCanvas(content);
                                }
                                renderProjectBar();
                            } catch(_) {}
                        })();

                        // ── Seed built-in games if missing ──
                        // Adds Snake (and optionally other defaults) if not present by name.
                        // Never overwrites or changes the currently active game.
                        (async () => {
                            try {
                                let tries = 0;
                                while (!window._traitsSDK && tries < 40) {
                                    await new Promise(r => setTimeout(r, 250));
                                    tries++;
                                }
                                const sdk = window._traitsSDK;
                                if (!sdk) return;
                                const col = readGamesCollection();
                                const names = Object.values(col.games || {}).map(g => (g.name || '').toLowerCase());
                                if (names.includes('snake')) return; // already seeded
                                const prevActive = col.active;
                                // Create snake game as a new inactive entry
                                await sdk.call('sys.canvas', ['new', 'Snake']);
                                await sdk.call('sys.canvas', ['set', SNAKE_GAME_HTML]);
                                // Restore the previously active game
                                if (prevActive) {
                                    await sdk.call('sys.canvas', ['activate', prevActive]);
                                    const col2 = readGamesCollection();
                                    const restored = col2.games[prevActive]?.content || '';
                                    if (restored) renderCanvas(restored);
                                }
                                renderProjectBar();
                            } catch(_) {}
                        })();

                        // Poll VFS for agent writes (1s backup for missed events)
                        const _pollId = setInterval(() => {
                            try {
                                if (sourceMode) return;
                                const content = getActiveGameContent() || readCanvasFromStorage();
                                if (content && content !== __lastContent) {
                                    __lastContent = content;
                                    renderCanvas(content);
                                    renderProjectBar();
                                }
                            } catch(_) {}
                        }, 1000);

                        // ── FAB menu ──
                        const fabToggle = document.getElementById('fabToggle');
                        const fabMenu = document.getElementById('fabMenu');
                        fabToggle.addEventListener('click', (e) => {
                            e.stopPropagation();
                            fabMenu.classList.toggle('show');
                            fabToggle.classList.toggle('open');
                        });
                        // Close FAB menu when clicking outside
                        document.addEventListener('click', (e) => {
                            if (!e.target.closest('#canvas-fab') && !e.target.closest('#voice-chat-modal')) {
                                fabMenu.classList.remove('show');
                                fabToggle.classList.remove('open');
                            }
                        });
                        // New Canvas button — creates a new game entry
                        document.getElementById('fabNew').addEventListener('click', async () => {
                            fabMenu.classList.remove('show');
                            fabToggle.classList.remove('open');
                            const sdk = window._traitsSDK;
                            if (sdk) await sdk.call('sys.canvas', ['new']);
                            __lastContent = '';
                            _currentContent = '';
                            renderCanvas('');
                            renderProjectBar();
                        });

                        // Voice button
                        document.getElementById('fabVoice').addEventListener('click', () => {
                            fabMenu.classList.remove('show');
                            fabToggle.classList.remove('open');
                            // Dispatch voice start via the global voice control bridge
                            window.dispatchEvent(new CustomEvent('traits-voice-control', { detail: { voice_control_action: 'start' } }));
                        });
                        // Splat viewer button
                        document.getElementById('fabSplats').addEventListener('click', async () => {
                            fabMenu.classList.remove('show');
                            fabToggle.classList.remove('open');
                            try {
                                const sdk = window._traitsSDK;
                                if (!sdk) return;
                                const splats = await sdk.call('www.splats', ['render']);
                                const html = (typeof splats === 'string') ? splats : splats?.result;
                                if (html && typeof html === 'string') {
                                    await sdk.call('sys.canvas', ['set', html]);
                                    renderCanvas(html);
                                }
                            } catch(e) { console.warn('splat load:', e); }
                        });

                        // Share / Receive project via WebRTC P2P
                        document.getElementById('fabShare').addEventListener('click', () => {
                            fabMenu.classList.remove('show'); fabToggle.classList.remove('open');
                            smOpen('send');
                        });
                        document.getElementById('fabReceive').addEventListener('click', () => {
                            fabMenu.classList.remove('show'); fabToggle.classList.remove('open');
                            smOpen('receive');
                        });

                        // ── P2P Share Modal ──
                        const shareModal   = document.getElementById('share-modal');
                        const smTitleEl    = document.getElementById('smTitle');
                        const smBodyEl     = document.getElementById('smBody');
                        const smCloseBtn   = document.getElementById('smClose');
                        const RELAY        = 'https://relay.slob.games';
                        let _smAbort = false;

                        smCloseBtn.addEventListener('click', (e) => { e.stopPropagation(); smHide(); });

                        // Drag
                        (() => {
                            let drag = false, ox = 0, oy = 0;
                            shareModal.querySelector('.sm-header').addEventListener('mousedown', (e) => {
                                drag = true;
                                shareModal.style.right = 'auto'; shareModal.style.bottom = 'auto';
                                ox = e.clientX - shareModal.offsetLeft;
                                oy = e.clientY - shareModal.offsetTop;
                            });
                            document.addEventListener('mousemove', (e) => {
                                if (!drag) return;
                                shareModal.style.left = (e.clientX - ox) + 'px';
                                shareModal.style.top  = (e.clientY - oy) + 'px';
                            });
                            document.addEventListener('mouseup', () => { drag = false; });
                        })();

                        function smOpen(mode) {
                            _smAbort = false;
                            shareModal.classList.add('sm-open');
                            smTitleEl.textContent = mode === 'send' ? '\ud83d\udce4\u2002Share Project' : '\ud83d\udce5\u2002Receive Project';
                            if (mode === 'send') smStartSend();
                            else smStartReceive();
                        }
                        function smHide() {
                            _smAbort = true;
                            shareModal.classList.remove('sm-open');
                        }
                        function smStatus(text, cls) {
                            const el = smBodyEl.querySelector('.sm-status');
                            if (!el) return;
                            el.textContent = text;
                            el.className = 'sm-status' + (cls ? ' ' + cls : '');
                        }
                        function smProgress(pct) {
                            const bar = smBodyEl.querySelector('.sm-progress-bar');
                            if (bar) bar.style.width = pct + '%';
                        }

                        // ── SENDER ──
                        // Strategy: register relay code, display it, wait for receiver to
                        // call path='project.recv' via relay/call, then respond with the
                        // full project content. No WebRTC — content travels through the
                        // relay (Cloudflare Worker). Two round-trips, always works.
                        async function smStartSend() {
                            smBodyEl.innerHTML = '<div class="sm-status">Registering\u2026</div>';

                            let code;
                            try {
                                const r = await fetch(RELAY + '/relay/register', { method: 'POST' });
                                if (!r.ok) throw new Error('HTTP ' + r.status);
                                const j = await r.json(); code = j.code;
                            } catch(e) {
                                smBodyEl.innerHTML = '<div class="sm-status err">\u26a0 Relay unreachable: ' + e.message + '</div>';
                                return;
                            }
                            if (_smAbort) return;

                            const shareUrl = location.origin + location.pathname + '#/canvas?receive=' + code;
                            smBodyEl.innerHTML = `
                                <div class="sm-label">Share this code with the receiver:</div>
                                <div class="sm-code-display">
                                    ${code.split('').map(c => `<div class="sm-code-char">${c}</div>`).join('')}
                                </div>
                                <div class="sm-link-row">
                                    <span class="sm-link" title="${shareUrl}">${shareUrl}</span>
                                    <button class="sm-btn" id="smCopyBtn">Copy</button>
                                </div>
                                <div class="sm-progress"><div class="sm-progress-bar" style="width:0%"></div></div>
                                <div class="sm-status">Waiting for receiver\u2026</div>
                            `;
                            smBodyEl.querySelector('#smCopyBtn').addEventListener('click', function() {
                                navigator.clipboard.writeText(shareUrl).then(() => {
                                    this.textContent = 'Copied!';
                                    setTimeout(() => { this.textContent = 'Copy'; }, 1600);
                                });
                            });

                            // Long-poll: wait for receiver to knock
                            let knock;
                            try {
                                const r = await fetch(RELAY + '/relay/poll?code=' + code, { signal: AbortSignal.timeout(120000) });
                                if (!r.ok) throw new Error('poll ' + r.status);
                                knock = await r.json();
                            } catch(e) {
                                if (!_smAbort) smStatus('\u26a0 Timed out — no receiver connected.', 'err');
                                return;
                            }
                            if (_smAbort) return;

                            smStatus('Receiver connected! Reading project\u2026');
                            smProgress(40);

                            // Read the current project
                            let content = '';
                            try {
                                const sdk = window._traitsSDK;
                                if (sdk) {
                                    const res = await sdk.call('sys.canvas', ['get']);
                                    content = res?.result?.content || res?.content || '';
                                }
                            } catch(_) {}
                            if (!content) content = document.getElementById('phone-viewport')?.srcdoc || '';

                            smStatus('Sending project\u2026');
                            smProgress(70);

                            // Respond with the project content
                            try {
                                await fetch(RELAY + '/relay/respond', {
                                    method: 'POST',
                                    headers: { 'Content-Type': 'application/json' },
                                    body: JSON.stringify({ code, id: knock.id, result: { content } }),
                                });
                            } catch(e) {
                                smStatus('\u26a0 Failed to send project: ' + e.message, 'err');
                                return;
                            }

                            smProgress(100);
                            smStatus('\u2713 Project sent!', 'ok');
                            setTimeout(() => smHide(), 1800);
                        }

                        // ── RECEIVER ──
                        async function smStartReceive() {
                            const autoCode = (() => {
                                try { const h = location.hash.split('?')[1]; return h ? new URLSearchParams(h).get('receive') || '' : ''; } catch(_) { return ''; }
                            })();
                            smBodyEl.innerHTML = `
                                <div class="sm-label">Enter the 4-letter code from the sender:</div>
                                <div class="sm-code-inputs" id="smCodeInputs">
                                    <input class="sm-ci" maxlength="1" inputmode="text" autocomplete="off" spellcheck="false">
                                    <input class="sm-ci" maxlength="1" inputmode="text" autocomplete="off" spellcheck="false">
                                    <input class="sm-ci" maxlength="1" inputmode="text" autocomplete="off" spellcheck="false">
                                    <input class="sm-ci" maxlength="1" inputmode="text" autocomplete="off" spellcheck="false">
                                </div>
                                <button class="sm-btn-primary" id="smConnectBtn">Receive \u2192</button>
                                <div class="sm-progress" style="opacity:0"><div class="sm-progress-bar" style="width:0%"></div></div>
                                <div class="sm-status"></div>
                            `;
                            const inputs = Array.from(smBodyEl.querySelectorAll('.sm-ci'));
                            inputs.forEach((inp, i) => {
                                inp.addEventListener('input', () => {
                                    inp.value = inp.value.toUpperCase().replace(/[^A-Z0-9]/g, '');
                                    if (inp.value && i < 3) inputs[i + 1].focus();
                                });
                                inp.addEventListener('keydown', (e) => {
                                    if (e.key === 'Backspace' && !inp.value && i > 0) inputs[i - 1].focus();
                                    if (e.key === 'Enter') smDoReceive(inputs);
                                });
                                inp.addEventListener('paste', (e) => {
                                    const txt = (e.clipboardData.getData('text') || '').toUpperCase().replace(/[^A-Z0-9]/g, '');
                                    inputs.forEach((b, j) => { b.value = txt[j] || ''; });
                                    e.preventDefault();
                                    inputs[Math.min(txt.length, 3)].focus();
                                });
                            });
                            smBodyEl.querySelector('#smConnectBtn').addEventListener('click', () => smDoReceive(inputs));
                            if (autoCode.length >= 4) {
                                autoCode.toUpperCase().split('').forEach((c, i) => { if (inputs[i]) inputs[i].value = c; });
                                setTimeout(() => smDoReceive(inputs), 300);
                            } else {
                                inputs[0].focus();
                            }
                        }

                        async function smDoReceive(inputs) {
                            const code = inputs.map(i => i.value).join('').toUpperCase();
                            if (code.length < 4) { smStatus('Enter all 4 characters.'); return; }

                            smBodyEl.querySelector('#smConnectBtn').disabled = true;
                            smBodyEl.querySelector('.sm-progress').style.opacity = '1';
                            smStatus('Contacting sender\u2026');
                            smProgress(30);

                            let result;
                            try {
                                const r = await fetch(RELAY + '/relay/call', {
                                    method: 'POST',
                                    headers: { 'Content-Type': 'application/json' },
                                    body: JSON.stringify({ code, path: 'project.recv', args: [] }),
                                });
                                if (!r.ok) throw new Error('HTTP ' + r.status);
                                const j = await r.json();
                                if (j.error) throw new Error(j.error);
                                result = j.result;
                            } catch(e) {
                                smStatus('\u26a0 ' + e.message, 'err');
                                smBodyEl.querySelector('#smConnectBtn').disabled = false;
                                return;
                            }
                            if (_smAbort) return;

                            smStatus('Applying project\u2026');
                            smProgress(80);
                            await smApplyProject(result?.content || result);
                        }

                        async function smApplyProject(content) {
                            if (!content) return;
                            try {
                                const sdk = window._traitsSDK;
                                if (sdk) {
                                    // Create a new game for the received project
                                    await sdk.call('sys.canvas', ['new', 'received']);
                                    await sdk.call('sys.canvas', ['set', content]);
                                }
                            } catch(_) {}
                            __lastContent = content;
                            _currentContent = '';
                            renderCanvas(content);
                            renderProjectBar();
                            smStatus('\u2713 Project received & saved!', 'ok');
                            smProgress(100);
                            setTimeout(() => smHide(), 1800);
                        }
                        window._pageCleanup = async () => {
                            clearInterval(_pollId);
                            fabMenu.classList.remove('show');
                            document.querySelectorAll('style[data-canvas]').forEach(s => s.remove());
                            try { delete window.traits; } catch(_) {}
                        };

                        // ── Voice Chat Modal ──
                        const vcModal   = document.getElementById('voice-chat-modal');
                        const vcmLog    = document.getElementById('vcmLog');
                        const vcmInput  = document.getElementById('vcmInput');
                        const vcmSendBtn = document.getElementById('vcmSend');
                        const vcmCloseBtn = document.getElementById('vcmClose');

                        function vcmOpen()  { vcModal.classList.add('vcm-open'); }
                        function vcmHide()  { vcModal.classList.remove('vcm-open'); }
                        function vcmToggle() {
                            vcModal.classList.contains('vcm-open') ? vcmHide() : vcmOpen();
                        }

                        function vcmAppend(role, text) {
                            if (!text) return;
                            const el = document.createElement('div');
                            el.className = 'vcm-msg ' + role;
                            el.textContent = text;
                            vcmLog.appendChild(el);
                            vcmLog.scrollTop = vcmLog.scrollHeight;
                        }

                        // Long-press (+) button to open the chat modal
                        (() => {
                            let _pressTimer = null;
                            fabToggle.addEventListener('pointerdown', () => {
                                _pressTimer = setTimeout(() => {
                                    _pressTimer = null;
                                    fabMenu.classList.remove('show');
                                    fabToggle.classList.remove('open');
                                    vcmToggle();
                                }, 500);
                            });
                            fabToggle.addEventListener('pointerup', () => { if (_pressTimer) clearTimeout(_pressTimer); });
                            fabToggle.addEventListener('pointercancel', () => { if (_pressTimer) clearTimeout(_pressTimer); });
                            // Double-click also opens the modal immediately
                            fabToggle.addEventListener('dblclick', (e) => {
                                e.stopPropagation();
                                fabMenu.classList.remove('show');
                                fabToggle.classList.remove('open');
                                vcmToggle();
                            });
                        })();

                        vcmCloseBtn.addEventListener('click', (e) => { e.stopPropagation(); vcmHide(); });

                        // Drag-to-reposition via header
                        (() => {
                            let drag = false, ox = 0, oy = 0;
                            const header = vcModal.querySelector('.vcm-header');
                            header.addEventListener('mousedown', (e) => {
                                drag = true;
                                vcModal.style.right = 'auto';
                                vcModal.style.bottom = 'auto';
                                ox = e.clientX - vcModal.offsetLeft;
                                oy = e.clientY - vcModal.offsetTop;
                            });
                            document.addEventListener('mousemove', (e) => {
                                if (!drag) return;
                                vcModal.style.left = (e.clientX - ox) + 'px';
                                vcModal.style.top  = (e.clientY - oy) + 'px';
                            });
                            document.addEventListener('mouseup', () => { drag = false; });
                        })();

                        // Voice events → chat log
                        window.addEventListener('voice-event', (e) => {
                            const d = e.detail;
                            switch (d.type) {
                                case 'started':
                                    vcmAppend('system', '🎤 Voice session started');
                                    break;
                                case 'stopped':
                                case 'disconnected':
                                    vcmAppend('system', '⏹ Voice session ended');
                                    break;
                                case 'transcript':
                                    vcmAppend('user', d.text);
                                    break;
                                case 'response':
                                    vcmAppend('assistant', d.text);
                                    break;
                                case 'tool_call': {
                                    let args = d.arguments || '';
                                    try { args = JSON.stringify(JSON.parse(args), null, 0).slice(0, 140); } catch(_) { args = args.slice(0, 100); }
                                    vcmAppend('tool', '\u26A1 ' + d.name + '(' + args + ')');
                                    break;
                                }
                                case 'tool_result': {
                                    const preview = (d.result || '').slice(0, 140);
                                    vcmAppend('tool-result', '\u2713 ' + d.name + ': ' + preview);
                                    break;
                                }
                                case 'error':
                                    vcmAppend('system', '\u26A0 ' + d.message);
                                    break;
                            }
                        });

                        // Send typed message to voice model
                        function vcmSendText() {
                            const text = vcmInput.value.trim();
                            if (!text) return;
                            const sdk = window._traitsSDK;
                            if (!sdk || !sdk.sendVoiceText) {
                                vcmAppend('system', '\u26A0 Voice not active — start voice first');
                                return;
                            }
                            const ok = sdk.sendVoiceText(text);
                            if (ok) {
                                vcmAppend('user', text);
                                vcmInput.value = '';
                            } else {
                                vcmAppend('system', '\u26A0 Voice not connected');
                            }
                        }
                        vcmSendBtn.addEventListener('click', vcmSendText);
                        vcmInput.addEventListener('keydown', (e) => { if (e.key === 'Enter') vcmSendText(); });
                    })();
                "#)) }
            }
        }
    };
    Value::String(markup.into_string())
}
