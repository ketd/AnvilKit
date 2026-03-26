//! # glTF 模型加载
//!
//! 提供 glTF/GLB 文件的网格数据提取功能。

use std::path::Path;
use glam::{Vec2, Vec3};
use log::info;

use anvilkit_core::error::{AnvilKitError, Result};
use crate::mesh::MeshData;
use crate::material::{TextureData, MaterialData};
use crate::scene::SceneData;

/// 从 glTF/GLB 文件加载第一个网格的第一个图元
///
/// 提取顶点位置（必须）、法线（必须）、纹理坐标（可选，默认为零）和索引（必须）。
///
/// # 参数
///
/// - `path`: glTF 或 GLB 文件路径
///
/// # 返回
///
/// 成功时返回 `MeshData`，失败时返回 `AnvilKitError::Asset`
///
/// # 示例
///
/// ```rust,no_run
/// use anvilkit_assets::gltf_loader::load_gltf_mesh;
///
/// let mesh = load_gltf_mesh("assets/suzanne.glb").expect("加载失败");
/// println!("顶点: {}, 索引: {}", mesh.vertex_count(), mesh.index_count());
/// ```
pub fn load_gltf_mesh(path: impl AsRef<Path>) -> Result<MeshData> {
    let path = path.as_ref();
    info!("加载 glTF 文件: {}", path.display());

    // 导入 glTF 文件
    let (document, buffers, _images) = gltf::import(path)
        .map_err(|e| AnvilKitError::asset_with_path(
            format!("glTF 导入失败: {}", e),
            path.to_string_lossy().to_string(),
        ))?;

    // 获取第一个网格
    let mesh = document.meshes().next()
        .ok_or_else(|| AnvilKitError::asset_with_path(
            "glTF 文件中没有网格".to_string(),
            path.to_string_lossy().to_string(),
        ))?;

    info!("网格名称: {:?}", mesh.name());

    // 获取第一个图元
    let primitive = mesh.primitives().next()
        .ok_or_else(|| AnvilKitError::asset_with_path(
            "网格中没有图元".to_string(),
            path.to_string_lossy().to_string(),
        ))?;

    // 创建缓冲区读取器
    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

    // 读取顶点位置（必须）
    let positions: Vec<Vec3> = reader.read_positions()
        .ok_or_else(|| AnvilKitError::asset_with_path(
            "网格缺少顶点位置属性".to_string(),
            path.to_string_lossy().to_string(),
        ))?
        .map(|p| Vec3::from(p))
        .collect();

    // 读取法线（必须）
    let normals: Vec<Vec3> = reader.read_normals()
        .ok_or_else(|| AnvilKitError::asset_with_path(
            "网格缺少法线属性".to_string(),
            path.to_string_lossy().to_string(),
        ))?
        .map(|n| Vec3::from(n))
        .collect();

    // 读取纹理坐标（可选，默认为零）
    let texcoords: Vec<Vec2> = reader.read_tex_coords(0)
        .map(|tc| tc.into_f32().map(|uv| Vec2::from(uv)).collect())
        .unwrap_or_else(|| vec![Vec2::ZERO; positions.len()]);

    // 读取切线（可选，默认为 [1,0,0,1] = +X tangent, +1 bitangent sign）
    let tangents: Vec<[f32; 4]> = reader.read_tangents()
        .map(|t| t.collect())
        .unwrap_or_else(|| vec![[1.0, 0.0, 0.0, 1.0]; positions.len()]);

    // 读取索引（必须）
    let indices: Vec<u32> = reader.read_indices()
        .ok_or_else(|| AnvilKitError::asset_with_path(
            "网格缺少索引数据".to_string(),
            path.to_string_lossy().to_string(),
        ))?
        .into_u32()
        .collect();

    info!("加载完成: {} 顶点, {} 索引", positions.len(), indices.len());

    Ok(MeshData {
        positions,
        normals,
        texcoords,
        tangents,
        indices,
    })
}

