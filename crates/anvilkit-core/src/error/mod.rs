//! # 错误处理系统
//! 
//! AnvilKit 的统一错误处理系统，提供结构化的错误类型和处理机制。
//! 
//! ## 设计原则
//! 
//! 1. **统一性**: 所有 AnvilKit 组件使用统一的错误类型
//! 2. **可扩展性**: 支持添加新的错误类别和上下文信息
//! 3. **用户友好**: 提供清晰的错误消息和调试信息
//! 4. **性能优化**: 错误路径的开销最小化
//! 
//! ## 错误分类
//! 
//! - **系统错误**: 渲染、物理、音频等子系统错误
//! - **资源错误**: 资源加载、管理相关错误
//! - **配置错误**: 配置文件、参数验证错误
//! - **运行时错误**: 游戏逻辑运行时错误
//! 
//! ## 使用示例
//! 
//! ```rust
//! use anvilkit_core::error::{AnvilKitError, Result};
//! 
//! fn load_texture(path: &str) -> Result<()> {
//!     if path.is_empty() {
//!         return Err(AnvilKitError::asset("纹理路径不能为空"));
//!     }
//!     
//!     // 模拟加载失败
//!     Err(AnvilKitError::asset(format!("无法加载纹理: {}", path)))
//! }
//! 
//! fn main() {
//!     match load_texture("") {
//!         Ok(_) => println!("加载成功"),
//!         Err(e) => {
//!             eprintln!("错误: {}", e);
//!             eprintln!("错误类型: {:?}", e.category());
//!         }
//!     }
//! }
//! ```

pub mod error;

// 重新导出主要类型
pub use error::{AnvilKitError, ErrorCategory};

/// AnvilKit 的标准 Result 类型
pub type Result<T> = std::result::Result<T, AnvilKitError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = AnvilKitError::render("测试渲染错误");
        assert_eq!(error.category(), ErrorCategory::Render);
        assert!(error.to_string().contains("测试渲染错误"));
    }

    #[test]
    fn test_result_type() {
        fn test_function() -> Result<i32> {
            Err(AnvilKitError::generic("测试错误"))
        }
        
        assert!(test_function().is_err());
    }
}
