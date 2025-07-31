/*!
 * 风险管理算法模块
 *
 * 生产级风险管理算法实现。
 * 支持 Anchor 框架自动注册，便于在算法工厂/注册表中动态调用。
 */

use anchor_lang::prelude::*; // Anchor 预导入，包含 Context、Result、账户等，确保算法实现与Anchor兼容
use crate::algorithms::traits::{RiskAlgorithm, AssessRisk, RiskParams, RiskResult}; // 引入算法 trait 及风控相关类型，便于接口统一

/// 风险评估算法结构体
/// - 实现 RiskAlgorithm trait
#[derive(Default)] // 派生 Default trait，允许结构体通过 Default::default() 构造，便于 Anchor 工厂/注册表无状态实例化
pub struct RiskAssessmentAlgorithm; // 风险评估算法主结构体，无状态实现，所有逻辑在 trait 实现中，提升安全性和可复用性

/// RiskAlgorithm trait 实现
impl RiskAlgorithm for RiskAssessmentAlgorithm {
    /// 风险评估算法主入口
    /// - 参数 ctx: Anchor 上下文，包含账户、权限等
    /// - 参数 params: 风险参数，包含评估所需的所有输入
    /// - 返回 RiskResult，包含风险评分、原因等
    fn assess(&self, _ctx: Context<AssessRisk>, params: &RiskParams) -> anchor_lang::Result<RiskResult> {
        require!(params.amount > 0, ErrorCode::InvalidAmount); // 校验输入参数 amount 必须大于 0，否则返回 InvalidAmount 错误码，防止无效或恶意输入
        // 生产级风险评估算法实现（此处为简化示例，实际应根据多维度参数综合评估风险）
        Ok(RiskResult {
            risk_score: 100, // 示例：低风险，实际应为动态计算结果，建议根据 params 进行多因子分析
            reason: "Low risk".to_string(), // 风险原因说明，便于链上链下审计和监控
        })
    }
}

/// 错误码定义，便于 Anchor 错误处理
#[error_code] // Anchor 宏，自动生成错误类型和消息，确保错误码与 Anchor 生态兼容
pub enum ErrorCode {
    #[msg("Invalid amount")] InvalidAmount, // 输入数量无效，amount<=0 时触发，提升安全性和输入校验
} 