//! # GPU 资产管理
//!
//! 管理 GPU 端的网格和材质资源，提供 Handle-based 的资产引用系统。
//! 支持管线共享：多个材质可引用同一渲染管线，避免重复创建。

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use bevy_ecs::prelude::*;
use wgpu::{Buffer, RenderPipeline, BindGroup, IndexFormat};

use crate::renderer::RenderDevice;
use crate::renderer::buffer::{Vertex, create_vertex_buffer, create_index_buffer, create_index_buffer_u32};

static NEXT_HANDLE_ID: AtomicU64 = AtomicU64::new(1);

fn next_id() -> u64 {
    NEXT_HANDLE_ID.fetch_add(1, Ordering::Relaxed)
}

/// 网格 GPU 句柄
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct MeshHandle(pub u64);

impl MeshHandle {
    /// 获取内部 ID（用于排序和批处理）
    pub fn index(&self) -> u64 { self.0 }
}

/// 材质 GPU 句柄
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct MaterialHandle(pub u64);

impl MaterialHandle {
    /// 获取内部 ID（用于排序和批处理）
    pub fn index(&self) -> u64 { self.0 }
}

/// 渲染管线句柄
///
/// 多个材质可引用同一管线，减少 GPU 管线对象的数量。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PipelineHandle(pub u64);

/// GPU 端网格数据
pub struct GpuMesh {
    /// GPU vertex buffer containing mesh vertex data.
    pub vertex_buffer: Buffer,
    /// GPU index buffer containing triangle indices.
    pub index_buffer: Buffer,
    /// Number of indices in the index buffer.
    pub index_count: u32,
    /// Index element format (Uint16 or Uint32).
    pub index_format: IndexFormat,
}

/// GPU 端材质数据
///
/// 材质通过 [`PipelineHandle`] 引用共享管线，而非直接持有 `RenderPipeline`。
pub struct GpuMaterial {
    /// Handle to the shared render pipeline used by this material.
    pub pipeline_handle: PipelineHandle,
    /// Material-specific bind group (textures, uniforms).
    pub bind_group: BindGroup,
}

/// Pipeline 缓存 key
///
/// 用于去重 pipeline 创建。相同 key 的 pipeline 可复用。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PipelineKey {
    /// 顶点格式标识
    pub vertex_format: u64,
    /// 混合模式
    pub blend_mode: BlendMode,
    /// 背面剔除模式
    pub cull_mode: CullMode,
}

/// 混合模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlendMode {
    /// 不透明
    Opaque,
    /// Alpha 混合
    AlphaBlend,
    /// 加法混合
    Additive,
}

/// 剔除模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CullMode {
    /// 无剔除
    None,
    /// 背面剔除
    Back,
    /// 正面剔除
    Front,
}

/// Pipeline 缓存
///
/// 缓存已创建的渲染管线，避免重复创建。
/// 使用 `PipelineKey` 作为缓存键。
pub struct PipelineCache {
    /// key → pipeline handle 映射
    cache: std::collections::HashMap<PipelineKey, PipelineHandle>,
}

impl PipelineCache {
    /// 创建空的 pipeline 缓存
    pub fn new() -> Self {
        Self {
            cache: std::collections::HashMap::new(),
        }
    }

    /// 获取或创建 pipeline
    ///
    /// 如果缓存中存在相同 key 的 pipeline，直接返回；
    /// 否则调用 `create_fn` 创建新 pipeline 并缓存。
    pub fn get_or_create(
        &mut self,
        key: PipelineKey,
        create_fn: impl FnOnce(&PipelineKey) -> PipelineHandle,
    ) -> PipelineHandle {
        *self.cache.entry(key.clone()).or_insert_with(|| create_fn(&key))
    }

    /// 缓存中的 pipeline 数量
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// 缓存是否为空
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// 清除所有缓存的 pipeline
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

impl Default for PipelineCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Bind group 缓存
///
/// 按 MaterialHandle 缓存 bind group，支持 dirty flag 重建。
pub struct BindGroupCache {
    /// material id → (bind_group_index, dirty)
    entries: std::collections::HashMap<u32, (u32, bool)>,
}

impl BindGroupCache {
    /// 创建空缓存
    pub fn new() -> Self {
        Self {
            entries: std::collections::HashMap::new(),
        }
    }

    /// 检查是否有缓存的 bind group
    pub fn get(&self, material_id: u32) -> Option<u32> {
        self.entries.get(&material_id)
            .filter(|(_, dirty)| !dirty)
            .map(|(idx, _)| *idx)
    }

    /// 插入或更新缓存
    pub fn insert(&mut self, material_id: u32, bind_group_index: u32) {
        self.entries.insert(material_id, (bind_group_index, false));
    }

    /// 标记为 dirty（需要重建）
    pub fn mark_dirty(&mut self, material_id: u32) {
        if let Some(entry) = self.entries.get_mut(&material_id) {
            entry.1 = true;
        }
    }

    /// 清除所有缓存
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// 缓存条目数量
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 缓存是否为空
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for BindGroupCache {
    fn default() -> Self {
        Self::new()
    }
}

/// GPU 资产存储
///
/// 管理所有已上传到 GPU 的网格、材质和渲染管线资源。
#[derive(Resource, Default)]
pub struct RenderAssets {
    meshes: HashMap<MeshHandle, GpuMesh>,
    materials: HashMap<MaterialHandle, GpuMaterial>,
    pipelines: HashMap<PipelineHandle, RenderPipeline>,
}

impl RenderAssets {
    /// 上传网格到 GPU 并返回句柄
    pub fn upload_mesh<V: Vertex>(
        &mut self,
        device: &RenderDevice,
        vertices: &[V],
        indices: &[u16],
        label: &str,
    ) -> MeshHandle {
        let vertex_buffer = create_vertex_buffer(device, &format!("{} VB", label), vertices);
        let index_buffer = create_index_buffer(device, &format!("{} IB", label), indices);
        let handle = MeshHandle(next_id());
        self.meshes.insert(handle, GpuMesh {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            index_format: IndexFormat::Uint16,
        });
        handle
    }

