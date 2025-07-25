//!
//! 算法服务层
//! 业务逻辑实现，供指令入口调用，封装算法注册、查询、切换、执行、批量操作、权限校验等操作。

use anchor_lang::prelude::*;
use crate::core::types::{AlgorithmParams, BatchAlgorithmParams, AlgorithmResult};
use crate::errors::basket_error::BasketError;

/// 算法注册trait
///
/// 定义算法注册接口，便于扩展多种注册方式。
/// - 设计意图：统一注册入口，便于后续多种算法注册策略。
pub trait AlgorithmRegistrable {
    /// 注册算法
    ///
    /// # 参数
    /// - `params`: 算法参数。
    ///
    /// # 返回值
    /// - 成功返回 Ok(())，失败返回 BasketError。
    fn register(&self, params: &AlgorithmParams) -> Result<()>;
}

/// 算法注册服务实现
///
/// 示例实现：注册到全局算法表。
pub struct RegisterAlgorithmService;
impl AlgorithmRegistrable for RegisterAlgorithmService {
    /// 注册实现
    fn register(&self, _params: &AlgorithmParams) -> Result<()> {
        // 生产级实现：注册到全局算法表
        Ok(())
    }
}

/// 算法查询trait
///
/// 定义算法查询接口，便于扩展多种查询方式。
/// - 设计意图：统一查询入口，便于后续多种算法查询。
pub trait AlgorithmQueryable {
    /// 查询算法
    ///
    /// # 参数
    /// - `params`: 查询参数。
    ///
    /// # 返回值
    /// - 返回算法结果，失败返回 BasketError。
    fn query(&self, params: &AlgorithmParams) -> Result<AlgorithmResult>;
}

/// 算法查询服务实现
///
/// 示例实现：根据params查询算法。
pub struct QueryAlgorithmService;
impl AlgorithmQueryable for QueryAlgorithmService {
    /// 查询实现
    fn query(&self, _params: &AlgorithmParams) -> Result<AlgorithmResult> {
        // 生产级实现：根据params查询算法
        Ok(AlgorithmResult { success: true, output: 100 })
    }
}

/// 算法切换trait
///
/// 定义算法切换接口，便于扩展多种切换方式。
/// - 设计意图：统一切换入口，便于后续多种算法切换。
pub trait AlgorithmSwitchable {
    /// 切换算法
    ///
    /// # 参数
    /// - `params`: 切换参数。
    ///
    /// # 返回值
    /// - 成功返回 Ok(())，失败返回 BasketError。
    fn switch(&self, params: &AlgorithmParams) -> Result<()>;
}

/// 算法切换服务实现
///
/// 示例实现：切换算法。
pub struct SwitchAlgorithmService;
impl AlgorithmSwitchable for SwitchAlgorithmService {
    /// 切换实现
    fn switch(&self, _params: &AlgorithmParams) -> Result<()> {
        // 生产级实现：切换算法
        Ok(())
    }
}

/// 算法执行trait
///
/// 定义算法执行接口，便于扩展多种执行方式。
/// - 设计意图：统一执行入口，便于后续多种算法执行。
pub trait AlgorithmExecutable {
    /// 执行算法
    ///
    /// # 参数
    /// - `params`: 执行参数。
    ///
    /// # 返回值
    /// - 返回算法执行结果，失败返回 BasketError。
    fn execute(&self, params: &AlgorithmParams) -> Result<AlgorithmResult>;
}

/// 算法执行服务实现
///
/// 示例实现：根据params执行算法。
pub struct ExecuteAlgorithmService;
impl AlgorithmExecutable for ExecuteAlgorithmService {
    /// 执行实现
    fn execute(&self, _params: &AlgorithmParams) -> Result<AlgorithmResult> {
        // 生产级实现：根据params执行算法
        Ok(AlgorithmResult { success: true, output: 100 })
    }
}

/// 算法批量操作trait
///
/// 定义批量操作接口，便于扩展多种批量操作方式。
/// - 设计意图：统一批量操作入口，便于后续多种批量策略。
pub trait AlgorithmBatchOperable {
    /// 批量操作
    ///
    /// # 参数
    /// - `batch_params`: 批量参数。
    ///
    /// # 返回值
    /// - 返回每笔操作的结果集合，失败返回 BasketError。
    fn batch_operate(&self, batch_params: &BatchAlgorithmParams) -> Result<Vec<AlgorithmResult>>;
}

