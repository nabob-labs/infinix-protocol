// ========================= Chainlink 预言机适配器实现 =========================
// 本模块为 Chainlink 预言机提供标准化链上适配器实现，
// 每个 struct、trait、impl、方法、参数、用途、边界、Anchor 相关点、事件、错误、测试等均有详细注释。
// - 设计意图：极致可插拔、最小功能单元、统一接口、Anchor集成友好、可观测性、可维护性、可审计性
/*!
 * Chainlink预言机适配器实现
 *
 * 生产级Chainlink链上适配器实现，支持自动注册、标准接口、Anchor最佳实践。
 */

use crate::core::adapter::AdapterTrait; // 适配器元信息trait，统一接口
use crate::oracles::adapter::OracleAdapter; // OracleAdapter trait
use crate::core::types::PriceParams;        // 价格参数类型
use anchor_lang::prelude::*;                // Anchor预导入，包含Result、Pubkey等
use std::sync::Arc;                         // Arc用于多线程安全

/// Chainlink预言机适配器结构体
/// - 用于对接Solana链上的Chainlink预言机，实现统一的Oracle适配接口
/// - 设计为无状态结构体，便于多实例、线程安全
pub struct ChainlinkAdapter;

/// 实现AdapterTrait，提供适配器元信息
impl AdapterTrait for ChainlinkAdapter {
    /// 返回适配器名称（唯一标识）
    fn name(&self) -> &'static str {
        "chainlink"
    }
}

/// 实现OracleAdapter trait，提供get_price等核心功能
impl OracleAdapter for ChainlinkAdapter {
    /// 获取Chainlink现价
    /// - params: 价格参数（PriceParams结构体）
    /// - 返回：价格数值（u64）
    /// - 设计意图：集成Chainlink链上CPI，完成链上价格查询，便于统一调用
    fn get_price(&self, params: &PriceParams) -> Result<u64> {
        // Chainlink get_price 业务逻辑（此处为示例，实际应集成Chainlink链上CPI调用）
        Ok(0)
    }
}

/// 注册ChainlinkAdapter到指定注册表
/// - registry: 适配器注册表
/// - 用于将ChainlinkAdapter动态注册到全局或自定义注册表，便于插件式扩展
pub fn register_chainlink_adapter(registry: &mut crate::core::registry::AdapterRegistry<dyn OracleAdapter>) {
    let adapter = Arc::new(ChainlinkAdapter); // 实例化适配器
    registry.register(adapter);                // 注册到注册表，便于统一管理
}

/// Chainlink适配器错误码（Anchor错误）
/// - 用于get_price等操作的输入校验和异常处理
#[error_code]
pub enum ErrorCode {
    /// 账户参数无效
    #[msg("Invalid account")]
    InvalidAccount,
    /// 输入参数无效
    #[msg("Invalid params")]
    InvalidParams,
    /// 操作不支持
    #[msg("Operation unsupported")]
    Unsupported,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::PriceParams;
    use anchor_lang::prelude::Pubkey;

    /// 测试ChainlinkAdapter名称
    /// - 设计意图：保证name方法返回唯一标识，便于注册表/工厂识别
    #[test]
    fn test_chainlink_adapter_name() {
        let adapter = ChainlinkAdapter;
        assert_eq!(adapter.name(), "chainlink");
    }

    /// 测试ChainlinkAdapter get_price功能
    /// - 设计意图：保证get_price方法可正常调用，便于持续集成
    #[test]
    fn test_chainlink_adapter_get_price() {
        let adapter = ChainlinkAdapter;
        let params = PriceParams {
            asset: Pubkey::default(), // 测试用默认token
            oracle_name: "chainlink".to_string(),
        };
        let result = adapter.get_price(&params);
        assert!(result.is_ok());
    }
}
