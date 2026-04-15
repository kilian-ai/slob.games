use rig::client::CompletionClient;
use rig::providers::groq;
use serde_json::{json, Value};

#[path = "../../openai/common.rs"]
mod common;

pub fn rig_groq_agent(args: &[Value]) -> Value {
    let prompt = match args.first().and_then(|v| v.as_str()) {
        Some(value) if !value.is_empty() => value,
        _ => return json!({ "ok": false, "error": "prompt is required" }),
    };

    let model = args.get(1)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("llama-3.1-70b-versatile");

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
    let client: groq::Client = groq::Client::new("shim").unwrap();

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

    let mut messages = Vec::new();
    if !preamble.is_empty() {
        messages.push(json!({ "role": "system", "content": preamble }));
    }
    let context_block = common::format_context_block(&docs);
    if !context_block.is_empty() {
        messages.push(json!({ "role": "system", "content": context_block }));
    }
    messages.push(json!({ "role": "user", "content": prompt }));

    let result = common::call_bearer_chat(
        "https://api.groq.com/openai/v1/chat/completions",
        messages,
        model,
        "groq_api_key",
    );
    let ok = result.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
    if !ok {
        let error = common::extract_error(&result, "Groq API call failed");
        return json!({ "ok": false, "error": error, "provider": "rig.groq.agent" });
    }

    let body = result.get("body").cloned().unwrap_or(Value::Null);
    let content = body.pointer("/choices/0/message/content")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    json!({
        "ok": true,
        "provider": "rig.groq.agent",
        "model": model,
        "content": content,
        "context_count": docs.len(),
        "rig_shim": true
    })
}