/// 从 glTF/GLB 文件加载场景数据（网格 + 材质 + 纹理）
///
/// 提取第一个网格的几何数据和对应的材质信息（含基础色纹理）。
///
/// # 示例
///
/// ```rust,no_run
/// use anvilkit_assets::gltf_loader::load_gltf_scene;
///
/// let scene = load_gltf_scene("assets/model.glb").expect("加载失败");
/// println!("顶点: {}, 有纹理: {}", scene.mesh.vertex_count(),
///     scene.material.base_color_texture.is_some());
/// ```
pub fn load_gltf_scene(path: impl AsRef<Path>) -> Result<SceneData> {
    let path = path.as_ref();
    info!("加载 glTF 场景: {}", path.display());

    let (document, buffers, images) = gltf::import(path)
        .map_err(|e| AnvilKitError::asset_with_path(
            format!("glTF 导入失败: {}", e),
            path.to_string_lossy().to_string(),
        ))?;

    // 获取第一个网格
    let gltf_mesh = document.meshes().next()
        .ok_or_else(|| AnvilKitError::asset_with_path(
            "glTF 文件中没有网格".to_string(),
            path.to_string_lossy().to_string(),
        ))?;

    let primitive = gltf_mesh.primitives().next()
        .ok_or_else(|| AnvilKitError::asset_with_path(
            "网格中没有图元".to_string(),
            path.to_string_lossy().to_string(),
        ))?;

    // 提取网格数据
    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

    let positions: Vec<Vec3> = reader.read_positions()
        .ok_or_else(|| AnvilKitError::asset_with_path(
            "网格缺少顶点位置属性".to_string(),
            path.to_string_lossy().to_string(),
        ))?
        .map(Vec3::from)
        .collect();

    let normals: Vec<Vec3> = reader.read_normals()
        .ok_or_else(|| AnvilKitError::asset_with_path(
            "网格缺少法线属性".to_string(),
            path.to_string_lossy().to_string(),
        ))?
        .map(Vec3::from)
        .collect();

    let texcoords: Vec<Vec2> = reader.read_tex_coords(0)
        .map(|tc| tc.into_f32().map(Vec2::from).collect())
        .unwrap_or_else(|| vec![Vec2::ZERO; positions.len()]);

    let tangents: Vec<[f32; 4]> = reader.read_tangents()
        .map(|t| t.collect())
        .unwrap_or_else(|| vec![[1.0, 0.0, 0.0, 1.0]; positions.len()]);

    let indices: Vec<u32> = reader.read_indices()
        .ok_or_else(|| AnvilKitError::asset_with_path(
            "网格缺少索引数据".to_string(),
            path.to_string_lossy().to_string(),
        ))?
        .into_u32()
        .collect();

    let mesh = MeshData { positions, normals, texcoords, tangents, indices };

    // 提取材质数据
    let material = extract_material(&primitive, &images);

    info!("场景加载完成: {} 顶点, {} 索引, 纹理: {}, 法线贴图: {}",
        mesh.vertex_count(), mesh.index_count(),
        material.base_color_texture.is_some(),
        material.normal_texture.is_some());

    Ok(SceneData { mesh, material })
}

/// 从 glTF/GLB 文件加载多子网格场景数据
///
/// 提取所有 mesh 的所有 primitive，每个 primitive 生成一个 Submesh。
///
/// # 示例
///
/// ```rust,no_run
/// use anvilkit_assets::gltf_loader::load_gltf_scene_multi;
///
/// let scene = load_gltf_scene_multi("assets/model.glb").expect("加载失败");
/// println!("子网格: {}, 总顶点: {}", scene.submesh_count(), scene.total_vertex_count());
/// ```
pub fn load_gltf_scene_multi(path: impl AsRef<Path>) -> Result<crate::scene::MultiMeshScene> {
    let path = path.as_ref();
    info!("加载 glTF 多子网格场景: {}", path.display());

    let (document, buffers, images) = gltf::import(path)
        .map_err(|e| AnvilKitError::asset_with_path(
            format!("glTF 导入失败: {}", e),
            path.to_string_lossy().to_string(),
        ))?;

    let mut submeshes = Vec::new();

    for gltf_mesh in document.meshes() {
        for primitive in gltf_mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            let Some(positions) = reader.read_positions().map(|p| p.map(Vec3::from).collect::<Vec<_>>()) else {
                continue;
            };

            let normals: Vec<Vec3> = reader.read_normals()
                .map(|n| n.map(Vec3::from).collect())
                .unwrap_or_else(|| {
                    log::warn!("Mesh missing normals, using default up direction");
                    vec![Vec3::Y; positions.len()]
                });

            let texcoords: Vec<Vec2> = reader.read_tex_coords(0)
                .map(|tc| tc.into_f32().map(Vec2::from).collect())
                .unwrap_or_else(|| vec![Vec2::ZERO; positions.len()]);

            let tangents: Vec<[f32; 4]> = reader.read_tangents()
                .map(|t| t.collect())
                .unwrap_or_else(|| vec![[1.0, 0.0, 0.0, 1.0]; positions.len()]);

            let Some(indices) = reader.read_indices().map(|i| i.into_u32().collect::<Vec<_>>()) else {
                continue;
            };

            let mesh = MeshData { positions, normals, texcoords, tangents, indices };
            let material = extract_material(&primitive, &images);

            info!("子网格: {} 顶点, {} 索引", mesh.vertex_count(), mesh.index_count());
            submeshes.push(crate::scene::Submesh { mesh, material });
        }
    }

    info!("多子网格场景加载完成: {} 个子网格", submeshes.len());
    Ok(crate::scene::MultiMeshScene { submeshes })
}

