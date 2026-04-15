use rig::client::embeddings::EmbeddingsClient;
use rig::providers::openai;
use serde_json::{json, Value};

#[path = "../common.rs"]
mod common;

pub fn rig_openai_embed(args: &[Value]) -> Value {
    let input = match args.first().and_then(|v| v.as_str()) {
        Some(value) if !value.is_empty() => value,
        _ => return json!({ "ok": false, "error": "input is required" }),
    };

    let model = args.get(1)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("text-embedding-3-small");

    let client: openai::Client = openai::Client::new("shim").unwrap();
    let _embedding_model = client.embedding_model(model);
    let result = common::call_openai_embeddings(input, model);
    let ok = result.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
    if !ok {
        let error = result.get("body")
            .and_then(|b| b.get("error"))
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .or_else(|| result.get("error").and_then(|e| e.as_str()))
            .unwrap_or("OpenAI embeddings API call failed");
        return json!({ "ok": false, "error": error, "provider": "rig.openai.embed" });
    }

    let body = result.get("body").cloned().unwrap_or(Value::Null);
    let embedding = body.pointer("/data/0/embedding")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    json!({
        "ok": true,
        "provider": "rig.openai.embed",
        "model": model,
        "dimensions": embedding.len(),
        "embedding": embedding,
        "rig_shim": true
    })
}