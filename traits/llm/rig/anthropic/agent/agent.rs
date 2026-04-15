use rig::client::CompletionClient;
use rig::providers::anthropic;
use serde_json::{json, Value};

#[path = "../../openai/common.rs"]
mod common;

pub fn rig_anthropic_agent(args: &[Value]) -> Value {
    let prompt = match args.first().and_then(|v| v.as_str()) {
        Some(value) if !value.is_empty() => value,
        _ => return json!({ "ok": false, "error": "prompt is required" }),
    };

    let model = args.get(1)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("claude-sonnet-4-5");

    let preamble = args.get(2)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("");

    let context_csv = args.get(3)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("");

    let temperature = args.get(4).and_then(|v| v.as_f64());

    let docs = common::read_context_docs(context_csv);
    let client: anthropic::Client = anthropic::Client::new("shim").unwrap();

    let mut builder = client.agent(model);
    if !preamble.is_empty() {
        builder = builder.preamble(preamble);
    }
    for doc in &docs {
        builder = builder.context(doc);
    }
    if let Some(value) = temperature {
        builder = builder.temperature(value);
    }
    let _agent = builder.build();

    let context_block = common::format_context_block(&docs);
    let user_text = if context_block.is_empty() {
        prompt.to_string()
    } else {
        format!("{}\n\n{}", context_block, prompt)
    };

    let mut body = json!({
        "model": model,
        "max_tokens": 1024,
        "messages": [
            { "role": "user", "content": user_text }
        ]
    });

    if !preamble.is_empty() {
        body["system"] = Value::String(preamble.to_string());
    }
    if let Some(value) = temperature {
        body["temperature"] = json!(value);
    }

    let api_key = match common::get_secret_any("anthropic_api_key") {
        Some(v) if !v.is_empty() => v,
        _ => {
            return json!({
                "ok": false,
                "error": "Missing secret anthropic_api_key",
                "provider": "rig.anthropic.agent"
            })
        }
    };

    let headers = json!({
        "x-api-key": api_key,
        "anthropic-version": "2023-06-01"
    });

    let result = common::call_json_with_headers("https://api.anthropic.com/v1/messages", body, headers);
    let ok = result.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
    if !ok {
        let error = common::extract_error(&result, "Anthropic API call failed");
        return json!({ "ok": false, "error": error, "provider": "rig.anthropic.agent" });
    }

    let body = result.get("body").cloned().unwrap_or(Value::Null);
    let content = body.pointer("/content/0/text")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    json!({
        "ok": true,
        "provider": "rig.anthropic.agent",
        "model": model,
        "content": content,
        "context_count": docs.len(),
        "rig_shim": true
    })
}
