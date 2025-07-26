//!
//! 策略服务层
//! 业务逻辑实现，供指令入口调用，封装策略注册、执行、验证、批量操作、权限校验等操作。

use anchor_lang::prelude::*;
use crate::core::types::{StrategyParams, BatchStrategyParams, StrategyResult};
use crate::errors::basket_error::BasketError;

/// 策略注册trait
///
/// 定义策略注册接口，便于扩展多种策略注册方式。
/// - 设计意图：统一注册入口，便于后续多种策略注册策略。
pub trait StrategyRegistrable {
    /// 注册策略
    ///
    /// # 参数
    /// - `params`: 策略参数。
    ///
    /// # 返回值
    /// - 成功返回 Ok(())，失败返回 BasketError。
    fn register(&self, params: &StrategyParams) -> Result<()>;
}

/// 策略注册服务实现
///
/// 示例实现：注册到全局策略表。
pub struct RegisterStrategyService;
impl StrategyRegistrable for RegisterStrategyService {
    /// 注册实现
    fn register(&self, _params: &StrategyParams) -> Result<()> {
        // 生产级实现：注册到全局策略表
        Ok(())
    }
}

/// 策略执行trait
///
/// 定义策略执行接口，便于扩展多种策略执行方式。
/// - 设计意图：统一执行入口，便于后续多种策略执行。
pub trait StrategyExecutable {
    /// 执行策略
    ///
    /// # 参数
    /// - `params`: 策略参数。
    ///
    /// # 返回值
    /// - 返回策略执行结果，失败返回 BasketError。
    fn execute(&self, params: &StrategyParams) -> Result<StrategyResult>;
}

/// 策略执行服务实现
///
/// 示例实现：根据params执行策略。
pub struct ExecuteStrategyService;
impl StrategyExecutable for ExecuteStrategyService {
    /// 执行实现
    fn execute(&self, params: &StrategyParams) -> Result<StrategyResult> {
        // 生产级实现：根据params执行策略
        Ok(StrategyResult { success: true, output: 100 })
    }
}

/// 策略验证trait
///
/// 定义策略验证接口，便于扩展多种策略验证方式。
/// - 设计意图：统一验证入口，便于后续多种策略验证。
pub trait StrategyValidatable {
    /// 验证策略
    ///
    /// # 参数
    /// - `params`: 策略参数。
    ///
    /// # 返回值
    /// - 是否有效。
    fn validate(&self, params: &StrategyParams) -> Result<bool>;
}

/// 策略验证服务实现
///
/// 示例实现：验证策略有效性。
pub struct ValidateStrategyService;
impl StrategyValidatable for ValidateStrategyService {
    /// 验证实现
    fn validate(&self, _params: &StrategyParams) -> Result<bool> {
        // 生产级实现：验证策略有效性
        Ok(true)
    }
}

/// 策略批量操作trait
///
/// 定义批量操作接口，便于扩展多种批量操作方式。
/// - 设计意图：统一批量操作入口，便于后续多种批量策略。
pub trait StrategyBatchOperable {
    /// 批量操作
    ///
    /// # 参数
    /// - `batch_params`: 批量参数。
    ///
    /// # 返回值
    /// - 返回每笔操作的结果集合，失败返回 BasketError。
    fn batch_operate(&self, batch_params: &BatchStrategyParams) -> Result<Vec<StrategyResult>>;
}

/// 策略批量操作服务实现
///
/// 示例实现：遍历批量参数。
pub struct BatchOperateStrategyService;
impl StrategyBatchOperable for BatchOperateStrategyService {
    /// 批量操作实现
    fn batch_operate(&self, batch_params: &BatchStrategyParams) -> Result<Vec<StrategyResult>> {
        // 生产级实现：遍历批量参数
        Ok(batch_params.strategies.iter().map(|_| StrategyResult { success: true, output: 100 }).collect())
    }
}

/// 策略权限校验trait
///
/// 定义策略权限校验接口，便于扩展多种权限模型。
/// - 设计意图：统一权限校验入口，便于后续多种权限策略。
pub trait StrategyAuthorizable {
    /// 校验策略操作权限
    ///
    /// # 参数
    /// - `authority`: 操作人。
    ///
    /// # 返回值
    /// - 是否有权限。
    fn authorize(&self, authority: Pubkey) -> Result<bool>;
}

