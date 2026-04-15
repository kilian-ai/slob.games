use serde_json::{json, Value};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue};

/// sys.lua — Execute Lua scripts in the browser runtime.
///
/// Args:
///   [code, input?]
///
/// - code: Lua source code string.
/// - input: Optional JSON value serialized and exposed to Lua as
///   `__traits_input_json` (string) and primitive globals for top-level scalar keys.
///
/// Returns:
///   {
///     ok: bool,
///     stdout: [string],
///     stderr: [string],
///     result: any,
///     error: string?
///   }
pub fn lua(args: &[Value]) -> Value {
    let code = match args.first().and_then(|v| v.as_str()) {
        Some(c) if !c.trim().is_empty() => c,
        _ => return json!({ "ok": false, "error": "code is required" }),
    };

    let input = args.get(1).cloned().unwrap_or_else(|| json!({}));

    #[cfg(target_arch = "wasm32")]
    {
        return run_in_browser_lua(code, &input);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = input;
        json!({
            "ok": false,
            "error": "sys.lua is only available in the browser WASM runtime"
        })
    }
}

#[cfg(target_arch = "wasm32")]
fn run_in_browser_lua(code: &str, input: &Value) -> Value {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return json!({ "ok": false, "error": "window is unavailable" }),
    };

    let runner = match js_sys::Reflect::get(&window, &JsValue::from_str("__traitsLuaRun")) {
        Ok(v) => v,
        Err(_) => return json!({ "ok": false, "error": "Lua runtime bridge not found" }),
    };

    if !runner.is_function() {
        return json!({ "ok": false, "error": "Lua runtime bridge is not callable" });
    }

    let func: js_sys::Function = runner.unchecked_into();
    let input_json = input.to_string();

    let out = match func.call2(
        &JsValue::NULL,
        &JsValue::from_str(code),
        &JsValue::from_str(&input_json),
    ) {
        Ok(v) => v,
        Err(e) => {
            let err = e.as_string().unwrap_or_else(|| "Lua runtime call failed".to_string());
            return json!({ "ok": false, "error": err });
        }
    };

    let out_str = out
        .as_string()
        .unwrap_or_else(|| "{\"ok\":false,\"error\":\"invalid lua runtime response\"}".to_string());

    match serde_json::from_str::<Value>(&out_str) {
        Ok(v) => v,
        Err(e) => json!({ "ok": false, "error": format!("invalid lua result json: {e}") }),
    }
}
