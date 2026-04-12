use serde_json::{json, Value};

/// llm.image — Generate game sprites/assets via OpenAI image API.
///
/// Wraps user prompts with mode-specific style directives inspired by
/// character-sheet pipelines (VNCCS, etc). Modes:
///   sprite — single sprite, pixel art, transparent bg (default)
///   sheet  — 2x2 character sheet: front/back/left/right or 4 expressions
///   icon   — clean UI icon, flat design, no background
///   bg     — seamless/panoramic game background
///   tile   — tileable texture for terrain/walls
///
/// Args: [prompt, path?, size?, model?, mode?]
pub fn image(args: &[Value]) -> Value {
    let prompt = match args.first().and_then(|v| v.as_str()) {
        Some(p) if !p.is_empty() => p,
        _ => return json!({ "ok": false, "error": "prompt is required" }),
    };

    let vfs_path = args.get(1)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty());

    let size_hint = args.get(2)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty());

    let model = args.get(3)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("dall-e-2");

    let mode = args.get(4)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("sprite");

    // Pick default size per mode if not explicitly given
    let size = size_hint.unwrap_or(match mode {
        "sheet" => "1024x1024",
        "bg"    => "1024x1024",
        "tile"  => "512x512",
        "icon"  => "256x256",
        _       => "256x256",   // sprite
    });

    let api_size = normalize_size(size, model);

    // ── Prompt engineering per mode ──
    let enhanced = enhance_prompt(prompt, mode);

    // Auto-generate VFS path if not provided
    let path = match vfs_path {
        Some(p) => p.to_string(),
        None => {
            let slug: String = prompt.chars()
                .filter(|c| c.is_alphanumeric() || *c == ' ')
                .take(30)
                .collect::<String>()
                .trim()
                .replace(' ', "_")
                .to_lowercase();
            let prefix = match mode {
                "sheet" => "sheets",
                "icon"  => "icons",
                "bg"    => "backgrounds",
                "tile"  => "tiles",
                _       => "sprites",
            };
            format!("{}/{}.png", prefix, if slug.is_empty() { "generated" } else { &slug })
        }
    };

    eprintln!("[llm.image] mode={} prompt={:?} enhanced={:?} path={} size={} model={}",
              mode, prompt, enhanced, path, api_size, model);

    // Build the API request
    let mut body = json!({
        "model": model,
        "prompt": enhanced,
        "n": 1,
        "size": api_size,
    });

    if model.starts_with("dall-e") {
        body["response_format"] = json!("b64_json");
    }

    let call_args = vec![
        Value::String("https://api.openai.com/v1/images/generations".into()),
        body,
        Value::String("openai_api_key".into()),
        Value::String("POST".into()),
        Value::Null,
    ];

    let result = kernel_logic::platform::dispatch("sys.call", &call_args)
        .unwrap_or_else(|| json!({"ok": false, "error": "sys.call not available"}));

    let ok = result.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
    if !ok {
        let error = result.get("body")
            .and_then(|b| b.get("error"))
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or("OpenAI image API call failed");
        return json!({ "ok": false, "error": error });
    }

    let resp_body = result.get("body").cloned().unwrap_or(Value::Null);

    // ── Extract image data ──

    // Try b64_json first (dall-e-2, dall-e-3)
    if let Some(b64_data) = resp_body.pointer("/data/0/b64_json").and_then(|v| v.as_str()) {
        if !b64_data.is_empty() {
            let data_url = format!("data:image/png;base64,{}", b64_data);
            kernel_logic::platform::vfs_write(&path, &data_url);
            eprintln!("[llm.image] saved b64 to VFS path={} bytes={}", path, data_url.len());

            let mut r = json!({
                "ok": true,
                "path": path,
                "data_url_length": data_url.len(),
                "format": "data_url",
                "size": api_size,
                "mode": mode,
                "enhanced_prompt": enhanced
            });
            if mode == "sheet" {
                r["usage_hint"] = json!(sheet_usage_hint(&path));
            }
            return r;
        }
    }

    // URL fallback (gpt-image-1)
    if let Some(url) = resp_body.pointer("/data/0/url").and_then(|v| v.as_str()) {
        if !url.is_empty() {
            kernel_logic::platform::vfs_write(&path, url);
            eprintln!("[llm.image] saved URL to VFS path={}", path);

            let mut r = json!({
                "ok": true,
                "path": path,
                "url": url,
                "format": "url",
                "size": api_size,
                "mode": mode,
                "enhanced_prompt": enhanced
            });
            if mode == "sheet" {
                r["usage_hint"] = json!(sheet_usage_hint(&path));
            }
            return r;
        }
    }

    json!({ "ok": false, "error": "No image data in API response" })
}

