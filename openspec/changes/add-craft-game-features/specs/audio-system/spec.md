## MODIFIED Requirements

### Requirement: Audio Playback System
The `audio_playback_system` SHALL support loading audio files both from direct file paths (`AudioSource.path`) and from `AssetServer`-managed assets (`AudioSource.asset_id`).

When an entity with `AudioSource` is despawned, the associated audio `Sink` SHALL be cleaned up automatically by an `audio_cleanup_system`.

#### Scenario: Path-based playback
- **WHEN** an entity with `AudioSource::new("sounds/break.wav")` transitions to `PlaybackState::Playing`
- **THEN** the sound file is loaded from disk and played through the audio output

#### Scenario: Entity despawn cleanup
- **WHEN** an entity with an active `AudioSource` is despawned
- **THEN** the audio sink is stopped and removed from the internal sink map within the next frame

## ADDED Requirements

### Requirement: Audio Cleanup System
The engine SHALL provide an `audio_cleanup_system` that detects removed `AudioSource` entities and releases their associated audio resources.

#### Scenario: Sink leak prevention
- **WHEN** 100 `AudioSource` entities are spawned and then despawned over 60 seconds
- **THEN** the internal sink map contains 0 entries after cleanup, not 100 leaked entries
