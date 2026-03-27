# audio-system Specification

## Purpose
TBD - created by archiving change add-engine-usability-overhaul. Update Purpose after archive.
## Requirements
### Requirement: Audio Playback System

The system SHALL provide an `AudioEngine` resource managing audio output via the rodio backend, with per-entity `Sink` management for play, pause, resume, and stop operations.

The `audio_playback_system` SHALL read and apply all fields of the `AudioSource` component:
- `looping` — when true, audio repeats indefinitely via `Sink::append()` with `Repeat::Infinite`
- `pitch` — applied via `Sink::set_speed(pitch)` where 1.0 is normal speed
- `volume` — applied via `Sink::set_volume(volume)`
- `spatial` — when true, enables distance-based attenuation (see Spatial Audio requirement)

#### Scenario: Looping playback
- **WHEN** an `AudioSource` entity has `looping: true` and transitions to `PlaybackState::Playing`
- **THEN** the audio loops continuously until stopped or the entity is despawned

#### Scenario: Pitch adjustment
- **WHEN** an `AudioSource` entity has `pitch: 2.0`
- **THEN** the audio plays at double speed (one octave higher)

#### Scenario: Volume control
- **WHEN** an `AudioSource` entity has `volume: 0.5`
- **THEN** the audio plays at 50% of full volume

### Requirement: Spatial Audio

The system SHALL implement distance-based audio attenuation using the `AudioListener` and `AudioSource` entity positions (from their `Transform` components).

The attenuation model SHALL be linear: `effective_volume = source_volume * max(0.0, 1.0 - distance / spatial_range)`.

When `AudioSource.spatial` is false, no distance attenuation SHALL be applied.

#### Scenario: Close source
- **WHEN** an AudioSource is 10 units from the AudioListener with `spatial_range: 100.0` and `volume: 1.0`
- **THEN** effective volume is `1.0 * (1.0 - 10/100) = 0.9`

#### Scenario: Out of range
- **WHEN** an AudioSource is 150 units from the AudioListener with `spatial_range: 100.0`
- **THEN** effective volume is 0.0 (silent)

#### Scenario: Non-spatial source
- **WHEN** an AudioSource has `spatial: false`
- **THEN** volume is `source_volume` regardless of distance

### Requirement: Audio Bus System

The system SHALL provide an `AudioBus` resource with a master volume and per-category volumes:
- `master` — global volume multiplier (default 1.0)
- `music` — background music volume (default 1.0)
- `sfx` — sound effects volume (default 1.0)
- `voice` — voice/dialogue volume (default 1.0)

Each `AudioSource` SHALL have a `bus: AudioBusCategory` field (default: SFX) that determines which category volume applies.

Final volume SHALL be: `source_volume * category_volume * master_volume * spatial_attenuation`.

#### Scenario: Master mute
- **WHEN** `AudioBus.master` is set to 0.0
- **THEN** all audio output is silent regardless of individual source volumes

#### Scenario: Category volume
- **WHEN** `AudioBus.music` is set to 0.3 and a music source has `volume: 1.0`
- **THEN** the effective volume is `1.0 * 0.3 * master = 0.3 * master`

### Requirement: Audio Asset Integration

The system SHALL load audio files through the `AssetServer` pipeline instead of directly opening files from disk paths.

Audio assets SHALL support WAV, OGG (Vorbis), and MP3 formats.

#### Scenario: Asset-based loading
- **WHEN** an `AudioSource` is created with `path: "sounds/explosion.ogg"`
- **THEN** the audio file is loaded via `AssetServer` (with caching and async support)

#### Scenario: Hot reload
- **WHEN** an audio file is modified on disk and hot-reload is enabled
- **THEN** the audio asset is automatically reloaded and new playbacks use the updated file

### Requirement: Thread-Safe Audio Engine
The system SHALL provide `AudioEngine` as an ECS `Resource` that is safe to access from any ECS system thread without undefined behavior.

The system SHALL NOT use `unsafe impl Send` or `unsafe impl Sync` to force thread safety. Instead, non-Send inner state SHALL be wrapped in a `Mutex` or the audio backend SHALL be accessed through a thread-safe channel.

#### Scenario: Parallel system access
- **WHEN** the ECS scheduler runs an audio system on a worker thread
- **THEN** the `AudioEngine` resource is safely accessed without undefined behavior

#### Scenario: No unsafe Send/Sync
- **WHEN** the `AudioEngine` type is inspected
- **THEN** it derives `Send + Sync` naturally through its fields (e.g., `Mutex<Inner>`) without any `unsafe impl`

### Requirement: Audio Error Handling
The system SHALL handle audio sink creation failures gracefully by returning `Result` instead of panicking.

All audio system functions SHALL propagate errors via `Result<T, AudioError>` rather than using `.expect()` or `.unwrap()`.

#### Scenario: Sink creation failure
- **WHEN** the audio backend fails to create a playback sink (e.g., no audio device available)
- **THEN** the system returns an `AudioError` and continues running without crashing

#### Scenario: Missing audio file
- **WHEN** `audio_playback_system` attempts to play a file that does not exist
- **THEN** an error is logged and the system continues processing remaining audio commands

### Requirement: Non-Blocking Audio Loading
The system SHALL NOT perform blocking file I/O (`File::open`, `Decoder::new`) within ECS systems running on the main game loop.

Audio file loading SHALL use pre-loaded buffers, a background loading thread, or an async task to prevent frame stalls.

#### Scenario: Large audio file
- **WHEN** a 50MB audio file is requested for playback
- **THEN** the file is loaded in the background and playback begins when ready, without stalling the main game loop

#### Scenario: Small audio file
- **WHEN** a small sound effect (<100KB) is requested
- **THEN** playback begins within 1-2 frames without blocking the update loop

### Requirement: Audio Engine Testing
The audio engine SHALL have unit tests covering initialization, sink management, volume control, and error handling.

#### Scenario: Engine initialization
- **WHEN** `AudioEngine::new()` is called
- **THEN** the engine initializes successfully or returns an error if no audio device is available

#### Scenario: Volume control
- **WHEN** `set_volume(channel, 0.5)` is called
- **THEN** the specified channel's volume is set to 50%

#### Scenario: Sink lifecycle
- **WHEN** a sink is created, used for playback, and then dropped
- **THEN** no resources leak and the audio device remains functional

