//! # 存档数据迁移框架
//!
//! 提供版本化存档数据的迁移机制。当游戏版本更新导致存档格式变化时，
//! `MigrationRunner` 按版本链依次执行 `SaveMigration`，将旧存档升级到当前版本。

use std::collections::HashMap;

/// 单步存档数据迁移
///
/// 每个实现对应一次版本升级操作。迁移在 `HashMap<String, String>` 上执行，
/// 即 RON 或其他格式解析后的键值对表示。
///
/// # 示例
///
/// ```rust
/// use std::collections::HashMap;
/// use anvilkit_core::persistence::SaveMigration;
///
/// struct MigrateV1ToV2;
///
/// impl SaveMigration for MigrateV1ToV2 {
///     fn from_version(&self) -> u32 { 1 }
///     fn to_version(&self) -> u32 { 2 }
///     fn migrate(&self, data: &mut HashMap<String, String>) -> Result<(), String> {
///         // 将旧的 "hp" 字段重命名为 "health"
///         if let Some(hp) = data.remove("hp") {
///             data.insert("health".to_string(), hp);
///         }
///         Ok(())
///     }
/// }
/// ```
pub trait SaveMigration: Send + Sync {
    /// 此迁移升级的源版本号。
    fn from_version(&self) -> u32;
    /// 此迁移升级的目标版本号。
    fn to_version(&self) -> u32;
    /// 对存档数据执行迁移变换。
    fn migrate(&self, data: &mut HashMap<String, String>) -> Result<(), String>;
}

/// 存档迁移运行器
///
/// 注册多个 `SaveMigration`，按版本链顺序执行所有必要的迁移步骤，
/// 将存档数据从任意旧版本升级到当前版本。
///
/// # 示例
///
/// ```rust
/// use std::collections::HashMap;
/// use anvilkit_core::persistence::{SaveMigration, MigrationRunner};
///
/// struct MigrateV1ToV2;
/// impl SaveMigration for MigrateV1ToV2 {
///     fn from_version(&self) -> u32 { 1 }
///     fn to_version(&self) -> u32 { 2 }
///     fn migrate(&self, data: &mut HashMap<String, String>) -> Result<(), String> {
///         if let Some(hp) = data.remove("hp") {
///             data.insert("health".to_string(), hp);
///         }
///         Ok(())
///     }
/// }
///
/// let mut runner = MigrationRunner::new(2);
/// runner.register(Box::new(MigrateV1ToV2));
///
/// let mut data = HashMap::new();
/// data.insert("hp".to_string(), "100".to_string());
///
/// let applied = runner.migrate(1, &mut data).unwrap();
/// assert_eq!(applied, 1);
/// assert_eq!(data.get("health").unwrap(), "100");
/// ```
#[cfg_attr(feature = "bevy_ecs", derive(bevy_ecs::system::Resource))]
pub struct MigrationRunner {
    migrations: Vec<Box<dyn SaveMigration>>,
    current_version: u32,
}

impl MigrationRunner {
    /// 创建迁移运行器。
    ///
    /// `current_version` 是游戏当前的存档格式版本号。
    pub fn new(current_version: u32) -> Self {
        Self {
            migrations: Vec::new(),
            current_version,
        }
    }

    /// 注册一个迁移步骤。
    ///
    /// 迁移无需按顺序注册，运行时会自动按 `from_version` 排序。
    pub fn register(&mut self, migration: Box<dyn SaveMigration>) -> &mut Self {
        self.migrations.push(migration);
        self
    }

    /// 执行所有必要的迁移，将数据从 `from_version` 升级到 `current_version`。
    ///
    /// 返回实际应用的迁移步骤数。如果 `from_version` 已经等于或大于
    /// `current_version`，则不执行任何迁移，返回 `Ok(0)`。
    ///
    /// # 错误
    ///
    /// - 如果某个迁移步骤执行失败，返回该步骤的错误信息。
    /// - 如果缺少必要的迁移步骤（版本链断裂），返回错误。
    pub fn migrate(
        &self,
        from_version: u32,
        data: &mut HashMap<String, String>,
    ) -> Result<u32, String> {
        if from_version >= self.current_version {
            return Ok(0);
        }

        // 收集并排序需要执行的迁移
        let mut applicable: Vec<&dyn SaveMigration> = self
            .migrations
            .iter()
            .map(|m| m.as_ref())
            .filter(|m| m.from_version() >= from_version && m.to_version() <= self.current_version)
            .collect();
        applicable.sort_by_key(|m| m.from_version());

        // 验证版本链的连续性并执行迁移
        let mut current = from_version;
        let mut applied = 0u32;

        for migration in &applicable {
            if migration.from_version() != current {
                // 跳过不属于当前链的迁移
                continue;
            }
            migration.migrate(data)?;
            current = migration.to_version();
            applied += 1;

            if current >= self.current_version {
                break;
            }
        }

        // 检查是否成功到达目标版本
        if current < self.current_version {
            return Err(format!(
                "Migration chain broken: reached version {} but target is {} \
                 (missing migration from v{} to v{})",
                current,
                self.current_version,
                current,
                current + 1,
            ));
        }

        Ok(applied)
    }