    /// 上传网格到 GPU（u32 索引）并返回句柄
    pub fn upload_mesh_u32<V: Vertex>(
        &mut self,
        device: &RenderDevice,
        vertices: &[V],
        indices: &[u32],
        label: &str,
    ) -> MeshHandle {
        let vertex_buffer = create_vertex_buffer(device, &format!("{} VB", label), vertices);
        let index_buffer = create_index_buffer_u32(device, &format!("{} IB", label), indices);
        let handle = MeshHandle(next_id());
        self.meshes.insert(handle, GpuMesh {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            index_format: IndexFormat::Uint32,
        });
        handle
    }

    /// 注册渲染管线并返回句柄
    ///
    /// 注册后的管线可被多个材质共享引用。
    pub fn register_pipeline(&mut self, pipeline: RenderPipeline) -> PipelineHandle {
        let handle = PipelineHandle(next_id());
        self.pipelines.insert(handle, pipeline);
        handle
    }

    /// 创建引用共享管线的材质
    ///
    /// # 参数
    ///
    /// - `pipeline_handle`: 通过 [`register_pipeline`](Self::register_pipeline) 获得的管线句柄
    /// - `bind_group`: 材质专属的绑定组
    pub fn create_material_with_pipeline(
        &mut self,
        pipeline_handle: PipelineHandle,
        bind_group: BindGroup,
    ) -> MaterialHandle {
        let handle = MaterialHandle(next_id());
        self.materials.insert(handle, GpuMaterial {
            pipeline_handle,
            bind_group,
        });
        handle
    }

    /// 创建材质并返回句柄（向后兼容 API）
    ///
    /// 内部自动注册管线并创建材质。适用于不需要管线共享的场景。
    pub fn create_material(
        &mut self,
        pipeline: RenderPipeline,
        bind_group: BindGroup,
    ) -> MaterialHandle {
        let pipeline_handle = self.register_pipeline(pipeline);
        self.create_material_with_pipeline(pipeline_handle, bind_group)
    }

    /// 获取 GPU 网格
    pub fn get_mesh(&self, handle: &MeshHandle) -> Option<&GpuMesh> {
        self.meshes.get(handle)
    }

    /// 获取 GPU 材质
    pub fn get_material(&self, handle: &MaterialHandle) -> Option<&GpuMaterial> {
        self.materials.get(handle)
    }

    /// 获取渲染管线
    pub fn get_pipeline(&self, handle: &PipelineHandle) -> Option<&RenderPipeline> {
        self.pipelines.get(handle)
    }

    /// 移除 GPU 网格资源，释放顶点和索引缓冲区
    pub fn remove_mesh(&mut self, handle: &MeshHandle) -> bool {
        self.meshes.remove(handle).is_some()
    }

    /// 移除 GPU 材质资源，释放绑定组
    pub fn remove_material(&mut self, handle: &MaterialHandle) -> bool {
        self.materials.remove(handle).is_some()
    }

    /// 移除渲染管线
    ///
    /// 注意：如果仍有材质引用此管线，那些材质的渲染将失败。
    /// 调用者应确保先移除所有引用此管线的材质。
    pub fn remove_pipeline(&mut self, handle: &PipelineHandle) -> bool {
        self.pipelines.remove(handle).is_some()
    }

    /// 已注册的网格数量
    pub fn mesh_count(&self) -> usize {
        self.meshes.len()
    }

    /// 已注册的材质数量
    pub fn material_count(&self) -> usize {
        self.materials.len()
    }

    /// 已注册的管线数量
    pub fn pipeline_count(&self) -> usize {
        self.pipelines.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_cache() {
        let mut cache = PipelineCache::new();
        assert!(cache.is_empty());

        let key = PipelineKey {
            vertex_format: 1,
            blend_mode: BlendMode::Opaque,
            cull_mode: CullMode::Back,
        };

        let handle = cache.get_or_create(key.clone(), |_| PipelineHandle(42));
        assert_eq!(handle.0, 42);
        assert_eq!(cache.len(), 1);

        // Same key should return cached handle
        let handle2 = cache.get_or_create(key, |_| PipelineHandle(99));
        assert_eq!(handle2.0, 42); // not 99 — was cached

        // Different key creates new
        let key2 = PipelineKey {
            vertex_format: 2,
            blend_mode: BlendMode::AlphaBlend,
            cull_mode: CullMode::None,
        };
        let handle3 = cache.get_or_create(key2, |_| PipelineHandle(77));
        assert_eq!(handle3.0, 77);
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_bind_group_cache() {
        let mut cache = BindGroupCache::new();
        assert!(cache.is_empty());

        cache.insert(1, 10);
        assert_eq!(cache.get(1), Some(10));

        cache.mark_dirty(1);
        assert_eq!(cache.get(1), None); // dirty = not returned

        cache.insert(1, 11); // re-create clears dirty
        assert_eq!(cache.get(1), Some(11));
        assert_eq!(cache.len(), 1);
    }
}
