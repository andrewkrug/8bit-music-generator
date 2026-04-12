use std::path::PathBuf;

use base64::Engine;
use rmcp::handler::server::tool::ToolRouter;
use rmcp::model::{
    CallToolResult, Content, Implementation, InitializeResult, ProtocolVersion, RawAudioContent,
    RawContent, ServerCapabilities, ServerInfo,
};
use rmcp::{ErrorData as McpError, ServerHandler, tool, tool_handler, tool_router};
use serde::{Deserialize, Serialize};

use crate::design;
use crate::lyria::LyriaClient;

fn audio_content(data: impl Into<String>, mime_type: impl Into<String>) -> Content {
    Content::new(
        RawContent::Audio(RawAudioContent {
            data: data.into(),
            mime_type: mime_type.into(),
        }),
        None,
    )
}

// --- Parameter types ---

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(description = "Parameters for generating 8-bit video game background music")]
pub struct GenerateMusicParams {
    #[schemars(
        description = "Description of the music to generate (e.g. 'upbeat overworld theme for a platformer')"
    )]
    pub prompt: String,
    #[schemars(
        description = "Game context: 'overworld', 'battle', 'boss', 'menu', 'dungeon', 'victory', 'game-over', 'title-screen', or custom. Defaults to 'overworld'"
    )]
    pub context: Option<String>,
    #[schemars(
        description = "Target tempo in BPM (e.g. '120'). If omitted, automatically chosen based on context"
    )]
    pub tempo: Option<String>,
    #[schemars(
        description = "Optional filename to save the audio as (without extension). If provided, saves to the output directory."
    )]
    pub save_as: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(description = "Parameters for generating an 8-bit sound effect")]
pub struct GenerateSfxParams {
    #[schemars(
        description = "Description of the sound effect (e.g. 'coin collect', 'player jump', 'explosion')"
    )]
    pub prompt: String,
    #[schemars(
        description = "Sound effect category: 'action' (jump, attack), 'collect' (coin, item), 'ui' (menu select, confirm), 'damage' (hit, death), 'environment' (door, chest). Defaults to 'action'"
    )]
    pub category: Option<String>,
    #[schemars(description = "Optional filename to save the sound effect as (without extension)")]
    pub save_as: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(description = "Parameters for generating a seamlessly looping background music track")]
pub struct GenerateLoopParams {
    #[schemars(
        description = "Description of the looping music (e.g. 'peaceful village theme with gentle melody')"
    )]
    pub prompt: String,
    #[schemars(
        description = "Mood: 'calm', 'tense', 'heroic', 'mysterious', 'energetic', 'melancholy', or custom. Defaults to 'calm'"
    )]
    pub mood: Option<String>,
    #[schemars(description = "Target loop duration in seconds (e.g. '30'). Defaults to '30'")]
    pub duration: Option<String>,
    #[schemars(description = "Optional filename to save the loop as (without extension)")]
    pub save_as: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(description = "Parameters for remixing or editing an existing audio track")]
pub struct RemixMusicParams {
    #[schemars(description = "Path to the existing audio file to remix")]
    pub audio_path: String,
    #[schemars(
        description = "Description of the changes to make (e.g. 'make it more intense', 'add a bass line', 'shift to minor key')"
    )]
    pub instruction: String,
    #[schemars(
        description = "Optional filename to save the remixed audio as (without extension). Defaults to overwriting the original."
    )]
    pub save_as: Option<String>,
}

// --- Asset context returned to callers for game build integration ---

#[derive(Serialize)]
struct AssetContext {
    asset_type: &'static str,
    context: String,
    prompt: String,
    tempo: String,
    mood: String,
    suggested_usage: String,
    file_path: Option<String>,
}

impl AssetContext {
    fn to_content(&self) -> Content {
        Content::text(serde_json::to_string_pretty(self).unwrap_or_default())
    }
}

// --- Server ---

#[derive(Clone)]
pub struct MusicGeneratorServer {
    client: LyriaClient,
    output_dir: PathBuf,
    tool_router: ToolRouter<Self>,
}

impl MusicGeneratorServer {
    pub fn new(client: LyriaClient, output_dir: PathBuf) -> Self {
        Self {
            client,
            output_dir,
            tool_router: Self::tool_router(),
        }
    }

    async fn save_audio(
        &self,
        filename: &str,
        data: &[u8],
        extension: &str,
    ) -> Result<PathBuf, McpError> {
        let path = self.output_dir.join(format!("{filename}.{extension}"));
        tokio::fs::create_dir_all(&self.output_dir)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        tokio::fs::write(&path, data)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(path)
    }

    fn extension_for_mime(mime: &str) -> &str {
        match mime {
            "audio/wav" | "audio/x-wav" => "wav",
            "audio/mpeg" | "audio/mp3" => "mp3",
            "audio/ogg" => "ogg",
            "audio/flac" => "flac",
            _ => "wav",
        }
    }
}

