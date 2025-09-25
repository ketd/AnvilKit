//! # 变换系统
//! 
//! 提供 3D 空间中的位置、旋转和缩放变换，以及层次变换的支持。
//! 
//! ## 核心概念
//! 
//! - [`Transform`]: 本地变换，相对于父对象的变换
//! - [`GlobalTransform`]: 全局变换，世界空间中的最终变换
//! 
//! ## 使用示例
//! 
//! ```rust
//! use anvilkit_core::math::Transform;
//! use glam::{Vec3, Quat};
//! 
//! // 创建基础变换
//! let mut transform = Transform::from_xyz(1.0, 2.0, 3.0);
//! 
//! // 链式调用设置属性
//! transform = transform
//!     .with_rotation(Quat::from_rotation_y(std::f32::consts::PI / 4.0))
//!     .with_scale(Vec3::splat(2.0));
//! 
//! // 应用变换到点
//! let point = Vec3::ZERO;
//! let transformed_point = transform.transform_point(point);
//! ```

use glam::{Vec3, Quat, Mat4};
use crate::error::{AnvilKitError, Result};

/// 表示 3D 空间中位置、旋转和缩放的变换组件。
/// 
/// `Transform` 是 AnvilKit 中用于表示对象空间变换的基础类型。
/// 它支持 2D 和 3D 使用场景，对于 2D 对象，通常将 Z 分量设置为 0。
/// 
/// ## 内存布局
/// 
/// 该结构体使用紧凑的内存布局，总大小为 40 字节：
/// - `translation`: 12 字节 (3 × f32)
/// - `rotation`: 16 字节 (4 × f32)
/// - `scale`: 12 字节 (3 × f32)
/// 
/// ## 线程安全
/// 
/// `Transform` 实现了 `Send` 和 `Sync`，可以安全地在线程间传递。
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bevy_ecs", derive(bevy_ecs::component::Component))]
pub struct Transform {
    /// 世界空间中的位置
    pub translation: Vec3,
    /// 四元数表示的旋转
    pub rotation: Quat,
    /// 各轴的缩放因子
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Transform {
    /// 单位变换（无平移、旋转或缩放）
    pub const IDENTITY: Self = Self {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };

    /// 创建一个新的变换实例
    /// 
    /// # 参数
    /// 
    /// - `translation`: 位置向量
    /// - `rotation`: 旋转四元数
    /// - `scale`: 缩放向量
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::math::Transform;
    /// use glam::{Vec3, Quat};
    /// 
    /// let transform = Transform::new(
    ///     Vec3::new(1.0, 2.0, 3.0),
    ///     Quat::IDENTITY,
    ///     Vec3::ONE
    /// );
    /// ```
    pub fn new(translation: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }

    /// 从位置创建变换
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::math::Transform;
    /// use glam::Vec3;
    /// 
    /// let transform = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
    /// assert_eq!(transform.translation, Vec3::new(1.0, 2.0, 3.0));
    /// ```
    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            ..Self::IDENTITY
        }
    }

    /// 从旋转创建变换
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::math::Transform;
    /// use glam::Quat;
    /// 
    /// let rotation = Quat::from_rotation_y(std::f32::consts::PI / 4.0);
    /// let transform = Transform::from_rotation(rotation);
    /// assert_eq!(transform.rotation, rotation);
    /// ```
    pub fn from_rotation(rotation: Quat) -> Self {
        Self {
            rotation,
            ..Self::IDENTITY
        }
    }

    /// 从缩放创建变换
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::math::Transform;
    /// use glam::Vec3;
    /// 
    /// let transform = Transform::from_scale(Vec3::splat(2.0));
    /// assert_eq!(transform.scale, Vec3::splat(2.0));
    /// ```
    pub fn from_scale(scale: Vec3) -> Self {
        Self {
            scale,
            ..Self::IDENTITY
        }
    }

    /// 从 XYZ 坐标创建变换
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::math::Transform;
    /// 
    /// let transform = Transform::from_xyz(1.0, 2.0, 3.0);
    /// assert_eq!(transform.translation.x, 1.0);
    /// assert_eq!(transform.translation.y, 2.0);
    /// assert_eq!(transform.translation.z, 3.0);
    /// ```
    pub fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        Self::from_translation(Vec3::new(x, y, z))
    }

    /// 从 XY 坐标创建 2D 变换（Z = 0）
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::math::Transform;
    /// 
    /// let transform = Transform::from_xy(1.0, 2.0);
    /// assert_eq!(transform.translation.z, 0.0);
    /// ```
    pub fn from_xy(x: f32, y: f32) -> Self {
        Self::from_translation(Vec3::new(x, y, 0.0))
    }

    /// 设置位置（链式调用）
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::math::Transform;
    /// use glam::Vec3;
    /// 
    /// let transform = Transform::IDENTITY
    ///     .with_translation(Vec3::new(1.0, 2.0, 3.0));
    /// ```
    pub fn with_translation(mut self, translation: Vec3) -> Self {
        self.translation = translation;
        self
    }

    /// 设置旋转（链式调用）
    pub fn with_rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }

    /// 设置缩放（链式调用）
    pub fn with_scale(mut self, scale: Vec3) -> Self {
        self.scale = scale;
        self
    }

    /// 创建朝向目标的变换
    /// 
    /// # 参数
    /// 
    /// - `eye`: 观察者位置
    /// - `target`: 目标位置
    /// - `up`: 上方向向量
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::math::Transform;
    /// use glam::Vec3;
    /// 
    /// let transform = Transform::looking_at(
    ///     Vec3::new(0.0, 0.0, 5.0),  // 相机位置
    ///     Vec3::ZERO,                // 看向原点
    ///     Vec3::Y                    // 上方向
    /// );
    /// ```
    pub fn looking_at(eye: Vec3, target: Vec3, up: Vec3) -> Result<Self> {
        let forward = (target - eye).normalize();
        
        // 检查前向向量是否有效
        if !forward.is_finite() || forward.length_squared() < f32::EPSILON {
            return Err(AnvilKitError::generic("无效的朝向向量：目标和眼睛位置相同或无效"));
        }

        let right = forward.cross(up).normalize();

        // 检查右向向量是否有效（避免平行向量）
        if !right.is_finite() || right.length_squared() < f32::EPSILON {
            return Err(AnvilKitError::generic("无效的上方向向量：与前向向量平行"));
        }

        let up = right.cross(forward);

        // 检查上向向量是否有效
        if !up.is_finite() {
            return Err(AnvilKitError::generic("计算上方向向量时出现数值错误"));
        }

        // 创建旋转矩阵并转换为四元数
        let rotation_matrix = glam::Mat3::from_cols(right, up, -forward);
        let rotation = Quat::from_mat3(&rotation_matrix);

        // 检查四元数是否有效
        if !rotation.is_finite() {
            return Err(AnvilKitError::generic("计算旋转四元数时出现数值错误"));
        }
        
        Ok(Self::new(eye, rotation, Vec3::ONE))
    }

    /// 将变换转换为 4x4 变换矩阵
    /// 
    /// 矩阵的计算顺序为：缩放 → 旋转 → 平移
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::math::Transform;
    /// use glam::Vec3;
    /// 
    /// let transform = Transform::from_xyz(1.0, 2.0, 3.0);
    /// let matrix = transform.compute_matrix();
    /// 
    /// // 验证平移部分
    /// assert_eq!(matrix.w_axis.truncate(), Vec3::new(1.0, 2.0, 3.0));
    /// ```
    pub fn compute_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    /// 应用变换到点
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::math::Transform;
    /// use glam::Vec3;
    /// 
    /// let transform = Transform::from_xyz(1.0, 2.0, 3.0);
    /// let point = Vec3::ZERO;
    /// let transformed = transform.transform_point(point);
    /// 
    /// assert_eq!(transformed, Vec3::new(1.0, 2.0, 3.0));
    /// ```
    pub fn transform_point(&self, point: Vec3) -> Vec3 {
        self.compute_matrix().transform_point3(point)
    }

    /// 应用变换到向量（忽略平移）
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::math::Transform;
    /// use glam::Vec3;
    /// 
    /// let transform = Transform::from_scale(Vec3::splat(2.0));
    /// let vector = Vec3::new(1.0, 1.0, 1.0);
    /// let transformed = transform.transform_vector(vector);
    /// 
    /// assert_eq!(transformed, Vec3::new(2.0, 2.0, 2.0));
    /// ```
    pub fn transform_vector(&self, vector: Vec3) -> Vec3 {
        self.compute_matrix().transform_vector3(vector)
    }

    /// 组合两个变换（self * other）
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::math::Transform;
    /// use glam::Vec3;
    /// 
    /// let parent = Transform::from_xyz(1.0, 0.0, 0.0);
    /// let child = Transform::from_xyz(0.0, 1.0, 0.0);
    /// let combined = parent.mul_transform(&child);
    /// 
    /// assert_eq!(combined.translation, Vec3::new(1.0, 1.0, 0.0));
    /// ```
    pub fn mul_transform(&self, other: &Transform) -> Transform {
        let matrix = self.compute_matrix() * other.compute_matrix();
        Transform::from_matrix(matrix)
    }

    /// 从变换矩阵创建变换
    /// 
    /// # 注意
    /// 
    /// 如果矩阵包含非均匀缩放或剪切，可能会丢失信息。
    pub fn from_matrix(matrix: Mat4) -> Self {
        let (scale, rotation, translation) = matrix.to_scale_rotation_translation();
        Self {
            translation,
            rotation,
            scale,
        }
    }

    /// 获取变换的逆变换
    ///
    /// # 错误
    ///
    /// 如果变换不可逆（例如缩放为零），返回错误。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use anvilkit_core::math::Transform;
    /// use glam::Vec3;
    ///
    /// let transform = Transform::from_xyz(1.0, 2.0, 3.0);
    /// let inverse = transform.inverse().unwrap();
    /// let identity = transform.mul_transform(&inverse);
    ///
    /// // 结果应该接近单位变换
    /// assert!((identity.translation.length() < 1e-5));
    /// ```
    pub fn inverse(&self) -> Result<Self> {
        // 检查缩放是否为零
        if self.scale.x.abs() < f32::EPSILON ||
           self.scale.y.abs() < f32::EPSILON ||
           self.scale.z.abs() < f32::EPSILON {
            return Err(AnvilKitError::generic("无法计算逆变换：缩放包含零值"));
        }

        let inv_scale = Vec3::new(1.0 / self.scale.x, 1.0 / self.scale.y, 1.0 / self.scale.z);
        let inv_rotation = self.rotation.inverse();
        let inv_translation = -(inv_rotation * (self.translation * inv_scale));

        Ok(Self {
            translation: inv_translation,
            rotation: inv_rotation,
            scale: inv_scale,
        })
    }

    /// 检查变换是否有效（不包含 NaN 或无穷大）
    pub fn is_finite(&self) -> bool {
        self.translation.is_finite() && 
        self.rotation.is_finite() && 
        self.scale.is_finite()
    }
}

