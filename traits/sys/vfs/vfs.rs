use serde_json::{json, Value};

/// VFS operations: read, write, list, delete files from the persistent virtual filesystem.
///
/// On WASM: backed by a global VFS with auto-persistence to localStorage.
/// On native: backed by the `data/vfs/` directory + project root for reads.
///
/// Actions:
///   read  <path>            — read file content
///   write <path> <content>  — write file content
///   append <path> <content> — append content to file
///   list  [prefix]          — list files (optional prefix filter)
///   list_dirs [prefix]      — list directories (optional prefix filter)
///   delete <path>           — delete a file
///   exists <path>           — check if file exists
///   is_dir <path>           — check if path is a directory
///   mkdir <path>            — create a directory
///   stat <path>             — file timestamps (created/modified)
pub fn vfs(args: &[Value]) -> Value {
    let action = args.get(0).and_then(|v| v.as_str()).unwrap_or("list");
    let path = args.get(1).and_then(|v| v.as_str()).unwrap_or("");
    let content = args.get(2).and_then(|v| v.as_str()).unwrap_or("");
    let mut vfs = kernel_logic::platform::make_vfs();

    eprintln!("[vfs] action={} path={} content_len={}", action, path, content.len());

    match action {
        "read" => {
            if path.is_empty() {
                return json!({"ok": false, "error": "Path required"});
            }
            match vfs.read(path) {
                Some(data) => {
                    eprintln!("[vfs] read OK path={} len={}", path, data.len());
                    json!({"ok": true, "path": path, "content": data})
                }
                None => {
                    eprintln!("[vfs] read FAIL path={} (not found)", path);
                    json!({"ok": false, "error": format!("File not found: {}", path)})
                }
            }
        }
        "write" => {
            if path.is_empty() {
                return json!({"ok": false, "error": "Path required"});
            }
            eprintln!("[vfs] write path={} bytes={}", path, content.len());
            vfs.write(path, content);
            json!({"ok": true, "path": path, "bytes": content.len()})
        }
        "append" => {
            if path.is_empty() {
                return json!({"ok": false, "error": "Path required"});
            }
            vfs.append(path, content);
            json!({"ok": true, "path": path, "bytes": content.len()})
        }
        "list" => {
            let all = vfs.list();
            let filtered: Vec<&str> = if path.is_empty() {
                all.iter().map(|s| s.as_str()).collect()
            } else {
                let prefix = path.trim_end_matches('/');
                all.iter()
                    .filter(|f| f.starts_with(prefix))
                    .map(|s| s.as_str())
                    .collect()
            };
            json!({"ok": true, "files": filtered, "count": filtered.len()})
        }
        "list_dirs" => {
            let all = vfs.list_dirs();
            let filtered: Vec<&str> = if path.is_empty() {
                all.iter().map(|s| s.as_str()).collect()
            } else {
                let prefix = path.trim_end_matches('/');
                all.iter()
                    .filter(|d| d.starts_with(prefix))
                    .map(|s| s.as_str())
                    .collect()
            };
            json!({"ok": true, "dirs": filtered, "count": filtered.len()})
        }
        "delete" => {
            if path.is_empty() {
                return json!({"ok": false, "error": "Path required"});
            }
            let deleted = vfs.delete(path);
            json!({"ok": true, "deleted": deleted, "path": path})
        }
        "exists" => {
            if path.is_empty() {
                return json!({"ok": false, "error": "Path required"});
            }
            let exists = vfs.exists(path);
            json!({"ok": true, "exists": exists, "path": path})
        }
        "is_dir" => {
            if path.is_empty() {
                return json!({"ok": false, "error": "Path required"});
            }
            let is_dir = vfs.is_dir(path);
            json!({"ok": true, "is_dir": is_dir, "path": path})
        }
        "mkdir" => {
            if path.is_empty() {
                return json!({"ok": false, "error": "Path required"});
            }
            let created = vfs.mkdir(path);
            json!({"ok": true, "created": created, "path": path})
        }
        "stat" => {
            if path.is_empty() {
                return json!({"ok": false, "error": "Path required"});
            }
            match vfs.stat(path) {
                Some((created, modified)) => json!({"ok": true, "path": path, "created": created, "modified": modified}),
                None => json!({"ok": false, "error": format!("Not found: {}", path)}),
            }
        }
        _ => json!({"ok": false, "error": format!("Unknown action: {}. Use: read, write, append, list, list_dirs, delete, exists, is_dir, mkdir, stat", action)}),
    }
}
