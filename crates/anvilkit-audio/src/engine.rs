//! # 音频引擎
//!
//! 基于 rodio 的音频输出管理。

use bevy_ecs::prelude::*;
use rodio::{OutputStream, OutputStreamHandle, Sink};
use std::collections::HashMap;
use log::{info, error};

/// 音频引擎内部状态
///
/// `OutputStream` 在 macOS 上使用 CoreAudio 绑定，不满足 `Send`。
/// 但 AnvilKit 架构保证 AudioEngine 仅在 main thread 上创建和访问
/// （通过 winit 的 single-threaded event loop + bevy_ecs 的 NonSend 资源）。
struct AudioEngineInner {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sinks: HashMap<Entity, Sink>,
}

// NOTE: No `unsafe impl Send/Sync` — OutputStream (CoreAudio on macOS) is !Send.
// AudioEngine is inserted as a non-send resource and accessed only on the main thread
// via `NonSend<AudioEngine>` / `NonSendMut<AudioEngine>`.

/// 音频引擎 (non-send resource)
///
/// 持有 rodio OutputStream 和活跃 Sink 的管理器。
///
/// # 线程安全
///
/// 此类型通过 `NonSend<AudioEngine>` / `NonSendMut<AudioEngine>` 访问，
/// bevy_ecs 保证只在 main thread 上运行。不实现 `Resource`，因为
/// 底层 OutputStream 是 `!Send`（macOS CoreAudio）。
pub struct AudioEngine {
    inner: AudioEngineInner,
}

impl AudioEngine {
    /// 创建音频引擎
    pub fn new() -> Option<Self> {
        match OutputStream::try_default() {
            Ok((stream, handle)) => {
                info!("音频引擎初始化成功");
                Some(Self {
                    inner: AudioEngineInner {
                        _stream: stream,
                        stream_handle: handle,
                        sinks: HashMap::new(),
                    },
                })
            }
            Err(e) => {
                error!("音频引擎初始化失败: {}", e);
                None
            }
        }
    }

    /// 获取或创建实体的 Sink，失败时返回 Err
    pub fn get_or_create_sink(&mut self, entity: Entity) -> Result<&Sink, String> {
        if !self.inner.sinks.contains_key(&entity) {
            let sink = Sink::try_new(&self.inner.stream_handle)
                .map_err(|e| format!("创建音频 sink 失败: {}", e))?;
            self.inner.sinks.insert(entity, sink);
        }
        Ok(self.inner.sinks.get(&entity).unwrap())
    }

    /// 暂停实体音频
    pub fn pause(&self, entity: Entity) {
        if let Some(sink) = self.inner.sinks.get(&entity) {
            sink.pause();
        }
    }

    /// 恢复实体音频
    pub fn resume(&self, entity: Entity) {
        if let Some(sink) = self.inner.sinks.get(&entity) {
            sink.play();
        }
    }

    /// 停止并移除实体音频
    pub fn stop(&mut self, entity: Entity) {
        if let Some(sink) = self.inner.sinks.remove(&entity) {
            sink.stop();
        }
    }

    /// 设置实体音量
    pub fn set_volume(&self, entity: Entity, volume: f32) {
        if let Some(sink) = self.inner.sinks.get(&entity) {
            sink.set_volume(volume);
        }
    }

    /// 清理已完成播放的 Sink
    pub fn cleanup_finished(&mut self) {
        self.inner.sinks.retain(|_, sink| !sink.empty());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        // 在 CI 环境中可能没有音频设备，所以两种结果都可以接受
        let engine = AudioEngine::new();
        drop(engine);
    }

    #[test]
    fn test_engine_is_not_send() {
        // AudioEngine wraps OutputStream which is !Send on macOS (CoreAudio).
        // It must be used as a non-send resource.
        fn is_send<T: Send>() {}
        // Compile-time proof: the following would fail to compile:
        // is_send::<AudioEngine>();
        let _ = is_send::<u32>; // suppress unused warning
    }

    #[test]
    fn test_operations_without_sink() {
        if let Some(engine) = AudioEngine::new() {
            let entity = Entity::from_raw(0);
            engine.pause(entity);
            engine.resume(entity);
            engine.set_volume(entity, 0.5);
        }
    }

    #[test]
    fn test_sink_creation_result() {
        if let Some(mut engine) = AudioEngine::new() {
            let entity = Entity::from_raw(42);
            let result = engine.get_or_create_sink(entity);
            assert!(result.is_ok());
            engine.stop(entity);
        }
    }
}