/// 全局变换组件，表示世界空间中的最终变换。
/// 
/// `GlobalTransform` 通常由层次变换系统计算，表示对象在世界空间中的最终位置、旋转和缩放。
/// 它使用 4x4 矩阵存储，以提供高效的变换操作。
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bevy_ecs", derive(bevy_ecs::component::Component))]
pub struct GlobalTransform(pub Mat4);

impl Default for GlobalTransform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl GlobalTransform {
    /// 单位全局变换
    pub const IDENTITY: Self = Self(Mat4::IDENTITY);

    /// 从变换矩阵创建全局变换
    pub fn from_matrix(matrix: Mat4) -> Self {
        Self(matrix)
    }

    /// 从本地变换创建全局变换
    pub fn from_transform(transform: &Transform) -> Self {
        Self(transform.compute_matrix())
    }

    /// 获取变换矩阵
    pub fn matrix(&self) -> Mat4 {
        self.0
    }

    /// 获取位置分量
    pub fn translation(&self) -> Vec3 {
        self.0.w_axis.truncate()
    }

    /// 获取旋转分量
    pub fn rotation(&self) -> Quat {
        let (_, rotation, _) = self.0.to_scale_rotation_translation();
        rotation
    }

    /// 获取缩放分量
    pub fn scale(&self) -> Vec3 {
        let (scale, _, _) = self.0.to_scale_rotation_translation();
        scale
    }

