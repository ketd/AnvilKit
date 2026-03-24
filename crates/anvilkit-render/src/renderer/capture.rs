//! # 帧捕获模块
//!
//! 提供 GPU 帧 readback 能力，支持单帧截图和连续录帧。
//! 通过 `capture` feature 启用。

use std::path::{Path, PathBuf};
use bevy_ecs::prelude::Resource;
use log::info;

/// 帧捕获状态（ECS Resource）
///
/// 控制截图和录制行为。插入到 ECS World 后由渲染循环自动检测。
///
/// # 示例
///
/// ```rust
/// use anvilkit_render::renderer::capture::CaptureState;
///
/// // 单帧截图
/// let state = CaptureState::screenshot("output/screenshot.png");
///
/// // 连续录帧（150 帧后自动退出）
/// let state = CaptureState::recording("output/frames", 150);
/// ```
#[derive(Debug, Clone, Resource)]
pub struct CaptureState {
    /// 单帧截图请求路径（截完自动清除）
    pub screenshot_path: Option<PathBuf>,
    /// 是否正在录制
    pub recording: bool,
    /// 录制帧输出目录
    pub output_dir: Option<PathBuf>,
    /// 最大录制帧数（0 = 无限）
    pub max_frames: u32,
    /// 当前已录制帧数
    pub frame_count: u32,
    /// 录完后自动退出
    pub auto_exit: bool,
    /// 内部标志：请求退出
    pub exit_requested: bool,
}

impl Default for CaptureState {
    fn default() -> Self {
        Self {
            screenshot_path: None,
            recording: false,
            output_dir: None,
            max_frames: 0,
            frame_count: 0,
            auto_exit: false,
            exit_requested: false,
        }
    }
}

impl CaptureState {
    /// 创建单帧截图请求
    pub fn screenshot(path: impl Into<PathBuf>) -> Self {
        Self {
            screenshot_path: Some(path.into()),
            ..Default::default()
        }
    }

    /// 创建录帧模式（自动退出）
    pub fn recording(output_dir: impl Into<PathBuf>, max_frames: u32) -> Self {
        Self {
            recording: true,
            output_dir: Some(output_dir.into()),
            max_frames,
            auto_exit: true,
            ..Default::default()
        }
    }

    /// 是否需要捕获当前帧
    pub fn should_capture(&self) -> bool {
        self.screenshot_path.is_some()
            || (self.recording && (self.max_frames == 0 || self.frame_count < self.max_frames))
    }

    /// 处理一帧捕获完成后的状态更新
    pub fn on_frame_captured(&mut self) {
        // 单帧截图：清除请求
        if self.screenshot_path.is_some() {
            self.screenshot_path = None;
        }

        // 录帧模式：递增计数
        if self.recording {
            self.frame_count += 1;
            if self.max_frames > 0 && self.frame_count >= self.max_frames {
                self.recording = false;
                if self.auto_exit {
                    self.exit_requested = true;
                }
                info!("录帧完成: {} 帧", self.frame_count);
            }
        }
    }

    /// 获取当前帧的输出路径
    pub fn current_output_path(&self) -> Option<PathBuf> {
        if let Some(ref path) = self.screenshot_path {
            return Some(path.clone());
        }
        if self.recording {
            if let Some(ref dir) = self.output_dir {
                return Some(dir.join(format!("frame_{:04}.png", self.frame_count)));
            }
        }
        None
    }

    /// 解析命令行参数
    pub fn from_args() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let mut state = Self::default();
        let mut max_frames: u32 = 150;

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--capture-dir" => {
                    if i + 1 < args.len() {
                        state.output_dir = Some(PathBuf::from(&args[i + 1]));
                        state.recording = true;
                        state.auto_exit = true;
                        i += 1;
                    }
                }
                "--capture-frames" => {
                    if i + 1 < args.len() {
                        max_frames = args[i + 1].parse().unwrap_or(150);
                        i += 1;
                    }
                }
                "--no-capture" => {
                    state.recording = false;
                    state.output_dir = None;
                    state.auto_exit = false;
                }
                _ => {}
            }
            i += 1;
        }

        if state.recording {
            state.max_frames = max_frames;
        }

        state
    }
}

/// GPU 帧捕获资源
///
/// 管理 capture texture 和 staging buffer，用于 GPU → CPU 像素回读。
pub struct CaptureResources {
    /// 捕获纹理（RENDER_ATTACHMENT | COPY_SRC）
    pub capture_texture: wgpu::Texture,
    /// 捕获纹理视图
    pub capture_view: wgpu::TextureView,
    /// Staging buffer（MAP_READ | COPY_DST）
    staging_buffer: wgpu::Buffer,
    /// 对齐后的每行字节数（256 对齐）
    padded_bytes_per_row: u32,
    /// 实际每行字节数（width * 4）
    unpadded_bytes_per_row: u32,
    /// 纹理宽度
    pub width: u32,
    /// 纹理高度
    pub height: u32,
    /// 纹理格式
    format: wgpu::TextureFormat,
}

