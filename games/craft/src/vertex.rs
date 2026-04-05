use bytemuck::{Pod, Zeroable};
use wgpu::{VertexBufferLayout, VertexStepMode, VertexAttribute, VertexFormat};
use anvilkit_render::renderer::buffer::Vertex;

/// Block vertex: position + UV + normal + AO + light.
///
/// 40 bytes per vertex. World-space positions are baked in so all chunks
/// share the same scene uniform and can be drawn in one render pass.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct BlockVertex {
    /// World-space position (x, y, z)
    pub position: [f32; 3],
    /// Texture atlas UV (u, v)
    pub uv: [f32; 2],
    /// Face normal (x, y, z)
    pub normal: [f32; 3],
    /// Ambient occlusion factor (0.0 = fully occluded, 1.0 = none)
    pub ao: f32,
    /// Packed light value: sky_light * 16.0 + block_light.
    /// Shader unpacks: sky = floor(light/16), block = light % 16.
    pub light: f32,
}

impl Vertex for BlockVertex {
    fn layout() -> VertexBufferLayout<'static> {
        const ATTRIBUTES: &[VertexAttribute] = &[
            // position: location 0
            VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: VertexFormat::Float32x3,
            },
            // uv: location 1
            VertexAttribute {
                offset: 12,
                shader_location: 1,
                format: VertexFormat::Float32x2,
            },
            // normal: location 2
            VertexAttribute {
                offset: 20,
                shader_location: 2,
                format: VertexFormat::Float32x3,
            },
            // ao: location 3
            VertexAttribute {
                offset: 32,
                shader_location: 3,
                format: VertexFormat::Float32,
            },
            // light: location 4 (packed sky*16 + block)
            VertexAttribute {
                offset: 36,
                shader_location: 4,
                format: VertexFormat::Float32,
            },
        ];

        VertexBufferLayout {
            array_stride: std::mem::size_of::<BlockVertex>() as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: ATTRIBUTES,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertex_size() {
        assert_eq!(std::mem::size_of::<BlockVertex>(), 40);
    }

    #[test]
    fn vertex_layout() {
        let layout = BlockVertex::layout();
        assert_eq!(layout.array_stride, 40);
        assert_eq!(layout.attributes.len(), 5);
    }
}
