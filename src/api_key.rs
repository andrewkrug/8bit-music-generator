use std::path::PathBuf;

use anyhow::{anyhow, Result};
use serde::Deserialize;
use tokio::sync::oneshot;

#[derive(Deserialize)]
struct Config {
    gemini_api_key: Option<String>,
}

fn config_path() -> PathBuf {
    dirs_or_default().join("music-generator.toml")
}

fn dirs_or_default() -> PathBuf {
    std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")))
}

fn load_from_config() -> Option<String> {
    let path = config_path();
    let content = std::fs::read_to_string(&path).ok()?;
    let config: Config = toml::from_str(&content).ok()?;
    let key = config.gemini_api_key?;
    if key.is_empty() {
        None
    } else {
        Some(key)
    }
}

fn mask_key(key: &str) -> String {
    if key.len() <= 8 {
        return "****".into();
    }
    format!("{}...{}", &key[..4], &key[key.len() - 4..])
}

/// Resolve the Gemini API key from (in order):
/// 1. `GEMINI_API_KEY` environment variable
/// 2. Config file at `~/.config/music-generator.toml`
/// 3. Interactive web prompt (opens browser, user pastes key)
pub async fn resolve_api_key() -> Result<String> {
    // 1. Environment variable
    if let Ok(key) = std::env::var("GEMINI_API_KEY") {
        if !key.is_empty() {
            let masked = mask_key(&key);
            tracing::info!(key = %masked, "API key found via GEMINI_API_KEY environment variable");
            return Ok(key);
        }
        tracing::debug!("GEMINI_API_KEY is set but empty, skipping");
    } else {
        tracing::debug!("GEMINI_API_KEY environment variable not set");
    }

    // 2. Config file
    let cfg = config_path();
    if let Some(key) = load_from_config() {
        let masked = mask_key(&key);
        tracing::info!(key = %masked, path = %cfg.display(), "API key found in config file");
        return Ok(key);
    }
    tracing::info!(path = %cfg.display(), "No config file found or no key in config");

    // 3. Web prompt — key will be held in memory only
    tracing::warn!("No API key found in environment or config file");
    tracing::info!(
        "Launching browser to collect API key (will be stored in memory only, not persisted to disk)"
    );
    prompt_via_browser().await
}

const HTML_PAGE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>8-Bit Music Generator - API Key Setup</title>
<style>
  * { box-sizing: border-box; margin: 0; padding: 0; }
  body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background: #0f172a; color: #e2e8f0;
    display: flex; justify-content: center; align-items: center;
    min-height: 100vh; padding: 1rem;
  }
  .card {
    background: #1e293b; border-radius: 12px; padding: 2rem;
    max-width: 480px; width: 100%; box-shadow: 0 4px 24px rgba(0,0,0,0.4);
  }
  h1 { font-size: 1.5rem; margin-bottom: 0.5rem; color: #f8fafc; }
  p { font-size: 0.9rem; color: #94a3b8; margin-bottom: 1.5rem; line-height: 1.5; }
  label { font-size: 0.85rem; color: #cbd5e1; display: block; margin-bottom: 0.5rem; }
  input[type="password"] {
    width: 100%; padding: 0.75rem; border-radius: 8px;
    border: 1px solid #334155; background: #0f172a; color: #f8fafc;
    font-size: 1rem; margin-bottom: 1rem;
  }
  input:focus { outline: none; border-color: #22d3ee; }
  button {
    width: 100%; padding: 0.75rem; border-radius: 8px; border: none;
    background: #22d3ee; color: #0f172a; font-size: 1rem; font-weight: 600;
    cursor: pointer; transition: background 0.2s;
  }
  button:hover { background: #06b6d4; }
  .hint { font-size: 0.8rem; color: #64748b; margin-top: 1rem; }
  .success { text-align: center; color: #34d399; font-size: 1.1rem; padding: 2rem 0; }
  .error { color: #f87171; font-size: 0.85rem; margin-bottom: 0.5rem; display: none; }
</style>
</head>
<body>
<div class="card">
  <h1>8-Bit Music Generator</h1>
  <p>Enter your Google Gemini API key to enable Lyria 3 music generation.
     The key is stored in memory only and never persisted to disk.</p>
  <div id="form-view">
    <label for="key">Gemini API Key</label>
    <div id="error" class="error">Please enter a valid API key.</div>
    <input type="password" id="key" placeholder="AIza..." autofocus>
    <button onclick="submit()">Start Server</button>
    <p class="hint">Get a key at <a href="https://aistudio.google.com/apikey" target="_blank" style="color:#22d3ee">aistudio.google.com/apikey</a></p>
  </div>
  <div id="success-view" style="display:none">
    <div class="success">API key received. You can close this tab.</div>
  </div>
</div>
<script>
async function submit() {
  const key = document.getElementById('key').value.trim();
  if (!key) {
    document.getElementById('error').style.display = 'block';
    return;
  }
  try {
    const res = await fetch('/api/key', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ key })
    });
    if (res.ok) {
      document.getElementById('form-view').style.display = 'none';
      document.getElementById('success-view').style.display = 'block';
    }
  } catch (e) {
    document.getElementById('error').textContent = 'Failed to submit. Is the server running?';
    document.getElementById('error').style.display = 'block';
  }
}
document.getElementById('key').addEventListener('keydown', e => {
  if (e.key === 'Enter') submit();
});
</script>
</body>
</html>"#;

async fn prompt_via_browser() -> Result<String> {
    use axum::{
        Json,
        Router,
        response::Html,
        routing::{get, post},
    };

    let (tx, rx) = oneshot::channel::<String>();
    let tx = std::sync::Arc::new(tokio::sync::Mutex::new(Some(tx)));

    let tx_clone = tx.clone();
    let app = Router::new()
        .route("/", get(|| async { Html(HTML_PAGE) }))
        .route(
            "/api/key",
            post(move |Json(body): Json<serde_json::Value>| {
                let tx = tx_clone.clone();
                async move {
                    let key = body
                        .get("key")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    if let Some(sender) = tx.lock().await.take() {
                        let _ = sender.send(key);
                    }
                    axum::http::StatusCode::OK
                }
            }),
        );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let url = format!("http://{addr}");

    tracing::info!(url = %url, "Opening browser for API key entry");
    eprintln!("Opening {url} in your browser to collect your Gemini API key...");
    eprintln!("If the browser doesn't open, navigate to {url} manually.");

    let _ = open::that(&url);

    let server = axum::serve(listener, app);
    tokio::spawn(async move {
        let _ = server.await;
    });

    let key = rx
        .await
        .map_err(|_| anyhow!("API key prompt was cancelled"))?;
    if key.is_empty() {
        return Err(anyhow!("Empty API key provided"));
    }

    Ok(key)
}
