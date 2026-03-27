## MODIFIED Requirements

### Requirement: Audio Engine Safety
The `AudioEngine` SHALL NOT use `unsafe impl Send` or `unsafe impl Sync`. If the underlying audio library types are not Send+Sync, the engine SHALL be accessed via `NonSend<AudioEngine>` / `NonSendMut<AudioEngine>`.

The `AudioPlugin` SHALL insert the engine using `app.insert_non_send_resource()` when the underlying types require it.

#### Scenario: Thread safety
- **WHEN** `AudioEngine` is created and inserted as a resource
- **THEN** no `unsafe` code is used for Send/Sync, and the resource access pattern matches the actual thread safety guarantees

### Requirement: Audio Asset Integration
The `audio_playback_system` SHALL load audio data through `AssetServer` asynchronously, NOT via direct `File::open()` on the game thread.

`AudioSource` SHALL reference audio data via an `AssetHandle<AudioAsset>` instead of a file path string.

#### Scenario: Async audio loading
- **WHEN** an `AudioSource` entity is spawned with an asset handle
- **THEN** the audio data is loaded asynchronously, and playback begins once loading completes

### Requirement: Stereo Panning
The `spatial_audio_system` SHALL compute stereo panning based on the listener's forward/right vectors and the source direction angle, in addition to distance-based volume attenuation.

#### Scenario: Left-right audio positioning
- **WHEN** an audio source is positioned to the left of the listener
- **THEN** the left channel volume is higher than the right channel volume