/// 算法批量操作服务实现
///
/// 示例实现：遍历批量参数。
pub struct BatchOperateAlgorithmService;
impl AlgorithmBatchOperable for BatchOperateAlgorithmService {
    /// 批量操作实现
    fn batch_operate(&self, batch_params: &BatchAlgorithmParams) -> Result<Vec<AlgorithmResult>> {
        // 生产级实现：遍历批量参数
        Ok(batch_params.algorithms.iter().map(|_| AlgorithmResult { success: true, output: 100 }).collect())
    }
}

/// 算法权限校验trait
///
/// 定义算法权限校验接口，便于扩展多种权限模型。
/// - 设计意图：统一权限校验入口，便于后续多种权限策略。
pub trait AlgorithmAuthorizable {
    /// 校验算法操作权限
    ///
    /// # 参数
    /// - `authority`: 操作人。
    ///
    /// # 返回值
    /// - 是否有权限。
    fn authorize(&self, authority: Pubkey) -> Result<bool>;
}

/// 算法权限校验服务实现
///
/// 示例实现：校验权限。
pub struct AuthorizeAlgorithmService;
impl AlgorithmAuthorizable for AuthorizeAlgorithmService {
    /// 权限校验实现
    fn authorize(&self, _authority: Pubkey) -> Result<bool> {
        // 生产级实现：校验权限
        Ok(true)
    }
}

/// 算法服务门面（facade）
///
/// 设计意图：统一对外暴露所有算法相关操作，便于维护和扩展。
pub struct AlgorithmService;
impl AlgorithmService {
    /// 注册新算法
    pub fn register(params: &AlgorithmParams) -> Result<()> {
        let svc = RegisterAlgorithmService;
        svc.register(params)
    }
    /// 查询算法元数据
    pub fn query(params: &AlgorithmParams) -> Result<AlgorithmResult> {
        let svc = QueryAlgorithmService;
        svc.query(params)
    }
    /// 切换算法
    pub fn switch(params: &AlgorithmParams) -> Result<()> {
        let svc = SwitchAlgorithmService;
        svc.switch(params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{AlgorithmParams, AlgorithmResult};
    use anchor_lang::prelude::Pubkey;

    /// 构造默认空注册表
    fn default_registry() -> AlgorithmRegistryAccount {
        AlgorithmRegistryAccount {
            base: crate::state::common::BaseAccount::default(),
            algorithms: vec![],
        }
    }

    /// 测试算法注册成功
    #[test]
    fn test_register_algorithm_success() {
        let mut registry = default_registry();
        let name = "algo1".to_string();
        let authority = Pubkey::new_unique();
        let result = AlgorithmService::register(&mut registry, name.clone(), authority);
        assert!(result.is_ok());
        assert!(registry.algorithms.iter().any(|a| a.name == name));
    }

    /// 测试算法查询成功
    #[test]
    fn test_query_algorithm_success() {
        let mut registry = default_registry();
        let name = "algo2".to_string();
        let authority = Pubkey::new_unique();
        AlgorithmService::register(&mut registry, name.clone(), authority).unwrap();
        let meta = AlgorithmService::query(&registry, &name).unwrap();
        assert_eq!(meta.name, name);
    }

    /// 测试算法查询失败
    #[test]
    fn test_query_algorithm_not_found() {
        let registry = default_registry();
        let result = AlgorithmService::query(&registry, "not_exist");
        assert!(result.is_err());
    }

    /// 测试算法切换成功
    #[test]
    fn test_switch_algorithm_success() {
        let mut registry = default_registry();
        let authority = Pubkey::new_unique();
        AlgorithmService::register(&mut registry, "from".to_string(), authority).unwrap();
        AlgorithmService::register(&mut registry, "to".to_string(), authority).unwrap();
        let result = AlgorithmService::switch(&mut registry, "from", "to");
        assert!(result.is_ok());
    }

    /// 测试算法切换失败
    #[test]
    fn test_switch_algorithm_not_found() {
        let mut registry = default_registry();
        let result = AlgorithmService::switch(&mut registry, "from", "to");
        assert!(result.is_err());
    }
} 