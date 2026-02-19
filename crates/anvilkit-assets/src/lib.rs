//! # AnvilKit 资源系统
//!
//! 提供 glTF 模型加载和 CPU 侧网格数据管理。
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use anvilkit_assets::gltf_loader::load_gltf_mesh;
//!
//! let mesh = load_gltf_mesh("assets/model.glb").expect("加载失败");
//! println!("顶点数: {}, 索引数: {}", mesh.vertex_count(), mesh.index_count());
//! ```

pub mod mesh;
pub mod material;
pub mod scene;
pub mod gltf_loader;

pub mod prelude {
    pub use crate::mesh::MeshData;
    pub use crate::material::{TextureData, MaterialData};
    pub use crate::scene::{SceneData, Submesh, MultiMeshScene};
    pub use crate::gltf_loader::{load_gltf_mesh, load_gltf_scene, load_gltf_scene_multi};
}
