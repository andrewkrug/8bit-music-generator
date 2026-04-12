use std::path::PathBuf;

use base64::Engine;
use serde_json::json;
use wiremock::matchers::{header, method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

use music_generator::lyria::LyriaClient;
use music_generator::server::MusicGeneratorServer;

/// Verifies the server struct can be constructed (no API call made).
#[test]
fn server_construction() {
    let client = LyriaClient::new("test-key".into());
    let _server = MusicGeneratorServer::new(client, PathBuf::from("/tmp/music"));
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Minimal valid WAV file — 1 sample of silence.
fn tiny_wav() -> Vec<u8> {
    let mut buf = Vec::new();
    // RIFF header
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&44u32.to_le_bytes()); // file size - 8
    buf.extend_from_slice(b"WAVE");
    // fmt chunk
    buf.extend_from_slice(b"fmt ");
    buf.extend_from_slice(&16u32.to_le_bytes()); // chunk size
    buf.extend_from_slice(&1u16.to_le_bytes()); // PCM
    buf.extend_from_slice(&1u16.to_le_bytes()); // mono
    buf.extend_from_slice(&44100u32.to_le_bytes()); // sample rate
    buf.extend_from_slice(&88200u32.to_le_bytes()); // byte rate
    buf.extend_from_slice(&2u16.to_le_bytes()); // block align
    buf.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
    // data chunk
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&2u32.to_le_bytes()); // data size
    buf.extend_from_slice(&0i16.to_le_bytes()); // one silent sample
    buf
}

fn lyria_success_response(audio_bytes: &[u8], description: &str) -> serde_json::Value {
    let b64 = base64::engine::general_purpose::STANDARD.encode(audio_bytes);
    json!({
        "candidates": [{
            "content": {
                "parts": [
                    { "text": description },
                    { "inlineData": { "mimeType": "audio/wav", "data": b64 } }
                ]
            }
        }]
    })
}

async fn setup_mock() -> (MockServer, LyriaClient) {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path_regex(r"/lyria-.*:generateContent"))
        .and(header("x-goog-api-key", "test-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(lyria_success_response(&tiny_wav(), "Generated audio")),
        )
        .mount(&mock_server)
        .await;

    let client = LyriaClient::with_base_url("test-key".into(), mock_server.uri());

    (mock_server, client)
}

// ---------------------------------------------------------------------------
// Music generation tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn generate_overworld_theme() {
    let (_server, client) = setup_mock().await;

    let result = client
        .generate_audio(
            "Compose an 8-bit chiptune overworld theme at ~120 BPM. \
             Bright, adventurous melody using square and triangle waves. \
             Classic NES-style with a memorable hook that loops seamlessly.",
        )
        .await;

    assert!(result.is_ok(), "generate_audio failed: {:?}", result.err());
    let audio = result.unwrap();
    assert!(
        !audio.audio_data.is_empty(),
        "audio_data should not be empty"
    );
    assert!(audio.description.is_some(), "should have a description");
}

#[tokio::test]
async fn generate_boss_battle_music() {
    let (_server, client) = setup_mock().await;

    let result = client
        .generate_audio(
            "Compose an intense 8-bit boss battle theme at ~150 BPM. \
             Driving bass line on triangle wave, aggressive square wave melody, \
             rapid arpeggios, and noise channel percussion. Minor key, urgent mood.",
        )
        .await;

    assert!(result.is_ok(), "generate_audio failed: {:?}", result.err());
    let audio = result.unwrap();
    assert!(!audio.audio_data.is_empty());
}

#[tokio::test]
async fn generate_coin_collect_sfx() {
    let (_server, client) = setup_mock().await;

    let result = client
        .generate_audio(
            "Create an 8-bit coin collect sound effect. Short ascending arpeggio \
             on a bright square wave, approximately 0.3 seconds long. \
             Classic NES coin pickup sound.",
        )
        .await;

    assert!(result.is_ok(), "generate_audio failed: {:?}", result.err());
    let audio = result.unwrap();
    assert!(!audio.audio_data.is_empty());
}

#[tokio::test]
async fn generate_jump_sfx() {
    let (_server, client) = setup_mock().await;

    let result = client
        .generate_audio(
            "Create an 8-bit jump sound effect. Quick upward pitch sweep on a \
             square wave, approximately 0.2 seconds. Classic platformer jump sound.",
        )
        .await;

    assert!(result.is_ok(), "generate_audio failed: {:?}", result.err());
    let audio = result.unwrap();
    assert!(!audio.audio_data.is_empty());
}

