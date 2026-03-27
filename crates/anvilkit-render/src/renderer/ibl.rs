//! # IBL (Image-Based Lighting) 工具
//!
//! 提供 BRDF 积分 LUT 的 CPU 生成。
//! BRDF LUT 用于 PBR 渲染中的 split-sum 近似，
//! 存储 (F0_scale, F0_bias) 以实现能量守恒的环境光镜面反射。

use std::f32::consts::PI;
use std::fs;
use std::io::Write;
use std::path::Path;

/// 生成 BRDF 积分查找表 (LUT)
///
/// 对每个 (NdotV, roughness) 组合进行重要性采样，计算 GGX BRDF 的
/// Fresnel 缩放因子和偏置因子。结果存储为 Rgba8Unorm 格式
/// （R=scale, G=bias, B=0, A=255）。
///
/// # 参数
///
/// - `size`: LUT 的宽高（正方形，推荐 256）
///
/// # 返回
///
/// RGBA8 像素数据，`size * size * 4` 字节
///
/// # 示例
///
/// ```rust
/// use anvilkit_render::renderer::ibl::generate_brdf_lut;
///
/// let data = generate_brdf_lut(64);
/// assert_eq!(data.len(), 64 * 64 * 4);
/// ```
pub fn generate_brdf_lut(size: u32) -> Vec<u8> {
    let sample_count = 1024u32;
    let mut data = Vec::with_capacity((size * size * 4) as usize);

    for y in 0..size {
        for x in 0..size {
            let n_dot_v = ((x as f32) + 0.5) / size as f32;
            let roughness = ((y as f32) + 0.5) / size as f32;
            let n_dot_v = n_dot_v.max(0.001);

            let (scale, bias) = integrate_brdf(n_dot_v, roughness, sample_count);

            data.push((scale.clamp(0.0, 1.0) * 255.0) as u8);
            data.push((bias.clamp(0.0, 1.0) * 255.0) as u8);
            data.push(0);
            data.push(255);
        }
    }

    data
}

/// 对单个 (NdotV, roughness) 点积分 BRDF
fn integrate_brdf(n_dot_v: f32, roughness: f32, sample_count: u32) -> (f32, f32) {
    let v = glam::Vec3::new((1.0 - n_dot_v * n_dot_v).sqrt(), 0.0, n_dot_v);
    let n = glam::Vec3::Z;

    let mut a = 0.0f32;
    let mut b = 0.0f32;

    for i in 0..sample_count {
        let xi = hammersley(i, sample_count);
        let h = importance_sample_ggx(xi, n, roughness);
        let l = (2.0 * v.dot(h) * h - v).normalize();

        let n_dot_l = l.z.max(0.0);
        let n_dot_h = h.z.max(0.0);
        let v_dot_h = v.dot(h).max(0.0);

        if n_dot_l > 0.0 {
            let g = geometry_smith_ibl(n_dot_v, n_dot_l, roughness);
            let g_vis = (g * v_dot_h) / (n_dot_h * n_dot_v).max(0.0001);
            let fc = (1.0 - v_dot_h).powf(5.0);

            a += (1.0 - fc) * g_vis;
            b += fc * g_vis;
        }
    }

    let inv = 1.0 / sample_count as f32;
    (a * inv, b * inv)
}

/// Hammersley 低差异序列
fn hammersley(i: u32, n: u32) -> glam::Vec2 {
    glam::Vec2::new(i as f32 / n as f32, radical_inverse_vdc(i))
}

/// Van der Corput 基数逆序列
fn radical_inverse_vdc(mut bits: u32) -> f32 {
    bits = (bits << 16) | (bits >> 16);
    bits = ((bits & 0x55555555) << 1) | ((bits & 0xAAAAAAAA) >> 1);
    bits = ((bits & 0x33333333) << 2) | ((bits & 0xCCCCCCCC) >> 2);
    bits = ((bits & 0x0F0F0F0F) << 4) | ((bits & 0xF0F0F0F0) >> 4);
    bits = ((bits & 0x00FF00FF) << 8) | ((bits & 0xFF00FF00) >> 8);
    bits as f32 * 2.3283064365386963e-10 // 0x100000000
}

