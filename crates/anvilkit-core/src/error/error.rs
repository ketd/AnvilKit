//! # 核心错误类型
//! 
//! 定义 AnvilKit 的主要错误类型和错误分类系统。

use thiserror::Error;
use std::fmt;

/// AnvilKit 的主要错误类型
/// 
/// 这个枚举涵盖了 AnvilKit 生态系统中可能出现的所有错误类型。
/// 每个变体都包含详细的错误信息和可选的上下文数据。
/// 
/// ## 设计特点
/// 
/// - **结构化**: 按功能模块分类错误
/// - **信息丰富**: 包含详细的错误描述和上下文
/// - **可序列化**: 支持错误的序列化和传输
/// - **链式错误**: 支持错误链和根因分析
#[derive(Error, Debug)]
pub enum AnvilKitError {
    /// 渲染系统错误
    /// 
    /// 包括 GPU 驱动错误、着色器编译错误、纹理加载错误等。
    #[error("渲染错误: {message}")]
    Render {
        /// 错误消息
        message: String,
        /// 可选的底层错误
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 物理系统错误
    /// 
    /// 包括物理世界初始化错误、碰撞检测错误、约束求解错误等。
    #[error("物理错误: {message}")]
    Physics {
        /// 错误消息
        message: String,
        /// 可选的底层错误
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 资源管理错误
    /// 
    /// 包括资源加载失败、格式不支持、资源不存在等。
    #[error("资源错误: {message}")]
    Asset {
        /// 错误消息
        message: String,
        /// 资源路径（如果适用）
        path: Option<String>,
        /// 可选的底层错误
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 音频系统错误
    /// 
    /// 包括音频设备初始化错误、音频格式错误、播放错误等。
    #[error("音频错误: {message}")]
    Audio {
        /// 错误消息
        message: String,
        /// 可选的底层错误
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 输入系统错误
    /// 
    /// 包括输入设备初始化错误、输入映射错误等。
    #[error("输入错误: {message}")]
    Input {
        /// 错误消息
        message: String,
        /// 可选的底层错误
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// ECS 系统错误
    /// 
    /// 包括组件注册错误、系统调度错误、世界状态错误等。
    #[error("ECS 错误: {message}")]
    Ecs {
        /// 错误消息
        message: String,
        /// 可选的底层错误
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 窗口和平台错误
    /// 
    /// 包括窗口创建错误、平台特定错误、事件处理错误等。
    #[error("窗口错误: {message}")]
    Window {
        /// 错误消息
        message: String,
        /// 可选的底层错误
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 配置和初始化错误
    /// 
    /// 包括配置文件解析错误、参数验证错误、初始化失败等。
    #[error("配置错误: {message}")]
    Config {
        /// 错误消息
        message: String,
        /// 配置键或路径（如果适用）
        key: Option<String>,
        /// 可选的底层错误
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 网络和通信错误
    /// 
    /// 包括网络连接错误、协议错误、序列化错误等。
    #[error("网络错误: {message}")]
    Network {
        /// 错误消息
        message: String,
        /// 可选的底层错误
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// I/O 操作错误
    /// 
    /// 文件系统操作、网络 I/O 等底层 I/O 错误的包装。
    #[error("I/O 错误: {0}")]
    Io(#[from] std::io::Error),

    /// 序列化和反序列化错误
    #[error("序列化错误: {message}")]
    Serialization {
        /// 错误消息
        message: String,
        /// 可选的底层错误
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 通用错误
    /// 
    /// 用于不属于其他特定类别的错误。
    #[error("AnvilKit 错误: {message}")]
    Generic {
        /// 错误消息
        message: String,
        /// 可选的底层错误
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

/// 错误类别枚举
/// 
/// 用于对错误进行分类，便于错误处理和统计。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    /// 渲染相关错误
    Render,
    /// 物理相关错误
    Physics,
    /// 资源相关错误
    Asset,
    /// 音频相关错误
    Audio,
    /// 输入相关错误
    Input,
    /// ECS 相关错误
    Ecs,
    /// 窗口相关错误
    Window,
    /// 配置相关错误
    Config,
    /// 网络相关错误
    Network,
    /// I/O 相关错误
    Io,
    /// 序列化相关错误
    Serialization,
    /// 通用错误
    Generic,
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            ErrorCategory::Render => "渲染",
            ErrorCategory::Physics => "物理",
            ErrorCategory::Asset => "资源",
            ErrorCategory::Audio => "音频",
            ErrorCategory::Input => "输入",
            ErrorCategory::Ecs => "ECS",
            ErrorCategory::Window => "窗口",
            ErrorCategory::Config => "配置",
            ErrorCategory::Network => "网络",
            ErrorCategory::Io => "I/O",
            ErrorCategory::Serialization => "序列化",
            ErrorCategory::Generic => "通用",
        };
        write!(f, "{}", name)
    }
}

impl AnvilKitError {
    /// 创建渲染错误
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::error::AnvilKitError;
    /// 
    /// let error = AnvilKitError::render("着色器编译失败");
    /// ```
    pub fn render(message: impl Into<String>) -> Self {
        Self::Render {
            message: message.into(),
            source: None,
        }
    }

    /// 创建带源错误的渲染错误
    pub fn render_with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::Render {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// 创建物理错误
    pub fn physics(message: impl Into<String>) -> Self {
        Self::Physics {
            message: message.into(),
            source: None,
        }
    }

    /// 创建带源错误的物理错误
    pub fn physics_with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::Physics {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// 创建资源错误
    pub fn asset(message: impl Into<String>) -> Self {
        Self::Asset {
            message: message.into(),
            path: None,
            source: None,
        }
    }

    /// 创建带路径的资源错误
    pub fn asset_with_path(message: impl Into<String>, path: impl Into<String>) -> Self {
        Self::Asset {
            message: message.into(),
            path: Some(path.into()),
            source: None,
        }
    }

    /// 创建音频错误
    pub fn audio(message: impl Into<String>) -> Self {
        Self::Audio {
            message: message.into(),
            source: None,
        }
    }

    /// 创建输入错误
    pub fn input(message: impl Into<String>) -> Self {
        Self::Input {
            message: message.into(),
            source: None,
        }
    }

    /// 创建 ECS 错误
    pub fn ecs(message: impl Into<String>) -> Self {
        Self::Ecs {
            message: message.into(),
            source: None,
        }
    }

    /// 创建窗口错误
    pub fn window(message: impl Into<String>) -> Self {
        Self::Window {
            message: message.into(),
            source: None,
        }
    }

    /// 创建配置错误
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
            key: None,
            source: None,
        }
    }

    /// 创建带键的配置错误
    pub fn config_with_key(message: impl Into<String>, key: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
            key: Some(key.into()),
            source: None,
        }
    }

    /// 创建网络错误
    pub fn network(message: impl Into<String>) -> Self {
        Self::Network {
            message: message.into(),
            source: None,
        }
    }

    /// 创建序列化错误
    pub fn serialization(message: impl Into<String>) -> Self {
        Self::Serialization {
            message: message.into(),
            source: None,
        }
    }

    /// 创建通用错误
    pub fn generic(message: impl Into<String>) -> Self {
        Self::Generic {
            message: message.into(),
            source: None,
        }
    }

    /// 获取错误类别
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::error::{AnvilKitError, ErrorCategory};
    /// 
    /// let error = AnvilKitError::render("测试错误");
    /// assert_eq!(error.category(), ErrorCategory::Render);
    /// ```
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::Render { .. } => ErrorCategory::Render,
            Self::Physics { .. } => ErrorCategory::Physics,
            Self::Asset { .. } => ErrorCategory::Asset,
            Self::Audio { .. } => ErrorCategory::Audio,
            Self::Input { .. } => ErrorCategory::Input,
            Self::Ecs { .. } => ErrorCategory::Ecs,
            Self::Window { .. } => ErrorCategory::Window,
            Self::Config { .. } => ErrorCategory::Config,
            Self::Network { .. } => ErrorCategory::Network,
            Self::Io(_) => ErrorCategory::Io,
            Self::Serialization { .. } => ErrorCategory::Serialization,
            Self::Generic { .. } => ErrorCategory::Generic,
        }
    }

    /// 获取错误消息
    ///
    /// 返回不包含错误类型前缀的纯错误消息。
    pub fn message(&self) -> String {
        match self {
            Self::Render { message, .. } => message.clone(),
            Self::Physics { message, .. } => message.clone(),
            Self::Asset { message, .. } => message.clone(),
            Self::Audio { message, .. } => message.clone(),
            Self::Input { message, .. } => message.clone(),
            Self::Ecs { message, .. } => message.clone(),
            Self::Window { message, .. } => message.clone(),
            Self::Config { message, .. } => message.clone(),
            Self::Network { message, .. } => message.clone(),
            Self::Io(err) => err.to_string(),
            Self::Serialization { message, .. } => message.clone(),
            Self::Generic { message, .. } => message.clone(),
        }
    }

    /// 检查是否为特定类别的错误
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use anvilkit_core::error::{AnvilKitError, ErrorCategory};
    /// 
    /// let error = AnvilKitError::render("测试错误");
    /// assert!(error.is_category(ErrorCategory::Render));
    /// assert!(!error.is_category(ErrorCategory::Physics));
    /// ```
    pub fn is_category(&self, category: ErrorCategory) -> bool {
        self.category() == category
    }

    /// 添加上下文信息
    /// 
    /// 返回一个包含额外上下文信息的新错误。
    pub fn with_context(self, context: impl Into<String>) -> Self {
        let context = context.into();
        match self {
            Self::Generic { message, source } => Self::Generic {
                message: format!("{}: {}", context, message),
                source,
            },
            _ => Self::Generic {
                message: format!("{}: {}", context, self),
                source: Some(Box::new(self)),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = AnvilKitError::render("测试渲染错误");
        assert_eq!(error.category(), ErrorCategory::Render);
        assert_eq!(error.message(), "测试渲染错误");
        assert!(error.is_category(ErrorCategory::Render));
    }

    #[test]
    fn test_error_with_path() {
        let error = AnvilKitError::asset_with_path("加载失败", "texture.png");
        if let AnvilKitError::Asset { path, .. } = &error {
            assert_eq!(path.as_ref().unwrap(), "texture.png");
        } else {
            panic!("Expected Asset error");
        }
    }

    #[test]
    fn test_error_with_context() {
        let original = AnvilKitError::render("着色器编译失败");
        let with_context = original.with_context("初始化渲染器时");
        
        assert!(with_context.to_string().contains("初始化渲染器时"));
        assert!(with_context.to_string().contains("着色器编译失败"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "文件未找到");
        let anvilkit_error: AnvilKitError = io_error.into();
        
        assert_eq!(anvilkit_error.category(), ErrorCategory::Io);
    }

    #[test]
    fn test_error_category_display() {
        assert_eq!(ErrorCategory::Render.to_string(), "渲染");
        assert_eq!(ErrorCategory::Physics.to_string(), "物理");
        assert_eq!(ErrorCategory::Asset.to_string(), "资源");
    }

    #[test]
    fn test_error_with_source() {
        let source_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "权限不足");
        let error = AnvilKitError::render_with_source("渲染初始化失败", source_error);
        
        assert!(std::error::Error::source(&error).is_some());
        assert_eq!(error.category(), ErrorCategory::Render);
    }
}
