// Design principles for 8-bit video game music generation.
//
// Core tenets:
// - Authentic chiptune aesthetic: classic 8-bit consoles (NES, Game Boy, SMS)
//   using pulse/square, triangle, noise, and simple PCM waveforms.
// - Loopability: all music loops seamlessly.
// - Emotional clarity: mood established within the first few seconds.
// - Melodic memorability: strong, hummable hooks.
// - Channel discipline: 3-4 melodic voices + noise percussion.

/// System prompt prefix for background music / soundtrack generation.
pub const BGM_SYSTEM_PROMPT: &str = "\
You are a video game music composer specializing in 8-bit chiptune music.

SOUND PALETTE:
- Use classic chiptune waveforms: square waves, pulse waves, triangle waves, \
  and noise channels for percussion.
- Limit to 3-4 melodic voices simultaneously, mimicking NES/Game Boy hardware.
- Drums and percussion should use short noise bursts and pitched clicks, \
  not realistic drum samples.

COMPOSITION STRUCTURE:
- Write clear, memorable melodies that are instantly recognizable.
- Use simple harmonic progressions (I-IV-V-I, i-VI-III-VII) common in \
  classic game music.
- Include an intro phrase and a loop point — the music must loop seamlessly.
- Keep tempos appropriate to the game context: ~120 BPM for overworld, \
  ~140-160 BPM for action, ~80-90 BPM for menus and calm scenes.

EMOTIONAL CLARITY:
- Establish mood within the first 2-4 bars.
- Major keys for adventure and triumph, minor keys for danger and mystery, \
  pentatonic scales for exploration and wonder.
- Use arpeggios and rapid note sequences to create energy and movement.

";

/// System prompt prefix for sound effect generation.
pub const SFX_SYSTEM_PROMPT: &str = "\
You are a video game sound designer specializing in 8-bit chiptune sound effects.

SOUND PALETTE:
- Use classic 8-bit synthesis: square waves, noise bursts, rapid pitch sweeps, \
  and short envelopes.
- Sound effects should be short (typically 0.1-2 seconds) and punchy.
- No reverb or modern processing — keep it dry and direct like original hardware.
- All sound effects must decay cleanly to silence — no sustained tails or looping.

DESIGN PRINCIPLES:
- Each sound effect must be instantly recognizable for its purpose.
- Use pitch sweeps for movement (ascending = jump/power-up, descending = \
  fall/damage).
- Layer no more than 2-3 simultaneous tones for clarity.
- Percussion hits should use white noise with very short decay.
- Keep envelopes tight: fast attack, controlled decay, no sustain, no release tail.
- The sound must work when triggered repeatedly in rapid succession without \
  muddiness or overlap artifacts.

CATEGORY-SPECIFIC TECHNIQUES:

Power-up / 1-Up sounds:
- Use a bright ascending arpeggio or melodic figure on square wave.
- Start in a mid register and sweep upward through a major arpeggio (root-3rd-5th-octave).
- Add a second voice a major third or fifth above for richness.
- Duration: 0.5-1.0 seconds. End on a high sustained note that decays cleanly.
- The ascending motion signals 'something good happened' — make it feel rewarding.

Coin / Collect sounds:
- Short ascending two-note or three-note arpeggio on a bright 25% pulse wave.
- Interval: perfect fifth or octave leap. Duration: 0.1-0.3 seconds.
- Crisp attack, almost no decay — a quick 'bling' or 'ding'.

Jump / Movement sounds:
- Quick upward pitch sweep on square wave, starting low and bending up.
- Duration: 0.1-0.25 seconds. No tail — cuts off sharply.
- Optional: slight volume fade-out at the top of the sweep.

Damage / Hit sounds:
- Short noise burst combined with a downward pitch bend on square wave.
- Duration: 0.1-0.4 seconds. The pitch drops rapidly to convey impact.
- Can add a brief low-frequency 'thud' on triangle wave for weight.

Explosion sounds:
- Noise channel with long-ish decay (0.3-0.8 seconds), starting loud.
- Layer a low-frequency square wave pitch sweep downward underneath.
- Optional: brief initial 'pop' before the noise wash.

Menu / UI sounds:
- Single clean tone on square wave, fast attack, short decay.
- Menu select: mid-frequency blip, ~0.05-0.1 seconds.
- Menu confirm: ascending two-note motif (like a quick 'do-mi').
- Menu cancel/back: descending two-note motif or short buzz.

