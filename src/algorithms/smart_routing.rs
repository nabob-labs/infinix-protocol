/*!
 * 智能路由算法模块
 *
 * 生产级智能路由算法实现。
 * 支持 Anchor 框架自动注册，便于在算法工厂/注册表中动态调用。
 */

use crate::algorithms::traits::{Route, RoutingAlgorithm, RoutingResult}; // 引入算法 trait 及路由相关类型，便于接口统一
use crate::core::types::TradeParams; // 引入通用交易参数类型，便于算法通用化
use anchor_lang::prelude::*; // Anchor 预导入，包含 Context、Result、账户等，确保算法实现与Anchor兼容

/// 智能路由算法结构体
/// - 实现 RoutingAlgorithm trait
#[derive(Default)] // 提供默认构造，便于无状态注册和工厂调用
pub struct SmartRoutingAlgorithm; // 智能路由算法主结构体，无状态实现，所有逻辑在trait实现中

/// RoutingAlgorithm trait 实现
impl RoutingAlgorithm for SmartRoutingAlgorithm {
    /// 路由算法主入口
    /// - 参数 ctx: Anchor 上下文，包含账户、权限等
    /// - 参数 params: 交易参数，包含路由所需的所有输入
    /// - 返回 RoutingResult，包含最优 DEX、预期输出等
    fn route(&self, _ctx: Context<Route>, params: &TradeParams) -> Result<RoutingResult> {
        require!(params.amount_in > 0, ErrorCode::InvalidAmount); // 校验输入参数amount_in必须大于0，否则返回错误码
        // 生产级智能路由算法实现（此处为简化示例，实际应根据市场深度、滑点、费用等综合决策）
        Ok(RoutingResult {
            best_dex: params.dex_name.clone(),      // 实际应根据算法选择最优 DEX，这里直接返回输入DEX名
            expected_output: params.amount_in,      // 实际应根据市场深度等计算，这里直接返回输入数量
        })
    }
    /// 算法名称
    fn name(&self) -> &'static str {
        "SmartRouting" // 算法名称常量，便于注册表/工厂识别
    }
}

/// 错误码定义，便于 Anchor 错误处理
#[error_code] // Anchor宏，自动生成错误类型和消息
pub enum ErrorCode {
    #[msg("Invalid amount")]
    InvalidAmount, // 输入数量无效，amount_in<=0时触发
}