/// GGX 重要性采样
fn importance_sample_ggx(xi: glam::Vec2, n: glam::Vec3, roughness: f32) -> glam::Vec3 {
    let a = roughness * roughness;

    let phi = 2.0 * PI * xi.x;
    let cos_theta = ((1.0 - xi.y) / (1.0 + (a * a - 1.0) * xi.y)).sqrt();
    let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

    // 球面坐标 → 切线空间笛卡尔坐标
    let h = glam::Vec3::new(phi.cos() * sin_theta, phi.sin() * sin_theta, cos_theta);

    // 切线空间 → 世界空间
    let up = if n.z.abs() < 0.999 {
        glam::Vec3::Z
    } else {
        glam::Vec3::X
    };
    let tangent = up.cross(n).normalize();
    let bitangent = n.cross(tangent);

    (tangent * h.x + bitangent * h.y + n * h.z).normalize()
}

/// Save the BRDF LUT to a binary file.
///
/// Generates the LUT of the given `size` and writes the raw RGBA8 bytes to `path`.
/// Parent directories are created automatically.
///
/// # Errors
///
/// Returns an `io::Error` if the file cannot be written.
pub fn save_brdf_lut(path: &str, size: u32) -> std::io::Result<()> {
    let data = generate_brdf_lut(size);
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::File::create(path)?;
    file.write_all(&data)?;
    Ok(())
}

/// Load a previously saved BRDF LUT from a binary file.
///
/// Returns `None` if the file does not exist or cannot be read.
pub fn load_brdf_lut(path: &str) -> Option<Vec<u8>> {
    fs::read(path).ok()
}

/// Load the BRDF LUT from cache, or generate + save it if the cache is missing.
///
/// The expected file size is `size * size * 4` bytes (RGBA8). If the cached file
/// exists but has an unexpected size, it is regenerated.
pub fn get_or_generate_brdf_lut(cache_path: &str, size: u32) -> Vec<u8> {
    let expected_len = (size * size * 4) as usize;

    if let Some(data) = load_brdf_lut(cache_path) {
        if data.len() == expected_len {
            log::info!("Loaded BRDF LUT from cache: {}", cache_path);
            return data;
        }
        log::warn!(
            "BRDF LUT cache size mismatch (expected {}, got {}), regenerating",
            expected_len,
            data.len()
        );
    }

    log::info!("Generating BRDF LUT ({}x{}) ...", size, size);
    let data = generate_brdf_lut(size);

    // Save to cache for next startup
    if let Some(parent) = Path::new(cache_path).parent() {
        let _ = fs::create_dir_all(parent);
    }
    match fs::File::create(cache_path) {
        Ok(mut f) => {
            if let Err(e) = f.write_all(&data) {
                log::warn!("Failed to write BRDF LUT cache: {}", e);
            } else {
                log::info!("Saved BRDF LUT cache to {}", cache_path);
            }
        }
        Err(e) => log::warn!("Failed to create BRDF LUT cache file: {}", e),
    }

    data
}

/// Smith GGX 几何函数 (IBL 版本，k = roughness² / 2)
fn geometry_smith_ibl(n_dot_v: f32, n_dot_l: f32, roughness: f32) -> f32 {
    let a = roughness;
    let k = (a * a) / 2.0;

    let ggx_v = n_dot_v / (n_dot_v * (1.0 - k) + k);
    let ggx_l = n_dot_l / (n_dot_l * (1.0 - k) + k);

    ggx_v * ggx_l
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_brdf_lut_size() {
        let data = generate_brdf_lut(32);
        assert_eq!(data.len(), 32 * 32 * 4);
    }

    #[test]
    fn test_brdf_lut_values_in_range() {
        let data = generate_brdf_lut(16);
        for chunk in data.chunks(4) {
            // B is always 0, A is always 255
            assert_eq!(chunk[2], 0);
            assert_eq!(chunk[3], 255);
        }
    }

    #[test]
    fn test_hammersley_sequence() {
        let h0 = hammersley(0, 16);
        assert_eq!(h0.x, 0.0);

        let h8 = hammersley(8, 16);
        assert!((h8.x - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_radical_inverse() {
        assert!((radical_inverse_vdc(0) - 0.0).abs() < 0.001);
        assert!((radical_inverse_vdc(1) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_brdf_lut_smooth_surface() {
        // At roughness near 0 and NdotV near 1, scale should be high, bias low
        let (scale, bias) = integrate_brdf(0.9, 0.05, 512);
        assert!(scale > 0.5, "scale={}", scale);
        assert!(bias < 0.2, "bias={}", bias);
    }
}
