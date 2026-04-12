use serde_json::{json, Value};

const HISTORY_VFS_PATH: &str = "voice/history.json";
/// Maximum turns to retain in storage.
const MAX_STORED: usize = 200;

/// sys.voice.history — rolling transcript store for the voice agent.
///
/// Backed by VFS so it works on both native and WASM (browser localStorage).
/// Every user + assistant turn is appended here and injected into the next
/// session via sys.voice.instruct build.
///
/// Actions:
///   append <role> <text>  — add one turn (role: user | assistant)
///   get    [n]            — return last n turns (default 20)
///   clear                 — erase all history
pub fn voice_history(args: &[Value]) -> Value {
    let action = args.first().and_then(|v| v.as_str()).unwrap_or("");

    match action {
        "append" => {
            let role = args.get(1).and_then(|v| v.as_str()).unwrap_or("user");
            let text = args.get(2).and_then(|v| v.as_str()).unwrap_or("").trim();
            if text.is_empty() {
                return json!({"ok": false, "error": "text required"});
            }
            let ts = now_stamp();
            let mut turns = load_turns();
            turns.push(json!({"role": role, "text": text, "ts": ts}));
            while turns.len() > MAX_STORED {
                turns.remove(0);
            }
            save_turns(&turns);
            json!({"ok": true, "count": turns.len()})
        }

        "get" => {
            let n = args
                .get(1)
                .and_then(|v| v.as_u64())
                .map(|n| n as usize)
                .unwrap_or(20)
                .min(MAX_STORED);
            let turns = load_turns();
            let total = turns.len();
            let start = total.saturating_sub(n);
            let recent: Vec<&Value> = turns[start..].iter().collect();
            json!({"ok": true, "turns": recent, "total": total})
        }

        "clear" => {
            save_turns(&[]);
            json!({"ok": true, "action": "clear"})
        }

        _ => json!({"ok": false, "error": format!("Unknown action '{}'. Use append, get, clear.", action)}),
    }
}

fn load_turns() -> Vec<Value> {
    match kernel_logic::platform::vfs_read(HISTORY_VFS_PATH) {
        Some(s) => serde_json::from_str(&s).unwrap_or_default(),
        None => Vec::new(),
    }
}

fn save_turns(turns: &[Value]) {
    if let Ok(s) = serde_json::to_string(turns) {
        kernel_logic::platform::vfs_write(HISTORY_VFS_PATH, &s);
    }
}

fn now_stamp() -> String {
    let (y, mo, d, h, mi, s) = kernel_logic::platform::time::now_utc();
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, mo, d, h, mi, s)
}
