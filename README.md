# 8-Bit Music Generator

A [Model Context Protocol](https://modelcontextprotocol.io) (MCP) server that generates authentic 8-bit chiptune video game music using Google's **Lyria 3** as the audio backend.

Drop it into any MCP-compatible client (Claude Desktop, Claude Code, etc.) and your assistant can compose original chiptune soundtracks, sound effects, and looping BGM tracks for your game on demand.

Architecture and project structure adopted from [sprite-generator](https://github.com/andrewkrug/sprite-generator).

## Sample Output

All six tracks below were generated end-to-end by this server. Every prompt is wrapped with a **universal looping directive**, so every track is composed to loop seamlessly.

### Dog Saves New York — Musical Theme

> *"Create an upbeat theme for a musical about a dog that saves new york. Pure 8-bit instrumental."*

<audio controls src="docs/samples/dog_saves_nyc_theme.mp3"></audio>

[▶ Download / play `dog_saves_nyc_theme.mp3`](docs/samples/dog_saves_nyc_theme.mp3)

### Overworld Adventure (~120 BPM)

Bright, heroic major-key theme — square-wave lead, triangle bass, noise-channel percussion.

<audio controls src="docs/samples/overworld_adventure.mp3"></audio>

[▶ Download / play `overworld_adventure.mp3`](docs/samples/overworld_adventure.mp3)

### Boss Battle (~150 BPM)

Driving minor-key combat theme with aggressive arpeggios and dual square-wave leads.

<audio controls src="docs/samples/boss_battle.mp3"></audio>

[▶ Download / play `boss_battle.mp3`](docs/samples/boss_battle.mp3)

### Peaceful Village (~90 BPM)

Cozy pentatonic melody with warm triangle bass — a classic JRPG town theme.

<audio controls src="docs/samples/peaceful_village.mp3"></audio>

[▶ Download / play `peaceful_village.mp3`](docs/samples/peaceful_village.mp3)

### Dungeon Crawl (~100 BPM)

Sparse, echoing minor-key arpeggios for hostile underground environments.

<audio controls src="docs/samples/dungeon_crawl.mp3"></audio>

[▶ Download / play `dungeon_crawl.mp3`](docs/samples/dungeon_crawl.mp3)

### Victory Fanfare (~6s)

Short, triumphant JRPG-style win jingle in a major key.

<audio controls src="docs/samples/victory_fanfare.mp3"></audio>

[▶ Download / play `victory_fanfare.mp3`](docs/samples/victory_fanfare.mp3)

> **Note:** GitHub strips `<audio>` tags when rendering READMEs, so the inline players only work in local previews (VS Code, Obsidian, etc.). On GitHub, click the download links above to listen.

---

## What It Does

The server exposes four MCP tools:

| Tool | Purpose |
|------|---------|
| `generate_music` | Compose a full chiptune BGM track for a game scene (overworld, battle, menu, etc.) |
| `generate_sfx` | Produce short punchy 8-bit sound effects (jump, coin, damage, power-up, UI) |
| `generate_loop` | Compose a seamlessly looping background track with explicit duration and mood |
| `remix_music` | Edit an existing audio file using a natural-language instruction |

Every generation request is wrapped with design-system prompts encoding authentic chiptune principles (square/triangle/noise channels, channel discipline, harmonic progressions) plus a **universal loop directive** — all generated audio is designed to repeat cleanly.

## Installation

### Prerequisites

- Rust stable toolchain ([rustup.rs](https://rustup.rs))
- A [Google Gemini API key](https://aistudio.google.com/apikey) with Lyria 3 access

### Build

```bash
git clone <this-repo>
cd 8bit-music-generator
make build          # or: cargo build --release
```

The server binary lives at `target/release/music-generator` (or `target/debug/music-generator`).

## API Key Setup

Three ways to provide your key, checked in order:

1. **Environment variable** (recommended for CI):
   ```bash
   export GEMINI_API_KEY="AIza..."
   ```

2. **Config file** at `~/.config/music-generator.toml`:
   ```toml
   gemini_api_key = "AIza..."
   ```

3. **Interactive browser prompt** — if no key is found, the server opens a local web page to collect one (kept in memory only).

## Using With an MCP Client

### Claude Desktop

Edit `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS) or the equivalent on your OS:

```json
{
  "mcpServers": {
    "8bit-music": {
      "command": "/absolute/path/to/target/release/music-generator",
      "env": {
        "GEMINI_API_KEY": "AIza..."
      }
    }
  }
}
```

Restart Claude Desktop. You should now see the four music tools available.

### Claude Code

```bash
claude mcp add 8bit-music /absolute/path/to/target/release/music-generator \
  --env GEMINI_API_KEY=AIza...
```

### Custom Output Directory

By default, generated audio is saved to `./music/`. Override with:

```bash
export MUSIC_OUTPUT_DIR="/path/to/game-assets/audio"
```

## Example Prompts

Once the server is wired up, try asking your assistant things like:

- *"Generate an energetic overworld theme for a platformer at 140 BPM and save it as `world1_bgm`."*
- *"Make me a coin-pickup sound effect."*
- *"Compose a 30-second loop for a mysterious dungeon, minor key. Save as `dungeon_theme`."*
- *"Remix `music/world1_bgm.mp3` — make it more intense for the boss room."*

## Generating Samples Locally

The repo ships with an example binary that regenerates the samples in `docs/samples/`:

```bash
export GEMINI_API_KEY="AIza..."
cargo run --example generate_samples
```

Output lands in `sample-output/` as MP3 files.

## Design Principles

System prompts encoding the rules live in [`src/design.rs`](src/design.rs):

- **`BGM_SYSTEM_PROMPT`** — chiptune waveforms, channel limits, harmonic conventions, tempo guidance.
- **`SFX_SYSTEM_PROMPT`** — short durations, pitch-sweep patterns, classic NES references.
- **`LOOP_SYSTEM_PROMPT`** — A/B structure, seamless loop points, mixing for games.
- **`LOOP_ALWAYS_DIRECTIVE`** — appended to every prompt; makes every generated track loopable.
- **`REMIX_SYSTEM_PROMPT`** — preserves key/tempo/chiptune aesthetic when editing.

## Project Layout

```
src/
  main.rs           Entry point, wires up MCP stdio transport
  lib.rs            Re-exports for integration tests
  server.rs         MCP tool definitions (generate_music, generate_sfx, …)
  lyria.rs          Lyria 3 API client (Gemini generativelanguage endpoint)
  design.rs         System-prompt constants for chiptune music design
  api_key.rs        Env → config → browser-prompt resolution
tests/
  integration.rs    wiremock-based integration tests
examples/
  generate_samples.rs  Live-API sample generator
docs/samples/       Embedded audio samples shown in this README
```

## Development

```bash
make check          # fmt + clippy + tests (zero warnings, zero failures)
make test           # cargo test --all-targets
make lint           # cargo clippy --all-targets -- -D warnings
make fmt            # cargo fmt --all
make run            # cargo run (launches the MCP server on stdio)
```

## Model

This server targets `lyria-3-pro-preview` on the Gemini `generativelanguage` v1beta endpoint. The model is accessed via `generateContent` and returns MP3 audio along with a structured textual description of the composition (BPM, section-by-section analysis, instrumentation notes).

## License

Apache-2.0
