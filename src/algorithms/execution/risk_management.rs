//! 风险管理算法实现模块
//! 实现 RiskManagement trait，支持多参数风险评估。
//! 支持 Anchor 自动注册，便于工厂/注册表动态调用。
use crate::algorithms::traits::{RiskManagement, RiskParams, RiskResult, AlgorithmError}; // 引入风控策略 trait 及相关类型，便于类型安全和接口统一
use crate::core::adapter::AdapterTrait; // 引入适配器 trait，便于统一管理和注册
use anchor_lang::prelude::*; // Anchor 预导入，包含 Context、Result、账户等，确保与Anchor兼容

/// 风险管理算法实现结构体
pub struct RiskManagementImpl; // 主结构体，无状态实现，所有逻辑在trait实现中，提升安全性和可复用性

/// AdapterTrait 实现，便于统一管理和注册
impl AdapterTrait for RiskManagementImpl {
    fn name(&self) -> &'static str { "risk_management" } // 算法唯一名称，便于注册表/工厂识别
    fn version(&self) -> &'static str { "1.0.0" } // 算法版本号，便于升级和兼容性管理
    fn supported_assets(&self) -> Vec<String> { vec!["SOL".to_string(), "USDC".to_string()] } // 支持资产类型，便于资产适配
    fn status(&self) -> Option<String> { Some("active".to_string()) } // 激活状态，便于运维监控
}

/// Anchor 自动注册宏，模块加载时自动注册到工厂
#[ctor::ctor]
fn auto_register_risk_management_impl() {
    let adapter = RiskManagementImpl; // 实例化无状态适配器
    let mut factory = crate::core::registry::ADAPTER_FACTORY.lock().unwrap(); // 获取全局工厂互斥锁，保证线程安全
    factory.register(adapter); // 注册适配器到工厂，支持热插拔和动态扩展
}

/// RiskManagement trait 实现
impl RiskManagement for RiskManagementImpl {
    /// 风控算法主入口
    /// - 参数 ctx: Anchor 上下文
    /// - 参数 params: 风控参数（如持仓、波动率、最大回撤等）
    /// - 返回 RiskResult，包含风险评分、是否可接受等
    fn assess(&self, _ctx: Context<crate::algorithms::traits::AssessRisk>, params: &RiskParams) -> anchor_lang::Result<RiskResult> {
        if params.position_size == 0 || params.volatility == 0 {
            return Err(AlgorithmError::InvalidInput.into()); // 输入参数校验，持仓和波动率必须大于0，防止无效或恶意调用
        }
        // 简化：风险评分 = 持仓*波动率/(最大回撤+1)
        let risk_score = ((params.position_size as u128 * params.volatility as u128) / (params.max_drawdown as u128 + 1)) as u8; // 类型安全，防止溢出
        Ok(RiskResult {
            risk_score, // 风险评分，类型安全
            is_acceptable: risk_score < 100, // 是否可接受，便于链上链下审计
        })
    }
    /// 算法名称
    fn name(&self) -> &'static str { "RiskManagement" } // 算法名称常量，便于注册表/工厂识别
}

#[cfg(test)]
mod tests {
    use super::*;
    /// 测试：正常风险评估
    #[test]
    fn test_risk_basic() {
        let algo = RiskManagementImpl; // 创建风险管理算法实例
        let params = RiskParams { position_size: 100, volatility: 10, max_drawdown: 5 }; // 有效参数
        let result = algo.assess_risk(params).unwrap(); // 执行算法，校验无错误
        assert!(result.is_acceptable); // 校验风险可接受性，确保算法正确
    }
    /// 测试：无效参数
    #[test]
    fn test_risk_invalid() {
        let algo = RiskManagementImpl; // 创建风险管理算法实例
        let params = RiskParams { position_size: 0, volatility: 0, max_drawdown: 0 }; // 无效参数
        assert!(algo.assess_risk(params).is_err()); // 应返回错误，防止无效输入
    }
} 