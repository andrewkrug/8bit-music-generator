use anyhow::{Result, anyhow};
use base64::Engine;
use serde::{Deserialize, Serialize};

const API_BASE: &str = "https://generativelanguage.googleapis.com/v1beta/models";
const MODEL_ID: &str = "lyria-realtime-exp";

#[derive(Clone)]
pub struct LyriaClient {
    http: reqwest::Client,
    api_key: String,
    base_url: String,
}

// --- Request types ---

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GenerateRequest {
    contents: Vec<Content>,
    generation_config: GenerationConfig,
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize)]
#[serde(untagged)]
enum Part {
    Text { text: String },
    InlineData { inline_data: InlineData },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InlineData {
    mime_type: String,
    data: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GenerationConfig {
    response_modalities: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    speech_config: Option<SpeechConfig>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SpeechConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    language_code: Option<String>,
}

// --- Response types ---

#[derive(Deserialize)]
struct GenerateResponse {
    candidates: Option<Vec<Candidate>>,
    error: Option<ApiError>,
}

#[derive(Deserialize)]
struct Candidate {
    content: CandidateContent,
}

#[derive(Deserialize)]
struct CandidateContent {
    parts: Vec<ResponsePart>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ResponsePart {
    Text {
        text: String,
    },
    InlineData {
        #[serde(rename = "inlineData")]
        inline_data: ResponseInlineData,
    },
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResponseInlineData {
    mime_type: String,
    data: String,
}

#[derive(Deserialize)]
struct ApiError {
    message: String,
}

/// Result of a music generation call.
#[derive(Debug)]
pub struct GeneratedAudio {
    pub audio_data: Vec<u8>,
    pub mime_type: String,
    pub description: Option<String>,
}

impl LyriaClient {
    pub fn new(api_key: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_key,
            base_url: API_BASE.to_string(),
        }
    }

    /// Create a client with a custom base URL (useful for testing with mock servers).
    pub fn with_base_url(api_key: String, base_url: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_key,
            base_url,
        }
    }

    /// Generate audio from a text prompt.
    pub async fn generate_audio(&self, prompt: &str) -> Result<GeneratedAudio> {
        let request = GenerateRequest {
            contents: vec![Content {
                parts: vec![Part::Text {
                    text: prompt.to_string(),
                }],
            }],
            generation_config: GenerationConfig {
                response_modalities: vec!["AUDIO".into()],
                speech_config: None,
            },
        };

        self.call_api(&request).await
    }

    /// Edit/remix existing audio with a text prompt.
    pub async fn edit_audio(
        &self,
        audio_data: &[u8],
        mime_type: &str,
        prompt: &str,
    ) -> Result<GeneratedAudio> {
        let encoded = base64::engine::general_purpose::STANDARD.encode(audio_data);

        let request = GenerateRequest {
            contents: vec![Content {
                parts: vec![
                    Part::Text {
                        text: prompt.to_string(),
                    },
                    Part::InlineData {
                        inline_data: InlineData {
                            mime_type: mime_type.to_string(),
                            data: encoded,
                        },
                    },
                ],
            }],
            generation_config: GenerationConfig {
                response_modalities: vec!["AUDIO".into()],
                speech_config: None,
            },
        };

        self.call_api(&request).await
    }

    async fn call_api(&self, request: &GenerateRequest) -> Result<GeneratedAudio> {
        let url = format!("{}/{MODEL_ID}:generateContent", self.base_url);

        let response = self
            .http
            .post(&url)
            .header("x-goog-api-key", &self.api_key)
            .json(request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("API request failed ({}): {}", status, body));
        }

        let body: GenerateResponse = response.json().await?;

        if let Some(err) = body.error {
            return Err(anyhow!("API error: {}", err.message));
        }

        let candidates = body
            .candidates
            .ok_or_else(|| anyhow!("No candidates in response"))?;
        let parts = &candidates
            .first()
            .ok_or_else(|| anyhow!("Empty candidates array"))?
            .content
            .parts;

        let mut audio_data = None;
        let mut audio_mime = String::from("audio/wav");
        let mut description = None;

        for part in parts {
            match part {
                ResponsePart::InlineData { inline_data } => {
                    if audio_data.is_none() {
                        let bytes =
                            base64::engine::general_purpose::STANDARD.decode(&inline_data.data)?;
                        tracing::info!(
                            mime_type = %inline_data.mime_type,
                            size = bytes.len(),
                            "Received audio from API"
                        );
                        audio_mime = inline_data.mime_type.clone();
                        audio_data = Some(bytes);
                    }
                }
                ResponsePart::Text { text } => {
                    if !text.is_empty() {
                        description = Some(text.clone());
                    }
                }
            }
        }

        let data = audio_data.ok_or_else(|| anyhow!("No audio data in API response"))?;

        Ok(GeneratedAudio {
            audio_data: data,
            mime_type: audio_mime,
            description,
        })
    }
}