/// 从 glTF primitive 提取材质数据
fn extract_material(
    primitive: &gltf::Primitive<'_>,
    images: &[gltf::image::Data],
) -> MaterialData {
    let gltf_material = primitive.material();
    let pbr = gltf_material.pbr_metallic_roughness();

    let base_color_factor = pbr.base_color_factor();

    // 提取基础色纹理
    let base_color_texture = pbr.base_color_texture()
        .and_then(|tex_info| {
            let texture = tex_info.texture();
            let source = texture.source();
            let image_index = source.index();

            if image_index < images.len() {
                let image = &images[image_index];
                // 转换为 RGBA8
                let rgba_data = convert_to_rgba8(image);
                rgba_data.map(|data| {
                    info!("加载基础色纹理: {}x{}", image.width, image.height);
                    TextureData {
                        width: image.width,
                        height: image.height,
                        data,
                    }
                })
            } else {
                None
            }
        });

    let metallic_factor = pbr.metallic_factor();
    let roughness_factor = pbr.roughness_factor();

    // 提取法线贴图
    let (normal_texture, normal_scale) = gltf_material.normal_texture()
        .map(|normal_tex| {
            let scale = normal_tex.scale();
            let texture = normal_tex.texture();
            let source = texture.source();
            let image_index = source.index();

            let tex_data = if image_index < images.len() {
                let image = &images[image_index];
                convert_to_rgba8(image).map(|data| {
                    info!("加载法线贴图: {}x{}", image.width, image.height);
                    TextureData {
                        width: image.width,
                        height: image.height,
                        data,
                    }
                })
            } else {
                None
            };
            (tex_data, scale)
        })
        .unwrap_or((None, 1.0));

    // 提取金属度/粗糙度纹理
    let metallic_roughness_texture = pbr.metallic_roughness_texture()
        .and_then(|tex_info| extract_texture_by_source(&tex_info.texture(), images, "金属度/粗糙度"));

    // 提取环境光遮蔽纹理
    let occlusion_texture = gltf_material.occlusion_texture()
        .and_then(|tex_info| extract_texture_by_source(&tex_info.texture(), images, "AO"));

    // 提取自发光纹理和因子
    let emissive_texture = gltf_material.emissive_texture()
        .and_then(|tex_info| extract_texture_by_source(&tex_info.texture(), images, "自发光"));
    let emissive_factor = gltf_material.emissive_factor();

    info!("材质: metallic={}, roughness={}, normal={}, mr_tex={}, ao={}, emissive={}",
        metallic_factor, roughness_factor, normal_texture.is_some(),
        metallic_roughness_texture.is_some(), occlusion_texture.is_some(),
        emissive_texture.is_some());

    MaterialData {
        base_color_texture,
        base_color_factor,
        metallic_factor,
        roughness_factor,
        normal_texture,
        normal_scale,
        metallic_roughness_texture,
        occlusion_texture,
        emissive_texture,
        emissive_factor,
    }
}

/// 从 glTF texture source 提取纹理数据
fn extract_texture_by_source(
    texture: &gltf::Texture<'_>,
    images: &[gltf::image::Data],
    label: &str,
) -> Option<TextureData> {
    let image_index = texture.source().index();
    if image_index < images.len() {
        let image = &images[image_index];
        convert_to_rgba8(image).map(|data| {
            info!("加载{}纹理: {}x{}", label, image.width, image.height);
            TextureData {
                width: image.width,
                height: image.height,
                data,
            }
        })
    } else {
        None
    }
}

