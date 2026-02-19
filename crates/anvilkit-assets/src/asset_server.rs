//! # 资产服务器
//!
//! 提供资产的异步加载、Handle 生命周期管理和加载状态追踪。
//!
//! ## 设计
//!
//! - `AssetServer`: ECS Resource，管理所有资产的加载和存储
//! - `AssetHandle<T>`: 带引用计数的资产句柄
//! - `LoadState`: 加载状态追踪 (NotLoaded → Loading → Loaded → Failed)

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// 资产 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssetId(u64);

static NEXT_ASSET_ID: AtomicU64 = AtomicU64::new(1);

impl AssetId {
    fn next() -> Self {
        Self(NEXT_ASSET_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// 资产加载状态
///
/// # 示例
///
/// ```rust
/// use anvilkit_assets::asset_server::LoadState;
///
/// let state = LoadState::NotLoaded;
/// assert!(!state.is_loaded());
///
/// let loaded = LoadState::Loaded;
/// assert!(loaded.is_loaded());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadState {
    /// 尚未开始加载
    NotLoaded,
    /// 正在加载中
    Loading,
    /// 加载完成
    Loaded,
    /// 加载失败
    Failed,
}

impl LoadState {
    pub fn is_loaded(&self) -> bool {
        matches!(self, LoadState::Loaded)
    }

    pub fn is_failed(&self) -> bool {
        matches!(self, LoadState::Failed)
    }
}

/// 带引用计数的资产句柄
///
/// 当所有句柄被丢弃时，资产可以被回收。
///
/// # 示例
///
/// ```rust
/// use anvilkit_assets::asset_server::AssetHandle;
///
/// let handle: AssetHandle<String> = AssetHandle::new(
///     anvilkit_assets::asset_server::AssetId::from_raw(1),
///     "test.txt".into(),
/// );
/// let handle2 = handle.clone();
/// assert_eq!(handle.id(), handle2.id());
/// ```
#[derive(Debug)]
pub struct AssetHandle<T> {
    inner: Arc<AssetHandleInner<T>>,
}

impl AssetId {
    /// 从原始 ID 创建（仅用于测试）
    pub fn from_raw(id: u64) -> Self {
        Self(id)
    }
}

#[derive(Debug)]
struct AssetHandleInner<T> {
    id: AssetId,
    path: PathBuf,
    _marker: std::marker::PhantomData<T>,
}

impl<T> AssetHandle<T> {
    /// 创建新句柄
    pub fn new(id: AssetId, path: PathBuf) -> Self {
        Self {
            inner: Arc::new(AssetHandleInner {
                id,
                path,
                _marker: std::marker::PhantomData,
            }),
        }
    }

    /// 获取资产 ID
    pub fn id(&self) -> AssetId {
        self.inner.id
    }

    /// 获取资产路径
    pub fn path(&self) -> &Path {
        &self.inner.path
    }

    /// 当前引用计数
    pub fn ref_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }
}

impl<T> Clone for AssetHandle<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> PartialEq for AssetHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.id == other.inner.id
    }
}

impl<T> Eq for AssetHandle<T> {}

impl<T> std::hash::Hash for AssetHandle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.id.hash(state);
    }
}

/// 资产存储
///
/// 按类型存储已加载的资产数据。
pub struct AssetStorage<T> {
    assets: HashMap<AssetId, T>,
    states: HashMap<AssetId, LoadState>,
}

impl<T> Default for AssetStorage<T> {
    fn default() -> Self {
        Self {
            assets: HashMap::new(),
            states: HashMap::new(),
        }
    }
}

impl<T> AssetStorage<T> {
    /// 创建新的资产存储
    pub fn new() -> Self {
        Self::default()
    }

    /// 插入已加载的资产
    pub fn insert(&mut self, id: AssetId, asset: T) {
        self.assets.insert(id, asset);
        self.states.insert(id, LoadState::Loaded);
    }

    /// 获取资产引用
    pub fn get(&self, id: AssetId) -> Option<&T> {
        self.assets.get(&id)
    }

    /// 获取加载状态
    pub fn load_state(&self, id: AssetId) -> LoadState {
        self.states.get(&id).copied().unwrap_or(LoadState::NotLoaded)
    }

    /// 设置加载状态
    pub fn set_state(&mut self, id: AssetId, state: LoadState) {
        self.states.insert(id, state);
    }

