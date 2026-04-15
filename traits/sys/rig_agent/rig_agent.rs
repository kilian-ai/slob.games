use serde_json::{json, Value};

pub fn rig_agent(args: &[Value]) -> Value {
    if args.is_empty() {
        return json!({ "ok": false, "error": "config required" });
    }
    
    let config_str = match args[0].as_str() {
        Some(s) => s,
        None => return json!({ "ok": false, "error": "config must be string" }),
    };

    let config: Value = match serde_json::from_str(config_str) {
        Ok(v) => v,
        Err(e) => return json!({ "ok": false, "error": format!("JSON parse error: {}", e) }),
    };

    match config.get("action").and_then(|a| a.as_str()).unwrap_or("query") {
        "create" => {
            let model = config.get("model")
                .and_then(|m| m.as_str())
                .unwrap_or("gpt-4o-mini");
            let session_id = format!("agent_{}", 
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos() as u32)
                    .unwrap_or(0)
            );
            
            json!({
                "ok": true,
                "sessionId": session_id,
                "model": model,
                "ready": true
            })
        }
        "query" => {
            let session_id = config.get("sessionId")
                .and_then(|s| s.as_str())
                .unwrap_or("");
            let message = config.get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("hello");
            
            if session_id.is_empty() {
                return json!({ "ok": false, "error": "sessionId required" });
            }
            
            let system_prompt = config.get("system_prompt")
                .and_then(|p| p.as_str())
                .unwrap_or("You are helpful");
            let tools_str = config.get("tools")
                .and_then(|t| t.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join(",")
                })
                .unwrap_or_default();
            let model = config.get("model")
                .and_then(|m| m.as_str())
                .unwrap_or("gpt-4o-mini");
            
            match kernel_logic::platform::dispatch(
                "llm.agent",
                &[
                    Value::String(message.to_string()),
                    Value::String(system_prompt.to_string()),
                    Value::String(tools_str),
                    Value::String(model.to_string()),
                    Value::Number(10.into()),
                ],
            ) {
                Some(result) => {
                    if let Some(resp) = result.get("response") {
                        json!({"ok": true, "response": resp})
                    } else if let Some(err) = result.get("error") {
                        json!({"ok": false, "error": err})
                    } else {
                        json!({"ok": true, "response": result})
                    }
                }
                None => json!({"ok": false, "error": "llm.agent dispatch failed"}),
            }
        }
        "stop" => {
            let session_id = config.get("sessionId")
                .and_then(|s| s.as_str())
                .unwrap_or("");
            
            if session_id.is_empty() {
                return json!({ "ok": false, "error": "sessionId required" });
            }
            
            json!({
                "ok": true,
                "sessionId": session_id,
                "stopped": true
            })
        }
        action => json!({ "ok": false, "error": format!("unknown action: {}", action) }),
    }
}
