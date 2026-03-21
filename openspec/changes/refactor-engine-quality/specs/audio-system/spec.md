## ADDED Requirements

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
