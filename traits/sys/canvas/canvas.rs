use serde_json::{json, Value};

/// VFS path for the games collection JSON.
const GAMES_JSON_PATH: &str = "canvas/games.json";

/// Read the games collection from VFS. Returns a mutable JSON object.
fn read_games() -> Value {
    kernel_logic::platform::vfs_read(GAMES_JSON_PATH)
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| json!({"active": null, "games": {}}))
}

/// Write the games collection back to VFS (auto-syncs to localStorage via pvfs).
fn write_games(col: &Value) {
    if let Ok(s) = serde_json::to_string(col) {
        kernel_logic::platform::vfs_write(GAMES_JSON_PATH, &s);
    }
    // Also keep canvas/app.html in sync for backward compat (poller, agent reads)
    if let Some(id) = col["active"].as_str() {
        if let Some(content) = col["games"][id]["content"].as_str() {
            kernel_logic::platform::vfs_write("canvas/app.html", content);
        }
    }
}

/// Generate a short unique ID: 8-char hex from timestamp + counter.
fn gen_id() -> String {
    use kernel_logic::platform::time::now_utc;
    let (y, mo, d, h, m, s) = now_utc();
    format!("{:02}{:02}{:02}{:02}{:02}{:02}",
            y % 100, mo, d, h, m, s)
}

fn now_ts() -> String {
    let now = kernel_logic::platform::time::now_utc();
    format!("{}-{:02}-{:02}T{:02}:{:02}:{:02}Z", now.0, now.1, now.2, now.3, now.4, now.5)
}

/// If the active game is external, fork it into an internal game before mutating.
fn ensure_internal_active(col: &mut Value) -> Option<String> {
    let active_id = col["active"].as_str().map(|s| s.to_string())?;
    if !col["games"][&active_id].is_object() {
        return Some(active_id);
    }

    let scope = col["games"][&active_id]["scope"]
        .as_str()
        .or_else(|| col["games"][&active_id]["_scope"].as_str())
        .unwrap_or("internal");

    if scope != "external" {
        return Some(active_id);
    }

    let mut new_id = format!("{}f", gen_id());
    let mut n = 1;
    while col["games"][&new_id].is_object() {
        n += 1;
        new_id = format!("{}f{}", gen_id(), n);
    }

    let mut forked = col["games"][&active_id].clone();
    let ts = now_ts();
    forked["scope"] = json!("internal");
    forked["_scope"] = json!("internal");
    forked["forked_from"] = json!(active_id.clone());
    if forked["created"].is_null() {
        forked["created"] = json!(ts.clone());
    }
    forked["updated"] = json!(ts);

    col["games"][&new_id] = forked;
    col["active"] = json!(new_id.clone());
    Some(new_id)
}

