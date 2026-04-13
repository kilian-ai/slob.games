use serde_json::{json, Map, Value};

const INDEX_PATH: &str = "canvas/revisions/index.json";
const REV_DIR: &str = "canvas/revisions";
const MAX_REVISIONS_PER_GAME: usize = 40;

fn now_iso() -> String {
    let t = kernel_logic::platform::time::now_utc();
    format!("{}-{:02}-{:02}T{:02}:{:02}:{:02}Z", t.0, t.1, t.2, t.3, t.4, t.5)
}

fn now_stamp() -> String {
    let t = kernel_logic::platform::time::now_utc();
    format!("{:02}{:02}{:02}{:02}{:02}{:02}", t.0 % 100, t.1, t.2, t.3, t.4, t.5)
}

fn normalize_key(input: &str) -> String {
    let mut out = String::new();
    for c in input.trim().to_lowercase().chars() {
        if c.is_ascii_alphanumeric() || c == '-' || c == '/' || c == '_' {
            out.push(c);
        } else if c.is_ascii_whitespace() {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

fn file_safe_key(game_key: &str) -> String {
    game_key.replace('/', "__")
}

fn read_index() -> Map<String, Value> {
    kernel_logic::platform::vfs_read(INDEX_PATH)
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default()
}

fn write_index(idx: &Map<String, Value>) {
    if let Ok(s) = serde_json::to_string(idx) {
        kernel_logic::platform::vfs_write(INDEX_PATH, &s);
    }
}

fn parse_resources(raw: &str) -> Map<String, Value> {
    if raw.trim().is_empty() {
        return Map::new();
    }
    match serde_json::from_str::<Value>(raw) {
        Ok(v) => {
            let mut out = Map::new();
            if let Some(obj) = v.as_object() {
                for (k, vv) in obj {
                    let path = k.trim();
                    if path.is_empty() || path.starts_with('/') || path.contains("..") {
                        continue;
                    }
                    if path == "canvas/app.html" || path == "canvas/games.json" {
                        continue;
                    }
                    if let Some(s) = vv.as_str() {
                        if !s.is_empty() {
                            out.insert(path.to_string(), Value::String(s.to_string()));
                        }
                    }
                }
            }
            out
        }
        Err(_) => Map::new(),
    }
}

fn read_revision_meta(idx: &Map<String, Value>, game_key: &str) -> Vec<Value> {
    idx.get(game_key)
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default()
}

fn write_revision_meta(idx: &mut Map<String, Value>, game_key: &str, revs: Vec<Value>) {
    idx.insert(game_key.to_string(), Value::Array(revs));
}

pub fn game_vcs(args: &[Value]) -> Value {
    let action = args.first().and_then(|v| v.as_str()).unwrap_or("list");
    let game_key_raw = args.get(1).and_then(|v| v.as_str()).unwrap_or("");
    let game_key = normalize_key(game_key_raw);

    match action {
        "commit" => {
            if game_key.is_empty() {
                return json!({"ok": false, "error": "game_key required"});
            }
            let content = args.get(2).and_then(|v| v.as_str()).unwrap_or("");
            if content.is_empty() {
                return json!({"ok": false, "error": "content required"});
            }
            let name = args.get(3).and_then(|v| v.as_str()).unwrap_or("untitled");
            let version = args.get(4).and_then(|v| v.as_str()).unwrap_or("");
            let resources = args.get(5).and_then(|v| v.as_str()).unwrap_or("{}");
            let resources_map = parse_resources(resources);

            let mut idx = read_index();
            let mut revs = read_revision_meta(&idx, &game_key);

            // Deduplicate if latest snapshot has same content.
            if let Some(last) = revs.last() {
                if let Some(last_id) = last.get("id").and_then(|v| v.as_str()) {
                    let last_path = format!("{}/{}.json", REV_DIR, last_id);
                    if let Some(raw) = kernel_logic::platform::vfs_read(&last_path) {
                        if let Ok(v) = serde_json::from_str::<Value>(&raw) {
                            if v.get("content").and_then(|x| x.as_str()) == Some(content) {
                                return json!({"ok": true, "action": "commit", "dedup": true, "id": last_id});
                            }
                        }
                    }
                }
            }

            let stamp = now_stamp();
            let id = format!("{}-{}-{}", file_safe_key(&game_key), stamp, revs.len() + 1);
            let path = format!("{}/{}.json", REV_DIR, id);
            let created = now_iso();

            let snapshot = json!({
                "id": id,
                "game_key": game_key,
                "name": name,
                "version": version,
                "created": created,
                "content": content,
                "resources": resources_map,
            });
            if let Ok(s) = serde_json::to_string(&snapshot) {
                kernel_logic::platform::vfs_write(&path, &s);
            }

            let meta = json!({
                "id": snapshot["id"],
                "name": snapshot["name"],
                "version": snapshot["version"],
                "created": snapshot["created"],
                "length": content.len(),
                "resource_count": snapshot["resources"].as_object().map(|m| m.len()).unwrap_or(0),
                "path": path,
            });
            revs.push(meta);
            if revs.len() > MAX_REVISIONS_PER_GAME {
                let drop_n = revs.len() - MAX_REVISIONS_PER_GAME;
                for old in revs.iter().take(drop_n) {
                    if let Some(p) = old.get("path").and_then(|v| v.as_str()) {
                        let _ = kernel_logic::platform::vfs_delete(p);
                    }
                }
                revs = revs.into_iter().skip(drop_n).collect();
            }

            write_revision_meta(&mut idx, &game_key, revs);
            write_index(&idx);
            json!({"ok": true, "action": "commit", "id": snapshot["id"]})
        }

        "list" => {
            if game_key.is_empty() {
                return json!({"ok": false, "error": "game_key required"});
            }
            let idx = read_index();
            let mut revs = read_revision_meta(&idx, &game_key);
            revs.reverse();
            json!({"ok": true, "action": "list", "game_key": game_key, "revisions": revs, "count": revs.len()})
        }

        "checkout" => {
            if game_key.is_empty() {
                return json!({"ok": false, "error": "game_key required"});
            }
            let rev_id = args.get(2).and_then(|v| v.as_str()).unwrap_or("");
            if rev_id.is_empty() {
                return json!({"ok": false, "error": "revision id required"});
            }
            let path = format!("{}/{}.json", REV_DIR, rev_id);
            let raw = match kernel_logic::platform::vfs_read(&path) {
                Some(s) => s,
                None => return json!({"ok": false, "error": "revision not found"}),
            };
            let snap = match serde_json::from_str::<Value>(&raw) {
                Ok(v) => v,
                Err(_) => return json!({"ok": false, "error": "invalid snapshot"}),
            };
            if snap.get("game_key").and_then(|v| v.as_str()) != Some(game_key.as_str()) {
                return json!({"ok": false, "error": "revision belongs to different game"});
            }
            json!({
                "ok": true,
                "action": "checkout",
                "revision": {
                    "id": snap["id"],
                    "name": snap["name"],
                    "version": snap["version"],
                    "created": snap["created"],
                    "content": snap["content"],
                    "resources": snap["resources"],
                }
            })
        }

        "delete" => {
            if game_key.is_empty() {
                return json!({"ok": false, "error": "game_key required"});
            }
            let rev_id = args.get(2).and_then(|v| v.as_str()).unwrap_or("");
            if rev_id.is_empty() {
                return json!({"ok": false, "error": "revision id required"});
            }

            let mut idx = read_index();
            let revs = read_revision_meta(&idx, &game_key);
            let mut kept = Vec::new();
            let mut deleted_path: Option<String> = None;

            for r in revs {
                let rid = r.get("id").and_then(|v| v.as_str()).unwrap_or("");
                if rid == rev_id {
                    deleted_path = r.get("path").and_then(|v| v.as_str()).map(|s| s.to_string());
                } else {
                    kept.push(r);
                }
            }

            if deleted_path.is_none() {
                return json!({"ok": false, "error": "revision not found"});
            }

            if let Some(p) = deleted_path {
                let _ = kernel_logic::platform::vfs_delete(&p);
            }

            write_revision_meta(&mut idx, &game_key, kept.clone());
            write_index(&idx);
            json!({"ok": true, "action": "delete", "revision_id": rev_id, "remaining": kept.len()})
        }

        _ => json!({"ok": false, "error": "Unknown action. Use: commit, list, checkout, delete"}),
    }
}