/// 从 glTF 文件提取骨骼和动画数据
///
/// 返回所有 skin 的 (Skeleton, Vec<AnimationClip>) 对。
/// 如果文件没有动画数据，返回空 Vec。
pub fn load_gltf_animations(path: impl AsRef<std::path::Path>) -> anvilkit_core::error::Result<Vec<(crate::animation::Skeleton, Vec<crate::animation::AnimationClip>)>> {
    let path = path.as_ref();
    let (document, buffers, _) = gltf::import(path).map_err(|e| {
        anvilkit_core::error::AnvilKitError::asset(format!("glTF 加载失败 {:?}: {}", path, e))
    })?;

    let mut results = Vec::new();

    // Extract skeletons from skins
    for skin in document.skins() {
        // Read inverse bind matrices via skin reader
        let skin_reader = skin.reader(|buffer| Some(&buffers[buffer.index()]));
        let ibms: Vec<glam::Mat4> = skin_reader
            .read_inverse_bind_matrices()
            .map(|iter| iter.map(|m| glam::Mat4::from_cols_array_2d(&m)).collect())
            .unwrap_or_default();

        let joint_nodes: Vec<_> = skin.joints().collect();
        let joints: Vec<crate::animation::Joint> = joint_nodes.iter().enumerate().map(|(i, joint)| {
            // Find parent index: check which other joint has this joint as a child
            let parent = joint_nodes.iter().position(|candidate| {
                candidate.children().any(|child| child.index() == joint.index())
            });
            crate::animation::Joint {
                name: joint.name().unwrap_or("").to_string(),
                parent,
                inverse_bind_matrix: ibms.get(i).copied().unwrap_or(glam::Mat4::IDENTITY),
            }
        }).collect();

        let skeleton = crate::animation::Skeleton { joints };

        // Extract animations targeting this skin's joints
        let mut clips = Vec::new();
        for anim in document.animations() {
            let mut channels = Vec::new();
            for channel in anim.channels() {
                let target = channel.target();
                let joint_index = skin.joints().position(|j| j.index() == target.node().index());
                let Some(joint_idx) = joint_index else { continue };

                let property = match target.property() {
                    gltf::animation::Property::Translation => crate::animation::AnimationProperty::Translation,
                    gltf::animation::Property::Rotation => crate::animation::AnimationProperty::Rotation,
                    gltf::animation::Property::Scale => crate::animation::AnimationProperty::Scale,
                    _ => continue,
                };

                let reader = channel.reader(|buffer| Some(&buffers[buffer.index()]));
                let timestamps: Vec<f32> = reader.read_inputs().map(|i| i.collect()).unwrap_or_default();
                let interpolation = match channel.sampler().interpolation() {
                    gltf::animation::Interpolation::Linear => crate::animation::Interpolation::Linear,
                    gltf::animation::Interpolation::Step => crate::animation::Interpolation::Step,
                    gltf::animation::Interpolation::CubicSpline => crate::animation::Interpolation::CubicSpline,
                };

                let values: Vec<[f32; 4]> = match reader.read_outputs() {
                    Some(gltf::animation::util::ReadOutputs::Translations(t)) =>
                        t.map(|v| [v[0], v[1], v[2], 0.0]).collect(),
                    Some(gltf::animation::util::ReadOutputs::Rotations(r)) =>
                        r.into_f32().map(|v| v).collect(),
                    Some(gltf::animation::util::ReadOutputs::Scales(s)) =>
                        s.map(|v| [v[0], v[1], v[2], 0.0]).collect(),
                    _ => continue,
                };

                let keyframes: Vec<crate::animation::Keyframe> = timestamps.into_iter()
                    .zip(values.into_iter())
                    .map(|(time, value)| crate::animation::Keyframe { time, value })
                    .collect();

                if !keyframes.is_empty() {
                    channels.push(crate::animation::AnimationChannel {
                        joint_index: joint_idx,
                        property,
                        interpolation,
                        keyframes,
                    });
                }
            }
            if !channels.is_empty() {
                clips.push(crate::animation::AnimationClip {
                    name: anim.name().unwrap_or("unnamed").to_string(),
                    channels,
                });
            }
        }

        results.push((skeleton, clips));
    }

    Ok(results)
}

/// 将 glTF 图像数据转换为 RGBA8 格式
fn convert_to_rgba8(image: &gltf::image::Data) -> Option<Vec<u8>> {
    match image.format {
        gltf::image::Format::R8G8B8A8 => Some(image.pixels.clone()),
        gltf::image::Format::R8G8B8 => {
            // RGB -> RGBA
            let mut rgba = Vec::with_capacity(image.pixels.len() / 3 * 4);
            for chunk in image.pixels.chunks_exact(3) {
                rgba.push(chunk[0]);
                rgba.push(chunk[1]);
                rgba.push(chunk[2]);
                rgba.push(255);
            }
            Some(rgba)
        }
        gltf::image::Format::R8 => {
            // Grayscale → RGBA
            let mut rgba = Vec::with_capacity(image.pixels.len() * 4);
            for &v in &image.pixels {
                rgba.push(v);
                rgba.push(v);
                rgba.push(v);
                rgba.push(255);
            }
            Some(rgba)
        }
        gltf::image::Format::R8G8 => {
            // RG → RGBA (common for metallic-roughness)
            let mut rgba = Vec::with_capacity(image.pixels.len() * 2);
            for chunk in image.pixels.chunks_exact(2) {
                rgba.push(chunk[0]);
                rgba.push(chunk[1]);
                rgba.push(0);
                rgba.push(255);
            }
            Some(rgba)
        }
        _ => {
            log::warn!("不支持的纹理格式: {:?}", image.format);
            None
        }
    }
}