pub fn canvas(args: &[Value]) -> Value {
    let action = args.first().and_then(|v| v.as_str()).unwrap_or("get");

    match action {
        // ── set: update active game content (or create new game) ──
        "set" => {
            let content = args.get(1).and_then(|v| v.as_str()).unwrap_or("");
            let mut col = read_games();
            let ts = now_ts();

            let active_id = ensure_internal_active(&mut col)
                .or_else(|| col["active"].as_str().map(|s| s.to_string()));

            match active_id {
                Some(id) if col["games"][&id].is_object() => {
                    // Update existing active game
                    col["games"][&id]["content"] = json!(content);
                    col["games"][&id]["updated"] = json!(ts);
                    col["games"][&id]["scope"] = json!("internal");
                    col["games"][&id]["_scope"] = json!("internal");
                }
                _ => {
                    // No active game — create a new one
                    let id = gen_id();
                    col["games"][&id] = json!({
                        "name": "untitled",
                        "content": content,
                        "scope": "internal",
                        "_scope": "internal",
                        "owner": "local",
                        "created": ts,
                        "updated": ts
                    });
                    col["active"] = json!(id);
                }
            }

            let len = content.len();
            write_games(&col);
            let active = col["active"].as_str().unwrap_or("");
            json!({"ok": true, "action": "set", "length": len, "game_id": active})
        }

        // ── append: append to active game ──
        "append" => {
            let content = args.get(1).and_then(|v| v.as_str()).unwrap_or("");
            let mut col = read_games();
            let ts = now_ts();

            let active_id = ensure_internal_active(&mut col)
                .or_else(|| col["active"].as_str().map(|s| s.to_string()));
            let combined = match &active_id {
                Some(id) if col["games"][id]["content"].is_string() => {
                    let existing = col["games"][id]["content"].as_str().unwrap_or("");
                    format!("{}{}", existing, content)
                }
                _ => content.to_string(),
            };

            match active_id {
                Some(id) if col["games"][&id].is_object() => {
                    col["games"][&id]["content"] = json!(combined);
                    col["games"][&id]["updated"] = json!(ts);
                    col["games"][&id]["scope"] = json!("internal");
                    col["games"][&id]["_scope"] = json!("internal");
                }
                _ => {
                    let id = gen_id();
                    col["games"][&id] = json!({
                        "name": "untitled",
                        "content": combined,
                        "scope": "internal",
                        "_scope": "internal",
                        "owner": "local",
                        "created": ts,
                        "updated": ts
                    });
                    col["active"] = json!(id);
                }
            }

            let len = combined.len();
            write_games(&col);
            json!({"ok": true, "action": "append", "length": len})
        }

        // ── get: return active game content ──
        "get" => {
            let col = read_games();
            let active_id = col["active"].as_str().unwrap_or("");
            let content = col["games"][active_id]["content"].as_str().unwrap_or("");
            let name = col["games"][active_id]["name"].as_str().unwrap_or("");
            let version = col["games"][active_id]["version"].as_str().unwrap_or("");
            json!({"ok": true, "content": content, "length": content.len(),
                "game_id": active_id, "name": name, "version": version})
        }

        // ── clear: clear active game content ──
        "clear" => {
            let mut col = read_games();
            let active_id = ensure_internal_active(&mut col)
                .or_else(|| col["active"].as_str().map(|s| s.to_string()));
            if let Some(id) = active_id {
                if col["games"][&id].is_object() {
                    col["games"][&id]["content"] = json!("");
                    col["games"][&id]["updated"] = json!(now_ts());
                    kernel_logic::platform::vfs_delete("canvas/app.html");
                }
            }
            write_games(&col);
            json!({"ok": true, "action": "clear"})
        }

        // ── new: create a new empty game and activate it ──
        "new" => {
            let name = args.get(1).and_then(|v| v.as_str()).unwrap_or("untitled");
            let version = args.get(2).and_then(|v| v.as_str()).unwrap_or("");
            let mut col = read_games();
            let ts = now_ts();
            let id = gen_id();
            col["games"][&id] = json!({
                "name": name,
                "version": version,
                "content": "",
                "scope": "internal",
                "_scope": "internal",
                "owner": "local",
                "created": ts,
                "updated": ts
            });
            col["active"] = json!(&id);
            write_games(&col);
            json!({"ok": true, "action": "new", "game_id": id, "name": name})
        }

        // ── games: list all games ──
        "games" => {
            let col = read_games();
            let active_id = col["active"].as_str().unwrap_or("");
            let games_obj = col["games"].as_object();
            let mut list = Vec::new();
            if let Some(games) = games_obj {
                for (id, g) in games {
                    list.push(json!({
                        "id": id,
                        "name": g["name"].as_str().unwrap_or("untitled"),
                        "version": g["version"].as_str().unwrap_or(""),
                        "owner": g["owner"].as_str().unwrap_or("local"),
                        "scope": g["scope"].as_str().or_else(|| g["_scope"].as_str()).unwrap_or("internal"),
                        "game_id": g["game_id"].as_str().unwrap_or(""),
                        "checksum": g["checksum"].as_str().unwrap_or(""),
                        "version": g["version"].as_str().unwrap_or(""),
                        "length": g["content"].as_str().map(|s| s.len()).unwrap_or(0),
                        "active": id == active_id,
                        "created": g["created"],
                        "updated": g["updated"]
                    }));
                }
            }
            // Sort by updated desc
            list.sort_by(|a, b| {
                let ua = a["updated"].as_str().unwrap_or("");
                let ub = b["updated"].as_str().unwrap_or("");
                ub.cmp(ua)
            });
            json!({"ok": true, "games": list, "count": list.len(), "active": active_id})
        }

        // ── activate: switch active game by id ──
        "activate" => {
            let id = args.get(1).and_then(|v| v.as_str()).unwrap_or("");
            if id.is_empty() {
                return json!({"ok": false, "error": "Game ID required"});
            }
            let mut col = read_games();
            if !col["games"][id].is_object() {
                return json!({"ok": false, "error": format!("Game not found: {}", id)});
            }
            col["active"] = json!(id);
            write_games(&col);
            let content = col["games"][id]["content"].as_str().unwrap_or("");
            let name = col["games"][id]["name"].as_str().unwrap_or("");
                let version = col["games"][id]["version"].as_str().unwrap_or("");
            json!({"ok": true, "action": "activate", "game_id": id,
                    "name": name, "version": version, "content": content, "length": content.len()})
        }

        // ── fork: if active is external, clone to internal and activate clone ──
        "fork" => {
            let mut col = read_games();
            let before = col["active"].as_str().unwrap_or("").to_string();
            let after = ensure_internal_active(&mut col)
                .or_else(|| col["active"].as_str().map(|s| s.to_string()))
                .unwrap_or_default();
            if !after.is_empty() {
                write_games(&col);
            }
            let forked = !before.is_empty() && before != after;
            let name = if after.is_empty() {
                "".to_string()
            } else {
                col["games"][&after]["name"].as_str().unwrap_or("untitled").to_string()
            };
            json!({
                "ok": true,
                "action": "fork",
                "forked": forked,
                "from": before,
                "game_id": after,
                "name": name
            })
        }

        // ── rename: rename active game or game by id ──
        "rename" => {
            let name = args.get(1).and_then(|v| v.as_str()).unwrap_or("");
            if name.is_empty() {
                return json!({"ok": false, "error": "Name required"});
            }
            // Optional second arg: game id (defaults to active)
            let target_id = args.get(2).and_then(|v| v.as_str())
                .or_else(|| read_games()["active"].as_str().map(|_| ""))
                .unwrap_or("");
            let mut col = read_games();
            if target_id.is_empty() {
                let _ = ensure_internal_active(&mut col);
            }
            let id = if target_id.is_empty() {
                col["active"].as_str().unwrap_or("").to_string()
            } else {
                target_id.to_string()
            };
            if id.is_empty() || !col["games"][&id].is_object() {
                return json!({"ok": false, "error": "No active game to rename"});
            }
            col["games"][&id]["name"] = json!(name);
            col["games"][&id]["updated"] = json!(now_ts());
            write_games(&col);
            json!({"ok": true, "action": "rename", "game_id": id, "name": name})
        }

        // ── delete: delete a game by id ──
        "delete" => {
            let id = args.get(1).and_then(|v| v.as_str()).unwrap_or("");
            if id.is_empty() {
                return json!({"ok": false, "error": "Game ID required"});
            }
            let mut col = read_games();
            if let Some(obj) = col["games"].as_object_mut() {
                obj.remove(id);
            }
            // If we deleted the active game, pick the next one
            if col["active"].as_str() == Some(id) {
                let next = col["games"].as_object()
                    .and_then(|m| m.keys().next().map(|k| k.clone()));
                col["active"] = match next {
                    Some(k) => json!(k),
                    None => json!(null),
                };
            }
            write_games(&col);
            json!({"ok": true, "action": "delete", "deleted": id})
        }

        // ── path: backward compat ──
        "path" => {
            json!({"ok": true, "vfs_path": "canvas/app.html", "games_path": GAMES_JSON_PATH})
        }

        // ── Legacy aliases ──
        "save" => {
            // save <name>: rename active game
            let name = args.get(1).and_then(|v| v.as_str()).unwrap_or("");
            if name.is_empty() {
                return json!({"ok": false, "error": "Name required"});
            }
            let mut col = read_games();
            let _ = ensure_internal_active(&mut col);
            let id = col["active"].as_str().unwrap_or("").to_string();
            if id.is_empty() || !col["games"][&id].is_object() {
                return json!({"ok": false, "error": "No active game"});
            }
            col["games"][&id]["name"] = json!(name);
            col["games"][&id]["updated"] = json!(now_ts());
            write_games(&col);
            json!({"ok": true, "action": "save", "game_id": id, "name": name})
        }
        "load" => {
            // load <id>: activate game by id
            let id = args.get(1).and_then(|v| v.as_str()).unwrap_or("");
            canvas(&[json!("activate"), json!(id)])
        }
        "projects" => {
            // Legacy alias for games
            canvas(&[json!("games")])
        }
        "delete_project" => {
            let id = args.get(1).and_then(|v| v.as_str()).unwrap_or("");
            canvas(&[json!("delete"), json!(id)])
        }

        _ => json!({"ok": false, "error": format!("Unknown action: {}. Use: set, append, get, clear, new, games, activate, fork, rename, delete", action)}),
    }
}
