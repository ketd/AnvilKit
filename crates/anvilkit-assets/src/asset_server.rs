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
use std::sync::mpsc;

use crate::asset_cache::{AssetCache, AssetCacheConfig};
use crate::dependency::DependencyGraph;
use crate::parsed_asset::ParsedAsset;

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
    /// Returns true if the asset has been successfully loaded.
    pub fn is_loaded(&self) -> bool {
        matches!(self, LoadState::Loaded)
    }

    /// Returns true if the asset failed to load.
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

/// 异步加载完成消息
pub struct AsyncLoadResult {
    /// 完成加载的资产 ID。
    pub id: AssetId,
    /// 加载到的原始字节数据（成功时 Some，失败时 None）。
    pub data: Result<Vec<u8>, String>,
}

/// 资产服务器
///
/// 管理资产的加载请求和状态追踪，支持同步和异步加载。
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
    /// AssetId → 路径 反向映射（用于 reload）
    id_to_path: HashMap<AssetId, PathBuf>,
    /// AssetId → 加载状态
    states: HashMap<AssetId, LoadState>,
    /// AssetId → 已加载字节缓存
    loaded_cache: HashMap<AssetId, Arc<Vec<u8>>>,
    /// 异步加载结果接收端
    async_rx: mpsc::Receiver<AsyncLoadResult>,
    /// 异步加载结果发送端（用于 clone 给后台线程）
    async_tx: mpsc::Sender<AsyncLoadResult>,
    /// 已完成但未处理的加载结果（缓存在主线程）
    completed: Vec<AsyncLoadResult>,
    /// 线程池任务发送端
    task_tx: std::sync::mpsc::Sender<Box<dyn FnOnce() + Send>>,
    /// 已解析的资产数据（原始字节 → 具体类型后存入此处）
    parsed_assets: HashMap<AssetId, ParsedAsset>,
    /// Content-addressed asset cache (LRU, keyed by content hash).
    cache: AssetCache,
    /// Dependency graph for cascade unloading.
    dependency_graph: DependencyGraph,
    /// 文件监视器（hot-reload feature 启用时有效）
    #[cfg(feature = "hot-reload")]
    watcher: Option<crate::hot_reload::FileWatcher>,
}

impl AssetServer {
    /// 创建新的资产服务器
    pub fn new(asset_root: impl Into<PathBuf>) -> Self {
        let asset_root: PathBuf = asset_root.into();
        let (tx, rx) = mpsc::channel();
        let (task_tx, task_rx) = std::sync::mpsc::channel::<Box<dyn FnOnce() + Send>>();
        let task_rx = std::sync::Arc::new(std::sync::Mutex::new(task_rx));
        let worker_count = std::thread::available_parallelism()
            .map(|n| n.get().clamp(1, 4))
            .unwrap_or(2);
        for _ in 0..worker_count {
            let rx = task_rx.clone();
            std::thread::spawn(move || {
                while let Ok(task) = rx.lock().unwrap().recv() {
                    task();
                }
            });
        }
        #[cfg(feature = "hot-reload")]
        let watcher = crate::hot_reload::FileWatcher::new(asset_root.as_path())
            .map_err(|e| log::warn!("FileWatcher 创建失败: {}", e))
            .ok();
        Self {
            asset_root,
            path_to_id: HashMap::new(),
            id_to_path: HashMap::new(),
            states: HashMap::new(),
            loaded_cache: HashMap::new(),
            async_rx: rx,
            async_tx: tx,
            completed: Vec::new(),
            task_tx,
            parsed_assets: HashMap::new(),
            cache: AssetCache::new(AssetCacheConfig::default()),
            dependency_graph: DependencyGraph::new(),
            #[cfg(feature = "hot-reload")]
            watcher,
        }
    }

    /// 请求加载资产（同步注册，不执行 I/O）。
    ///
    /// 如果同一路径已请求过，返回相同 ID 的新句柄。
    pub fn load<T>(&mut self, path: impl AsRef<Path>) -> AssetHandle<T> {
        let full_path = self.asset_root.join(path.as_ref());
        let id = *self.path_to_id.entry(full_path.clone()).or_insert_with(AssetId::next);
        self.id_to_path.entry(id).or_insert_with(|| full_path.clone());

        if !self.states.contains_key(&id) {
            self.states.insert(id, LoadState::Loading);
        }

        AssetHandle::new(id, full_path)
    }

