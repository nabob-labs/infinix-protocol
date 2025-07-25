//!
//! 路由服务层
//! 业务逻辑实现，供指令入口调用，封装路由策略注册、路径选择、批量路由、权限校验等操作。

use anchor_lang::prelude::*;
use crate::core::types::{RoutingParams, BatchRoutingParams, RoutingResult};
use crate::errors::basket_error::BasketError;

/// 路由策略注册trait
///
/// 定义路由策略注册接口，便于扩展多种策略。
/// - 设计意图：统一注册入口，便于后续多种路由策略。
pub trait RoutingStrategyRegistrable {
    /// 注册路由策略
    ///
    /// # 参数
    /// - `strategy`: 策略实例。
    ///
    /// # 返回值
    /// - 成功返回 Ok(())，失败返回 BasketError。
    fn register_strategy(&self, strategy: Box<dyn RoutingStrategy>) -> Result<()>;
}

/// 路由策略注册服务实现
///
/// 示例实现：注册到全局策略表。
pub struct RegisterRoutingStrategyService;
impl RoutingStrategyRegistrable for RegisterRoutingStrategyService {
    /// 注册实现
    fn register_strategy(&self, _strategy: Box<dyn RoutingStrategy>) -> Result<()> {
        // 生产级实现：注册到全局策略表
        Ok(())
    }
}

/// 路径选择trait
///
/// 定义路径选择接口，便于扩展多种路径选择方式。
/// - 设计意图：统一路径选择入口，便于后续多种路由算法。
pub trait PathSelectable {
    /// 选择最佳路径
    ///
    /// # 参数
    /// - `params`: 路由参数。
    ///
    /// # 返回值
    /// - 返回路由结果，失败返回 BasketError。
    fn select_path(&self, params: &RoutingParams) -> Result<RoutingResult>;
}

/// 路径选择服务实现
///
/// 示例实现：根据params选择最佳路径。
pub struct SelectPathService;
impl PathSelectable for SelectPathService {
    /// 路径选择实现
    fn select_path(&self, params: &RoutingParams) -> Result<RoutingResult> {
        // 生产级实现：根据params选择最佳路径
        Ok(RoutingResult { best_path: vec![params.input, params.output], expected_out: 1000 })
    }
}

/// 批量路由trait
///
/// 定义批量路由接口，便于扩展多种批量路由方式。
/// - 设计意图：统一批量路由入口，便于后续多种批量策略。
pub trait BatchRoutable {
    /// 批量路由
    ///
    /// # 参数
    /// - `batch_params`: 批量参数。
    ///
    /// # 返回值
    /// - 返回每笔路由的结果集合，失败返回 BasketError。
    fn batch_route(&self, batch_params: &BatchRoutingParams) -> Result<Vec<RoutingResult>>;
}

/// 批量路由服务实现
///
/// 示例实现：遍历批量参数。
pub struct BatchRouteService;
impl BatchRoutable for BatchRouteService {
    /// 批量路由实现
    fn batch_route(&self, batch_params: &BatchRoutingParams) -> Result<Vec<RoutingResult>> {
        // 生产级实现：遍历批量参数
        Ok(batch_params.routes.iter().map(|route| RoutingResult { best_path: vec![route.input, route.output], expected_out: 1000 }).collect())
    }
}

/// 路由权限校验trait
///
/// 定义路由权限校验接口，便于扩展多种权限模型。
/// - 设计意图：统一权限校验入口，便于后续多种权限策略。
pub trait RouterAuthorizable {
    /// 校验路由操作权限
    ///
    /// # 参数
    /// - `authority`: 操作人。
    ///
    /// # 返回值
    /// - 是否有权限。
    fn authorize(&self, authority: Pubkey) -> Result<bool>;
}

/// 路由权限校验服务实现
///
/// 示例实现：校验权限。
pub struct AuthorizeRouterService;
impl RouterAuthorizable for AuthorizeRouterService {
    /// 权限校验实现
    fn authorize(&self, _authority: Pubkey) -> Result<bool> {
        // 生产级实现：校验权限
        Ok(true)
    }
} 