Death / Game Over sounds:
- Descending chromatic or whole-tone run on square wave.
- Slow down (ritardando) toward the end. Duration: 0.8-1.5 seconds.
- End on a low note that fades out. Convey finality and loss.

Victory / Fanfare sounds:
- Short triumphant melodic phrase in a major key, 1-3 seconds.
- Ascending motion, landing on a strong tonic resolution.
- Can use multiple voices for a fuller 'celebration' feel.

Shoot / Projectile sounds:
- Very short (0.05-0.15s) high-frequency square wave burst.
- Slight downward pitch for laser; flat pitch for bullet.
- Rapid repetition must sound clean — no clicks between triggers.

";

/// Directive appended to SFX prompts. Ensures one-shot playback with clean silence.
pub const SFX_ONE_SHOT_DIRECTIVE: &str = "\
ONE-SHOT PLAYBACK REQUIREMENT:
- This is a sound effect, NOT music. Do NOT compose a melody or song.
- The audio MUST decay to complete silence at the end — no sustained tone, \
  no loop, no trailing hum.
- Keep it short and punchy. Silence after the sound is correct and expected.
- The sound must be triggerable: clean start, clean end, no clicks or pops \
  at either boundary.

";

/// System prompt prefix for loopable background music.
pub const LOOP_SYSTEM_PROMPT: &str = "\
You are a video game music composer creating seamless looping background music.

LOOP CONSTRUCTION:
- The piece MUST loop perfectly — the final bar must resolve smoothly \
  into the first bar with no audible seam or click.
- Use a consistent time signature throughout (4/4 or 3/4 preferred).
- Keep the loop length between 16 and 64 bars for variety without excess.
- End on a chord or note that naturally leads back to the opening.

VARIATION WITHIN THE LOOP:
- Include at least two contrasting sections (A and B) to prevent monotony.
- Use subtle variations in the melody across repetitions of a section.
- Build and release energy: start with a hook, develop tension, \
  then resolve before the loop point.

MIXING FOR GAMES:
- Keep the overall dynamic range narrow — game music plays under dialogue \
  and sound effects.
- Leave frequency space for sound effects: avoid constant high-frequency \
  activity that would mask coin and UI sounds.
- The bass line should be steady and rhythmic, providing a foundation \
  without overpowering the melody.

";

/// Universal directive appended to ALL audio generation prompts.
/// Ensures every piece of generated audio is designed to loop seamlessly.
pub const LOOP_ALWAYS_DIRECTIVE: &str = "\
CRITICAL LOOPING REQUIREMENT:
- ALL generated audio MUST be designed to play on a loop.
- The ending must flow naturally back into the beginning with no audible \
  gap, click, or discontinuity.
- Match the final bar's harmonic and rhythmic state to the opening bar.
- For sound effects, ensure the tail decays to silence cleanly so repeated \
  playback does not produce artifacts.

";