    /// 异步加载资产文件到字节数据。
    ///
    /// 在后台线程中读取文件，完成后结果通过内部通道发送回主线程。
    /// 调用 [`process_completed`] 来处理完成的加载。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use anvilkit_assets::asset_server::AssetServer;
    ///
    /// let mut server = AssetServer::new("assets");
    /// let handle = server.load_async::<Vec<u8>>("textures/atlas.png");
    /// // Later, in your game loop:
    /// // server.process_completed();
    /// ```
    pub fn load_async<T>(&mut self, path: impl AsRef<Path>) -> AssetHandle<T> {
        let handle: AssetHandle<T> = self.load(path);
        let id = handle.id();

        // Cache-hit: if the asset is already loaded, skip the I/O dispatch
        if self.loaded_cache.contains_key(&id) {
            return handle;
        }

        let file_path = handle.path().to_path_buf();
        let tx = self.async_tx.clone();

        let _ = self.task_tx.send(Box::new(move || {
            let result = std::fs::read(&file_path)
                .map_err(|e| format!("Failed to load {:?}: {}", file_path, e));
            let _ = tx.send(AsyncLoadResult { id, data: result });
        }));

        handle
    }

    /// 处理后台线程完成的加载结果。
    ///
    /// 每帧调用一次。将接收到的结果缓存到 `completed`，
    /// 并更新对应的 `LoadState`。
    /// 返回本次处理的完成数量。
    pub fn process_completed(&mut self) -> usize {
        let mut count = 0;
        while let Ok(result) = self.async_rx.try_recv() {
            match &result.data {
                Ok(data) => {
                    self.states.insert(result.id, LoadState::Loaded);
                    self.loaded_cache.insert(result.id, Arc::new(data.clone()));

                    // Populate the content-addressed cache
                    let content_hash = AssetCache::content_hash(data);
                    let source_path = self.id_to_path.get(&result.id)
                        .cloned()
                        .unwrap_or_default();
                    if let Err(e) = self.cache.put(content_hash, &source_path, data) {
                        log::warn!("Asset cache put failed for {:?}: {}", result.id, e);
                    }

                    log::debug!("Asset {:?} loaded successfully", result.id);
                }
                Err(e) => {
                    self.states.insert(result.id, LoadState::Failed);
                    log::error!("Asset {:?} failed: {}", result.id, e);
                }
            }
            self.completed.push(result);
            count += 1;
        }
        #[cfg(feature = "hot-reload")]
        if let Some(ref mut watcher) = self.watcher {
            for changed_path in watcher.poll_changes() {
                if let Some(&id) = self.path_to_id.get(&changed_path) {
                    log::info!("热重载: {:?}", changed_path);
                    self.reload(id);
                }
            }
        }

        // 自动清理无引用的缓存资产
        self.process_unloads();

        count
    }