// ─────────────────────────────────────────────────────────────
// Prompt engineering — wraps raw user description with style
// directives tuned for each asset mode.
//
// Inspired by VNCCS character-sheet approach: generating multiple
// consistent views in a single image for cross-sprite coherence.
// ─────────────────────────────────────────────────────────────

fn enhance_prompt(raw: &str, mode: &str) -> String {
    match mode {
        "sheet" => format!(
            "Character reference sheet, 2x2 grid layout on a plain white background. \
             Each cell shows the same character in a different pose: front view, back view, \
             left side view, right side view. Consistent proportions, colors, and details \
             across all four cells. Clean separation between cells with thin dividing lines. \
             Character: {}. \
             Style: clean digital illustration, flat colors, sharp outlines, \
             full body visible in each cell, centered in frame, game sprite art.",
            raw
        ),
        "icon" => format!(
            "Game UI icon, flat design, centered subject, no background, \
             clean vector style, bold outlines, vibrant colors, \
             simple and instantly recognizable at small sizes. Subject: {}.",
            raw
        ),
        "bg" => format!(
            "Game background scene, wide landscape composition, atmospheric perspective, \
             no characters or UI elements, seamless horizontal scrolling friendly, \
             rich detail, painted style. Scene: {}.",
            raw
        ),
        "tile" => format!(
            "Seamless tileable texture pattern, top-down view, \
             edges wrap perfectly for tiling in all directions, \
             consistent lighting with no directional shadows, game terrain style. \
             Texture: {}.",
            raw
        ),
        _ => {
            // sprite mode — check if user already specified style keywords
            let lower = raw.to_lowercase();
            let has_style = lower.contains("pixel art")
                || lower.contains("sprite")
                || lower.contains("vector")
                || lower.contains("flat")
                || lower.contains("illustration");
            let has_bg = lower.contains("transparent")
                || lower.contains("background")
                || lower.contains("no bg");

            let mut parts = Vec::new();
            if !has_style {
                parts.push("Pixel art game sprite, clean sharp pixels, limited color palette");
            }
            parts.push(raw);
            if !has_bg {
                parts.push("on a plain solid-color background for easy extraction");
            }
            parts.push("centered in frame, full subject visible");
            parts.join(". ")
        }
    }
}

/// JS code hint for slicing a 2x2 character sheet into individual sprites.
fn sheet_usage_hint(vfs_path: &str) -> String {
    format!(
        "This is a 2x2 character sheet. Slice it in your game JS:\n\
         ```\n\
         const resp = await traits.call('sys.vfs', ['read', '{}']);\n\
         const sheet = new Image();\n\
         sheet.src = resp.content;\n\
         sheet.onload = () => {{\n\
         const W = sheet.width / 2, H = sheet.height / 2;\n\
         ['front','back','left','right'].forEach((dir, i) => {{\n\
           const c = document.createElement('canvas');\n\
           c.width = W; c.height = H;\n\
           const x = (i % 2) * W, y = Math.floor(i / 2) * H;\n\
           c.getContext('2d').drawImage(sheet, x, y, W, H, 0, 0, W, H);\n\
           sprites[dir] = c;  // use as drawImage source\n\
         }});\n\
         }};\n\
         ```",
        vfs_path
    )
}

/// Normalize the requested size to a valid API size for the given model.
fn normalize_size(requested: &str, model: &str) -> &'static str {
    let parts: Vec<&str> = requested.split('x').collect();
    let max_dim = parts.iter()
        .filter_map(|p| p.parse::<u32>().ok())
        .max()
        .unwrap_or(256);

    if model == "dall-e-2" {
        if max_dim <= 256 { "256x256" }
        else if max_dim <= 512 { "512x512" }
        else { "1024x1024" }
    } else if model == "dall-e-3" {
        "1024x1024"
    } else {
        if requested == "auto" { "auto" }
        else { "1024x1024" }
    }
}
