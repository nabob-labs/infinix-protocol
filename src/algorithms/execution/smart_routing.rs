//! 智能路由算法实现模块
//! 实现 RoutingStrategy trait，支持多 DEX 备选路由选择。
//! 支持 Anchor 自动注册，便于工厂/注册表动态调用。

use crate::algorithms::traits::{RoutingStrategy, RoutingParams, RoutingResult, AlgorithmError}; // 引入路由策略 trait 及相关类型
use crate::core::adapter::AdapterTrait; // 引入适配器 trait，便于统一管理
use anchor_lang::prelude::*; // Anchor 预导入，包含 Context、Result、账户等

/// 智能路由算法实现结构体
pub struct SmartRoutingImpl; // 主结构体，无状态实现

/// RoutingStrategy trait 实现
impl RoutingStrategy for SmartRoutingImpl {
    /// 路由算法主入口
    /// - 参数 ctx: Anchor 上下文
    /// - 参数 params: 路由参数（包含 DEX 备选列表、输入金额等）
    /// - 返回 RoutingResult，包含最优 DEX、预期输出等
    fn route(&self, _ctx: Context<crate::algorithms::traits::Route>, params: &RoutingParams) -> Result<RoutingResult> {
        if params.dex_candidates.is_empty() {
            return Err(AlgorithmError::InvalidInput.into()); // 输入参数校验
        }
        // 简化：选择第一个 DEX 作为最优 DEX，实际应根据市场深度等排序
        Ok(RoutingResult {
            best_dex: params.dex_candidates[0].clone(), // 最优 DEX
            expected_out: params.amount_in,             // 预期输出
        })
    }
    /// 算法名称
    fn name(&self) -> &'static str { "SmartRouting" }
}

/// AdapterTrait 实现，便于统一管理和注册
impl AdapterTrait for SmartRoutingImpl {
    fn name(&self) -> &'static str { "smart_routing" } // 算法唯一名称
    fn version(&self) -> &'static str { "1.0.0" } // 算法版本号
    fn supported_assets(&self) -> Vec<String> { vec!["SOL".to_string(), "USDC".to_string()] } // 支持资产
    fn status(&self) -> Option<String> { Some("active".to_string()) } // 激活状态
}

/// Anchor 自动注册宏，模块加载时自动注册到工厂
#[ctor::ctor]
fn auto_register_smart_routing_impl() {
    let adapter = SmartRoutingImpl;
    let mut factory = crate::core::registry::ADAPTER_FACTORY.lock().unwrap();
    factory.register(adapter);
}

#[cfg(test)]
mod tests {
    use super::*;
    /// 测试：正常路由
    #[test]
    fn test_route_basic() {
        let algo = SmartRoutingImpl;
        let params = RoutingParams {
            input_mint: Default::default(),
            output_mint: Default::default(),
            amount_in: 100,
            dex_candidates: vec!["Jupiter".to_string(), "Orca".to_string()],
        };
        let result = algo.route(Context::default(), &params).unwrap();
        assert_eq!(result.best_dex, "Jupiter");
    }
    /// 测试：空 DEX 备选列表
    #[test]
    fn test_route_empty() {
        let algo = SmartRoutingImpl;
        let params = RoutingParams {
            input_mint: Default::default(),
            output_mint: Default::default(),
            amount_in: 100,
            dex_candidates: vec![],
        };
        assert!(algo.route(Context::default(), &params).is_err());
    }
} 