    /// 应用全局变换到点
    pub fn transform_point(&self, point: Vec3) -> Vec3 {
        self.0.transform_point3(point)
    }

    /// 应用全局变换到向量（忽略平移）
    pub fn transform_vector(&self, vector: Vec3) -> Vec3 {
        self.0.transform_vector3(vector)
    }

    /// 组合全局变换
    pub fn mul_transform(&self, other: &GlobalTransform) -> GlobalTransform {
        GlobalTransform(self.0 * other.0)
    }

    /// 获取全局变换的逆变换
    pub fn inverse(&self) -> Result<Self> {
        let inv_matrix = self.0.inverse();
        if !inv_matrix.is_finite() {
            return Err(AnvilKitError::generic("无法计算全局变换的逆变换"));
        }
        Ok(Self(inv_matrix))
    }

    /// 检查全局变换是否有效
    pub fn is_finite(&self) -> bool {
        self.0.is_finite()
    }
}

impl From<Transform> for GlobalTransform {
    fn from(transform: Transform) -> Self {
        Self::from_transform(&transform)
    }
}

impl From<Mat4> for GlobalTransform {
    fn from(matrix: Mat4) -> Self {
        Self::from_matrix(matrix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    // 自定义的近似相等比较函数
    fn vec3_approx_eq(a: Vec3, b: Vec3, epsilon: f32) -> bool {
        (a - b).length() < epsilon
    }

    fn quat_approx_eq(a: glam::Quat, b: glam::Quat, epsilon: f32) -> bool {
        (a.dot(b) - 1.0).abs() < epsilon || (a.dot(b) + 1.0).abs() < epsilon
    }

    #[test]
    fn test_transform_identity() {
        let transform = Transform::IDENTITY;
        assert_eq!(transform.translation, Vec3::ZERO);
        assert_eq!(transform.rotation, Quat::IDENTITY);
        assert_eq!(transform.scale, Vec3::ONE);
    }

    #[test]
    fn test_transform_creation() {
        let transform = Transform::from_xyz(1.0, 2.0, 3.0);
        assert_eq!(transform.translation, Vec3::new(1.0, 2.0, 3.0));
        
        let transform = Transform::from_xy(1.0, 2.0);
        assert_eq!(transform.translation, Vec3::new(1.0, 2.0, 0.0));
    }

    #[test]
    fn test_transform_chaining() {
        let transform = Transform::IDENTITY
            .with_translation(Vec3::new(1.0, 2.0, 3.0))
            .with_scale(Vec3::splat(2.0));
        
        assert_eq!(transform.translation, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(transform.scale, Vec3::splat(2.0));
    }

    #[test]
    fn test_transform_point() {
        let transform = Transform::from_xyz(1.0, 2.0, 3.0);
        let point = Vec3::ZERO;
        let transformed = transform.transform_point(point);
        
        assert!(vec3_approx_eq(transformed, Vec3::new(1.0, 2.0, 3.0), 1e-6));
    }

    #[test]
    fn test_transform_vector() {
        let transform = Transform::from_scale(Vec3::splat(2.0));
        let vector = Vec3::new(1.0, 1.0, 1.0);
        let transformed = transform.transform_vector(vector);
        
        assert!(vec3_approx_eq(transformed, Vec3::new(2.0, 2.0, 2.0), 1e-6));
    }

    #[test]
    fn test_transform_composition() {
        let parent = Transform::from_xyz(1.0, 0.0, 0.0);
        let child = Transform::from_xyz(0.0, 1.0, 0.0);
        let combined = parent.mul_transform(&child);
        
        assert!(vec3_approx_eq(combined.translation, Vec3::new(1.0, 1.0, 0.0), 1e-6));
    }

    #[test]
    fn test_transform_matrix_roundtrip() {
        let original = Transform::from_xyz(1.0, 2.0, 3.0)
            .with_rotation(Quat::from_rotation_y(0.5))
            .with_scale(Vec3::new(2.0, 1.5, 0.5));
        
        let matrix = original.compute_matrix();
        let reconstructed = Transform::from_matrix(matrix);
        
        assert!(vec3_approx_eq(original.translation, reconstructed.translation, 1e-5));
        assert!(quat_approx_eq(original.rotation, reconstructed.rotation, 1e-5));
        assert!(vec3_approx_eq(original.scale, reconstructed.scale, 1e-5));
    }

    #[test]
    fn test_transform_inverse() {
        let transform = Transform::from_xyz(1.0, 2.0, 3.0)
            .with_rotation(Quat::from_rotation_y(0.5))
            .with_scale(Vec3::splat(2.0));
        
        let inverse = transform.inverse().unwrap();
        let identity = transform.mul_transform(&inverse);
        
        assert!(vec3_approx_eq(identity.translation, Vec3::ZERO, 1e-5));
        assert!(quat_approx_eq(identity.rotation, Quat::IDENTITY, 1e-5));
        assert!(vec3_approx_eq(identity.scale, Vec3::ONE, 1e-5));
    }

    #[test]
    fn test_transform_inverse_zero_scale() {
        let transform = Transform::from_scale(Vec3::new(0.0, 1.0, 1.0));
        assert!(transform.inverse().is_err());
    }

    #[test]
    fn test_looking_at() {
        let transform = Transform::looking_at(
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::ZERO,
            Vec3::Y
        ).unwrap();
        
        // 验证变换后的前向向量指向目标
        let forward = transform.transform_vector(-Vec3::Z);
        let expected_direction = (Vec3::ZERO - Vec3::new(0.0, 0.0, 5.0)).normalize();
        
        assert!(vec3_approx_eq(forward, expected_direction, 1e-5));
    }

    #[test]
    fn test_looking_at_invalid() {
        // 相同的眼睛和目标位置
        assert!(Transform::looking_at(Vec3::ZERO, Vec3::ZERO, Vec3::Y).is_err());

        // 平行的前向和上向量
        // 从 (0,0,0) 看向 (0,0,1)，前向向量是 (0,0,1)
        // 如果上向量也是 (0,0,1)，那么它们平行，叉积为零
        let result = Transform::looking_at(Vec3::ZERO, Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(result.is_err(), "Expected error for parallel forward and up vectors, but got: {:?}", result);

        // 另一个平行向量的例子：前向和上向都是 Y 轴
        assert!(Transform::looking_at(Vec3::ZERO, Vec3::Y, Vec3::Y).is_err());
    }

    #[test]
    fn test_global_transform() {
        let transform = Transform::from_xyz(1.0, 2.0, 3.0);
        let global = GlobalTransform::from_transform(&transform);
        
        assert_eq!(global.translation(), Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_global_transform_composition() {
        let global1 = GlobalTransform::from_matrix(Mat4::from_translation(Vec3::X));
        let global2 = GlobalTransform::from_matrix(Mat4::from_translation(Vec3::Y));
        let combined = global1.mul_transform(&global2);
        
        assert!(vec3_approx_eq(combined.translation(), Vec3::new(1.0, 1.0, 0.0), 1e-6));
    }

    #[test]
    fn test_finite_checks() {
        let valid_transform = Transform::from_xyz(1.0, 2.0, 3.0);
        assert!(valid_transform.is_finite());
        
        let invalid_transform = Transform::from_xyz(f32::NAN, 2.0, 3.0);
        assert!(!invalid_transform.is_finite());
    }
}