#[tokio::test]
async fn generate_looping_dungeon_theme() {
    let (_server, client) = setup_mock().await;

    let result = client
        .generate_audio(
            "Compose a seamlessly looping 8-bit dungeon theme, approximately 30 \
             seconds long. Mysterious minor key, slow tempo ~100 BPM. \
             Sparse melody with echoing arpeggios on square wave, \
             low triangle wave bass. Must loop perfectly.",
        )
        .await;

    assert!(result.is_ok(), "generate_audio failed: {:?}", result.err());
    let audio = result.unwrap();
    assert!(!audio.audio_data.is_empty());
}

#[tokio::test]
async fn generate_victory_fanfare() {
    let (_server, client) = setup_mock().await;

    let result = client
        .generate_audio(
            "Compose a short 8-bit victory fanfare, approximately 5 seconds. \
             Triumphant major key melody, ascending arpeggios, bright square waves. \
             Similar in feel to classic JRPG battle victory jingles.",
        )
        .await;

    assert!(result.is_ok(), "generate_audio failed: {:?}", result.err());
    let audio = result.unwrap();
    assert!(!audio.audio_data.is_empty());
}

// ---------------------------------------------------------------------------
// Sample prompt — dog saves NYC musical theme
// ---------------------------------------------------------------------------

#[tokio::test]
async fn generate_dog_saves_nyc_theme() {
    let (_server, client) = setup_mock().await;

    let result = client
        .generate_audio(
            "Create an upbeat theme for a musical about a dog that saves new york. \
             Pure 8-bit instrumental.",
        )
        .await;

    assert!(result.is_ok(), "generate_audio failed: {:?}", result.err());
    let audio = result.unwrap();
    assert!(
        !audio.audio_data.is_empty(),
        "audio_data should not be empty"
    );
}

// ---------------------------------------------------------------------------
// Verify LOOP_ALWAYS_DIRECTIVE is present in all prompts
// ---------------------------------------------------------------------------

#[test]
fn loop_always_directive_exists() {
    use music_generator::design::LOOP_ALWAYS_DIRECTIVE;
    assert!(LOOP_ALWAYS_DIRECTIVE.contains("MUST be designed to play on a loop"));
    assert!(LOOP_ALWAYS_DIRECTIVE.contains("no audible"));
}

// ---------------------------------------------------------------------------
// Design prompt tests — verify design principles are injected
// ---------------------------------------------------------------------------

#[test]
fn bgm_prompt_includes_design_principles() {
    use music_generator::design::BGM_SYSTEM_PROMPT;
    assert!(BGM_SYSTEM_PROMPT.contains("square wave"));
    assert!(BGM_SYSTEM_PROMPT.contains("triangle wave"));
    assert!(BGM_SYSTEM_PROMPT.contains("noise channel"));
    assert!(BGM_SYSTEM_PROMPT.contains("3-4 melodic voices"));
    assert!(BGM_SYSTEM_PROMPT.contains("loop seamlessly"));
}

#[test]
fn sfx_prompt_includes_design_principles() {
    use music_generator::design::SFX_SYSTEM_PROMPT;
    assert!(SFX_SYSTEM_PROMPT.contains("pitch sweep"));
    assert!(SFX_SYSTEM_PROMPT.contains("noise burst"));
    assert!(SFX_SYSTEM_PROMPT.contains("0.1-2 seconds"));
    assert!(SFX_SYSTEM_PROMPT.contains("Coin collect"));
}

#[test]
fn loop_prompt_includes_design_principles() {
    use music_generator::design::LOOP_SYSTEM_PROMPT;
    assert!(LOOP_SYSTEM_PROMPT.contains("loop perfectly"));
    assert!(LOOP_SYSTEM_PROMPT.contains("16 and 64 bars"));
    assert!(LOOP_SYSTEM_PROMPT.contains("dynamic range"));
    assert!(LOOP_SYSTEM_PROMPT.contains("A and B"));
}

#[test]
fn remix_prompt_includes_design_principles() {
    use music_generator::design::REMIX_SYSTEM_PROMPT;
    assert!(REMIX_SYSTEM_PROMPT.contains("key and tempo"));
    assert!(REMIX_SYSTEM_PROMPT.contains("chiptune aesthetic"));
    assert!(REMIX_SYSTEM_PROMPT.contains("channel limit"));
}

// ---------------------------------------------------------------------------
// API serialization tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn api_sends_correct_headers_and_model() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path_regex(r"/lyria-realtime-exp:generateContent"))
        .and(header("x-goog-api-key", "my-secret-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(lyria_success_response(&tiny_wav(), "ok")),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = LyriaClient::with_base_url("my-secret-key".into(), mock_server.uri());

    let result = client.generate_audio("test prompt").await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn api_error_response_is_handled() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path_regex(r"/lyria-.*:generateContent"))
        .respond_with(ResponseTemplate::new(403).set_body_string("forbidden"))
        .mount(&mock_server)
        .await;

    let client = LyriaClient::with_base_url("bad-key".into(), mock_server.uri());

    let result = client.generate_audio("test").await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("403"),
        "error should mention status code: {err}"
    );
}