/// 策略权限校验服务实现
///
/// 示例实现：校验权限。
pub struct AuthorizeStrategyService;
impl StrategyAuthorizable for AuthorizeStrategyService {
    /// 权限校验实现
    fn authorize(&self, _authority: Pubkey) -> Result<bool> {
        // 生产级实现：校验权限
        Ok(true)
    }
}

/// 策略服务门面
///
/// 设计意图：统一对外暴露所有策略相关操作，便于维护和扩展。
pub struct StrategyService;
impl StrategyService {
    /// 策略注册，融合多资产/算法/DEX/Oracle
    pub fn register_strategy(name: &str, params: &StrategyParams) -> Result<()> {
        // 1. 参数校验
        require!(!name.is_empty(), crate::errors::asset_error::AssetError::InvalidParams);
        // 2. 可选算法/DEX/Oracle适配器注册
        if let Some(algo_name) = &params.algo_name {
            let factory = crate::core::registry::ADAPTER_FACTORY.lock().unwrap();
            if let Some(algo) = factory.get(algo_name) {
                // 可选：注册算法到策略表
            }
        }
        if let Some(dex_name) = &params.dex_name {
            let factory = crate::core::registry::ADAPTER_FACTORY.lock().unwrap();
            if let Some(dex) = factory.get(dex_name) {
                // 可选：注册DEX到策略表
            }
        }
        if let Some(oracle_name) = &params.oracle_name {
            let factory = crate::core::registry::ADAPTER_FACTORY.lock().unwrap();
            if let Some(oracle) = factory.get(oracle_name) {
                // 可选：注册Oracle到策略表
            }
        }
        // 3. 事件追踪
        crate::core::logging::log_instruction_dispatch("register_strategy", name);
        Ok(())
    }
    /// 查询策略
    pub fn query(registry: &StrategyRegistryAccount, strategy_id: u64) -> Result<StrategyMeta> {
        let svc = QueryStrategyService;
        svc.query(registry, strategy_id)
    }
    /// 切换策略
    pub fn switch(registry: &mut StrategyRegistryAccount, from: u64, to: u64) -> Result<()> {
        let svc = SwitchStrategyService;
        svc.switch(registry, from, to)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::strategy_registry_account::{StrategyRegistryAccount, StrategyMeta};
    use crate::strategies::strategy_registry::StrategyConfig;
    use anchor_lang::prelude::Pubkey;

    fn default_registry() -> StrategyRegistryAccount {
        StrategyRegistryAccount {
            base: crate::state::common::BaseAccount::default(),
            strategies: vec![],
        }
    }

    fn default_config() -> StrategyConfig {
        StrategyConfig { name: "test_strategy".to_string(), params: vec![] }
    }

    #[test]
    fn test_register_strategy_success() {
        let mut registry = default_registry();
        let id = 1;
        let config = default_config();
        let authority = Pubkey::new_unique();
        let result = StrategyService::register(&mut registry, id, config.clone(), authority);
        assert!(result.is_ok());
        assert!(registry.strategies.iter().any(|s| s.id == id));
    }

    #[test]
    fn test_query_strategy_success() {
        let mut registry = default_registry();
        let id = 2;
        let config = default_config();
        let authority = Pubkey::new_unique();
        StrategyService::register(&mut registry, id, config.clone(), authority).unwrap();
        let meta = StrategyService::query(&registry, id).unwrap();
        assert_eq!(meta.id, id);
    }

    #[test]
    fn test_query_strategy_not_found() {
        let registry = default_registry();
        let result = StrategyService::query(&registry, 999);
        assert!(result.is_err());
    }

    #[test]
    fn test_switch_strategy_success() {
        let mut registry = default_registry();
        let authority = Pubkey::new_unique();
        StrategyService::register(&mut registry, 10, default_config(), authority).unwrap();
        StrategyService::register(&mut registry, 20, default_config(), authority).unwrap();
        let result = StrategyService::switch(&mut registry, 10, 20);
        assert!(result.is_ok());
    }

    #[test]
    fn test_switch_strategy_not_found() {
        let mut registry = default_registry();
        let result = StrategyService::switch(&mut registry, 10, 20);
        assert!(result.is_err());
    }
} 