//! Generate sample 8-bit music tracks using the Lyria 3 API.
//!
//! Requires `GEMINI_API_KEY` to be set.
//!
//! Usage:
//!   cargo run --example generate_samples

use std::path::PathBuf;

use music_generator::lyria::LyriaClient;

const OUTPUT_DIR: &str = "sample-output";

struct Sample {
    name: &'static str,
    prompt: &'static str,
}

const SAMPLES: &[Sample] = &[
    Sample {
        name: "dog_saves_nyc_theme",
        prompt: "Create an upbeat theme for a musical about a dog that saves new york. \
                 Pure 8-bit instrumental. The music must loop seamlessly.",
    },
    Sample {
        name: "overworld_adventure",
        prompt: "Compose an 8-bit chiptune overworld adventure theme at ~120 BPM. \
                 Bright, heroic melody on square wave with triangle bass and noise percussion. \
                 Must loop seamlessly.",
    },
    Sample {
        name: "boss_battle",
        prompt: "Compose an intense 8-bit boss battle theme at ~150 BPM. \
                 Driving bass, aggressive arpeggios, minor key urgency. \
                 NES-style square and triangle waves. Must loop seamlessly.",
    },
    Sample {
        name: "peaceful_village",
        prompt: "Compose a calm 8-bit village theme at ~90 BPM. \
                 Gentle pentatonic melody, warm triangle wave bass, \
                 light percussion. Cozy and inviting. Must loop seamlessly.",
    },
    Sample {
        name: "dungeon_crawl",
        prompt: "Compose a mysterious 8-bit dungeon theme at ~100 BPM. \
                 Sparse, echoing arpeggios on square wave, low triangle bass, \
                 occasional noise hits. Minor key, suspenseful. Must loop seamlessly.",
    },
    Sample {
        name: "victory_fanfare",
        prompt: "Compose a short 8-bit victory fanfare, ~5 seconds. \
                 Triumphant major key, ascending arpeggios, bright square waves. \
                 Classic JRPG style. Must loop seamlessly.",
    },
];

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("music_generator=info")
        .with_writer(std::io::stderr)
        .init();

    let api_key =
        std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set to generate samples");

    let client = LyriaClient::new(api_key);
    let output_dir = PathBuf::from(OUTPUT_DIR);
    tokio::fs::create_dir_all(&output_dir).await?;

    for sample in SAMPLES {
        eprintln!("Generating: {} ...", sample.name);

        match client.generate_audio(sample.prompt).await {
            Ok(audio) => {
                let ext = match audio.mime_type.as_str() {
                    "audio/mpeg" | "audio/mp3" => "mp3",
                    "audio/ogg" => "ogg",
                    _ => "wav",
                };
                let path = output_dir.join(format!("{}.{ext}", sample.name));
                tokio::fs::write(&path, &audio.audio_data).await?;
                eprintln!(
                    "  Saved: {} ({} bytes)",
                    path.display(),
                    audio.audio_data.len()
                );
                if let Some(desc) = &audio.description {
                    eprintln!("  Description: {desc}");
                }
            }
            Err(e) => {
                eprintln!("  FAILED: {e}");
            }
        }
    }

    eprintln!("Done. Samples written to {OUTPUT_DIR}/");
    Ok(())
}
