//! # GPU 资产管理
//!
//! 管理 GPU 端的网格和材质资源，提供 Handle-based 的资产引用系统。

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

/// 材质 GPU 句柄
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct MaterialHandle(pub u64);

/// GPU 端网格数据
pub struct GpuMesh {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub index_count: u32,
    pub index_format: IndexFormat,
}

/// GPU 端材质数据
pub struct GpuMaterial {
    pub pipeline: RenderPipeline,
    pub bind_group: BindGroup,
}

/// GPU 资产存储
///
/// 管理所有已上传到 GPU 的网格和材质资源。
#[derive(Resource, Default)]
pub struct RenderAssets {
    meshes: HashMap<MeshHandle, GpuMesh>,
    materials: HashMap<MaterialHandle, GpuMaterial>,
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

    /// 创建材质并返回句柄
    pub fn create_material(
        &mut self,
        pipeline: RenderPipeline,
        bind_group: BindGroup,
    ) -> MaterialHandle {
        let handle = MaterialHandle(next_id());
        self.materials.insert(handle, GpuMaterial {
            pipeline,
            bind_group,
        });
        handle
    }

    /// 获取 GPU 网格
    pub fn get_mesh(&self, handle: &MeshHandle) -> Option<&GpuMesh> {
        self.meshes.get(handle)
    }

    /// 获取 GPU 材质
    pub fn get_material(&self, handle: &MaterialHandle) -> Option<&GpuMaterial> {
        self.materials.get(handle)
    }
}
