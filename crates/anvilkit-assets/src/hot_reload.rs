//! # 资源热重载
//!
//! 监视资源目录的文件变更，自动触发重新加载。
//! 需要启用 `hot-reload` feature flag。
//!
//! # 示例
//!
//! ```rust,no_run
//! use anvilkit_assets::hot_reload::FileWatcher;
//!
//! let mut watcher = FileWatcher::new("assets").unwrap();
//! // In your game loop:
//! for path in watcher.poll_changes() { // requires &mut self
//!     println!("File changed: {:?}", path);
//! }
//! ```

#[cfg(feature = "hot-reload")]
mod inner {
    use notify::{Watcher, RecursiveMode, Event, EventKind};
    use std::path::{Path, PathBuf};
    use std::sync::mpsc;

    /// 文件变更监视器
    ///
    /// 使用 `notify` crate 监视目录，检测文件创建/修改/删除事件。
    /// 每帧调用 `poll_changes()` 获取变更文件列表。
    pub struct FileWatcher {
        _watcher: notify::RecommendedWatcher,
        rx: mpsc::Receiver<PathBuf>,
        watch_root: PathBuf,
        pending_changes: std::collections::HashMap<PathBuf, std::time::Instant>,
    }

    impl FileWatcher {
        /// 创建文件监视器并开始监视指定目录。
        pub fn new(watch_dir: impl AsRef<Path>) -> Result<Self, String> {
            let watch_root = watch_dir.as_ref().to_path_buf();
            let (tx, rx) = mpsc::channel();

            let sender = tx.clone();
            let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    match event.kind {
                        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                            for path in event.paths {
                                let _ = sender.send(path);
                            }
                        }
                        _ => {}
                    }
                }
            }).map_err(|e| format!("Failed to create file watcher: {}", e))?;

            watcher.watch(&watch_root, RecursiveMode::Recursive)
                .map_err(|e| format!("Failed to watch {:?}: {}", watch_root, e))?;

            log::info!("File watcher started on {:?}", watch_root);

            Ok(Self {
                _watcher: watcher,
                rx,
                watch_root,
                pending_changes: std::collections::HashMap::new(),
            })
        }

        /// 轮询所有待处理的文件变更。
        ///
        /// 返回已稳定（经过 200ms 防抖）的变更文件路径列表。
        pub fn poll_changes(&mut self) -> Vec<PathBuf> {
            let now = std::time::Instant::now();
            let debounce_duration = std::time::Duration::from_millis(200);

            // Collect new events
            while let Ok(path) = self.rx.try_recv() {
                self.pending_changes.insert(path, now);
            }

            // Return paths that have been stable for debounce_duration
            let mut ready = Vec::new();
            self.pending_changes.retain(|path, last_event| {
                if now.duration_since(*last_event) >= debounce_duration {
                    ready.push(path.clone());
                    false // remove from pending
                } else {
                    true // keep pending
                }
            });
            ready
        }

        /// 检查路径是否是特定类型的资源文件。
        pub fn is_shader(path: &Path) -> bool {
            path.extension().map_or(false, |e| e == "wgsl")
        }

        /// 检查路径是否是纹理文件。
        pub fn is_texture(path: &Path) -> bool {
            path.extension().map_or(false, |e| {
                e == "png" || e == "jpg" || e == "jpeg" || e == "bmp" || e == "tga"
            })
        }

        /// 监视根目录。
        pub fn watch_root(&self) -> &Path {
            &self.watch_root
        }
    }
}

#[cfg(feature = "hot-reload")]
pub use inner::FileWatcher;

// When hot-reload is disabled, provide a no-op stub
#[cfg(not(feature = "hot-reload"))]
mod stub {
    use std::path::{Path, PathBuf};

    /// No-op file watcher (hot-reload feature disabled).
    pub struct FileWatcher;

    impl FileWatcher {
        /// Returns Err when hot-reload feature is not enabled.
        pub fn new(_watch_dir: impl AsRef<Path>) -> Result<Self, String> {
            Err("hot-reload feature not enabled".to_string())
        }

        /// Always returns empty (no-op).
        pub fn poll_changes(&mut self) -> Vec<PathBuf> { Vec::new() }

        /// Always false.
        pub fn is_shader(_path: &Path) -> bool { false }

        /// Always false.
        pub fn is_texture(_path: &Path) -> bool { false }

        /// Returns empty path.
        pub fn watch_root(&self) -> &Path { Path::new("") }
    }
}

#[cfg(not(feature = "hot-reload"))]
pub use stub::FileWatcher;
