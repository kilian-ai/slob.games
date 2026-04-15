use rig::client::CompletionClient;
use rig::client::embeddings::EmbeddingsClient;
use rig::providers::openai;
use serde_json::{json, Value};

pub fn rig_providers(_args: &[Value]) -> Value {
    // Validate that rig-core is linked and its OpenAI builder surface is usable in WASM.
    let client: openai::Client = openai::Client::new("shim").unwrap();
    let _agent = client.clone().agent("gpt-4o-mini").build();
    let _embedding_model = client.embedding_model("text-embedding-3-small");

    json!({
        "ok": true,
        "rig_core": true,
        "wasm_shim": true,
        "providers": [
            {
                "provider": "openai",
                "chat_model": "gpt-4o-mini",
                "embedding_model": "text-embedding-3-small",
                "traits": ["llm.rig.openai.agent", "llm.rig.openai.embed"]
            },
            {
                "provider": "anthropic",
                "chat_model": "claude-sonnet-4-5",
                "note": "available in rig-core, not yet shimmed here"
            },
            {
                "provider": "groq",
                "chat_model": "llama-3.1-70b-versatile",
                "note": "available in rig-core, not yet shimmed here"
            },
            {
                "provider": "openrouter",
                "chat_model": "anthropic/claude-3.5-sonnet",
                "note": "available in rig-core, not yet shimmed here"
            },
            {
                "provider": "together",
                "chat_model": "meta-llama/Meta-Llama-3.1-70B-Instruct-Turbo",
                "note": "available in rig-core, not yet shimmed here"
            }
        ]
    })
}