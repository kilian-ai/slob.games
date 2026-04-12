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
                        :root { --bg: #0a0a0a; --fg: #e0e0e0; --accent: #00e0ff; --border: #1a1a2e; }
                        html, body { margin: 0; padding: 0; background: var(--bg); color: var(--fg); font-family: 'Courier New', Menlo, monospace; overflow: hidden; height: 100%; }
                        .canvas-header {
                            display: flex; align-items: center; justify-content: space-between;
                            padding: 10px 20px; border-bottom: 1px solid rgba(0,224,255,0.1);
                            background: rgba(8,8,14,0.95);
                        }
                        .canvas-header h1 { font-size: 13px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.06em; color: #8899a6; }
                        .canvas-header h1 .accent { color: var(--accent); text-shadow: 0 0 8px rgba(0,224,255,0.4); }
                        .canvas-header .actions { display: flex; gap: 8px; }
                        .canvas-header button {
                            background: transparent; border: 1px solid #1e1e2e;
                            color: #8899a6; padding: 4px 12px; border-radius: 4px;
                            cursor: pointer; font-size: 11px; font-family: 'Courier New', Menlo, monospace;
                            text-transform: uppercase; letter-spacing: 0.04em; transition: all 0.2s;
                        }
                        .canvas-header button:hover { border-color: var(--accent); color: var(--accent); box-shadow: 0 0 8px rgba(0,224,255,0.15); }
                        .canvas-header button.save-btn { border-color: rgba(0,255,136,0.3); color: #00ff88; }
                        .canvas-header button.save-btn:hover { border-color: rgba(0,255,136,0.5); color: #00ff88; box-shadow: 0 0 8px rgba(0,255,136,0.2); }

                        /* Game selector dropdown */
                        #game-select {
                            background: #0c0c16; color: #8899a6;
                            border: 1px solid #1e1e2e; border-radius: 4px;
                            padding: 4px 8px; font-size: 11px;
                            cursor: pointer; outline: none; max-width: 180px;
                            font-family: 'Courier New', Menlo, monospace;
                            text-transform: uppercase; letter-spacing: 0.02em;
                            -webkit-appearance: none; appearance: none;
                            background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='6'%3E%3Cpath d='M0 0l5 6 5-6z' fill='%23556'/%3E%3C/svg%3E");
                            background-repeat: no-repeat; background-position: right 8px center;
                            padding-right: 22px;
                        }
                        #game-select:hover { border-color: var(--accent); }
                        #game-select:focus { border-color: var(--accent); box-shadow: 0 0 6px rgba(0,224,255,0.15); }
                        #game-select option { background: #0c0c16; color: #8899a6; }

                        #canvas-container {
                            width: 100%; height: calc(100vh - 49px);
                            padding: 0 20px; position: relative;
                            display: flex; justify-content: center; align-items: center;
                            overflow: hidden; box-sizing: border-box;
                        }
                        @keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
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
                            background: #111118;
                            border-radius: 50px;
                            border: 2px solid #2a2a3c;
                            box-shadow: 0 0 0 1px #0a0a0f, 0 0 40px rgba(0,224,255,0.06), 0 30px 80px rgba(0,0,0,0.8), inset 0 0 0 1px #1e1e2e;
                            padding: 18px 16px 22px;
                            flex-shrink: 0;
                        }
                        #phone-frame.visible { display: block; }
                        .phone-notch {
                            width: 120px; height: 34px;
                            background: #111118;
                            border-radius: 0 0 22px 22px;
                            margin: 0 auto 10px;
                            position: relative; z-index: 2;
                            display: flex; align-items: center; justify-content: center;
                            gap: 8px;
                        }
                        .phone-game-label {
                            position: absolute;
                            top: 8px;
                            left: 18px;
                            z-index: 4;
                            font: 600 11px/1.2 monospace;
                            color: #c7d1da;
                            background: rgba(10, 16, 26, 0.72);
                            border: 1px solid rgba(0, 224, 255, 0.2);
                            border-radius: 8px;
                            padding: 3px 7px;
                            max-width: 260px;
                            white-space: nowrap;
                            overflow: hidden;
                            text-overflow: ellipsis;
                        }
                        .phone-runtime-label {
                            position: absolute;
                            top: 8px;
                            right: 18px;
                            z-index: 4;
                            font: 600 11px/1 monospace;
                            color: #00e0ff;
                            opacity: 0.9;
                            background: rgba(0, 224, 255, 0.08);
                            border: 1px solid rgba(0, 224, 255, 0.28);
                            border-radius: 8px;
                            padding: 3px 7px;
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
                        .phone-nav-bracket {
                            position: absolute;
                            top: 50%;
                            width: 28px;
                            height: 92px;
                            border: 1px solid rgba(0, 224, 255, 0.32);
                            background: linear-gradient(180deg, rgba(0, 224, 255, 0.12), rgba(0, 224, 255, 0.04));
                            color: rgba(0, 224, 255, 0.78);
                            cursor: pointer;
                            transform: translateY(-50%);
                            transition: opacity 0.18s ease, border-color 0.18s ease, box-shadow 0.18s ease;
                            opacity: 0.42;
                            z-index: 4;
                            display: none;
                            user-select: none;
                            -webkit-user-select: none;
                        }
                        .phone-nav-bracket::after {
                            content: '';
                            position: absolute;
                            inset: 2px;
                            background: linear-gradient(180deg, rgba(255,255,255,0.08), rgba(0,0,0,0));
                            pointer-events: none;
                        }
                        .phone-nav-bracket:hover {
                            opacity: 0.85;
                            border-color: rgba(0, 224, 255, 0.72);
                            box-shadow: 0 0 12px rgba(0, 224, 255, 0.22);
                        }
                        .phone-nav-bracket:active { opacity: 1; }
                        .phone-nav-bracket.left {
                            left: -16px;
                            border-right: none;
                            border-radius: 11px 0 0 11px;
                            clip-path: polygon(100% 0, 36% 0, 0 17%, 58% 50%, 0 83%, 36% 100%, 100% 100%, 78% 82%, 22% 50%, 78% 18%);
                        }
                        .phone-nav-bracket.right {
                            right: -16px;
                            border-left: none;
                            border-radius: 0 11px 11px 0;
                            clip-path: polygon(0 0, 64% 0, 100% 17%, 42% 50%, 100% 83%, 64% 100%, 0 100%, 22% 82%, 78% 50%, 22% 18%);
                        }
                        .phone-nav-bracket .glyph {
                            position: absolute;
                            top: 50%;
                            left: 50%;
                            transform: translate(-50%, -50%);
                            font-size: 14px;
                            font-weight: 700;
                            text-shadow: 0 0 6px rgba(0, 224, 255, 0.35);
                            pointer-events: none;
                        }

                        /* FAB menu */
                        #canvas-fab {
                            position: fixed; bottom: 20px; right: 20px; z-index: 9990;
                        }
                        #canvas-fab .fab-btn {
                            width: 44px; height: 44px; border-radius: 50%;
                            background: rgba(0,224,255,0.06); border: 1px solid rgba(0,224,255,0.25);
                            color: #00e0ff; font-size: 22px; cursor: pointer;
                            display: flex; align-items: center; justify-content: center;
                            backdrop-filter: blur(8px); transition: all 0.2s;
                            box-shadow: 0 0 12px rgba(0,224,255,0.12);
                            font-family: 'Courier New', monospace;
                        }
                        #canvas-fab .fab-btn:hover { background: rgba(0,224,255,0.12); transform: scale(1.08); box-shadow: 0 0 20px rgba(0,224,255,0.25); }
                        #canvas-fab .fab-btn.open { transform: rotate(45deg); }
                        #canvas-fab .fab-menu {
                            display: none; position: absolute; bottom: 52px; right: 0;
                            background: rgba(10,10,18,0.97); border: 1px solid rgba(0,224,255,0.1);
                            border-radius: 8px; padding: 4px 0; min-width: 170px;
                            backdrop-filter: blur(12px); box-shadow: 0 4px 24px rgba(0,0,0,0.6);
                            font-family: 'Courier New', Menlo, monospace;
                        }
                        #canvas-fab .fab-menu.show { display: block; }
                        #canvas-fab .fab-menu button {
                            display: flex; align-items: center; gap: 8px; width: 100%;
                            padding: 8px 14px; border: none; background: none;
                            color: #8899a6; font-size: 12px; cursor: pointer; text-align: left;
                            text-transform: uppercase; letter-spacing: 0.02em;
                        }
                        #canvas-fab .fab-menu button:hover { background: rgba(0,224,255,0.06); color: #00e0ff; }
                        #canvas-fab .fab-menu button .fab-icon { width: 18px; text-align: center; flex-shrink: 0; }

                        /* Voice Chat Modal */
                        #voice-chat-modal {
                            display: none; position: fixed;
                            bottom: 74px; right: 20px;
                            width: 340px; height: 460px;
                            background: rgba(10,10,18,0.97);
                            border: 1px solid rgba(0,224,255,0.2);
                            border-radius: 12px; z-index: 9995;
                            box-shadow: 0 8px 32px rgba(0,0,0,0.6), 0 0 20px rgba(0,224,255,0.05);
                            backdrop-filter: blur(16px);
                            flex-direction: column;
                            font-family: 'Courier New', Menlo, monospace;
                        }
                        #voice-chat-modal.vcm-open { display: flex; }
                        .vcm-header {
                            display: flex; justify-content: space-between; align-items: center;
                            padding: 10px 14px; border-bottom: 1px solid rgba(255,255,255,0.07);
                            cursor: move; flex-shrink: 0;
                        }
                        .vcm-title { color: #00e0ff; font-size: 13px; font-weight: 600; letter-spacing: 0.04em; text-transform: uppercase; }
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
                        .vcm-msg.user { background: rgba(0,224,255,0.1); color: #66f0ff; align-self: flex-end; }
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
                        #vcmInput:focus { border-color: rgba(0,224,255,0.4); }
                        #vcmSend {
                            background: rgba(0,224,255,0.12); border: 1px solid rgba(0,224,255,0.3);
                            border-radius: 6px; color: #00e0ff; padding: 6px 12px;
                            cursor: pointer; font-size: 16px; transition: background 0.15s;
                        }
                        #vcmSend:hover { background: rgba(0,224,255,0.25); }

                        /* P2P Share Modal */
                        #share-modal {
                            display: none; position: fixed;
                            bottom: 74px; right: 20px;
                            width: 320px;
                            background: rgba(10,10,18,0.97);
                            border: 1px solid rgba(0,224,255,0.2);
                            border-radius: 12px; z-index: 9996;
                            box-shadow: 0 8px 40px rgba(0,0,0,0.7), 0 0 20px rgba(0,224,255,0.05);
                            backdrop-filter: blur(16px);
                            flex-direction: column;
                            font-family: 'Courier New', Menlo, monospace;
                        }
                        #share-modal.sm-open { display: flex; }
                        .sm-header {
                            display: flex; justify-content: space-between; align-items: center;
                            padding: 10px 14px; border-bottom: 1px solid rgba(255,255,255,0.07);
                            cursor: move; flex-shrink: 0;
                        }
                        .sm-title { color: #00e0ff; font-size: 13px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.04em; }
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
                            background: rgba(0,224,255,0.06);
                            border: 1px solid rgba(0,224,255,0.25);
                            border-radius: 8px; color: #00e0ff;
                            font-size: 28px; font-weight: 700; font-family: monospace;
                            display: flex; align-items: center; justify-content: center;
                            letter-spacing: 0;
                        }
                        .sm-code-inputs { display: flex; gap: 8px; }
                        .sm-ci {
                            width: 52px; height: 64px; text-align: center;
                            background: rgba(255,255,255,0.04);
                            border: 1px solid rgba(0,224,255,0.2);
                            border-radius: 8px; color: #66f0ff;
                            font-size: 26px; font-weight: 700; font-family: monospace;
                            outline: none; text-transform: uppercase; caret-color: #00e0ff;
                        }
                        .sm-ci:focus { border-color: rgba(0,224,255,0.6); background: rgba(0,224,255,0.04); }
                        .sm-link-row {
                            display: flex; gap: 6px; width: 100%; align-items: center;
                        }
                        .sm-link {
                            flex: 1; font-size: 10px; color: #555; overflow: hidden;
                            text-overflow: ellipsis; white-space: nowrap; user-select: text;
                        }
                        .sm-btn {
                            background: rgba(0,224,255,0.1); border: 1px solid rgba(0,224,255,0.25);
                            border-radius: 6px; color: #00e0ff; padding: 5px 12px;
                            font-size: 12px; cursor: pointer; flex-shrink: 0; transition: background 0.15s;
                        }
                        .sm-btn:hover { background: rgba(0,224,255,0.2); }
                        .sm-btn-primary {
                            background: rgba(0,224,255,0.15); border: 1px solid rgba(0,224,255,0.35);
                            border-radius: 8px; color: #66f0ff; padding: 9px 28px;
                            font-size: 14px; cursor: pointer; transition: background 0.15s; width: 100%;
                            text-transform: uppercase; letter-spacing: 0.04em;
                        }
                        .sm-btn-primary:hover { background: rgba(0,224,255,0.25); }
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

                        /* ── Desktop: scale phone frame to fit viewport ── */
                        #phone-frame {
                            transform-origin: top center;
                        }
                        .phone-nav-bracket { display: block; }


                        /* ── Mobile fullscreen ── */
                        @media (max-width: 768px) and (pointer: coarse) {
                            .canvas-header { display: none !important; }
                            #canvas-container {
                                height: 100vh !important; padding: 0 !important;
                                align-items: stretch !important;
                            }
                            #phone-frame {
                                width: 100% !important; border-radius: 0 !important;
                                border: none !important; box-shadow: none !important;
                                padding: 0 !important; background: #000 !important;
                                transform: none !important;
                            }
                            .phone-notch, .phone-home-bar { display: none !important; }
                            .phone-nav-bracket { display: none !important; }
                            #phone-viewport {
                                width: 100vw !important; height: 100vh !important;
                                border-radius: 0 !important;
                            }
                            #canvas-fab {
                                transition: opacity 0.3s;
                            }
                            #canvas-fab.mob-hidden { opacity: 0; pointer-events: none; }
                            .canvas-empty { height: 100vh; }



                            /* FAB mobile action items */
                            .fab-menu .fab-mob-only { display: flex !important; }
                            .fab-menu .fab-mob-divider {
                                height: 1px; background: rgba(255,255,255,0.08);
                                margin: 4px 0;
                            }

                            /* Voice chat modal — full width on mobile */
                            #voice-chat-modal {
                                left: 8px; right: 8px; bottom: 8px; width: auto;
                                max-height: 70vh;
                            }
                            #share-modal {
                                left: 8px; right: 8px; bottom: 8px; width: auto;
                            }
                        }

                        /* Hide mobile-only FAB items on desktop */
                        .fab-menu .fab-mob-only { display: none !important; }
                        @media (max-width: 768px) and (pointer: coarse) {
                            .fab-menu .fab-mob-only { display: flex !important; }
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
                        div #phoneGameLabel .phone-game-label { "untitled v—" }
                        div #phoneRuntimeLabel .phone-runtime-label { "wasm()" }
                        div .phone-notch {
                            div .speaker {}
                            div .camera {}
                        }
                        button #btnPrevBracket .phone-nav-bracket .left title="Previous game" aria-label="Previous game" { span .glyph { "<" } }
                        iframe #phone-viewport sandbox="allow-scripts allow-same-origin allow-forms" {}
                        button #btnNextBracket .phone-nav-bracket .right title="Next game" aria-label="Next game" { span .glyph { ">" } }
                        div .phone-home-bar {}
                    }
                    div #canvasLoading style="display:none;position:absolute;top:0;left:0;right:0;bottom:0;background:rgba(10,10,10,0.85);z-index:9999;align-items:center;justify-content:center;flex-direction:column;border-radius:24px" {
                        div style="font-size:32px;margin-bottom:12px;animation:spin 1.2s linear infinite" { "⚙️" }
                        div style="color:#00ff88;font:bold 14px monospace" { "Updating game..." }
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
                        button #fabSaveQuick {
                            span .fab-icon { "💾" }
                            span { "Save" }
                        }
                        button #fabVoice {
                            span .fab-icon { "🎤" }
                            span #fabVoiceLabel { "Start Voice" }
                        }
                        button #fabSplats {
                            span .fab-icon { "🔮" }
                            span { "Splat Viewer" }
                        }
                        button #fabShare {
                            span .fab-icon { "📤" }
                            span { "Share Project" }
                        }
                        button #fabFavorite {
                            span .fab-icon { "❤" }
                            span { "Like / Keep Internal" }
                        }
                        button #fabReceive {
                            span .fab-icon { "📥" }
                            span { "Receive Project" }
                        }
                        // Mobile-only: divider + header controls
                        div .fab-mob-divider .fab-mob-only {}
                        button #fabGameSelect .fab-mob-only {
                            span .fab-icon { "🎮" }
                            span { "Switch Game" }
                        }
                        button #fabSaveMob .fab-mob-only {
                            span .fab-icon { "💾" }
                            span { "Save" }
                        }
                        button #fabClearMob .fab-mob-only {
                            span .fab-icon { "🗑" }
                            span { "Clear" }
                        }
                        button #fabSourceMob .fab-mob-only {
                            span .fab-icon { "📄" }
                            span { "View Source" }
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
                        const SNAKE_GAME_HTML = `<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8"><title>Snake</title><style>*{margin:0;padding:0;box-sizing:border-box}body{width:390px;height:844px;overflow:hidden;background:#0a0a0a;color:#e0e0e0;font-family:system-ui,sans-serif;user-select:none}#hdr{display:flex;align-items:center;justify-content:space-between;padding:18px 20px 10px}#hdr .title{font-size:22px;font-weight:700}.accent{color:#00ff88}#score{font-size:22px;font-weight:700;color:#00ff88}#hs{font-size:11px;color:#555;margin-top:2px}#cw{display:flex;justify-content:center;padding:0 20px}canvas{border-radius:8px;border:1px solid #1a1a1a}#ctrl{display:flex;flex-direction:column;align-items:center;gap:4px;padding:14px}#ctrl .row{display:flex;gap:4px}.btn{width:68px;height:52px;background:rgba(255,255,255,.06);border:1px solid rgba(255,255,255,.1);border-radius:12px;font-size:20px;display:flex;align-items:center;justify-content:center;cursor:pointer;-webkit-tap-highlight-color:transparent;transition:background .1s}.btn:active,.btn.pressed{background:rgba(0,255,136,.15);border-color:#00ff88}#overlay{position:absolute;inset:0;display:flex;flex-direction:column;align-items:center;justify-content:center;background:rgba(10,10,10,.92);text-align:center;z-index:20}#overlay.off{display:none}#overlay h2{font-size:30px;font-weight:700;margin-bottom:8px}#overlay p{color:#666;font-size:13px;margin-bottom:24px}#overlay button{background:rgba(0,255,136,.15);border:1px solid #00ff88;color:#00ff88;padding:12px 36px;border-radius:10px;font-size:15px;cursor:pointer;font-family:inherit}</style></head><body><div id="hdr"><div><div class="title"><span class="accent">Snake</span></div><div id="hs">best: 0</div></div><div id="score">0</div></div><div id="cw"><canvas id="c" width="350" height="560"></canvas></div><div id="ctrl"><div class="row"><div class="btn" id="bU">\u25b2</div></div><div class="row"><div class="btn" id="bL">\u25c4</div><div class="btn" style="visibility:hidden">\u25bc</div><div class="btn" id="bR">\u25ba</div></div><div class="row"><div class="btn" id="bD">\u25bc</div></div></div><div id="overlay"><h2 id="otitle">\uD83D\uDC0D Snake</h2><p id="otext">Tap buttons or use arrow keys</p><button id="obtn">Start</button></div><script>const cv=document.getElementById('c'),ctx=cv.getContext('2d'),C=25,R=40,CW=cv.width/C,CH=cv.height/R;let sn,dir,nd,fd,sc,hs=0,running=false,iv;const seEl=document.getElementById('score'),hsEl=document.getElementById('hs'),ov=document.getElementById('overlay'),ot=document.getElementById('otitle'),op=document.getElementById('otext'),ob=document.getElementById('obtn');function rn(n){return Math.floor(Math.random()*n)}function spawnFood(){let p;do{p={x:rn(C),y:rn(R)}}while(sn.some(s=>s.x===p.x&&s.y===p.y));fd=p}function init(){const mx=Math.floor(C/2),my=Math.floor(R/2);sn=[{x:mx,y:my},{x:mx-1,y:my},{x:mx-2,y:my}];dir={x:1,y:0};nd={x:1,y:0};sc=0;seEl.textContent=0;spawnFood()}function step(){dir=nd;const h={(x:(sn[0].x+dir.x+C)%C),(y:(sn[0].y+dir.y+R)%R)};if(sn.some(s=>s.x===h.x&&s.y===h.y)){over();return}sn.unshift(h);if(h.x===fd.x&&h.y===fd.y){sc++;seEl.textContent=sc;if(sc>hs){hs=sc;hsEl.textContent='best: '+hs;if(window.traits&&window.traits.score)window.traits.score(hs)}spawnFood()}else{sn.pop()}draw()}function draw(){ctx.fillStyle='#0a0a0a';ctx.fillRect(0,0,cv.width,cv.height);ctx.strokeStyle='#111';ctx.lineWidth=.5;for(let i=0;i<=C;i++){ctx.beginPath();ctx.moveTo(i*CW,0);ctx.lineTo(i*CW,cv.height);ctx.stroke()}for(let j=0;j<=R;j++){ctx.beginPath();ctx.moveTo(0,j*CH);ctx.lineTo(cv.width,j*CH);ctx.stroke()}ctx.shadowColor='#ff4444';ctx.shadowBlur=10;ctx.fillStyle='#ff4444';ctx.beginPath();ctx.arc(fd.x*CW+CW/2,fd.y*CH+CH/2,Math.min(CW,CH)*.38,0,Math.PI*2);ctx.fill();ctx.shadowBlur=0;sn.forEach((s,i)=>{const t=i/sn.length;ctx.fillStyle=i===0?'#00ff88':\`hsl(\${145-t*30},\${80-t*20}%,\${50-t*15}%)\`;const p=i===0?1:2;if(ctx.roundRect){ctx.beginPath();ctx.roundRect(s.x*CW+p,s.y*CH+p,CW-p*2,CH-p*2,i===0?4:3);ctx.fill()}else{ctx.fillRect(s.x*CW+p,s.y*CH+p,CW-p*2,CH-p*2)}})}function over(){clearInterval(iv);running=false;ot.textContent='\uD83D\uDC80 Game Over';op.textContent='Score: '+sc;ob.textContent='Play Again';ov.classList.remove('off')}function start(){ov.classList.add('off');init();clearInterval(iv);running=true;iv=setInterval(step,120)}ob.addEventListener('click',start);const DM={ArrowUp:{x:0,y:-1},ArrowDown:{x:0,y:1},ArrowLeft:{x:-1,y:0},ArrowRight:{x:1,y:0},w:{x:0,y:-1},s:{x:0,y:1},a:{x:-1,y:0},d:{x:1,y:0}};function tryDir(d){if(running&&!(d.x===-dir.x&&d.y===-dir.y))nd=d}function onKey(e){if(!running){if(e.key==='Enter')start();return}const d=DM[e.key];if(d){tryDir(d);e.preventDefault()}}window.addEventListener('keydown',onKey);document.addEventListener('keydown',onKey);document.getElementById('bU').addEventListener('click',()=>tryDir({x:0,y:-1}));document.getElementById('bD').addEventListener('click',()=>tryDir({x:0,y:1}));document.getElementById('bL').addEventListener('click',()=>tryDir({x:-1,y:0}));document.getElementById('bR').addEventListener('click',()=>tryDir({x:1,y:0}));let tx=0,ty=0;cv.addEventListener('touchstart',e=>{tx=e.touches[0].clientX;ty=e.touches[0].clientY},{passive:true});cv.addEventListener('touchend',e=>{const dx=e.changedTouches[0].clientX-tx,dy=e.changedTouches[0].clientY-ty;if(!running)return;if(Math.abs(dx)>Math.abs(dy)){tryDir(dx>0?{x:1,y:0}:{x:-1,y:0})}else{tryDir(dy>0?{x:0,y:1}:{x:0,y:-1})}});draw();if(window.traits&&window.traits.score){var _sh=window.traits.score();var _shv=(typeof _sh==='object'&&_sh)?_sh.score:(_sh||0);if(_shv>hs){hs=_shv;hsEl.textContent='best: '+hs}}window.addEventListener('message',function(e){if(e.data&&e.data.type==='highscore-update'&&e.data.score>hs){hs=e.data.score;hsEl.textContent='best: '+(e.data.player?e.data.player+' ':'') +hs}});<\/script></body></html>`;
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

                        function writeGamesCollection(col) {
                            try {
                                const raw = localStorage.getItem('traits.pvfs') || '{}';
                                const files = JSON.parse(raw);
                                files['canvas/games.json'] = JSON.stringify(col || { active: null, games: {} });
                                localStorage.setItem('traits.pvfs', JSON.stringify(files));
                            } catch(_) {}
                        }

                        function nowIso() {
                            return new Date().toISOString();
                        }

                        function makeLocalId() {
                            return String(Date.now()) + Math.random().toString(36).slice(2, 6);
                        }

                        function createLocalGameFallback(name) {
                            try {
                                const raw = localStorage.getItem('traits.pvfs') || '{}';
                                const files = JSON.parse(raw);
                                const col = files['canvas/games.json']
                                    ? JSON.parse(files['canvas/games.json'])
                                    : { active: null, games: {} };
                                const id = makeLocalId();
                                const ts = nowIso();
                                col.games[id] = {
                                    name: name || 'untitled',
                                    content: '',
                                    scope: 'internal',
                                    _scope: 'internal',
                                    owner: 'local',
                                    created: ts,
                                    updated: ts
                                };
                                col.active = id;
                                files['canvas/games.json'] = JSON.stringify(col);
                                files['canvas/app.html'] = '';
                                localStorage.setItem('traits.pvfs', JSON.stringify(files));
                                return id;
                            } catch(_) { return ''; }
                        }

                        function dedupeLocalGames() {
                            try {
                                const raw = localStorage.getItem('traits.pvfs') || '{}';
                                const files = JSON.parse(raw);
                                const col = files['canvas/games.json']
                                    ? JSON.parse(files['canvas/games.json'])
                                    : { active: null, games: {} };
                                var byIdentity = {};
                                for (var id in col.games) {
                                    var g = col.games[id];
                                    var scope = (g.scope || g._scope || 'internal');
                                    var identity = '';
                                    if ((g._sync_owner || g.owner) && (g._sync_game_id || g.game_id)) {
                                        identity = 'relay|' + String(g._sync_owner || g.owner).trim().toLowerCase() + '|' + String(g._sync_game_id || g.game_id).trim().toLowerCase();
                                    } else if (scope === 'external' && g.owner && g.game_id) {
                                        identity = 'external|' + String(g.owner).trim().toLowerCase() + '|' + String(g.game_id).trim().toLowerCase();
                                    } else {
                                        // Keep local/internal games distinct even when names match.
                                        // Multiple "untitled" or "received" projects are valid.
                                        identity = 'local-id|' + id;
                                    }
                                    if (!byIdentity[identity]) byIdentity[identity] = [];
                                    byIdentity[identity].push(id);
                                }
                                var removed = 0;
                                for (var n in byIdentity) {
                                    var ids = byIdentity[n];
                                    if (ids.length <= 1) continue;
                                    // Sort preference: internal edits first, then newest updated,
                                    // then content length only as a final tiebreaker.
                                    ids.sort(function(a, b) {
                                        var ga = col.games[a] || {};
                                        var gb = col.games[b] || {};
                                        var sa = (ga.scope || ga._scope || 'internal');
                                        var sb = (gb.scope || gb._scope || 'internal');
                                        var ia = (sa === 'internal') ? 1 : 0;
                                        var ib = (sb === 'internal') ? 1 : 0;
                                        if (ib !== ia) return ib - ia;
                                        var ua = String(ga.updated || '');
                                        var ub = String(gb.updated || '');
                                        if (ub !== ua) return ub.localeCompare(ua);
                                        var la = (ga.content || '').length;
                                        var lb = (gb.content || '').length;
                                        if (lb !== la) return lb - la;
                                        return String(b).localeCompare(String(a));
                                    });
                                    var keep = ids[0];
                                    for (var i = 1; i < ids.length; i++) {
                                        var del = ids[i];
                                        if (col.active === del) col.active = keep;
                                        delete col.games[del];
                                        removed++;
                                    }
                                }

                                var syncedBySlug = {};
                                for (var sid in col.games) {
                                    if (!Object.prototype.hasOwnProperty.call(col.games, sid)) continue;
                                    var sg = col.games[sid] || {};
                                    var hasRelayIdentity = !!_relayIdentityOf(sg);
                                    var sscope = (sg.scope || sg._scope || 'internal');
                                    if (!hasRelayIdentity || sscope === 'external') continue;
                                    syncedBySlug[_slugifyGameId(sg._sync_game_id || sg.game_id || sg.name || sid)] = sid;
                                }
                                for (var lid in col.games) {
                                    if (!Object.prototype.hasOwnProperty.call(col.games, lid)) continue;
                                    var lg = col.games[lid] || {};
                                    if (_relayIdentityOf(lg)) continue;
                                    var lscope = (lg.scope || lg._scope || 'internal');
                                    if (lscope === 'external') continue;
                                    var lslug = _slugifyGameId(lg.game_id || lg.name || lid);
                                    var preferredId = syncedBySlug[lslug];
                                    if (preferredId && preferredId !== lid) {
                                        if (col.active === lid) col.active = preferredId;
                                        delete col.games[lid];
                                        removed++;
                                    }
                                }

                                if (removed > 0) {
                                    files['canvas/games.json'] = JSON.stringify(col);
                                    localStorage.setItem('traits.pvfs', JSON.stringify(files));
                                }
                                return removed;
                            } catch(_) { return 0; }
                        }

                        function runOneTimeHistoricalDedupe() {
                            try {
                                const FLAG = 'traits.env.GAMES_DEDUPE_V2';
                                if (localStorage.getItem(FLAG)) return 0;

                                const raw = localStorage.getItem('traits.pvfs') || '{}';
                                const files = JSON.parse(raw);
                                const col = files['canvas/games.json']
                                    ? JSON.parse(files['canvas/games.json'])
                                    : { active: null, games: {} };
                                const games = col.games || {};

                                var byName = {};
                                for (var id in games) {
                                    if (!games.hasOwnProperty(id)) continue;
                                    var g = games[id] || {};
                                    // Historical cleanup should only merge relay/external duplicates,
                                    // never local/internal projects that merely share a display name.
                                    var nk = '';
                                    if ((g._sync_owner || g.owner) && (g._sync_game_id || g.game_id)) {
                                        nk = 'relay|' + String(g._sync_owner || g.owner).trim().toLowerCase() + '|' + String(g._sync_game_id || g.game_id).trim().toLowerCase();
                                    } else if ((g.scope || g._scope) === 'external' && g.owner && g.game_id) {
                                        nk = 'external|' + String(g.owner).trim().toLowerCase() + '|' + String(g.game_id).trim().toLowerCase();
                                    } else if (g._sync_hash || g.checksum) {
                                        nk = 'hash|' + String(g._sync_hash || g.checksum).trim().toLowerCase();
                                    } else {
                                        nk = 'local-id|' + id;
                                    }
                                    if (!byName[nk]) byName[nk] = [];
                                    byName[nk].push(id);
                                }

                                var removed = 0;
                                for (var nk in byName) {
                                    var ids = byName[nk];
                                    if (!ids || ids.length <= 1) continue;

                                    // Prefer internal edits, then relay identity, then most recently updated,
                                    // then content length only as a last tiebreaker.
                                    ids.sort(function(a, b) {
                                        var ga = games[a] || {};
                                        var gb = games[b] || {};
                                        var sca = (ga.scope || ga._scope || 'internal');
                                        var scb = (gb.scope || gb._scope || 'internal');
                                        var ia = (sca === 'internal') ? 1 : 0;
                                        var ib = (scb === 'internal') ? 1 : 0;
                                        if (ib !== ia) return ib - ia;
                                        var sa = ((ga._sync_owner || ga.owner) && (ga._sync_game_id || ga.game_id)) ? 1 : 0;
                                        var sb = ((gb._sync_owner || gb.owner) && (gb._sync_game_id || gb.game_id)) ? 1 : 0;
                                        if (sb !== sa) return sb - sa;
                                        var ua = String(ga.updated || '');
                                        var ub = String(gb.updated || '');
                                        if (ub !== ua) return ub.localeCompare(ua);
                                        var la = (ga.content || '').length;
                                        var lb = (gb.content || '').length;
                                        if (lb !== la) return lb - la;
                                        return String(b).localeCompare(String(a));
                                    });

                                    var keep = ids[0];
                                    for (var i = 1; i < ids.length; i++) {
                                        var del = ids[i];
                                        if (col.active === del) col.active = keep;
                                        delete games[del];
                                        removed++;
                                    }
                                }

                                col.games = games;
                                files['canvas/games.json'] = JSON.stringify(col);
                                localStorage.setItem('traits.pvfs', JSON.stringify(files));
                                localStorage.setItem(FLAG, '1');
                                return removed;
                            } catch(_) { return 0; }
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
                                    version: g.version || '',
                                    scope: g.scope || g._scope || 'internal',
                                    length: (g.content || '').length,
                                    active: id === col.active,
                                    updated: g.updated || ''
                                });
                            }
                            list.sort((a, b) => (b.updated || '').localeCompare(a.updated || ''));
                            return list;
                        }

                        const gameSelect = document.getElementById('game-select');
                        const phoneGameLabel = document.getElementById('phoneGameLabel');

                        function renderActiveGameBadge() {
                            if (!phoneGameLabel) return;
                            const col = readGamesCollection();
                            const a = col.active ? (col.games || {})[col.active] : null;
                            const nm = (a && a.name) ? String(a.name) : 'untitled';
                            const ver = (a && a.version) ? String(a.version) : '—';
                            phoneGameLabel.textContent = nm + ' v' + ver;
                        }

                        function renderProjectBar() {
                            dedupeLocalGames();
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
                                const scopeTag = (g.scope === 'external') ? '🌐' : '🏠';
                                const vv = g.version ? (' v' + g.version) : '';
                                opt.textContent = scopeTag + ' ' + (g.name || 'untitled') + vv;
                                if (g.active) opt.selected = true;
                                sel.appendChild(opt);
                            });
                            renderActiveGameBadge();
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

                        function _authToken() {
                            try { return (localStorage.getItem('traits.secret.SLOB_USER_TOKEN') || '').trim(); }
                            catch(_) { return ''; }
                        }

                        function _slugifyGameId(s) {
                            return String(s || '')
                                .trim()
                                .toLowerCase()
                                .replace(/[^a-z0-9]+/g, '-')
                                .replace(/^-+|-+$/g, '') || 'untitled';
                        }

                        function _relayIdentityOf(g) {
                            const owner = String((g && (g._sync_owner || g.owner)) || '').trim().toLowerCase();
                            const gameId = String((g && (g._sync_game_id || g.game_id)) || '').trim().toLowerCase();
                            return (owner && gameId) ? (owner + '|' + gameId) : '';
                        }

                        async function _shortContentHash(text) {
                            const buf = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(String(text || '')));
                            const arr = new Uint8Array(buf);
                            let hex = '';
                            for (let i = 0; i < arr.length; i++) hex += arr[i].toString(16).padStart(2, '0');
                            return hex.slice(0, 16);
                        }

                        async function _mergeActiveGameWithRelayInternal(owner, gameId, relayInfo) {
                            try {
                                const col = readGamesCollection();
                                const activeId = col.active;
                                const active = activeId ? (col.games || {})[activeId] : null;
                                if (!active) return false;

                                const relayOwner = String(owner || relayInfo?.owner || active._sync_owner || active.owner || '').trim();
                                const relayGameId = String(gameId || relayInfo?.game_id || active._sync_game_id || active.game_id || _slugifyGameId(active.name || activeId)).trim();
                                if (!relayOwner || !relayGameId) return false;

                                active.scope = 'internal';
                                active._scope = 'internal';
                                active.owner = relayOwner;
                                active.game_id = relayGameId;
                                active._sync_owner = relayOwner;
                                active._sync_game_id = relayGameId;
                                active._sync_hash = relayInfo?.content_hash || relayInfo?.checksum || active._sync_hash || '';
                                active.checksum = relayInfo?.checksum || relayInfo?.content_hash || active.checksum || '';
                                active.version = relayInfo?.version || active.version || '';
                                active.updated = nowIso();

                                const activeIdentity = _relayIdentityOf(active);
                                const activeHash = await _shortContentHash(active.content || '');
                                for (const [id, other] of Object.entries(col.games || {})) {
                                    if (id === activeId || !other) continue;
                                    const sameIdentity = _relayIdentityOf(other) === activeIdentity;
                                    const otherScope = other.scope || other._scope || 'internal';
                                    let sameUnsyncedCopy = false;
                                    if (!sameIdentity && otherScope !== 'external' && !_relayIdentityOf(other)) {
                                        const sameSlug = _slugifyGameId(other.game_id || other.name || id) === relayGameId;
                                        if (sameSlug) {
                                            const otherHash = await _shortContentHash(other.content || '');
                                            sameUnsyncedCopy = otherHash === activeHash;
                                        }
                                    }
                                    if (sameIdentity || sameUnsyncedCopy) {
                                        delete col.games[id];
                                    }
                                }
                                col.games[activeId] = active;
                                col.active = activeId;
                                writeGamesCollection(col);
                                dedupeLocalGames();
                                renderProjectBar();
                                return true;
                            } catch(_) { return false; }
                        }

                        let __relayInternalSyncTimer = null;
                        let __lastRelayInternalSyncKey = '';
                        async function _syncActiveToRelayInternal(opts) {
                            opts = opts || {};
                            const immediate = !!opts.immediate;

                            const run = async function() {
                                try {
                                    const token = _authToken();
                                    if (!token) return false;

                                    const colNow = readGamesCollection();
                                    const activeId = colNow.active;
                                    const active = activeId ? (colNow.games || {})[activeId] : null;
                                    if (!active || !active.content) return false;

                                    const gameId = active._sync_game_id || active.game_id || _slugifyGameId(active.name || activeId);
                                    const syncKey = [activeId, gameId, active.name || '', active.updated || '', (active.content || '').length].join('|');
                                    if (syncKey === __lastRelayInternalSyncKey) return true;

                                    const resp = await fetch('https://relay.slob.games/sync/internal/game/' + encodeURIComponent(gameId), {
                                        method: 'PUT',
                                        headers: {
                                            'Content-Type': 'application/json',
                                            'Authorization': 'Bearer ' + token,
                                        },
                                        body: JSON.stringify({
                                            name: active.name || 'untitled',
                                            content: active.content,
                                            version: active.version || ''
                                        })
                                    });
                                    if (!resp.ok) return false;
                                    const data = await resp.json().catch(() => ({}));
                                    await _mergeActiveGameWithRelayInternal(data.owner, data.game_id || gameId, data);
                                    __lastRelayInternalSyncKey = syncKey;
                                    return true;
                                } catch(_) { return false; }
                            };

                            if (immediate) {
                                if (__relayInternalSyncTimer) clearTimeout(__relayInternalSyncTimer);
                                __relayInternalSyncTimer = null;
                                return await run();
                            }

                            if (__relayInternalSyncTimer) clearTimeout(__relayInternalSyncTimer);
                            __relayInternalSyncTimer = setTimeout(() => {
                                __relayInternalSyncTimer = null;
                                run().catch(() => {});
                            }, 900);
                            return true;
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
                                // Make saved games visible in Settings/Admin (relay-backed views).
                                // If not logged in, save still succeeds locally.
                                await _syncActiveToRelayInternal({ immediate: true });
                                renderActiveGameBadge();
                            }
                        }

                        document.getElementById('btnSave').addEventListener('click', saveProject);
                        window.addEventListener('traits-canvas-projects-changed', renderProjectBar);
                        runOneTimeHistoricalDedupe();
                        dedupeLocalGames();
                        renderProjectBar();

                        (async function installCanvasSdkHooks() {
                            try {
                                let tries = 0;
                                while (!window._traitsSDK && tries < 40) {
                                    await new Promise(r => setTimeout(r, 250));
                                    tries++;
                                }
                                const sdk = window._traitsSDK;
                                if (!sdk || sdk.__canvasHooksInstalled) return;
                                const origCall = sdk.call.bind(sdk);
                                sdk.call = async function(path, args, opts) {
                                    const result = await origCall(path, args, opts);
                                    try {
                                        const cleanPath = String(path || '').split('@')[0];
                                        if (cleanPath === 'sys.canvas' && result && result.ok) {
                                            const r = result.result || result || {};
                                            if (r.action === 'set' || r.action === 'append' || r.action === 'clear') {
                                                try {
                                                    const getRes = await origCall('sys.canvas', ['get']);
                                                    const content = getRes?.result?.content ?? getRes?.content ?? '';
                                                    window.dispatchEvent(new CustomEvent('traits-canvas-update', { detail: { content } }));
                                                } catch(_) {
                                                    window.dispatchEvent(new CustomEvent('traits-canvas-update', {}));
                                                }
                                            }
                                            if (r.canvas_project_action || r.action === 'new' || r.action === 'rename' || r.action === 'activate' || r.action === 'fork' || r.action === 'delete') {
                                                window.dispatchEvent(new CustomEvent('traits-canvas-project', { detail: r }));
                                                window.dispatchEvent(new CustomEvent('traits-canvas-projects-changed'));
                                            }
                                        }
                                    } catch(_) {}
                                    return result;
                                };
                                sdk.__canvasHooksInstalled = true;
                            } catch(_) {}
                        })();

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
                            // Don't intercept keys when user is typing in an input/textarea/select
                            var tag = (document.activeElement || {}).tagName;
                            if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return;
                            if (document.activeElement && document.activeElement.isContentEditable) return;
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
                            function _cropSpriteImage(img, opts) {
                                opts = opts || {};
                                var whiteThreshold = opts.whiteThreshold == null ? 245 : Number(opts.whiteThreshold);
                                var alphaThreshold = opts.alphaThreshold == null ? 8 : Number(opts.alphaThreshold);
                                var padding = opts.padding == null ? 0 : Math.max(0, Number(opts.padding) || 0);
                                var preserveWhite = !!opts.preserveWhite;
                                var w = img.naturalWidth || img.width || 0;
                                var h = img.naturalHeight || img.height || 0;
                                if (!w || !h) return img;
                                var srcCanvas = document.createElement('canvas');
                                srcCanvas.width = w;
                                srcCanvas.height = h;
                                var srcCtx = srcCanvas.getContext('2d', { willReadFrequently: true });
                                if (!srcCtx) return img;
                                srcCtx.drawImage(img, 0, 0);
                                var imageData = srcCtx.getImageData(0, 0, w, h);
                                var data = imageData.data;
                                var minX = w, minY = h, maxX = -1, maxY = -1;
                                for (var y = 0; y < h; y++) {
                                    for (var x = 0; x < w; x++) {
                                        var i = (y * w + x) * 4;
                                        var r = data[i];
                                        var g = data[i + 1];
                                        var b = data[i + 2];
                                        var a = data[i + 3];
                                        var isWhiteBg = !preserveWhite && a > alphaThreshold && r >= whiteThreshold && g >= whiteThreshold && b >= whiteThreshold;
                                        var isBg = a <= alphaThreshold || isWhiteBg;
                                        if (isWhiteBg) data[i + 3] = 0;
                                        if (!isBg) {
                                            if (x < minX) minX = x;
                                            if (y < minY) minY = y;
                                            if (x > maxX) maxX = x;
                                            if (y > maxY) maxY = y;
                                        }
                                    }
                                }
                                if (maxX < minX || maxY < minY) return img;
                                srcCtx.putImageData(imageData, 0, 0);
                                minX = Math.max(0, minX - padding);
                                minY = Math.max(0, minY - padding);
                                maxX = Math.min(w - 1, maxX + padding);
                                maxY = Math.min(h - 1, maxY + padding);
                                var outW = maxX - minX + 1;
                                var outH = maxY - minY + 1;
                                var outCanvas = document.createElement('canvas');
                                outCanvas.width = outW;
                                outCanvas.height = outH;
                                var outCtx = outCanvas.getContext('2d');
                                if (!outCtx) return img;
                                outCtx.drawImage(srcCanvas, minX, minY, outW, outH, 0, 0, outW, outH);
                                return outCanvas;
                            }
                            function _loadVFSImage(path, opts) {
                                return (sdk() && sdk().call('sys.vfs', ['read', path])).then(function(r) {
                                    var src = (r && (r.content || (r.result && r.result.content))) || r;
                                    return new Promise(function(resolve, reject) {
                                        var img = new Image();
                                        img.onload = function() {
                                            try { resolve(_cropSpriteImage(img, opts)); }
                                            catch (_) { resolve(img); }
                                        };
                                        img.onerror = reject;
                                        img.src = src || '';
                                    });
                                });
                            }
                            window.traits = {
                                call:   function(p,a)   { return sdk() && sdk().call(p, a||[]); },
                                list:   function(ns)    { return sdk() && sdk().call('sys.list', ns?[ns]:[]); },
                                info:   function(p)     { return sdk() && sdk().call('sys.info', [p]); },
                                echo:   function(t)     { return sdk() && sdk().call('sys.echo', [t]); },
                                canvas: function(a,c)   { return sdk() && sdk().call('sys.canvas', c!==undefined?[a,c]:[a]); },
                                loadVFSImage: function(path, opts) { return _loadVFSImage(path, opts); },
                                cropSpriteImage: function(img, opts) { return _cropSpriteImage(img, opts); },
                                audio:  function(a)     {
                                    var r = Array.prototype.slice.call(arguments, 1);
                                    return sdk() && sdk().call('sys.audio', [a].concat(r));
                                },
                                onPause: null,
                                onResume: null,
                                score: function(val, player) {
                                    if (val !== undefined) {
                                        var n = Math.floor(Number(val));
                                        var p = (typeof player === 'string') ? player.slice(0, 50) : '';
                                        if (n >= 0 && isFinite(n)) {
                                            window.parent.postMessage({type:'canvas-score', score:n, player:p}, '*');
                                        }
                                    }
                                    var hs = window.parent.__highScores;
                                    var hash = window.parent.__activeGameHash;
                                    if (hs && hash && hs[hash]) {
                                        return hs[hash];
                                    }
                                    return {score: 0, player: ''};
                                },
                            };
                            if (!window.loadVFSImage) {
                                window.loadVFSImage = function(path, opts) { return window.traits.loadVFSImage(path, opts); };
                            }
                            var _qs = document.querySelector.bind(document);
                            document.querySelector = function(s) {
                                return _qs(s.replace(/^#phone-viewport\s+/,'').replace(/^#canvas-container\s+/,''));
                            };
                            // ── Pause engine: freeze rAF + timers + audio instantly ──
                            var _paused = false;
                            var _origRAF = window.requestAnimationFrame;
                            var _origCAF = window.cancelAnimationFrame;
                            var _origST = window.setTimeout;
                            var _origCT = window.clearTimeout;
                            var _origSI = window.setInterval;
                            var _origCI = window.clearInterval;
                            var _rafQueue = [];
                            var _timerQueue = [];
                            var _activeIntervals = {}; // id -> {cb, ms, realId}
                            var _nextFakeId = 900000;
                            // Track AudioContext instances for suspension
                            var _audioContexts = [];
                            var _OrigAudioCtx = window.AudioContext || window.webkitAudioContext;
                            if (_OrigAudioCtx) {
                                var PatchedAudioCtx = function() {
                                    var ctx = new _OrigAudioCtx();
                                    _audioContexts.push(ctx);
                                    return ctx;
                                };
                                PatchedAudioCtx.prototype = _OrigAudioCtx.prototype;
                                window.AudioContext = PatchedAudioCtx;
                                if (window.webkitAudioContext) window.webkitAudioContext = PatchedAudioCtx;
                            }
                            // rAF: wrap callback so already-scheduled frames freeze instantly
                            window.requestAnimationFrame = function(cb) {
                                if (_paused) { var id = _nextFakeId++; _rafQueue.push({id:id,cb:cb}); return id; }
                                return _origRAF.call(window, function(ts) {
                                    if (_paused) { _rafQueue.push({id:0,cb:cb}); return; }
                                    cb(ts);
                                });
                            };
                            window.cancelAnimationFrame = function(id) {
                                _rafQueue = _rafQueue.filter(function(e){ return e.id !== id; });
                                _origCAF.call(window, id);
                            };
                            // setTimeout: wrap callback
                            window.setTimeout = function(cb, ms) {
                                if (_paused) { var id = _nextFakeId++; _timerQueue.push({id:id,cb:cb,ms:ms,type:'timeout'}); return id; }
                                return _origST.call(window, function() {
                                    if (_paused) { _timerQueue.push({id:0,cb:cb,ms:ms,type:'timeout'}); return; }
                                    if (typeof cb === 'function') cb();
                                    else if (typeof cb === 'string') eval(cb);
                                }, ms);
                            };
                            window.clearTimeout = function(id) {
                                _timerQueue = _timerQueue.filter(function(e){ return e.id !== id; });
                                _origCT.call(window, id);
                            };
                            // setInterval: track all active intervals so we can clear them on pause
                            window.setInterval = function(cb, ms) {
                                if (_paused) { var id = _nextFakeId++; _timerQueue.push({id:id,cb:cb,ms:ms,type:'interval'}); return id; }
                                var realId = _origSI.call(window, function() {
                                    if (_paused) return; // skip silently; interval stays alive to be cleared in _doPause
                                    if (typeof cb === 'function') cb();
                                }, ms);
                                _activeIntervals[realId] = {cb:cb, ms:ms, realId:realId};
                                return realId;
                            };
                            window.clearInterval = function(id) {
                                _timerQueue = _timerQueue.filter(function(e){ return e.id !== id; });
                                delete _activeIntervals[id];
                                _origCI.call(window, id);
                            };
                            function _doPause() {
                                if (_paused) return;
                                _paused = true;
                                // Clear all active intervals and stash them for resume
                                var keys = Object.keys(_activeIntervals);
                                for (var i = 0; i < keys.length; i++) {
                                    var info = _activeIntervals[keys[i]];
                                    _origCI.call(window, info.realId);
                                    _timerQueue.push({id:0, cb:info.cb, ms:info.ms, type:'interval'});
                                }
                                _activeIntervals = {};
                                // Suspend all AudioContexts
                                _audioContexts.forEach(function(ctx){ try { ctx.suspend(); } catch(_){} });
                                try { if (typeof window.traits.onPause === 'function') window.traits.onPause(); } catch(_e){}
                            }
                            function _doResume() {
                                if (!_paused) return;
                                _paused = false;
                                try { if (typeof window.traits.onResume === 'function') window.traits.onResume(); } catch(_e){}
                                // Resume AudioContexts
                                _audioContexts.forEach(function(ctx){ try { ctx.resume(); } catch(_){} });
                                // Flush queued rAFs
                                var rafs = _rafQueue.slice(); _rafQueue = [];
                                rafs.forEach(function(e){ _origRAF.call(window, e.cb); });
                                // Flush queued timers
                                var timers = _timerQueue.slice(); _timerQueue = [];
                                timers.forEach(function(e){
                                    if (e.type === 'interval') {
                                        var rid = _origSI.call(window, function(){
                                            if (_paused) return;
                                            if (typeof e.cb === 'function') e.cb();
                                        }, e.ms);
                                        _activeIntervals[rid] = {cb:e.cb, ms:e.ms, realId:rid};
                                    }
                                    else { _origST.call(window, e.cb, e.ms); }
                                });
                            }
                            window.addEventListener('message', function(evt) {
                                if (!evt.data) return;
                                if (evt.data.type === 'canvas-pause') _doPause();
                                else if (evt.data.type === 'canvas-resume') _doResume();
                            });
                            // Capture console output and forward to parent for voice agent context
                            ['log','warn','error'].forEach(function(level){
                                var orig = console[level].bind(console);
                                console[level] = function(){
                                    orig.apply(console, arguments);
                                    try {
                                        var msg = Array.prototype.slice.call(arguments).map(function(a){
                                            return typeof a === 'object' ? JSON.stringify(a) : String(a);
                                        }).join(' ');
                                        window.parent.postMessage({type:'canvas-console', level:level, message:msg}, '*');
                                    } catch(_){}
                                };
                            });
                            // Capture uncaught errors
                            window.addEventListener('error', function(e){
                                try {
                                    var msg = (e.message||'') + (e.filename ? ' at '+e.filename+':'+e.lineno : '');
                                    window.parent.postMessage({type:'canvas-console', level:'error', message:msg}, '*');
                                } catch(_){}
                            });
                            // Two-finger contact: notify parent immediately (parent handles gestures)
                            document.addEventListener('touchstart', function(e){
                                if (e.touches.length >= 2) {
                                    e.preventDefault();
                                    var t0 = e.touches[0], t1 = e.touches[1];
                                    window.parent.postMessage({type:'canvas-two-finger-start', x:(t0.clientX+t1.clientX)/2, y:(t0.clientY+t1.clientY)/2}, '*');
                                }
                            }, {passive:false});
                            document.addEventListener('touchmove', function(e){
                                if (e.touches.length >= 2) {
                                    e.preventDefault();
                                    var t0 = e.touches[0], t1 = e.touches[1];
                                    window.parent.postMessage({type:'canvas-two-finger-move', x:(t0.clientX+t1.clientX)/2, y:(t0.clientY+t1.clientY)/2}, '*');
                                }
                            }, {passive:false});
                            document.addEventListener('touchend', function(e){
                                if (e.touches.length === 0) {
                                    window.parent.postMessage({type:'canvas-two-finger-end'}, '*');
                                }
                            }, {passive:true});
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
                            _updateActiveGameHash(content);
                            empty.style.display = 'none';
                            phoneFrame.classList.add('visible');
                            // Hide loading overlay when content renders
                            const _lo = document.getElementById('canvasLoading');
                            if (_lo) _lo.style.display = 'none';
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
                            // Clear game logs on new content load
                            if (window.__canvasGameLogs) window.__canvasGameLogs.length = 0;
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

                        let __lastPersistedContent = '';

                        function _extractTitleFromContent(content) {
                            const m = String(content || '').match(/<title[^>]*>([^<]+)<\/title>/i);
                            return (m && m[1] ? m[1].trim() : '');
                        }

                        function _isGenericGameName(name) {
                            const n = String(name || '').trim().toLowerCase();
                            return !n || n === 'untitled' || n === 'received';
                        }

                        function _uniqueGameName(base, col, activeId) {
                            const root = String(base || 'new game').trim() || 'new game';
                            const taken = new Set();
                            for (const [id, g] of Object.entries((col && col.games) || {})) {
                                if (id === activeId) continue;
                                taken.add(String((g && g.name) || '').trim().toLowerCase());
                            }
                            if (!taken.has(root.toLowerCase())) return root;
                            for (let i = 2; i <= 9999; i++) {
                                const candidate = root + ' ' + i;
                                if (!taken.has(candidate.toLowerCase())) return candidate;
                            }
                            return root + ' ' + Date.now();
                        }

                        async function _autoNameActiveGame(content) {
                            try {
                                const sdk = window._traitsSDK;
                                if (!sdk) return;
                                const col = readGamesCollection();
                                const activeId = col.active;
                                const active = activeId ? (col.games || {})[activeId] : null;
                                if (!active) return;
                                if (!_isGenericGameName(active.name)) return;
                                const title = _extractTitleFromContent(content);
                                const base = title || 'new game ' + new Date().toLocaleTimeString();
                                const nextName = _uniqueGameName(base, col, activeId);
                                await sdk.call('sys.canvas', ['rename', nextName]);
                            } catch (_) {}
                        }

                        async function persistActiveContent(content) {
                            const text = String(content || '');
                            if (!text || text === __lastPersistedContent) return;
                            try {
                                const sdk = window._traitsSDK;
                                if (!sdk) return;
                                __lastPersistedContent = text;
                                await sdk.call('sys.canvas', ['set', text]);
                                await _autoNameActiveGame(text);
                                _syncActiveToRelayInternal({ immediate: false }).catch(() => {});
                            } catch(_) {}
                        }

                        async function autosaveAfterRefresh() {
                            try {
                                const sdk = window._traitsSDK;
                                if (!sdk) return;
                                const col = readGamesCollection();
                                const activeId = col.active;
                                const activeGame = activeId ? (col.games || {})[activeId] : null;
                                const activeContent = (activeGame && activeGame.content) ? String(activeGame.content) : '';
                                const fallbackContent = String(readCanvasFromStorage() || '');
                                const content = activeContent || fallbackContent;
                                if (!content) return;

                                // If refresh recovered orphaned content with no active game, create a new one first.
                                if (!activeGame) {
                                    const title = _extractTitleFromContent(content) || ('new game ' + new Date().toLocaleTimeString());
                                    const name = _uniqueGameName(title, col, null);
                                    await sdk.call('sys.canvas', ['new', name]);
                                }

                                await persistActiveContent(content);
                            } catch (_) {}
                        }

                        // Listen for live updates from voice/SDK
                        window.addEventListener('traits-canvas-update', (e) => {
                            const content = e.detail?.content;
                            if (content !== undefined) {
                                __lastContent = content;
                                renderCanvas(content);
                                renderProjectBar();
                                // Safety net: ensure updates are persisted continuously.
                                persistActiveContent(content);
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
                                        _syncActiveToRelayInternal({ immediate: false }).catch(() => {});
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
                                await autosaveAfterRefresh();
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

                        // ── Game console log ring buffer (last 50 entries) ──
                        const __gameLogs = [];
                        const __GAME_LOG_MAX = 50;
                        window.__highScores = {};    // game_hash → best score (synced via relay)
                        window.__activeGameHash = null;
                        // Compute content hash for active game (same algo as sync module)
                        async function _updateActiveGameHash(content) {
                            if (!content) { window.__activeGameHash = null; return; }
                            try {
                                const buf = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(content));
                                const arr = new Uint8Array(buf);
                                let hex = '';
                                for (let i = 0; i < arr.length; i++) hex += arr[i].toString(16).padStart(2, '0');
                                window.__activeGameHash = hex.slice(0, 16);
                                // Immediately push any already-received relay score for this game to the iframe
                                const _relayScore = window.__highScores[window.__activeGameHash];
                                if (_relayScore && _relayScore.score > 0) {
                                    try {
                                        const _vp = document.getElementById('phone-viewport');
                                        if (_vp && _vp.contentWindow) _vp.contentWindow.postMessage({type:'highscore-update', score:_relayScore.score, player:_relayScore.player||''}, '*');
                                    } catch(_) {}
                                }
                            } catch(_) { window.__activeGameHash = null; }
                        }
                        window.addEventListener('message', (e) => {
                            if (e.data?.type === 'canvas-console') {
                                const entry = '[' + e.data.level.toUpperCase() + '] ' + (e.data.message || '').slice(0, 300);
                                __gameLogs.push(entry);
                                if (__gameLogs.length > __GAME_LOG_MAX) __gameLogs.shift();
                            }
                            if (e.data?.type === 'canvas-score') {
                                // Game reported a score — forward to sync WebSocket
                                const hash = window.__activeGameHash;
                                if (!hash) return;
                                const score = Math.floor(Number(e.data.score));
                                if (!Number.isFinite(score) || score < 0) return;
                                const player = (typeof e.data.player === 'string') ? e.data.player.slice(0, 50) : '';
                                // Update local high score immediately
                                const current = window.__highScores[hash] || {score:0, player:''};
                                if (score > current.score || (score === current.score && player && !current.player)) {
                                    window.__highScores[hash] = {score, player: player || current.player};
                                }
                                // Forward to relay via the sync WebSocket
                                if (window.__syncWs && window.__syncWs.readyState === WebSocket.OPEN) {
                                    window.__syncWs.send(JSON.stringify({
                                        type: 'score', game_hash: hash, score: score, player: player || current.player
                                    }));
                                }
                            }
                        });
                        // Expose for the SDK canvas agent to read
                        window.__canvasGameLogs = __gameLogs;

                        // Track canvas agent status — suppress poll while agent is running
                        let __canvasAgentRunning = false;
                        window.addEventListener('traits-canvas-agent-status', (e) => {
                            __canvasAgentRunning = !!e.detail?.running;
                            const overlay = document.getElementById('canvasLoading');
                            if (overlay) overlay.style.display = __canvasAgentRunning ? 'flex' : 'none';
                        });

                        // Poll VFS for agent writes (1s backup for missed events)
                        const _pollId = setInterval(() => {
                            try {
                                if (sourceMode || __canvasAgentRunning) return;
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
                        const fabSaveQuick = document.getElementById('fabSaveQuick');
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
                            if (sdk) {
                                await sdk.call('sys.canvas', ['new']);
                                await sdk.call('sys.canvas', ['clear']);
                            } else {
                                // Early-click fallback before SDK init: still create/store a new game.
                                createLocalGameFallback('untitled');
                            }
                            __lastContent = '';
                            __lastPersistedContent = '';
                            renderCanvas('');
                            renderProjectBar();
                        });

                        if (fabSaveQuick) {
                            fabSaveQuick.addEventListener('click', () => {
                                fabMenu.classList.remove('show');
                                fabToggle.classList.remove('open');
                                document.getElementById('btnSave')?.click();
                            });
                        }

                        // Global save hotkey fallback (Cmd+S / Ctrl+S)
                        document.addEventListener('keydown', (e) => {
                            if (!(e.metaKey || e.ctrlKey) || String(e.key || '').toLowerCase() !== 's') return;
                            const tag = (e.target && e.target.tagName) ? e.target.tagName.toLowerCase() : '';
                            if (tag === 'input' || tag === 'textarea' || (e.target && e.target.isContentEditable)) return;
                            e.preventDefault();
                            document.getElementById('btnSave')?.click();
                        });

                        // Voice button — toggles start/stop
                        let _voiceActive = false;
                        const fabVoiceBtn = document.getElementById('fabVoice');
                        const fabVoiceLabel = document.getElementById('fabVoiceLabel');
                        function updateVoiceBtn(active) {
                            _voiceActive = active;
                            fabVoiceLabel.textContent = active ? 'Stop Voice' : 'Start Voice';
                            fabVoiceBtn.querySelector('.fab-icon').textContent = active ? '⏹' : '🎤';
                        }
                        fabVoiceBtn.addEventListener('click', () => {
                            fabMenu.classList.remove('show');
                            fabToggle.classList.remove('open');
                            const action = _voiceActive ? 'stop' : 'start';
                            window.dispatchEvent(new CustomEvent('traits-voice-control', { detail: { voice_control_action: action } }));
                            updateVoiceBtn(!_voiceActive);
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
                        document.getElementById('fabFavorite').addEventListener('click', async () => {
                            fabMenu.classList.remove('show'); fabToggle.classList.remove('open');
                            try {
                                const sdk = window._traitsSDK;
                                if (!sdk) return;
                                const res = await sdk.call('sys.canvas', ['fork']);
                                const r = res?.result || res || {};
                                // Best-effort: also fork the source external game into relay internal room for logged-in users.
                                try {
                                    const token = (localStorage.getItem('traits.secret.SLOB_USER_TOKEN') || '').trim();
                                    const col = readGamesCollection();
                                    const active = col.active && col.games ? col.games[col.active] : null;
                                    const sourceHash = active ? (active._sync_hash || active.checksum || '') : '';
                                    if (token && sourceHash) {
                                        await fetch('https://relay.slob.games/sync/internal/fork', {
                                            method: 'POST',
                                            headers: {
                                                'Content-Type': 'application/json',
                                                'Authorization': 'Bearer ' + token,
                                            },
                                            body: JSON.stringify({ source_hash: sourceHash }),
                                        });
                                    }
                                } catch(_) {}
                                if (r.forked) {
                                    renderProjectBar();
                                    alert('Saved to your internal games: ' + (r.name || 'untitled'));
                                } else {
                                    alert('Already internal. This game is already yours.');
                                }
                            } catch (e) {
                                alert('Could not save internally: ' + (e?.message || e));
                            }
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
                                    const _title = (String(content).match(/<title[^>]*>([^<]+)<\/title>/i) || [])[1];
                                    const _name = (_title && _title.trim())
                                        ? _title.trim().slice(0, 80)
                                        : ('received ' + new Date().toLocaleTimeString());
                                    // Create a new game for the received project
                                    await sdk.call('sys.canvas', ['new', _name]);
                                    await sdk.call('sys.canvas', ['set', content]);
                                    await _syncActiveToRelayInternal({ immediate: true });
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
                            if (!vcModal.classList.contains('vcm-open')) vcmOpen();
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

                        // Voice events → chat log + sync button state
                        window.addEventListener('voice-event', (e) => {
                            const d = e.detail;
                            switch (d.type) {
                                case 'started':
                                    updateVoiceBtn(true);
                                    vcmAppend('system', '🎤 Voice session started');
                                    break;
                                case 'stopped':
                                case 'disconnected':
                                    updateVoiceBtn(false);
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

                        // ── Mobile fullscreen: auto-hide chrome, two-finger carousel ──
                        // Build sorted list of public/external games for carousel rotation
                        function _publicGamesList() {
                            const col = readGamesCollection();
                            const list = [];
                            for (const [id, g] of Object.entries(col.games || {})) {
                                if ((g.scope || g._scope || 'internal') !== 'external') continue;
                                list.push({ id, name: g.name || 'untitled' });
                            }
                            list.sort((a, b) => a.name.localeCompare(b.name, undefined, { sensitivity: 'base' }));
                            return { list, activeId: col.active };
                        }

                        const isMobile = window.matchMedia('(max-width:768px) and (pointer:coarse)').matches;
                        if (isMobile) {
                            const shellNav = document.getElementById('shell-nav');
                            const fab = document.getElementById('canvas-fab');
                            let hideTimer = null;
                            let chromeVisible = true;
                            let gamePaused = false;
                            const HIDE_DELAY = 3000;

                            function showChrome() {
                                chromeVisible = true;
                                if (shellNav) { shellNav.style.transition = 'opacity 0.3s, transform 0.3s'; shellNav.style.opacity = '1'; shellNav.style.transform = 'translateY(0)'; shellNav.style.pointerEvents = ''; }
                                if (fab) fab.classList.remove('mob-hidden');
                                clearTimeout(hideTimer);
                                if (!gamePaused) hideTimer = setTimeout(hideChrome, HIDE_DELAY);
                            }

                            function hideChrome() {
                                const fabMenu = document.getElementById('fabMenu');
                                const vcm = document.getElementById('voice-chat-modal');
                                const sm = document.getElementById('share-modal');
                                if (fabMenu?.classList.contains('show')) return;
                                if (vcm?.classList.contains('vcm-open')) return;
                                if (sm?.classList.contains('sm-open')) return;

                                chromeVisible = false;
                                if (shellNav) { shellNav.style.opacity = '0'; shellNav.style.transform = 'translateY(-100%)'; shellNav.style.pointerEvents = 'none'; }
                                if (fab) fab.classList.add('mob-hidden');
                            }

                            function pauseGame() {
                                if (gamePaused) return;
                                gamePaused = true;
                                const vp = document.getElementById('phone-viewport');
                                if (vp) vp.contentWindow?.postMessage({type:'canvas-pause'}, '*');
                            }

                            function resumeGame() {
                                if (!gamePaused) return;
                                gamePaused = false;
                                const vp = document.getElementById('phone-viewport');
                                if (vp) vp.contentWindow?.postMessage({type:'canvas-resume'}, '*');
                            }

                            function switchGame(direction) {
                                const { list, activeId } = _publicGamesList();
                                if (list.length < 1) return;
                                let idx = list.findIndex(g => g.id === activeId);
                                if (idx < 0) idx = 0; // current game not in public list — start at first
                                const next = direction === 'next'
                                    ? (idx + 1) % list.length
                                    : (idx - 1 + list.length) % list.length;
                                activateGame(list[next].id);
                            }

                            function getGameLabel(direction) {
                                const { list, activeId } = _publicGamesList();
                                if (list.length < 1) return '';
                                let idx = list.findIndex(g => g.id === activeId);
                                if (idx < 0) idx = 0;
                                const target = direction === 'next'
                                    ? (idx + 1) % list.length
                                    : (idx - 1 + list.length) % list.length;
                                return list[target].name;
                            }

                            // ── Carousel gesture state ──
                            const SWIPE_THRESHOLD = 0.15; // fraction of screen width
                            let _gesture = null; // { startX, startY, tracking, swiping }
                            let _carouselPrevLabel = null;
                            let _carouselNextLabel = null;

                            function createCarouselLabels() {
                                if (_carouselPrevLabel) return;
                                _carouselPrevLabel = document.createElement('div');
                                _carouselPrevLabel.style.cssText = 'position:fixed;top:50%;left:0;transform:translateY(-50%) translateX(-100%);' +
                                    'color:#fff;font-size:16px;font-weight:600;padding:12px 20px;background:rgba(0,0,0,0.7);' +
                                    'border-radius:0 12px 12px 0;z-index:9999;pointer-events:none;transition:none;white-space:nowrap;';
                                document.body.appendChild(_carouselPrevLabel);
                                _carouselNextLabel = document.createElement('div');
                                _carouselNextLabel.style.cssText = 'position:fixed;top:50%;right:0;transform:translateY(-50%) translateX(100%);' +
                                    'color:#fff;font-size:16px;font-weight:600;padding:12px 20px;background:rgba(0,0,0,0.7);' +
                                    'border-radius:12px 0 0 12px;z-index:9999;pointer-events:none;transition:none;white-space:nowrap;';
                                document.body.appendChild(_carouselNextLabel);
                            }

                            function beginGesture(x, y) {
                                if (_gesture) return; // already tracking
                                _gesture = { startX: x, startY: y, tracking: true, swiping: false, wasPaused: gamePaused };
                                pauseGame();
                                showChrome();
                                clearTimeout(hideTimer);
                                hideTimer = null;
                                // Prepare carousel labels
                                createCarouselLabels();
                                _carouselPrevLabel.textContent = '\u25C0 ' + (getGameLabel('prev') || 'prev');
                                _carouselNextLabel.textContent = (getGameLabel('next') || 'next') + ' \u25B6';
                                const vp = document.getElementById('phone-viewport');
                                if (vp) vp.style.transition = 'none';
                            }

                            function moveGesture(x) {
                                if (!_gesture || !_gesture.tracking) return;
                                const dx = x - _gesture.startX;
                                const sw = window.innerWidth;
                                const pct = dx / sw;
                                if (!_gesture.swiping && Math.abs(dx) > 10) _gesture.swiping = true;
                                const vp = document.getElementById('phone-viewport');
                                if (vp) vp.style.transform = 'translateX(' + dx + 'px)';
                                // Show peeking labels
                                if (_carouselPrevLabel) {
                                    const show = pct > 0.05;
                                    _carouselPrevLabel.style.transform = 'translateY(-50%) translateX(' + (show ? '0' : '-100%') + ')';
                                    _carouselPrevLabel.style.opacity = show ? Math.min(1, pct * 3) : 0;
                                }
                                if (_carouselNextLabel) {
                                    const show = pct < -0.05;
                                    _carouselNextLabel.style.transform = 'translateY(-50%) translateX(' + (show ? '0' : '100%') + ')';
                                    _carouselNextLabel.style.opacity = show ? Math.min(1, Math.abs(pct) * 3) : 0;
                                }
                            }

                            function endGesture() {
                                if (!_gesture) return;
                                const g = _gesture;
                                _gesture = null;
                                const vp = document.getElementById('phone-viewport');

                                // Hide labels
                                if (_carouselPrevLabel) { _carouselPrevLabel.style.transform = 'translateY(-50%) translateX(-100%)'; _carouselPrevLabel.style.opacity = 0; }
                                if (_carouselNextLabel) { _carouselNextLabel.style.transform = 'translateY(-50%) translateX(100%)'; _carouselNextLabel.style.opacity = 0; }

                                if (!g.swiping) {
                                    // It was a two-finger tap (no significant drag)
                                    if (vp) { vp.style.transition = ''; vp.style.transform = ''; }
                                    if (g.wasPaused) {
                                        // Was already paused → unpause + hide chrome
                                        resumeGame();
                                        hideTimer = setTimeout(hideChrome, HIDE_DELAY);
                                    }
                                    // If wasn't paused, beginGesture already paused it — stay paused
                                    return;
                                }

                                // Calculate final displacement
                                const currentX = parseFloat(vp?.style.transform?.match(/translateX\(([\-\d.]+)px\)/)?.[1] || 0);
                                const sw = window.innerWidth;
                                const pct = currentX / sw;

                                if (Math.abs(pct) >= SWIPE_THRESHOLD) {
                                    // Animate off-screen then switch
                                    const direction = pct > 0 ? 'prev' : 'next';
                                    if (vp) {
                                        vp.style.transition = 'transform 0.2s ease-out';
                                        vp.style.transform = 'translateX(' + (pct > 0 ? sw : -sw) + 'px)';
                                    }
                                    setTimeout(() => {
                                        switchGame(direction);
                                        // Snap to other side then animate in
                                        if (vp) {
                                            vp.style.transition = 'none';
                                            vp.style.transform = 'translateX(' + (pct > 0 ? -sw : sw) + 'px)';
                                            requestAnimationFrame(() => {
                                                if (vp) {
                                                    vp.style.transition = 'transform 0.25s ease-out';
                                                    vp.style.transform = '';
                                                }
                                            });
                                        }
                                        // New game loads resumed
                                        gamePaused = false;
                                        hideTimer = setTimeout(hideChrome, HIDE_DELAY);
                                    }, 200);
                                } else {
                                    // Snap back — stay paused
                                    if (vp) { vp.style.transition = 'transform 0.25s ease-out'; vp.style.transform = ''; }
                                }
                            }

                            // Listen for bridge messages from iframe
                            window.addEventListener('message', (e) => {
                                if (!e.data?.type) return;
                                switch (e.data.type) {
                                    case 'canvas-two-finger-start':
                                        beginGesture(e.data.x, e.data.y);
                                        break;
                                    case 'canvas-two-finger-move':
                                        moveGesture(e.data.x);
                                        break;
                                    case 'canvas-two-finger-end':
                                        endGesture();
                                        break;
                                }
                            });

                            // Two-finger gestures on parent document itself (outside iframe)
                            document.addEventListener('touchstart', (e) => {
                                if (e.touches.length >= 2) {
                                    e.preventDefault();
                                    const t0 = e.touches[0], t1 = e.touches[1];
                                    beginGesture((t0.clientX+t1.clientX)/2, (t0.clientY+t1.clientY)/2);
                                }
                            }, { passive: false });
                            document.addEventListener('touchmove', (e) => {
                                if (_gesture && _gesture.tracking && e.touches.length >= 2) {
                                    e.preventDefault();
                                    const t0 = e.touches[0], t1 = e.touches[1];
                                    moveGesture((t0.clientX+t1.clientX)/2);
                                }
                            }, { passive: false });
                            document.addEventListener('touchend', (e) => {
                                if (_gesture && e.touches.length === 0) endGesture();
                            });

                            // Taps on FAB/shell-nav/modals: reset hide timer
                            document.addEventListener('click', (e) => {
                                if (e.target.closest('#canvas-fab, #voice-chat-modal, #share-modal, #shell-nav')) {
                                    clearTimeout(hideTimer);
                                    hideTimer = setTimeout(hideChrome, HIDE_DELAY);
                                }
                            });

                            // Mobile FAB items → trigger the real header buttons
                            const fabGameSelect = document.getElementById('fabGameSelect');
                            const fabSaveMob = document.getElementById('fabSaveMob');
                            const fabClearMob = document.getElementById('fabClearMob');
                            const fabSourceMob = document.getElementById('fabSourceMob');
                            const gameSelect = document.getElementById('game-select');

                            if (fabGameSelect && gameSelect) {
                                fabGameSelect.addEventListener('click', () => {
                                    // Cycle to next public game alphabetically
                                    const { list, activeId } = _publicGamesList();
                                    if (list.length < 1) return;
                                    let idx = list.findIndex(g => g.id === activeId);
                                    if (idx < 0) idx = 0;
                                    const next = (idx + 1) % list.length;
                                    activateGame(list[next].id);
                                    fabGameSelect.querySelector('span:last-child').textContent = list[next].name;
                                });
                            }
                            if (fabSaveMob) fabSaveMob.addEventListener('click', () => document.getElementById('btnSave')?.click());
                            if (fabClearMob) fabClearMob.addEventListener('click', () => document.getElementById('btnClear')?.click());
                            if (fabSourceMob) fabSourceMob.addEventListener('click', () => document.getElementById('btnSource')?.click());

                            // Initial auto-hide after load
                            hideTimer = setTimeout(hideChrome, HIDE_DELAY);

                            // Shell nav: fixed on mobile
                            if (shellNav) {
                                shellNav.style.position = 'fixed';
                                shellNav.style.left = '0';
                                shellNav.style.right = '0';
                                shellNav.style.transition = 'opacity 0.3s, transform 0.3s';
                            }
                        } else {
                            // ── Desktop gesture parity with mobile two-finger controls ──
                            let gamePaused = false;
                            let desktopGesture = null; // {dx, active, timer}
                            const SWIPE_THRESHOLD = 0.15; // fraction of viewport width

                            function pauseGame() {
                                if (gamePaused) return;
                                gamePaused = true;
                                const vp = document.getElementById('phone-viewport');
                                if (vp) vp.contentWindow?.postMessage({type:'canvas-pause'}, '*');
                            }

                            function resumeGame() {
                                if (!gamePaused) return;
                                gamePaused = false;
                                const vp = document.getElementById('phone-viewport');
                                if (vp) vp.contentWindow?.postMessage({type:'canvas-resume'}, '*');
                            }

                            function switchGame(direction) {
                                const { list, activeId } = _publicGamesList();
                                if (list.length < 1) return;
                                let idx = list.findIndex(g => g.id === activeId);
                                if (idx < 0) idx = 0;
                                const next = direction === 'next'
                                    ? (idx + 1) % list.length
                                    : (idx - 1 + list.length) % list.length;
                                activateGame(list[next].id);
                            }

                            function animateSwitch(direction) {
                                const vp = document.getElementById('phone-viewport');
                                const frame = document.getElementById('phone-frame');
                                const w = frame ? frame.clientWidth : window.innerWidth;
                                if (vp) {
                                    vp.style.transition = 'transform 0.2s ease-out';
                                    vp.style.transform = 'translateX(' + (direction === 'prev' ? w : -w) + 'px)';
                                }
                                setTimeout(() => {
                                    switchGame(direction);
                                    if (vp) {
                                        vp.style.transition = 'none';
                                        vp.style.transform = 'translateX(' + (direction === 'prev' ? -w : w) + 'px)';
                                        requestAnimationFrame(() => {
                                            if (vp) {
                                                vp.style.transition = 'transform 0.22s ease-out';
                                                vp.style.transform = '';
                                            }
                                        });
                                    }
                                    resumeGame();
                                }, 200);
                            }

                            function resetViewportTransform() {
                                const vp = document.getElementById('phone-viewport');
                                if (!vp) return;
                                vp.style.transition = 'transform 0.2s ease-out';
                                vp.style.transform = '';
                            }

                            function endDesktopGesture() {
                                if (!desktopGesture || !desktopGesture.active) return;
                                const vp = document.getElementById('phone-viewport');
                                const frame = document.getElementById('phone-frame');
                                const w = frame ? frame.clientWidth : window.innerWidth;
                                const dx = desktopGesture.dx || 0;
                                const pct = w ? (dx / w) : 0;

                                desktopGesture.active = false;
                                if (desktopGesture.timer) {
                                    clearTimeout(desktopGesture.timer);
                                    desktopGesture.timer = null;
                                }

                                if (Math.abs(pct) >= SWIPE_THRESHOLD) {
                                    const direction = pct > 0 ? 'prev' : 'next';
                                    animateSwitch(direction);
                                } else {
                                    // Small swipe: snap back and stay paused, like mobile.
                                    resetViewportTransform();
                                }
                            }

                            function beginDesktopGesture() {
                                if (desktopGesture && desktopGesture.active) return;
                                desktopGesture = { dx: 0, active: true, timer: null };
                                pauseGame();
                                const vp = document.getElementById('phone-viewport');
                                if (vp) vp.style.transition = 'none';
                            }

                            function scheduleDesktopGestureEnd() {
                                if (!desktopGesture) return;
                                if (desktopGesture.timer) clearTimeout(desktopGesture.timer);
                                desktopGesture.timer = setTimeout(endDesktopGesture, 140);
                            }

                            function onDesktopWheel(e) {
                                // Trackpad two-finger horizontal swipe maps to wheel deltaX.
                                if (Math.abs(e.deltaX) <= Math.abs(e.deltaY)) return;
                                e.preventDefault();
                                beginDesktopGesture();
                                desktopGesture.dx += (-e.deltaX);
                                const vp = document.getElementById('phone-viewport');
                                if (vp) vp.style.transform = 'translateX(' + desktopGesture.dx + 'px)';
                                scheduleDesktopGestureEnd();
                            }

                            function onDesktopContextMenu(e) {
                                // Right-click / two-finger tap on trackpad toggles pause/resume.
                                e.preventDefault();
                                if (desktopGesture && desktopGesture.active) return;
                                if (gamePaused) {
                                    resumeGame();
                                    resetViewportTransform();
                                } else {
                                    pauseGame();
                                }
                            }

                            function onDesktopKeydown(e) {
                                const tag = (e.target && e.target.tagName) ? e.target.tagName.toLowerCase() : '';
                                if (tag === 'input' || tag === 'textarea' || (e.target && e.target.isContentEditable)) return;
                                if (e.repeat) return;
                                if (String(e.key || '').toLowerCase() === 'p') {
                                    e.preventDefault();
                                    if (gamePaused) {
                                        resumeGame();
                                        resetViewportTransform();
                                    } else {
                                        pauseGame();
                                    }
                                }
                            }

                            function onDesktopBracket(direction) {
                                if (desktopGesture && desktopGesture.active) return;
                                pauseGame();
                                animateSwitch(direction);
                            }

                            // Apply handlers to the phone frame area.
                            const frameEl = document.getElementById('phone-frame');
                            const vpEl = document.getElementById('phone-viewport');
                            const prevBracket = document.getElementById('btnPrevBracket');
                            const nextBracket = document.getElementById('btnNextBracket');
                            if (frameEl) {
                                frameEl.addEventListener('wheel', onDesktopWheel, { passive: false });
                                frameEl.addEventListener('contextmenu', onDesktopContextMenu);
                            }
                            if (vpEl) {
                                vpEl.addEventListener('wheel', onDesktopWheel, { passive: false });
                                vpEl.addEventListener('contextmenu', onDesktopContextMenu);
                            }
                            if (prevBracket) prevBracket.addEventListener('click', () => onDesktopBracket('prev'));
                            if (nextBracket) nextBracket.addEventListener('click', () => onDesktopBracket('next'));
                            document.addEventListener('keydown', onDesktopKeydown);
                        }

                        // ── Game Sync: auto-share games via relay WebSocket ──
                        (async function initGameSync() {
                            const RELAY_WS = 'wss://relay.slob.games/sync';
                            const MAX_PUSH_SIZE = 256 * 1024; // 256KB per game

                            function slugify(s) {
                                return String(s || '')
                                    .trim()
                                    .toLowerCase()
                                    .replace(/[^a-z0-9]+/g, '-')
                                    .replace(/^-+|-+$/g, '') || 'untitled';
                            }

                            function localUsername() {
                                try {
                                    const u = (localStorage.getItem('traits.env.SLOB_USERNAME') || '').trim();
                                    return u ? slugify(u) : 'public';
                                } catch (_) {
                                    return 'public';
                                }
                            }

                            async function contentHash(str) {
                                const buf = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(str || ''));
                                const arr = new Uint8Array(buf);
                                let hex = '';
                                for (let i = 0; i < arr.length; i++) hex += arr[i].toString(16).padStart(2, '0');
                                return hex.slice(0, 16);
                            }

                            // Read local games and compute their content hashes
                            async function localGamesWithHashes() {
                                const col = readGamesCollection();
                                const result = [];
                                for (const [id, g] of Object.entries(col.games || {})) {
                                    if (!g.content) continue;
                                    const hash = await contentHash(g.content);
                                    result.push({
                                        id,
                                        name: g.name || 'untitled',
                                        content: g.content,
                                        hash,
                                        owner: g.owner || localUsername(),
                                        game_id: g.game_id || slugify(g.name || id),
                                        scope: g.scope || 'internal',
                                        version: g.version || ''
                                    });
                                }
                                return result;
                            }

                            // Add synced games to local collection without disrupting active game
                            function addSyncedGames(games, localHashes) {
                                if (!games.length) return 0;
                                try {
                                    const raw = localStorage.getItem('traits.pvfs') || '{}';
                                    const files = JSON.parse(raw);
                                    const col = files['canvas/games.json']
                                        ? JSON.parse(files['canvas/games.json'])
                                        : { active: null, games: {} };

                                    // Build set of existing content hashes
                                    const existing = new Set(localHashes || []);
                                    // Also check _sync_hash on stored games
                                    for (const g of Object.values(col.games || {})) {
                                        if (g._sync_hash) existing.add(g._sync_hash);
                                    }

                                    let added = 0;
                                    for (const g of games) {
                                        if (!g.content || !g.content_hash) continue;
                                        const owner = g.owner || 'public';
                                        const gid = g.game_id || slugify(g.name || g.content_hash);
                                        const gameId = ('s-' + slugify(owner + '-' + gid)).slice(0, 48);

                                        // Match by name: reuse existing entry if same name already present.
                                        const nameKey = (g.name || '').trim().toLowerCase();
                                        let matchedId = null;
                                        if (nameKey) {
                                            for (const [eid, eg] of Object.entries(col.games)) {
                                                const es = (eg.scope || eg._scope || 'internal');
                                                if (es === 'external' && (eg.name || '').trim().toLowerCase() === nameKey) {
                                                    matchedId = eid;
                                                    break;
                                                }
                                            }
                                        }
                                        const targetId = matchedId || gameId;

                                        // Skip if we already have this exact content version
                                        if (existing.has(g.content_hash) && !matchedId && !col.games[gameId]) continue;

                                        // Update in place if we already have this identity.
                                        const prev = col.games[targetId] || {};
                                        col.games[targetId] = {
                                            name: g.name || prev.name || 'untitled',
                                            content: g.content,
                                            created: prev.created || g.updated || new Date().toISOString(),
                                            updated: g.updated || new Date().toISOString(),
                                            owner: owner,
                                            game_id: gid,
                                            scope: 'external',
                                            version: g.version || prev.version || '',
                                            checksum: g.checksum || g.content_hash,
                                            _sync_hash: g.content_hash
                                        };
                                        existing.add(g.content_hash);
                                        added++;
                                    }

                                    if (added > 0) {
                                        if (!col.active && Object.keys(col.games).length > 0) {
                                            col.active = Object.keys(col.games)[0];
                                        }
                                        files['canvas/games.json'] = JSON.stringify(col);
                                        localStorage.setItem('traits.pvfs', JSON.stringify(files));
                                        dedupeLocalGames();
                                        renderProjectBar();
                                    }
                                    return added;
                                } catch (e) {
                                    console.warn('[sync] add failed:', e);
                                    return 0;
                                }
                            }

                            let ws = null;
                            let reconnectDelay = 2000;
                            let _syncing = false; // prevent re-entrant sync from storage events
                            let serverHashSet = new Set(); // track what relay already has

                            function connect() {
                                if (ws) return;
                                try {
                                    let wsUrl = RELAY_WS + '?user=' + encodeURIComponent(localUsername());
                                    try {
                                        const token = localStorage.getItem('traits.secret.SLOB_USER_TOKEN') || '';
                                        if (token) wsUrl += '&token=' + encodeURIComponent(token);
                                    } catch (_) {}
                                    ws = new WebSocket(wsUrl);
                                } catch (e) { scheduleReconnect(); return; }
                                window.__syncWs = ws; // expose for score forwarding

                                ws.onopen = () => {
                                    reconnectDelay = 2000;
                                    console.log('[sync] connected');
                                };

                                ws.onmessage = async (e) => {
                                    let data;
                                    try { data = JSON.parse(e.data); } catch (_) { return; }

                                    if (data.type === 'catalog') {
                                        // Server sent its hash catalog — compare with local
                                        serverHashSet = new Set(data.hashes || []);
                                        const local = await localGamesWithHashes();
                                        const localHashSet = new Set(local.map(g => g.hash));

                                        // Request games we don't have
                                        const need = [...serverHashSet].filter(h => !localHashSet.has(h));
                                        if (need.length > 0) {
                                            ws.send(JSON.stringify({ type: 'need', hashes: need }));
                                        }

                                        // Push games server doesn't have
                                        const toPush = local.filter(g =>
                                            g.scope !== 'internal' &&
                                            !serverHashSet.has(g.hash) &&
                                            g.content.length > 0 &&
                                            g.content.length <= MAX_PUSH_SIZE
                                        );
                                        if (toPush.length > 0) {
                                            ws.send(JSON.stringify({
                                                type: 'push',
                                                games: toPush.map(g => ({
                                                    name: g.name,
                                                    content: g.content,
                                                    content_hash: g.hash,
                                                    checksum: g.hash,
                                                    owner: g.owner,
                                                    game_id: g.game_id,
                                                    scope: g.scope || 'internal',
                                                    version: g.version || ''
                                                }))
                                            }));
                                            toPush.forEach(g => serverHashSet.add(g.hash));
                                        }
                                    }

                                    if (data.type === 'games') {
                                        // Full game content from server (response to 'need')
                                        const hadGames = getGamesList().length > 0;
                                        const localHashes = (await localGamesWithHashes()).map(g => g.hash);
                                        _syncing = true;
                                        const added = addSyncedGames(data.games, localHashes);
                                        _syncing = false;
                                        // Auto-activate first game when syncing into empty collection
                                        if (added > 0 && !hadGames) {
                                            const col = readGamesCollection();
                                            const firstId = col.active || Object.keys(col.games || {})[0];
                                            if (firstId) activateGame(firstId);
                                        }
                                    }

                                    if (data.type === 'sync') {
                                        // Real-time broadcast from another client
                                        const hadGames = getGamesList().length > 0;
                                        const localHashes = (await localGamesWithHashes()).map(g => g.hash);
                                        _syncing = true;
                                        const added = addSyncedGames(data.games, localHashes);
                                        _syncing = false;
                                        if (added > 0) {
                                            console.log('[sync] received', added, 'new game(s)');
                                            if (!hadGames) {
                                                const col = readGamesCollection();
                                                const firstId = col.active || Object.keys(col.games || {})[0];
                                                if (firstId) activateGame(firstId);
                                            }
                                        }
                                    }

                                    if (data.type === 'scores') {
                                        // Initial high score catalog from server — highest always wins
                                        for (const s of (data.scores || [])) {
                                            const cur = window.__highScores[s.game_hash] || {score:0, player:''};
                                            if (s.score > cur.score) {
                                                window.__highScores[s.game_hash] = {score: s.score, player: s.player || ''};
                                                // If this is the active game, notify the iframe immediately
                                                if (s.game_hash === window.__activeGameHash) {
                                                    try {
                                                        const vp = document.getElementById('phone-viewport');
                                                        if (vp && vp.contentWindow) vp.contentWindow.postMessage({type:'highscore-update', score:s.score, player:s.player||''}, '*');
                                                    } catch(_) {}
                                                }
                                            }
                                        }
                                    }

                                    if (data.type === 'score-update') {
                                        // New high score broadcast
                                        const cur = window.__highScores[data.game_hash] || {score:0, player:''};
                                        if (data.score > cur.score) {
                                            window.__highScores[data.game_hash] = {score: data.score, player: data.player || ''};
                                            // Notify iframe so game can update display
                                            try {
                                                const vp = document.getElementById('phone-viewport');
                                                if (vp && vp.contentWindow) {
                                                    vp.contentWindow.postMessage({
                                                        type: 'highscore-update',
                                                        score: data.score,
                                                        player: data.player || ''
                                                    }, '*');
                                                }
                                            } catch(_) {}
                                        }
                                    }

                                    if (data.type === 'game-deleted') {
                                        try {
                                            const dead = String(data.content_hash || '');
                                            if (!dead) return;
                                            const raw = localStorage.getItem('traits.pvfs') || '{}';
                                            const files = JSON.parse(raw);
                                            const col = files['canvas/games.json']
                                                ? JSON.parse(files['canvas/games.json'])
                                                : { active: null, games: {} };
                                            var removed = 0;
                                            for (const gid in (col.games || {})) {
                                                if (!Object.prototype.hasOwnProperty.call(col.games, gid)) continue;
                                                const g = col.games[gid] || {};
                                                const isExternal = (g.scope || g._scope || '') === 'external';
                                                const h = String(g._sync_hash || g.checksum || '');
                                                if (isExternal && h === dead) {
                                                    if (col.active === gid) col.active = null;
                                                    delete col.games[gid];
                                                    removed++;
                                                }
                                            }
                                            if (removed > 0) {
                                                files['canvas/games.json'] = JSON.stringify(col);
                                                localStorage.setItem('traits.pvfs', JSON.stringify(files));
                                                renderProjectBar();
                                            }
                                        } catch(_) {}
                                    }
                                };

                                ws.onclose = () => { ws = null; window.__syncWs = null; scheduleReconnect(); };
                                ws.onerror = () => {};
                            }

                            function scheduleReconnect() {
                                setTimeout(() => {
                                    reconnectDelay = Math.min(reconnectDelay * 1.5, 30000);
                                    connect();
                                }, reconnectDelay);
                            }

                            // Push new games when local collection changes
                            window.addEventListener('traits-canvas-projects-changed', async () => {
                                if (_syncing) return; // don't echo sync-adds
                                if (!ws || ws.readyState !== WebSocket.OPEN) return;
                                const local = await localGamesWithHashes();
                                const toPush = local.filter(g =>
                                    g.scope !== 'internal' &&
                                    !serverHashSet.has(g.hash) &&
                                    g.content.length > 0 &&
                                    g.content.length <= MAX_PUSH_SIZE
                                );
                                if (toPush.length > 0) {
                                    ws.send(JSON.stringify({
                                        type: 'push',
                                        games: toPush.map(g => ({
                                            name: g.name,
                                            content: g.content,
                                            content_hash: g.hash,
                                            checksum: g.hash,
                                            owner: g.owner,
                                            game_id: g.game_id,
                                            scope: g.scope || 'internal',
                                            version: g.version || ''
                                        }))
                                    }));
                                    toPush.forEach(g => serverHashSet.add(g.hash));
                                }
                            });

                            // ── One-time migration: push local internal games to relay ──
                            async function migrateLocalToRelay() {
                                try {
                                    if (localStorage.getItem('traits.env.GAMES_MIGRATED')) return;
                                    const token = localStorage.getItem('traits.secret.SLOB_USER_TOKEN') || '';
                                    if (!token) return; // need auth to push internal games

                                    const col = readGamesCollection();
                                    const games = col.games || {};
                                    const internalGames = [];
                                    for (const [id, g] of Object.entries(games)) {
                                        const scope = g.scope || g._scope || 'internal';
                                        if (scope !== 'external' && g.content && g.content.length > 0 && g.content.length <= MAX_PUSH_SIZE) {
                                            internalGames.push({ id, g });
                                        }
                                    }
                                    if (!internalGames.length) {
                                        localStorage.setItem('traits.env.GAMES_MIGRATED', '1');
                                        return;
                                    }

                                    console.log('[migration] pushing', internalGames.length, 'internal game(s) to relay');
                                    const headers = { 'Content-Type': 'application/json', 'Authorization': 'Bearer ' + token };
                                    let pushed = 0;
                                    for (const { id, g } of internalGames) {
                                        const gameId = g.game_id || slugify(g.name || id);
                                        try {
                                            const resp = await fetch('https://relay.slob.games/sync/internal/game/' + encodeURIComponent(gameId), {
                                                method: 'PUT',
                                                headers,
                                                body: JSON.stringify({
                                                    name: g.name || 'untitled',
                                                    content: g.content,
                                                    version: g.version || 'v1'
                                                })
                                            });
                                            if (resp.ok) pushed++;
                                        } catch (_) {}
                                    }
                                    console.log('[migration] pushed', pushed, '/', internalGames.length, 'games');

                                    // Remove migrated internal games from local pvfs (keep external ones)
                                    if (pushed > 0) {
                                        try {
                                            const raw = localStorage.getItem('traits.pvfs') || '{}';
                                            const files = JSON.parse(raw);
                                            const c = files['canvas/games.json'] ? JSON.parse(files['canvas/games.json']) : { active: null, games: {} };
                                            for (const { id } of internalGames) {
                                                if (c.games[id]) {
                                                    if (c.active === id) {
                                                        // Find an external game to activate instead
                                                        const ext = Object.entries(c.games).find(([eid, eg]) => eid !== id && (eg.scope === 'external'));
                                                        c.active = ext ? ext[0] : null;
                                                    }
                                                    delete c.games[id];
                                                }
                                            }
                                            files['canvas/games.json'] = JSON.stringify(c);
                                            localStorage.setItem('traits.pvfs', JSON.stringify(files));
                                        } catch (_) {}
                                    }

                                    localStorage.setItem('traits.env.GAMES_MIGRATED', '1');
                                    console.log('[migration] done — internal games moved to relay');
                                } catch (e) {
                                    console.warn('[migration] failed:', e);
                                }
                            }

                            // Start sync
                            connect();

                            // Run migration after a short delay to ensure WS/auth are ready
                            setTimeout(migrateLocalToRelay, 3000);
                        })();

                        // ── Desktop: scale phone frame to fit viewport height ──
                        if (!isMobile) {
                            const phoneFrame = document.getElementById('phone-frame');
                            const container = document.getElementById('canvas-container');
                            function scaleFrame() {
                                if (!phoneFrame || phoneFrame.style.display === 'none') return;
                                const available = container.clientHeight;
                                const natural = phoneFrame.scrollHeight || 900;
                                const scale = Math.min(1, available / natural);
                                phoneFrame.style.transform = 'scale(' + scale + ')';
                                // CSS transform doesn't affect layout — the layout box stays
                                // at the original height, causing align-items:center to mis-position.
                                // Use align-self + margin to center based on the visual (scaled) height.
                                phoneFrame.style.alignSelf = 'flex-start';
                                const visualH = natural * scale;
                                phoneFrame.style.marginTop = Math.max(0, (available - visualH) / 2) + 'px';
                            }
                            window.addEventListener('resize', scaleFrame);
                            // Run after a short delay to ensure layout is settled
                            setTimeout(scaleFrame, 100);
                            // Also run when a game loads
                            const vp = document.getElementById('phone-viewport');
                            if (vp) vp.addEventListener('load', () => setTimeout(scaleFrame, 50));
                        }
                    })();
                "#)) }
            }
        }
    };
    Value::String(markup.into_string())
}
