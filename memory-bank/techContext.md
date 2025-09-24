# σ₃: Technical Context
*v1.0 | Created: 2025-09-24 | Updated: 2025-09-24*
*Π: 🚧INITIALIZING | Ω: 💡INNOVATE*

## 🛠️ Technology Stack

### 🎮 Core Game Engine
- **ECS**: `bevy_ecs` - 高性能实体组件系统
- **渲染**: `wgpu` - 现代跨平台图形API
- **窗口**: `winit` - 跨平台窗口管理
- **数学**: `glam` - 游戏优化的数学库

### 🎨 Rendering & Graphics
- **3D渲染**: PBR管线，基于 `wgpu`
- **2D渲染**: 高效精灵批处理系统
- **着色器**: WGSL (WebGPU Shading Language)
- **纹理**: 支持多种格式 (PNG, JPG, HDR, KTX2)

### ⚡ Physics & Simulation
- **2D物理**: `rapier2d` - 高性能2D物理引擎
- **3D物理**: `rapier3d` - 高性能3D物理引擎
- **碰撞检测**: 基于 Rapier 的统一碰撞系统

### 📦 Assets & Resources
- **3D模型**: `gltf` - glTF 2.0 格式支持
- **音频**: `kira` - 游戏音频引擎
- **图像**: `image` - 多格式图像处理
- **序列化**: `serde` - 配置和存档系统

## 🌐 Environment Setup
### Development
- [开发环境配置]

### Testing
- [测试环境配置]

### Production
- [生产环境配置]

## 📦 Dependencies
### Core Dependencies
- [核心依赖待确定]

### Development Dependencies
- [开发依赖待确定]

## 🔧 Build Tools
- [构建工具待选择]

## 📋 Development Standards
- [编码标准]
- [代码审查流程]
- [测试策略]

## 🔒 Security Considerations
- [安全考虑事项]

## 📝 Notes
- 技术选型将基于项目需求进行
- 优先考虑团队熟悉度和项目适配性
