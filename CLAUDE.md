# 8-Bit Music Generator

MCP server for generating 8-bit chiptune video game music using Google Lyria 3.

## Architecture

Follows the same pattern as [sprite-generator](https://github.com/andrewkrug/sprite-generator):

- `src/main.rs` — entry point, wires up MCP stdio transport
- `src/server.rs` — MCP tool definitions (generate_music, generate_sfx, generate_loop, remix_music)
- `src/lyria.rs` — Lyria 3 API client (Gemini generativelanguage endpoint)
- `src/design.rs` — system prompt constants encoding 8-bit music design principles
- `src/api_key.rs` — API key resolution: env var → config file → browser prompt
- `src/lib.rs` — re-exports for integration tests
- `tests/integration.rs` — wiremock-based integration tests

## Commands

```bash
make check    # fmt + clippy + tests
make build    # cargo build
make test     # cargo test
make lint     # cargo clippy
```

## API Key

Set `GEMINI_API_KEY` env var, or create `~/.config/music-generator.toml`:

```toml
gemini_api_key = "AIza..."
```

## Output

Generated audio files are saved to `./music/` (or `MUSIC_OUTPUT_DIR`).