    /// 获取当前目标版本号。
    pub fn current_version(&self) -> u32 {
        self.current_version
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试用迁移：v1 → v2，将 "hp" 重命名为 "health"
    struct MigrateV1ToV2;
    impl SaveMigration for MigrateV1ToV2 {
        fn from_version(&self) -> u32 { 1 }
        fn to_version(&self) -> u32 { 2 }
        fn migrate(&self, data: &mut HashMap<String, String>) -> Result<(), String> {
            if let Some(hp) = data.remove("hp") {
                data.insert("health".to_string(), hp);
            }
            Ok(())
        }
    }

    /// 测试用迁移：v2 → v3，添加 "armor" 默认值
    struct MigrateV2ToV3;
    impl SaveMigration for MigrateV2ToV3 {
        fn from_version(&self) -> u32 { 2 }
        fn to_version(&self) -> u32 { 3 }
        fn migrate(&self, data: &mut HashMap<String, String>) -> Result<(), String> {
            data.entry("armor".to_string()).or_insert_with(|| "0".to_string());
            Ok(())
        }
    }

    /// 测试用迁移：v3 → v4，固定会失败
    struct MigrateV3ToV4Fail;
    impl SaveMigration for MigrateV3ToV4Fail {
        fn from_version(&self) -> u32 { 3 }
        fn to_version(&self) -> u32 { 4 }
        fn migrate(&self, _data: &mut HashMap<String, String>) -> Result<(), String> {
            Err("Intentional migration failure".to_string())
        }
    }

    #[test]
    fn test_new_runner() {
        let runner = MigrationRunner::new(5);
        assert_eq!(runner.current_version(), 5);
    }

    #[test]
    fn test_no_migration_needed() {
        let runner = MigrationRunner::new(3);
        let mut data = HashMap::new();

        // 已经是当前版本
        let applied = runner.migrate(3, &mut data).unwrap();
        assert_eq!(applied, 0);

        // 比当前版本更新（未来版本存档）
        let applied = runner.migrate(5, &mut data).unwrap();
        assert_eq!(applied, 0);
    }

    #[test]
    fn test_single_migration() {
        let mut runner = MigrationRunner::new(2);
        runner.register(Box::new(MigrateV1ToV2));

        let mut data = HashMap::new();
        data.insert("hp".to_string(), "100".to_string());
        data.insert("name".to_string(), "Player".to_string());

        let applied = runner.migrate(1, &mut data).unwrap();
        assert_eq!(applied, 1);
        assert_eq!(data.get("health").unwrap(), "100");
        assert!(!data.contains_key("hp"));
        assert_eq!(data.get("name").unwrap(), "Player");
    }

    #[test]
    fn test_chained_migrations() {
        let mut runner = MigrationRunner::new(3);
        runner.register(Box::new(MigrateV1ToV2));
        runner.register(Box::new(MigrateV2ToV3));

        let mut data = HashMap::new();
        data.insert("hp".to_string(), "50".to_string());

        let applied = runner.migrate(1, &mut data).unwrap();
        assert_eq!(applied, 2);
        assert_eq!(data.get("health").unwrap(), "50");
        assert_eq!(data.get("armor").unwrap(), "0");
        assert!(!data.contains_key("hp"));
    }

    #[test]
    fn test_partial_chain() {
        let mut runner = MigrationRunner::new(3);
        runner.register(Box::new(MigrateV1ToV2));
        runner.register(Box::new(MigrateV2ToV3));

        // 只需要 v2 → v3
        let mut data = HashMap::new();
        data.insert("health".to_string(), "75".to_string());

        let applied = runner.migrate(2, &mut data).unwrap();
        assert_eq!(applied, 1);
        assert_eq!(data.get("health").unwrap(), "75");
        assert_eq!(data.get("armor").unwrap(), "0");
    }

    #[test]
    fn test_migration_failure() {
        let mut runner = MigrationRunner::new(4);
        runner.register(Box::new(MigrateV1ToV2));
        runner.register(Box::new(MigrateV2ToV3));
        runner.register(Box::new(MigrateV3ToV4Fail));

        let mut data = HashMap::new();
        data.insert("hp".to_string(), "100".to_string());

        let result = runner.migrate(1, &mut data);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Intentional migration failure"));
    }

    #[test]
    fn test_broken_chain() {
        // 缺少 v2 → v3 迁移
        let mut runner = MigrationRunner::new(3);
        runner.register(Box::new(MigrateV1ToV2));

        let mut data = HashMap::new();
        data.insert("hp".to_string(), "100".to_string());

        let result = runner.migrate(1, &mut data);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Migration chain broken"));
        assert!(err.contains("reached version 2"));
    }

    #[test]
    fn test_register_out_of_order() {
        // 迁移注册顺序不影响执行顺序
        let mut runner = MigrationRunner::new(3);
        runner.register(Box::new(MigrateV2ToV3)); // 先注册 v2→v3
        runner.register(Box::new(MigrateV1ToV2)); // 后注册 v1→v2

        let mut data = HashMap::new();
        data.insert("hp".to_string(), "80".to_string());

        let applied = runner.migrate(1, &mut data).unwrap();
        assert_eq!(applied, 2);
        assert_eq!(data.get("health").unwrap(), "80");
        assert_eq!(data.get("armor").unwrap(), "0");
    }

    #[test]
    fn test_register_chaining() {
        let mut runner = MigrationRunner::new(3);
        runner
            .register(Box::new(MigrateV1ToV2))
            .register(Box::new(MigrateV2ToV3));

        let mut data = HashMap::new();
        data.insert("hp".to_string(), "60".to_string());

        let applied = runner.migrate(1, &mut data).unwrap();
        assert_eq!(applied, 2);
    }

    #[test]
    fn test_migration_preserves_unrelated_data() {
        let mut runner = MigrationRunner::new(2);
        runner.register(Box::new(MigrateV1ToV2));

        let mut data = HashMap::new();
        data.insert("hp".to_string(), "100".to_string());
        data.insert("level".to_string(), "5".to_string());
        data.insert("gold".to_string(), "999".to_string());

        runner.migrate(1, &mut data).unwrap();
        assert_eq!(data.get("level").unwrap(), "5");
        assert_eq!(data.get("gold").unwrap(), "999");
    }

    #[test]
    fn test_migration_trait_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<MigrationRunner>();
    }
}
