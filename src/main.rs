use std::path::PathBuf;

use anyhow::Result;
use rmcp::ServiceExt;
use rmcp::transport::stdio;

use music_generator::api_key;
use music_generator::lyria::LyriaClient;
use music_generator::server::MusicGeneratorServer;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("music_generator=info".parse()?),
        )
        .with_writer(std::io::stderr)
        .init();

    let api_key = api_key::resolve_api_key().await?;

    let output_dir = std::env::var("MUSIC_OUTPUT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("./music"));

    tracing::info!("Starting 8-Bit Music Generator MCP server");
    tracing::info!(output_dir = %output_dir.display(), "Output directory");

    let client = LyriaClient::new(api_key);
    let server = MusicGeneratorServer::new(client, output_dir);

    let service = server.serve(stdio()).await?;
    service.waiting().await?;

    Ok(())
}
