use serde_json::{json, Value};

pub fn read_context_docs(patterns_csv: &str) -> Vec<String> {
    let patterns: Vec<&str> = patterns_csv
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if patterns.is_empty() {
        return Vec::new();
    }

    let vfs = kernel_logic::platform::make_vfs();
    let all_paths = vfs.list();
    let mut docs = Vec::new();
    let mut seen = std::collections::BTreeSet::new();

    for pattern in patterns {
        if let Some(content) = vfs.read(pattern) {
            if seen.insert(pattern.to_string()) {
                docs.push(content);
            }
            continue;
        }

        for path in &all_paths {
            if simple_glob_match(pattern, path) {
                if let Some(content) = vfs.read(path) {
                    if seen.insert(path.to_string()) {
                        docs.push(content);
                    }
                }
            }
        }
    }

    docs
}

pub fn format_context_block(docs: &[String]) -> String {
    if docs.is_empty() {
        return String::new();
    }

    let mut out = String::from("<context>\n");
    for (index, doc) in docs.iter().enumerate() {
        out.push_str(&format!("<doc index=\"{}\">\n{}\n</doc>\n", index, doc));
    }
    out.push_str("</context>");
    out
}

pub fn call_openai_chat(messages: Vec<Value>, model: &str) -> Value {
    let body = json!({
        "model": model,
        "messages": messages,
    });

    kernel_logic::platform::dispatch(
        "sys.call",
        &[
            Value::String("https://api.openai.com/v1/chat/completions".into()),
            body,
            Value::String("openai_api_key".into()),
            Value::String("POST".into()),
            Value::Null,
        ],
    )
    .unwrap_or_else(|| json!({ "ok": false, "error": "sys.call not available" }))
}

pub fn call_bearer_chat(
    url: &str,
    messages: Vec<Value>,
    model: &str,
    secret_id: &str,
) -> Value {
    let body = json!({
        "model": model,
        "messages": messages,
    });

    kernel_logic::platform::dispatch(
        "sys.call",
        &[
            Value::String(url.to_string()),
            body,
            Value::String(secret_id.to_string()),
            Value::String("POST".into()),
            Value::Null,
        ],
    )
    .unwrap_or_else(|| json!({ "ok": false, "error": "sys.call not available" }))
}

pub fn call_json_with_headers(url: &str, body: Value, headers: Value) -> Value {
    kernel_logic::platform::dispatch(
        "sys.call",
        &[
            Value::String(url.to_string()),
            body,
            Value::Null,
            Value::String("POST".into()),
            headers,
        ],
    )
    .unwrap_or_else(|| json!({ "ok": false, "error": "sys.call not available" }))
}

pub fn get_secret_any(secret_id: &str) -> Option<String> {
    let upper = secret_id.to_uppercase();
    let lower = secret_id.to_lowercase();

    kernel_logic::platform::secret_get(secret_id)
        .or_else(|| kernel_logic::platform::secret_get(&upper))
        .or_else(|| kernel_logic::platform::secret_get(&lower))
}

pub fn extract_error(result: &Value, fallback: &str) -> String {
    result.get("body")
        .and_then(|b| b.get("error"))
        .and_then(|e| e.get("message").or_else(|| e.get("error")))
        .and_then(|m| m.as_str())
        .or_else(|| result.get("error").and_then(|e| e.as_str()))
        .unwrap_or(fallback)
        .to_string()
}

pub fn call_openai_embeddings(input: &str, model: &str) -> Value {
    let body = json!({
        "model": model,
        "input": input,
    });

    kernel_logic::platform::dispatch(
        "sys.call",
        &[
            Value::String("https://api.openai.com/v1/embeddings".into()),
            body,
            Value::String("openai_api_key".into()),
            Value::String("POST".into()),
            Value::Null,
        ],
    )
    .unwrap_or_else(|| json!({ "ok": false, "error": "sys.call not available" }))
}

fn simple_glob_match(pattern: &str, text: &str) -> bool {
    glob_match_inner(pattern.as_bytes(), text.as_bytes())
}

fn glob_match_inner(pattern: &[u8], text: &[u8]) -> bool {
    if pattern.is_empty() {
        return text.is_empty();
    }
    match pattern[0] {
        b'*' => {
            glob_match_inner(&pattern[1..], text)
                || (!text.is_empty() && glob_match_inner(pattern, &text[1..]))
        }
        b'?' => !text.is_empty() && glob_match_inner(&pattern[1..], &text[1..]),
        c => !text.is_empty() && c == text[0] && glob_match_inner(&pattern[1..], &text[1..]),
    }
}