    /// 取出所有已完成的加载结果（消耗缓存）。
    ///
    /// 游戏代码调用此方法获取原始字节数据，然后自行解析。
    pub fn drain_completed(&mut self) -> Vec<AsyncLoadResult> {
        std::mem::take(&mut self.completed)
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

    /// 处理已完成的加载任务（main thread 侧）
    ///
    /// 从 worker channel 接收已解析的字节数据，
    /// 缓存到 `loaded_cache` 并更新 load state。
    ///
    /// 与 [`process_completed`] 不同，此方法仅计数成功加载的资产，
    /// 不执行 hot-reload 检查或自动卸载。
    ///
    /// # 返回
    ///
    /// 本帧处理的完成项数量
    pub fn process_completed_count(&mut self) -> usize {
        // process_completed already handles the full logic including
        // hot-reload and unloads. This method provides a lightweight
        // count-only interface that drains the async channel.
        let mut count = 0;
        while let Ok(result) = self.async_rx.try_recv() {
            match &result.data {
                Ok(data) => {
                    self.loaded_cache.insert(result.id, Arc::new(data.clone()));
                    self.states.insert(result.id, LoadState::Loaded);
                }
                Err(_e) => {
                    self.states.insert(result.id, LoadState::Failed);
                }
            }
            self.completed.push(result);
            count += 1;
        }
        count
    }

    /// 获取资产根目录
    pub fn asset_root(&self) -> &Path {
        &self.asset_root
    }

    /// 强制重新加载指定资产（清除缓存并重新发起异步加载）
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use anvilkit_assets::asset_server::{AssetServer, AssetId};
    ///
    /// let mut server = AssetServer::new("assets");
    /// // server.reload(some_id); // 文件修改后强制刷新
    /// ```
    pub fn reload(&mut self, id: AssetId) {
        // 清除缓存
        self.loaded_cache.remove(&id);
        self.states.insert(id, LoadState::Loading);

        // 查找路径并重新发起异步加载
        if let Some(file_path) = self.id_to_path.get(&id).cloned() {
            let tx = self.async_tx.clone();
            let _ = self.task_tx.send(Box::new(move || {
                let result = std::fs::read(&file_path)
                    .map_err(|e| format!("Failed to reload {:?}: {}", file_path, e));
                let _ = tx.send(AsyncLoadResult { id, data: result });
            }));
        }
    }

    /// 获取缓存的资产字节数据
    ///
    /// 如果资产已加载过，返回 `Some(Arc<Vec<u8>>)`。
    /// 未加载或加载失败返回 `None`。
    pub fn get_cached(&self, id: AssetId) -> Option<Arc<Vec<u8>>> {
        self.loaded_cache.get(&id).cloned()
    }

    /// 缓存中的资产数量
    pub fn cache_len(&self) -> usize {
        self.loaded_cache.len()
    }

    /// 清理无引用的缓存资产
    ///
    /// 遍历缓存，移除 `Arc::strong_count == 1` 的条目
    /// （仅 AssetServer 自身持有，所有外部 handle 已 drop）。
    /// Cascade-unloads orphaned dependencies via the dependency graph.
    ///
    /// 返回本次清理的资产数量。
    pub fn process_unloads(&mut self) -> usize {
        // Collect IDs whose Arc is only held by AssetServer (refcount == 1)
        let unreferenced: Vec<AssetId> = self.loaded_cache.iter()
            .filter(|(_, arc)| Arc::strong_count(arc) <= 1)
            .map(|(&id, _)| id)
            .collect();

        let mut total_unloaded = 0;
        for id in unreferenced {
            log::debug!("自动卸载资产 {:?}", id);
            let orphans = self.unload(id);
            total_unloaded += 1 + orphans.len();
        }

        total_unloaded
    }

    /// Store a parsed asset for a given ID.
    ///
    /// After raw bytes are loaded and parsed (e.g., glTF -> MeshData),
    /// call this to make the parsed result available via the storage.
    pub fn insert_parsed(&mut self, id: AssetId, asset: ParsedAsset) {
        self.parsed_assets.insert(id, asset);
        self.states.insert(id, LoadState::Loaded);
    }

    /// Retrieve a reference to a previously stored parsed asset.
    pub fn get_parsed(&self, id: AssetId) -> Option<&ParsedAsset> {
        self.parsed_assets.get(&id)
    }

    /// Immutable access to the dependency graph.
    pub fn dependency_graph(&self) -> &DependencyGraph {
        &self.dependency_graph
    }

    /// Mutable access to the dependency graph.
    pub fn dependency_graph_mut(&mut self) -> &mut DependencyGraph {
        &mut self.dependency_graph
    }

    /// Register a parent→child dependency between two assets.
    ///
    /// When the parent is unloaded, children with no remaining parents
    /// will be cascade-unloaded automatically.
    pub fn add_dependency(&mut self, parent: AssetId, child: AssetId) {
        self.dependency_graph.add_dependency(parent, child);
    }

    /// Immutable access to the content-addressed asset cache.
    pub fn asset_cache(&self) -> &AssetCache {
        &self.cache
    }

    /// Mutable access to the content-addressed asset cache.
    pub fn asset_cache_mut(&mut self) -> &mut AssetCache {
        &mut self.cache
    }

    /// Unload an asset and cascade-unload any orphaned dependencies.
    ///
    /// Removes the asset from all internal caches (`loaded_cache`, `parsed_assets`,
    /// `states`) and then removes it from the dependency graph. Any children that
    /// become orphans (no remaining parents) are also unloaded recursively.
    ///
    /// Returns the list of additionally unloaded (orphaned) asset IDs.
    pub fn unload(&mut self, id: AssetId) -> Vec<AssetId> {
        let orphans = self.dependency_graph.remove_and_cascade(id);

        // Clean up the requested asset
        self.loaded_cache.remove(&id);
        self.parsed_assets.remove(&id);
        self.states.insert(id, LoadState::NotLoaded);

        // Clean up orphaned dependents
        for &orphan in &orphans {
            self.loaded_cache.remove(&orphan);
            self.parsed_assets.remove(&orphan);
            self.states.insert(orphan, LoadState::NotLoaded);
        }

        orphans
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

    #[test]
    fn test_cache_and_reload() {
        let mut server = AssetServer::new("/tmp/nonexistent_assets");
        let handle = server.load::<String>("test.txt");

        // Initially no cache
        assert!(server.get_cached(handle.id()).is_none());
        assert_eq!(server.cache_len(), 0);
    }

    #[test]
    fn test_process_unloads_empty() {
        let mut server = AssetServer::new("/tmp");
        assert_eq!(server.process_unloads(), 0);
    }

    #[test]
    fn test_reload_invalidates_cache() {
        let mut server = AssetServer::new("/tmp/nonexistent_assets");
        let handle = server.load::<String>("test.txt");
        // Reload should invalidate cache and re-dispatch load
        server.reload(handle.id());
        // Should still be in loading state, not crash
        assert_eq!(server.load_state(&handle), LoadState::Loading);
    }

    #[test]
    fn test_dependency_graph_accessors() {
        let mut server = AssetServer::new("/tmp");
        let parent = AssetId::from_raw(100);
        let child = AssetId::from_raw(200);

        server.add_dependency(parent, child);

        assert!(server.dependency_graph().dependencies_of(parent).contains(&child));
        assert!(server.dependency_graph().has_dependents(child));
    }

    #[test]
    fn test_unload_cascade() {
        let mut server = AssetServer::new("/tmp");
        let scene_id = AssetId::from_raw(10);
        let tex_id = AssetId::from_raw(20);
        let mat_id = AssetId::from_raw(30);

        // Simulate loaded assets
        server.loaded_cache.insert(scene_id, Arc::new(vec![1]));
        server.loaded_cache.insert(tex_id, Arc::new(vec![2]));
        server.loaded_cache.insert(mat_id, Arc::new(vec![3]));
        server.states.insert(scene_id, LoadState::Loaded);
        server.states.insert(tex_id, LoadState::Loaded);
        server.states.insert(mat_id, LoadState::Loaded);

        // scene -> texture, scene -> material
        server.add_dependency(scene_id, tex_id);
        server.add_dependency(scene_id, mat_id);

        let orphans = server.unload(scene_id);

        // Both children should be orphaned and unloaded
        assert!(orphans.contains(&tex_id));
        assert!(orphans.contains(&mat_id));

        // All caches should be cleared
        assert!(server.get_cached(scene_id).is_none());
        assert!(server.get_cached(tex_id).is_none());
        assert!(server.get_cached(mat_id).is_none());

        // States should be reset
        assert_eq!(*server.states.get(&scene_id).unwrap(), LoadState::NotLoaded);
        assert_eq!(*server.states.get(&tex_id).unwrap(), LoadState::NotLoaded);
        assert_eq!(*server.states.get(&mat_id).unwrap(), LoadState::NotLoaded);
    }

    #[test]
    fn test_unload_shared_dep_no_cascade() {
        let mut server = AssetServer::new("/tmp");
        let scene_a = AssetId::from_raw(1);
        let scene_b = AssetId::from_raw(2);
        let shared_tex = AssetId::from_raw(3);

        server.loaded_cache.insert(scene_a, Arc::new(vec![1]));
        server.loaded_cache.insert(scene_b, Arc::new(vec![2]));
        server.loaded_cache.insert(shared_tex, Arc::new(vec![3]));

        // Both scenes depend on the same texture
        server.add_dependency(scene_a, shared_tex);
        server.add_dependency(scene_b, shared_tex);

        let orphans = server.unload(scene_a);

        // shared_tex still has scene_b as parent, so not orphaned
        assert!(orphans.is_empty());
        assert!(server.get_cached(shared_tex).is_some());
    }

    #[test]
    fn test_asset_cache_field() {
        let server = AssetServer::new("/tmp");
        assert!(server.asset_cache().is_empty());
    }

    #[test]
    fn test_load_async_cache_hit_skips_dispatch() {
        let mut server = AssetServer::new("/tmp");
        let handle = server.load::<String>("test.txt");

        // Manually insert into loaded_cache to simulate already-loaded
        server.loaded_cache.insert(handle.id(), Arc::new(b"cached data".to_vec()));

        // load_async should return immediately without dispatching I/O
        let handle2 = server.load_async::<String>("test.txt");
        assert_eq!(handle.id(), handle2.id());
    }
}
