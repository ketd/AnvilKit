//! # 音频引擎
//!
//! 基于 rodio 的音频输出管理。

use bevy_ecs::prelude::*;
use rodio::{OutputStream, OutputStreamHandle, Sink};
use std::collections::HashMap;
use log::{info, error};

/// 音频引擎资源
///
/// 持有 rodio OutputStream 和活跃 Sink 的管理器。
///
/// # Safety
/// OutputStream 内部使用 cpal 平台绑定，某些平台不自动 Send/Sync，
/// 但在 winit 单窗口应用中始终在同一线程使用，是安全的。
pub struct AudioEngine {
    // OutputStream 必须保持存活，否则音频会停止
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    /// Entity → Sink 映射
    sinks: HashMap<Entity, Sink>,
}

// SAFETY: AudioEngine is only accessed from the main thread in our single-window architecture.
unsafe impl Send for AudioEngine {}
unsafe impl Sync for AudioEngine {}

impl Resource for AudioEngine {}

impl AudioEngine {
    /// 创建音频引擎
    pub fn new() -> Option<Self> {
        match OutputStream::try_default() {
            Ok((stream, handle)) => {
                info!("音频引擎初始化成功");
                Some(Self {
                    _stream: stream,
                    stream_handle: handle,
                    sinks: HashMap::new(),
                })
            }
            Err(e) => {
                error!("音频引擎初始化失败: {}", e);
                None
            }
        }
    }

    /// 获取输出流句柄
    pub fn stream_handle(&self) -> &OutputStreamHandle {
        &self.stream_handle
    }

    /// 获取或创建实体的 Sink
    pub fn get_or_create_sink(&mut self, entity: Entity) -> &Sink {
        self.sinks.entry(entity).or_insert_with(|| {
            Sink::try_new(&self.stream_handle)
                .expect("Failed to create audio sink")
        })
    }

    /// 获取实体的 Sink（如果存在）
    pub fn get_sink(&self, entity: Entity) -> Option<&Sink> {
        self.sinks.get(&entity)
    }

    /// 移除实体的 Sink
    pub fn remove_sink(&mut self, entity: Entity) {
        if let Some(sink) = self.sinks.remove(&entity) {
            sink.stop();
        }
    }

    /// 暂停实体音频
    pub fn pause(&self, entity: Entity) {
        if let Some(sink) = self.sinks.get(&entity) {
            sink.pause();
        }
    }

    /// 恢复实体音频
    pub fn resume(&self, entity: Entity) {
        if let Some(sink) = self.sinks.get(&entity) {
            sink.play();
        }
    }

    /// 停止实体音频
    pub fn stop(&mut self, entity: Entity) {
        self.remove_sink(entity);
    }

    /// 设置实体音量
    pub fn set_volume(&self, entity: Entity, volume: f32) {
        if let Some(sink) = self.sinks.get(&entity) {
            sink.set_volume(volume);
        }
    }

    /// 清理已完成播放的 Sink
    pub fn cleanup_finished(&mut self) {
        self.sinks.retain(|_, sink| !sink.empty());
    }
}