fn default_tempo_for_context(context: &str) -> &str {
    match context {
        "battle" | "boss" => "150",
        "overworld" | "title-screen" => "120",
        "menu" | "game-over" => "90",
        "dungeon" | "mystery" => "100",
        "victory" => "130",
        _ => "120",
    }
}

use rmcp::handler::server::wrapper::Parameters;

#[tool_router]
impl MusicGeneratorServer {
    #[tool(
        description = "Generate 8-bit chiptune background music for a video game. Produces authentic retro game music suitable for NES/Game Boy style games."
    )]
    async fn generate_music(
        &self,
        Parameters(params): Parameters<GenerateMusicParams>,
    ) -> Result<CallToolResult, McpError> {
        let context = params.context.unwrap_or_else(|| "overworld".into());
        let tempo = params
            .tempo
            .unwrap_or_else(|| default_tempo_for_context(&context).into());
        let original_prompt = params.prompt.clone();

        let enhanced_prompt = format!(
            "{}{}\nCompose an 8-bit chiptune track for a video game {context} scene at ~{tempo} BPM: {}. \
             Use classic NES-style square wave, triangle wave, and noise channels. \
             The music should be immediately recognizable as retro video game music \
             with a strong, memorable melody.",
            design::BGM_SYSTEM_PROMPT,
            design::LOOP_ALWAYS_DIRECTIVE,
            params.prompt
        );

        let result = self
            .client
            .generate_audio(&enhanced_prompt)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let ext = Self::extension_for_mime(&result.mime_type);
        let mut contents = Vec::new();
        let mut file_path = None;

        if let Some(filename) = params.save_as {
            let path = self.save_audio(&filename, &result.audio_data, ext).await?;
            contents.push(Content::text(format!("Music saved to: {}", path.display())));
            file_path = Some(path.display().to_string());
        }

        let b64 = base64::engine::general_purpose::STANDARD.encode(&result.audio_data);
        contents.push(audio_content(b64, &result.mime_type));

        if let Some(desc) = result.description {
            contents.push(Content::text(desc));
        }

        contents.push(
            AssetContext {
                asset_type: "bgm",
                context: context.clone(),
                prompt: original_prompt,
                tempo: tempo.clone(),
                mood: context.clone(),
                suggested_usage: format!(
                    "Background music for a {context} scene. Play as a looping BGM track \
                     in your game engine's audio system. Tempo: ~{tempo} BPM."
                ),
                file_path,
            }
            .to_content(),
        );

        Ok(CallToolResult::success(contents))
    }

    #[tool(
        description = "Generate an 8-bit chiptune sound effect for a video game. Produces short, punchy retro sound effects (jumps, coins, explosions, UI sounds)."
    )]
    async fn generate_sfx(
        &self,
        Parameters(params): Parameters<GenerateSfxParams>,
    ) -> Result<CallToolResult, McpError> {
        let category = params.category.unwrap_or_else(|| "action".into());
        let original_prompt = params.prompt.clone();

        let enhanced_prompt = format!(
            "{}{}\nCreate an 8-bit chiptune sound effect in the '{}' category: {}. \
             The sound should be short, punchy, and immediately recognizable. \
             Use classic NES-era synthesis techniques: square waves, noise bursts, \
             and rapid pitch sweeps.",
            design::SFX_SYSTEM_PROMPT,
            design::LOOP_ALWAYS_DIRECTIVE,
            category,
            params.prompt
        );

        let result = self
            .client
            .generate_audio(&enhanced_prompt)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let ext = Self::extension_for_mime(&result.mime_type);
        let mut contents = Vec::new();
        let mut file_path = None;

        if let Some(filename) = params.save_as {
            let path = self.save_audio(&filename, &result.audio_data, ext).await?;
            contents.push(Content::text(format!("SFX saved to: {}", path.display())));
            file_path = Some(path.display().to_string());
        }

        let b64 = base64::engine::general_purpose::STANDARD.encode(&result.audio_data);
        contents.push(audio_content(b64, &result.mime_type));

        if let Some(desc) = result.description {
            contents.push(Content::text(desc));
        }

        contents.push(
            AssetContext {
                asset_type: "sfx",
                context: category.clone(),
                prompt: original_prompt,
                tempo: "n/a".into(),
                mood: category.clone(),
                suggested_usage: format!(
                    "Sound effect for '{category}' events. Trigger on the corresponding \
                     game event. Keep volume balanced relative to background music."
                ),
                file_path,
            }
            .to_content(),
        );

        Ok(CallToolResult::success(contents))
    }

    #[tool(
        description = "Generate a seamlessly looping 8-bit music track. The audio is designed to repeat perfectly with no audible seam, ideal for game background music."
    )]
    async fn generate_loop(
        &self,
        Parameters(params): Parameters<GenerateLoopParams>,
    ) -> Result<CallToolResult, McpError> {
        let mood = params.mood.unwrap_or_else(|| "calm".into());
        let duration = params.duration.unwrap_or_else(|| "30".into());
        let original_prompt = params.prompt.clone();

        let enhanced_prompt = format!(
            "{}{}\nCompose a seamlessly looping 8-bit chiptune track, approximately {duration} \
             seconds long, with a {mood} mood: {}. \
             The track MUST loop perfectly — the ending must flow naturally back \
             into the beginning with no click or gap. Use NES-style synthesis.",
            design::LOOP_SYSTEM_PROMPT,
            design::LOOP_ALWAYS_DIRECTIVE,
            params.prompt
        );

        let result = self
            .client
            .generate_audio(&enhanced_prompt)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let ext = Self::extension_for_mime(&result.mime_type);
        let mut contents = Vec::new();
        let mut file_path = None;

        if let Some(filename) = params.save_as {
            let path = self.save_audio(&filename, &result.audio_data, ext).await?;
            contents.push(Content::text(format!("Loop saved to: {}", path.display())));
            file_path = Some(path.display().to_string());
        }

        let b64 = base64::engine::general_purpose::STANDARD.encode(&result.audio_data);
        contents.push(audio_content(b64, &result.mime_type));

        if let Some(desc) = result.description {
            contents.push(Content::text(desc));
        }

        contents.push(
            AssetContext {
                asset_type: "loop",
                context: "background".into(),
                prompt: original_prompt,
                tempo: "auto".into(),
                mood: mood.clone(),
                suggested_usage: format!(
                    "Seamlessly looping background music (~{duration}s). Set your audio \
                     engine to loop mode. Mood: {mood}."
                ),
                file_path,
            }
            .to_content(),
        );

        Ok(CallToolResult::success(contents))
    }

    #[tool(
        description = "Remix or edit an existing audio track with 8-bit chiptune style. Provide a path to an existing audio file and describe the changes you want."
    )]
    async fn remix_music(
        &self,
        Parameters(params): Parameters<RemixMusicParams>,
    ) -> Result<CallToolResult, McpError> {
        let original_instruction = params.instruction.clone();
        let audio_data = tokio::fs::read(&params.audio_path)
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to read audio: {e}"), None))?;

        let mime_type = if params.audio_path.ends_with(".wav") {
            "audio/wav"
        } else if params.audio_path.ends_with(".mp3") {
            "audio/mpeg"
        } else if params.audio_path.ends_with(".ogg") {
            "audio/ogg"
        } else if params.audio_path.ends_with(".flac") {
            "audio/flac"
        } else {
            "audio/wav"
        };

        let enhanced_instruction = format!(
            "{}{}\n{}",
            design::REMIX_SYSTEM_PROMPT,
            design::LOOP_ALWAYS_DIRECTIVE,
            params.instruction
        );

        let result = self
            .client
            .edit_audio(&audio_data, mime_type, &enhanced_instruction)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let ext = Self::extension_for_mime(&result.mime_type);
        let save_path = if let Some(filename) = params.save_as {
            self.save_audio(&filename, &result.audio_data, ext).await?
        } else {
            let path = PathBuf::from(&params.audio_path);
            tokio::fs::write(&path, &result.audio_data)
                .await
                .map_err(|e| McpError::internal_error(e.to_string(), None))?;
            path
        };

        let mut contents = vec![Content::text(format!(
            "Remixed audio saved to: {}",
            save_path.display()
        ))];

        let b64 = base64::engine::general_purpose::STANDARD.encode(&result.audio_data);
        contents.push(audio_content(b64, &result.mime_type));

        if let Some(desc) = result.description {
            contents.push(Content::text(desc));
        }

        contents.push(
            AssetContext {
                asset_type: "remix",
                context: "edited".into(),
                prompt: original_instruction,
                tempo: "original".into(),
                mood: "original".into(),
                suggested_usage: format!(
                    "Remixed version of {}. Drop-in replacement for the original \
                     asset in your game build.",
                    params.audio_path
                ),
                file_path: Some(save_path.display().to_string()),
            }
            .to_content(),
        );

        Ok(CallToolResult::success(contents))
    }
}

#[tool_handler]
impl ServerHandler for MusicGeneratorServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2025_06_18,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "8-Bit Music Generator MCP server - generates chiptune video game music, \
                 sound effects, and looping tracks using Lyria 3 as the audio generation backend."
                    .into(),
            ),
        }
    }

    async fn initialize(
        &self,
        _request: rmcp::model::InitializeRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        Ok(self.get_info())
    }
}
