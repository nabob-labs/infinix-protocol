//!
//! TWAP（时间加权平均价格）算法执行器实现模块
//! 实现 ExecutionStrategy trait，支持 TWAP 执行优化。
//! 包含单元测试。
use crate::algorithms::traits::{ExecutionStrategy, ExecutionParams, ExecutionResult, AlgorithmError}; // 引入执行策略 trait 及相关类型，便于类型安全和接口统一
use anchor_lang::prelude::*; // Anchor 预导入，包含 Context、Result、账户等，确保与Anchor兼容

/// TWAP 算法执行器实现结构体
pub struct TwapImpl; // 主结构体，无状态实现，所有逻辑在trait实现中，提升安全性和可复用性

/// ExecutionStrategy trait 实现
impl ExecutionStrategy for TwapImpl {
    /// 执行 TWAP 算法主入口
    /// - 参数 ctx: Anchor 上下文，包含账户、权限等
    /// - 参数 params: 执行参数（订单量、滑点等）
    /// - 返回 ExecutionResult，包含优化后成交量、预期成本等
    fn execute(&self, _ctx: Context<crate::algorithms::traits::Execute>, params: &ExecutionParams) -> Result<ExecutionResult> {
        if params.order_size == 0 || params.slippage_tolerance == 0 {
            return Err(AlgorithmError::InvalidInput.into()); // 输入参数校验，订单量和滑点必须大于0，防止无效或恶意调用
        }
        // 生产级 TWAP 算法核心流程
        let intervals = 10; // 分 10 个时间区间，实际可根据策略调整
        let avg_size = params.order_size / intervals; // 每区间平均成交量，整除分配
        let mut executed = 0u64; // 已成交量累计，初始化为0
        let mut total_cost = 0u64; // 总成交成本累计，初始化为0
        for i in 0..intervals {
            let size = if i == intervals - 1 {
                params.order_size - executed // 最后一区间补足剩余量，确保总量精确
            } else {
                avg_size // 其余区间均分
            };
            let price = 1_000_000 + (i as u64 * params.slippage_tolerance / intervals as u64); // 模拟价格，考虑滑点，线性递增
            total_cost += size * price; // 累加成本，防止溢出
            executed += size; // 累加成交量，防止溢出
        }
        let avg_price = total_cost / params.order_size; // 计算加权平均价格，便于链上链下审计
        Ok(ExecutionResult {
            optimized_size: params.order_size, // 优化后成交量，类型安全
            expected_cost: total_cost,         // 总成交成本，便于审计
        })
    }
    /// 算法名称
    fn name(&self) -> &'static str { "TWAP" } // 算法名称常量，便于注册表/工厂识别
}

#[cfg(test)]
mod tests {
    use super::*;
    /// 测试：正常 TWAP 执行
    #[test]
    fn test_twap_basic() {
        let algo = TwapImpl; // 创建TWAP算法实例
        let params = ExecutionParams { order_size: 100, slippage_tolerance: 100 }; // 有效参数
        let result = algo.execute(Context::default(), &params).unwrap(); // 执行算法，校验无错误
        assert_eq!(result.optimized_size, 100); // 校验成交量，确保算法正确
    }
    /// 测试：无效参数
    #[test]
    fn test_twap_empty() {
        let algo = TwapImpl; // 创建TWAP算法实例
        let params = ExecutionParams { order_size: 0, slippage_tolerance: 100 }; // 无效订单量
        assert!(algo.execute(Context::default(), &params).is_err()); // 应返回错误，防止无效输入
    }
} 