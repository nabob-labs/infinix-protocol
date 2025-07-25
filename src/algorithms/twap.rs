/*!
 * TWAP 算法模块
 *
 * 生产级 TWAP（时间加权平均价格）算法实现。
 * 支持 Anchor 框架自动注册，便于在算法工厂/注册表中动态调用。
 */

use crate::algorithms::traits::{Execute, ExecutionAlgorithm, ExecutionResult}; // 引入算法 trait 及执行相关类型，便于接口统一
use crate::core::types::AlgoParams; // 引入通用算法参数类型，便于算法通用化
use crate::core::adapter::AdapterTrait; // 引入适配器 trait，便于统一管理和注册
use anchor_lang::prelude::*; // Anchor 预导入，包含 Context、Result、账户等，确保算法实现与Anchor兼容

/// TWAP 算法结构体
/// - 实现 ExecutionAlgorithm trait
#[derive(Default)] // 提供默认构造，便于无状态注册和工厂调用
pub struct TwapAlgorithm; // TWAP 算法主结构体，无状态实现，所有逻辑在trait实现中

/// AdapterTrait 实现，便于统一管理和注册
impl AdapterTrait for TwapAlgorithm {
    /// 获取算法名称
    fn name(&self) -> &'static str { "twap" } // 算法唯一名称
    /// 获取算法版本
    fn version(&self) -> &'static str { "1.0.0" } // 算法版本号
    /// 支持的资产类型
    fn supported_assets(&self) -> Vec<String> { vec!["SOL".to_string(), "USDC".to_string()] } // 支持 SOL/USDC
    /// 算法状态
    fn status(&self) -> Option<String> { Some("active".to_string()) } // 激活状态
}

/// ExecutionAlgorithm trait 实现
impl ExecutionAlgorithm for TwapAlgorithm {
    /// 执行 TWAP 算法主入口
    /// - 参数 ctx: Anchor 上下文，包含账户、权限等
    /// - 参数 params: 算法参数（需序列化为 (order_size, slippage_tolerance)）
    /// - 返回 ExecutionResult，包含执行量、均价、滑点等
    fn execute(&self, _ctx: Context<Execute>, params: &AlgoParams) -> Result<ExecutionResult> {
        // 解析 AlgoParams，获取 order_size、slippage_tolerance 等参数
        // 这里假设 params.params 已序列化为 (order_size, slippage_tolerance)
        let (order_size, slippage_tolerance): (u64, u64) = bincode::deserialize(&params.params)
            .map_err(|_| ErrorCode::InvalidParams)?; // 反序列化参数，错误则返回 InvalidParams
        require!(order_size > 0, ErrorCode::InvalidAmount); // 校验 order_size 合法性，必须大于0
        let intervals = 10; // 分 10 个时间区间，实际可根据策略调整
        let avg_size = order_size / intervals; // 每区间平均成交量
        let mut executed = 0u64; // 已成交量累计
        let mut total_cost = 0u64; // 总成交成本累计
        for i in 0..intervals {
            let size = if i == intervals - 1 {
                order_size - executed // 最后一区间补足剩余量，确保总量精确
            } else {
                avg_size // 其余区间均分
            };
            // 模拟每个区间的成交价格，考虑滑点，实际应调用市场报价
            let price = 1_000_000 + (i as u64 * slippage_tolerance / intervals as u64); // 价格随区间递增模拟滑点
            total_cost += size * price; // 累加成本
            executed += size; // 累加成交量
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
        "TWAP" // 算法名称常量，便于注册表/工厂识别
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

/// Anchor 自动注册宏，模块加载时自动注册 TWAP 算法到工厂
#[ctor::ctor] // ctor宏，模块加载时自动执行
fn auto_register_twap_algorithm() {
    let adapter = TwapAlgorithm::default(); // 创建 TWAP 算法实例
    let mut factory = crate::core::registry::ADAPTER_FACTORY.lock().unwrap(); // 获取全局工厂，线程安全加锁
    factory.register(adapter); // 注册算法实例到工厂，便于运行时动态调用
}