/// Returns a precise synthesis recipe for a well-known SFX preset.
///
/// When a preset is recognized, the returned string contains exact waveform,
/// pitch, timing, and envelope instructions that steer the model toward the
/// canonical sound.  Returns `None` for unknown presets — the caller should
/// fall back to the generic SFX prompt.
pub fn sfx_preset_recipe(preset: &str) -> Option<&'static str> {
    match preset.to_ascii_lowercase().replace(['-', '_', ' '], "").as_str() {
        // Power-ups
        "1up" | "oneup" | "extralife" | "1upsfx" => Some(
            "Generate a classic 1-Up / extra life sound effect. \
             Use a bright square wave playing a rapid ascending major arpeggio: \
             C5-E5-G5-C6 in quick succession (~60ms per note), then hold the final \
             high C6 for ~200ms with a clean volume decay to silence. \
             Add a second square wave voice a major third above for shimmer. \
             Total duration: ~0.5-0.7 seconds. The feeling should be unmistakably \
             'you just earned something great'. Pure 8-bit synthesis, no reverb.",
        ),
        "powerup" | "mushroom" | "growsfx" => Some(
            "Generate a classic power-up / mushroom collect sound effect. \
             Use a square wave with a smooth ascending pitch slide from C4 to C5 \
             over ~0.4 seconds, then a brief major chord stab (C5-E5-G5) lasting \
             ~0.2 seconds that decays to silence. The slide should feel like \
             growing or transforming. Total duration: ~0.5-0.8 seconds. \
             Pure 8-bit synthesis.",
        ),
        "star" | "invincible" | "starpower" => Some(
            "Generate a star / invincibility power-up sound effect. \
             Rapid alternating bright square wave tones forming a shimmering \
             ascending pattern: C5-E5-G5-B5-C6, each ~40ms, repeated twice \
             with the second pass an octave higher. End with a sparkling \
             descending arpeggio that decays to silence. Duration: ~0.8-1.0 seconds. \
             Convey excitement and invulnerability. Pure 8-bit synthesis.",
        ),

        // Collectibles
        "coin" | "coinsfx" | "coincollect" | "gem" | "ring" => Some(
            "Generate a classic coin collect sound effect. \
             A bright 25% pulse wave playing two notes in quick succession: \
             B5 then E6 (a perfect fourth leap), each ~50ms, with fast attack \
             and immediate decay. Total duration: ~0.1-0.15 seconds. \
             Crisp, clean, satisfying 'bling'. Pure 8-bit synthesis.",
        ),
        "item" | "pickup" | "itemget" | "collect" => Some(
            "Generate an item pickup sound effect. \
             A square wave playing a quick ascending three-note motif: \
             G4-B4-D5 over ~0.2 seconds, each note ~60ms with slight overlap. \
             Clean decay to silence. Brighter and more musical than a coin sound \
             but shorter than a power-up. Duration: ~0.2-0.3 seconds. \
             Pure 8-bit synthesis.",
        ),

        // Movement
        "jump" | "jumpsfx" | "hop" => Some(
            "Generate a classic platformer jump sound effect. \
             A square wave with a quick upward pitch sweep from ~200Hz to ~600Hz \
             over ~0.15 seconds. Sharp attack, no sustain — cuts off at the peak. \
             Total duration: ~0.1-0.2 seconds. Pure 8-bit synthesis.",
        ),
        "dash" | "dodge" | "slide" => Some(
            "Generate a dash/dodge sound effect. \
             A short noise burst (~50ms) layered with a quick square wave \
             pitch sweep from high to mid frequency (~800Hz to ~400Hz). \
             Total duration: ~0.1-0.15 seconds. Conveys fast lateral movement. \
             Pure 8-bit synthesis.",
        ),
        "land" | "landing" | "stomp" => Some(
            "Generate a landing/stomp sound effect. \
             A short noise hit combined with a triangle wave thud — start at \
             ~150Hz and drop to ~60Hz over ~0.1 seconds. Brief and weighty. \
             Duration: ~0.1-0.15 seconds. Pure 8-bit synthesis.",
        ),

        // Combat
        "shoot" | "laser" | "fire" | "projectile" | "pew" => Some(
            "Generate a laser/shoot sound effect. \
             A very short square wave burst at high frequency (~1200Hz) with a \
             slight downward pitch bend over ~0.08 seconds. Crisp attack, \
             instant cutoff. Duration: ~0.05-0.1 seconds. Must sound clean \
             when triggered in rapid succession. Pure 8-bit synthesis.",
        ),
        "explosion" | "boom" | "explode" => Some(
            "Generate an 8-bit explosion sound effect. \
             Start with a brief pop (square wave at ~300Hz for ~20ms), then \
             transition to a noise channel wash that decays over ~0.5 seconds. \
             Layer a low triangle wave (~80Hz) underneath that fades with the noise. \
             Duration: ~0.4-0.7 seconds. Sounds destructive but retro. \
             Pure 8-bit synthesis.",
        ),
        "hit" | "damage" | "hurt" | "ouch" => Some(
            "Generate a damage/hit sound effect. \
             A short noise burst (~80ms) combined with a square wave that sweeps \
             sharply downward from ~500Hz to ~100Hz over ~0.15 seconds. \
             Conveys impact and pain. Duration: ~0.15-0.25 seconds. \
             Pure 8-bit synthesis.",
        ),
        "sword" | "slash" | "melee" | "swipe" => Some(
            "Generate a sword slash / melee attack sound effect. \
             A quick noise sweep with a high-to-low filter feel — starts bright \
             and decays to nothing over ~0.15 seconds. Layer a very brief square \
             wave 'ting' at the start (~1000Hz, ~20ms) for the blade edge. \
             Duration: ~0.1-0.2 seconds. Pure 8-bit synthesis.",
        ),

        // Death / Game state
        "death" | "die" | "gameover" | "dead" => Some(
            "Generate a player death sound effect. \
             A square wave playing a descending chromatic run from C5 down to C3, \
             slowing down (ritardando) as it descends. Each note slightly longer \
             than the last. End on a low note that fades to silence. \
             Duration: ~1.0-1.5 seconds. Conveys loss and finality. \
             Pure 8-bit synthesis.",
        ),
        "victory" | "win" | "fanfare" | "levelcomplete" => Some(
            "Generate a short victory fanfare sound effect. \
             A bright square wave playing a triumphant ascending phrase: \
             C4-E4-G4-C5, then a resolving figure back down to G4-C5 with a \
             sustained final note that decays. Two voices in harmony for richness. \
             Duration: ~1.5-2.5 seconds. Major key, celebratory. \
             Pure 8-bit synthesis.",
        ),

        // UI / Menu
        "select" | "menuselect" | "cursor" | "blip" => Some(
            "Generate a menu select / cursor move sound effect. \
             A single clean square wave tone at ~800Hz, ~50ms duration. \
             Very fast attack, fast decay, no sustain. A quick 'blip'. \
             Duration: ~0.04-0.08 seconds. Pure 8-bit synthesis.",
        ),
        "confirm" | "ok" | "accept" | "menuconfirm" => Some(
            "Generate a menu confirm sound effect. \
             Two quick ascending square wave notes: C5 then E5, ~40ms each, \
             played in rapid succession. Clean and decisive. A quick 'do-mi'. \
             Duration: ~0.1-0.15 seconds. Pure 8-bit synthesis.",
        ),
        "cancel" | "back" | "menucancel" | "deny" => Some(
            "Generate a menu cancel / back sound effect. \
             Two quick descending square wave notes: E5 then C5, ~40ms each. \
             Or a very short low buzz (~200Hz noise, ~60ms). Conveys rejection \
             or backing away. Duration: ~0.08-0.12 seconds. Pure 8-bit synthesis.",
        ),
        "pause" | "menuopen" => Some(
            "Generate a pause menu open sound effect. \
             A clean square wave tone at ~600Hz with a slight upward bend, \
             ~0.1 seconds, ending with a brief silence gap. Neutral and clean. \
             Duration: ~0.1-0.15 seconds. Pure 8-bit synthesis.",
        ),

        // Environment
        "door" | "dooropen" | "gate" => Some(
            "Generate a door opening sound effect. \
             A square wave playing a short ascending two-note motif (F4-C5) with \
             a brief noise 'creak' layered on top (~0.1s noise at low volume). \
             Duration: ~0.2-0.3 seconds. Pure 8-bit synthesis.",
        ),
        "chest" | "chestopen" | "treasure" => Some(
            "Generate a treasure chest opening sound effect. \
             An ascending arpeggio on square wave: C4-E4-G4-C5, ~80ms per note, \
             with the final note ringing briefly and decaying. Similar to a power-up \
             but more deliberate and 'revealing'. Duration: ~0.4-0.6 seconds. \
             Pure 8-bit synthesis.",
        ),
        "warp" | "teleport" | "portal" => Some(
            "Generate a warp/teleport sound effect. \
             A square wave with rapid oscillating pitch (vibrato) that sweeps from \
             low (~200Hz) to high (~1500Hz) over ~0.4 seconds, then cuts off. \
             The fast vibrato creates a 'warbling' quality. Duration: ~0.3-0.5 seconds. \
             Pure 8-bit synthesis.",
        ),

        _ => None,
    }
}

/// System prompt for remixing or editing existing audio.
pub const REMIX_SYSTEM_PROMPT: &str = "\
You are a chiptune music remixer and editor. Given an existing audio track \
and an instruction, modify the music while preserving:

PRESERVATION RULES:
- Maintain the original key and tempo unless explicitly asked to change them.
- Keep the 8-bit chiptune aesthetic — do not introduce modern synthesis \
  or realistic instruments.
- Preserve the overall structure and loop points if the original loops.

MODIFICATION APPROACH:
- Apply changes musically — if asked to make it 'more intense', increase \
  tempo slightly, add harmony voices, or use faster arpeggios.
- If asked to change mood, shift the mode (major to minor or vice versa) \
  while keeping the melodic contour.
- Layer additions should respect the channel limit (3-4 voices + noise).

";