    /// 移除资产
    pub fn remove(&mut self, id: AssetId) -> Option<T> {
        self.states.remove(&id);
        self.assets.remove(&id)
    }

    /// 已加载资产数量
    pub fn len(&self) -> usize {
        self.assets.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.assets.is_empty()
    }
}

/// 资产服务器
///
/// 管理资产的加载请求和状态追踪。
///
/// # 示例
///
/// ```rust
/// use anvilkit_assets::asset_server::{AssetServer, LoadState};
///
/// let mut server = AssetServer::new("assets");
/// let handle = server.load::<String>("test.txt");
/// assert_eq!(server.load_state(&handle), LoadState::Loading);
/// ```
pub struct AssetServer {
    /// 资产根目录
    asset_root: PathBuf,
    /// 路径 → AssetId 映射（去重）
    path_to_id: HashMap<PathBuf, AssetId>,
    /// AssetId → 加载状态
    states: HashMap<AssetId, LoadState>,
}

impl AssetServer {
    /// 创建新的资产服务器
    pub fn new(asset_root: impl Into<PathBuf>) -> Self {
        Self {
            asset_root: asset_root.into(),
            path_to_id: HashMap::new(),
            states: HashMap::new(),
        }
    }

    /// 请求加载资产
    ///
    /// 如果同一路径已请求过，返回相同 ID 的新句柄。
    pub fn load<T>(&mut self, path: impl AsRef<Path>) -> AssetHandle<T> {
        let full_path = self.asset_root.join(path.as_ref());
        let id = *self.path_to_id.entry(full_path.clone()).or_insert_with(|| {
            let id = AssetId::next();
            id
        });

        if !self.states.contains_key(&id) {
            self.states.insert(id, LoadState::Loading);
        }

        AssetHandle::new(id, full_path)
    }

    /// 获取资产加载状态
    pub fn load_state<T>(&self, handle: &AssetHandle<T>) -> LoadState {
        self.states.get(&handle.id()).copied().unwrap_or(LoadState::NotLoaded)
    }

    /// 标记资产为已加载
    pub fn mark_loaded(&mut self, id: AssetId) {
        self.states.insert(id, LoadState::Loaded);
    }

    /// 标记资产为失败
    pub fn mark_failed(&mut self, id: AssetId) {
        self.states.insert(id, LoadState::Failed);
    }

    /// 获取资产根目录
    pub fn asset_root(&self) -> &Path {
        &self.asset_root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_state() {
        assert!(LoadState::Loaded.is_loaded());
        assert!(!LoadState::Loading.is_loaded());
        assert!(LoadState::Failed.is_failed());
        assert!(!LoadState::Loaded.is_failed());
    }

    #[test]
    fn test_asset_handle_refcount() {
        let handle: AssetHandle<String> = AssetHandle::new(
            AssetId::from_raw(1),
            "test.txt".into(),
        );
        assert_eq!(handle.ref_count(), 1);

        let handle2 = handle.clone();
        assert_eq!(handle.ref_count(), 2);
        assert_eq!(handle2.ref_count(), 2);

        drop(handle2);
        assert_eq!(handle.ref_count(), 1);
    }

    #[test]
    fn test_asset_storage() {
        let mut storage = AssetStorage::new();
        let id = AssetId::from_raw(1);

        assert!(storage.is_empty());
        assert_eq!(storage.load_state(id), LoadState::NotLoaded);

        storage.insert(id, "hello".to_string());
        assert_eq!(storage.len(), 1);
        assert_eq!(storage.get(id), Some(&"hello".to_string()));
        assert_eq!(storage.load_state(id), LoadState::Loaded);
    }

    #[test]
    fn test_asset_server_dedup() {
        let mut server = AssetServer::new("assets");
        let h1: AssetHandle<String> = server.load("test.txt");
        let h2: AssetHandle<String> = server.load("test.txt");

        // Same path → same ID
        assert_eq!(h1.id(), h2.id());
    }

    #[test]
    fn test_asset_server_load_state() {
        let mut server = AssetServer::new("assets");
        let handle: AssetHandle<String> = server.load("test.txt");
        assert_eq!(server.load_state(&handle), LoadState::Loading);

        server.mark_loaded(handle.id());
        assert_eq!(server.load_state(&handle), LoadState::Loaded);
    }
}