impl CaptureResources {
    /// 创建捕获资源
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> Self {
        let capture_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Capture Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let capture_view = capture_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bytes_per_pixel = 4u32; // BGRA8 or RGBA8
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row = (unpadded_bytes_per_row + align - 1) / align * align;

        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Capture Staging Buffer"),
            size: (padded_bytes_per_row * height) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            capture_texture,
            capture_view,
            staging_buffer,
            padded_bytes_per_row,
            unpadded_bytes_per_row,
            width,
            height,
            format,
        }
    }

    /// 向 encoder 添加 copy_texture_to_buffer 命令
    pub fn encode_copy(&self, encoder: &mut wgpu::CommandEncoder) {
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &self.capture_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &self.staging_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(self.padded_bytes_per_row),
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
    }

    /// 同步读取 staging buffer 中的像素数据（RGBA 格式）
    ///
    /// 调用此方法前必须已经 submit 了包含 copy 命令的 encoder。
    pub fn read_pixels(&self, device: &wgpu::Device) -> Vec<u8> {
        let buffer_slice = self.staging_buffer.slice(..);

        let (sender, receiver) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).unwrap();
        });
        device.poll(wgpu::Maintain::Wait);
        receiver.recv().unwrap().unwrap();

        let data = buffer_slice.get_mapped_range();

        // 去除行 padding + BGRA → RGBA 转换
        let is_bgra = matches!(
            self.format,
            wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb
        );

        let mut pixels = Vec::with_capacity((self.width * self.height * 4) as usize);
        for row in 0..self.height {
            let offset = (row * self.padded_bytes_per_row) as usize;
            let row_data = &data[offset..offset + self.unpadded_bytes_per_row as usize];

            if is_bgra {
                // BGRA → RGBA: swap B and R channels
                for chunk in row_data.chunks_exact(4) {
                    pixels.push(chunk[2]); // R
                    pixels.push(chunk[1]); // G
                    pixels.push(chunk[0]); // B
                    pixels.push(chunk[3]); // A
                }
            } else {
                pixels.extend_from_slice(row_data);
            }
        }

        drop(data);
        self.staging_buffer.unmap();

        pixels
    }

    /// 窗口 resize 时重建资源
    pub fn resize(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) {
        if width == self.width && height == self.height {
            return;
        }
        *self = Self::new(device, width, height, self.format);
    }
}

/// 保存 RGBA 像素数据为 PNG 文件
pub fn save_png(pixels: &[u8], width: u32, height: u32, path: &Path) {
    use image::{ImageBuffer, Rgba};

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let img: ImageBuffer<Rgba<u8>, _> =
        ImageBuffer::from_raw(width, height, pixels.to_vec())
            .expect("像素数据大小与图像尺寸不匹配");

    img.save(path).unwrap_or_else(|e| {
        log::error!("保存截图失败 {:?}: {}", path, e);
    });

    info!("截图已保存: {:?}", path);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_state_default() {
        let state = CaptureState::default();
        assert!(!state.recording);
        assert!(!state.should_capture());
        assert!(!state.auto_exit);
    }

    #[test]
    fn test_capture_state_screenshot() {
        let state = CaptureState::screenshot("test.png");
        assert!(state.should_capture());
        assert!(state.screenshot_path.is_some());
        assert!(!state.recording);
    }

    #[test]
    fn test_capture_state_recording() {
        let mut state = CaptureState::recording("/tmp/frames", 3);
        assert!(state.should_capture());
        assert!(state.recording);
        assert!(state.auto_exit);
        assert_eq!(state.max_frames, 3);

        // 模拟 3 帧
        state.on_frame_captured();
        assert_eq!(state.frame_count, 1);
        assert!(state.should_capture());

        state.on_frame_captured();
        assert_eq!(state.frame_count, 2);
        assert!(state.should_capture());

        state.on_frame_captured();
        assert_eq!(state.frame_count, 3);
        assert!(!state.should_capture());
        assert!(!state.recording);
        assert!(state.exit_requested);
    }

    #[test]
    fn test_capture_state_output_path() {
        let state = CaptureState::recording("/tmp/out", 10);
        assert_eq!(
            state.current_output_path().unwrap(),
            PathBuf::from("/tmp/out/frame_0000.png")
        );
    }

    #[test]
    fn test_capture_state_screenshot_clears() {
        let mut state = CaptureState::screenshot("shot.png");
        assert!(state.should_capture());
        state.on_frame_captured();
        assert!(!state.should_capture());
        assert!(state.screenshot_path.is_none());
    }

    #[test]
    fn test_row_alignment() {
        // 验证行对齐计算
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let width = 640u32;
        let unpadded = width * 4;
        let padded = (unpadded + align - 1) / align * align;
        assert_eq!(padded % align, 0);
        assert!(padded >= unpadded);
    }
}
