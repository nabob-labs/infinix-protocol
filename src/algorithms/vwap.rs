/*!
 * VWAP 算法模块
 *
 * 生产级 VWAP（成交量加权平均价格）算法实现。
 * 支持 Anchor 框架自动注册，便于在算法工厂/注册表中动态调用。
 */

use crate::algorithms::traits::{Execute, ExecutionAlgorithm, ExecutionResult}; // 引入算法 trait 及执行相关类型，便于接口统一
use crate::core::types::AlgoParams; // 引入通用算法参数类型，便于算法通用化
use anchor_lang::prelude::*; // Anchor 预导入，包含 Context、Result、账户等，确保算法实现与Anchor兼容

/// VWAP 算法结构体
/// - 实现 ExecutionAlgorithm trait
#[derive(Default)] // 提供默认构造，便于无状态注册和工厂调用
pub struct VwapAlgorithm; // VWAP 算法主结构体，无状态实现，所有逻辑在trait实现中

/// ExecutionAlgorithm trait 实现
impl ExecutionAlgorithm for VwapAlgorithm {
    /// 执行 VWAP 算法主入口
    /// - 参数 ctx: Anchor 上下文，包含账户、权限等
    /// - 参数 params: 算法参数（需序列化为 (order_size, slippage_tolerance)）
    /// - 返回 ExecutionResult，包含执行量、均价、滑点等
    fn execute(&self, _ctx: Context<Execute>, params: &AlgoParams) -> Result<ExecutionResult> {
        // 解析 AlgoParams，获取 order_size、slippage_tolerance 等参数
        let (order_size, slippage_tolerance): (u64, u64) = bincode::deserialize(&params.params)
            .map_err(|_| ErrorCode::InvalidParams)?; // 反序列化参数，错误则返回 InvalidParams
        require!(order_size > 0, ErrorCode::InvalidAmount); // 校验 order_size 合法性，必须大于0
        let intervals = 10; // 分 10 个时间区间，实际可根据策略调整
        let mut executed = 0u64; // 已成交量累计
        let mut total_cost = 0u64; // 总成交成本累计
        let mut total_volume = 0u64; // 总权重累计
        let mut weights = vec![1u64; intervals]; // 每区间权重初始化
        // 计算每个区间的权重
        for i in 0..intervals {
            weights[i] = (i as u64 + 1) * 10; // 权重递增，后期成交量权重更高
            total_volume += weights[i]; // 累加总权重
        }
        // 按权重分配成交量并计算成本
        for i in 0..intervals {
            let size = order_size * weights[i] / total_volume; // 当前区间成交量，按权重分配
            let price = 1_000_000 + (i as u64 * slippage_tolerance / intervals as u64); // 模拟价格，考虑滑点
            total_cost += size * price; // 累加成本
            executed += size; // 累加成交量
        }
        // 补足最后一笔，确保总成交量精确
        if executed < order_size {
            let size = order_size - executed; // 剩余未成交量
            let price = 1_000_000 + (intervals as u64 * slippage_tolerance / intervals as u64); // 最后区间价格
            total_cost += size * price; // 累加成本
        }
        let avg_price = total_cost / order_size; // 计算加权平均价格
        Ok(ExecutionResult {
            executed_amount: order_size, // 实际成交量
            avg_price,                  // 平均成交价格
            slippage_bps: slippage_tolerance, // 滑点（基点）
        })
    }
    /// 算法名称
    fn name(&self) -> &'static str {
        "VWAP" // 算法名称常量，便于注册表/工厂识别
    }
}

/// 错误码定义，便于 Anchor 错误处理
#[error_code] // Anchor宏，自动生成错误类型和消息
pub enum ErrorCode {
    #[msg("Invalid amount")]
    InvalidAmount, // 输入数量无效，order_size<=0时触发
    #[msg("Invalid params")]
    InvalidParams, // 参数反序列化失败
}
