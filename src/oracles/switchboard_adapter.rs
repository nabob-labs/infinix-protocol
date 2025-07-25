use super::{GetPrice, GetTwap, GetVwap, OracleAdapter};
use anchor_lang::prelude::*;

// ========================= Switchboard 预言机适配器桥接实现 =========================
// 本模块为 Switchboard 预言机提供 Anchor 兼容的桥接适配器，
// 每个 struct、trait、impl、方法、参数、用途、边界、Anchor 相关点、事件、注册、测试等均有详细注释。
/// Switchboard预言机适配器结构体
/// 用于对接Solana链上的Switchboard预言机，实现统一的Oracle适配接口
pub struct SwitchboardAdapter;

/// 实现OracleAdapter trait，集成Switchboard链上CPI调用（待补充）
impl OracleAdapter for SwitchboardAdapter {
    /// 获取现价（待集成CPI）
    /// 参数：
    ///   - ctx: Anchor上下文
    ///   - base_mint: 基础资产mint
    ///   - quote_mint: 报价资产mint
    /// 返回：现价（u64）
    fn get_price(
        &self,
        _ctx: Context<GetPrice>,
        _base_mint: Pubkey,
        _quote_mint: Pubkey,
    ) -> Result<u64> {
        // TODO: 集成 Switchboard CPI
        Ok(0)
    }
    /// 获取TWAP（待集成CPI）
    /// 参数：
    ///   - ctx: Anchor上下文
    ///   - base_mint: 基础资产mint
    ///   - quote_mint: 报价资产mint
    ///   - interval: 时间区间
    /// 返回：TWAP数值（u64）
    fn get_twap(
        &self,
        _ctx: Context<GetTwap>,
        _base_mint: Pubkey,
        _quote_mint: Pubkey,
        _interval: u64,
    ) -> Result<u64> {
        // TODO: 集成 Switchboard CPI
        Ok(0)
    }
    /// 获取VWAP（待集成CPI）
    /// 参数：
    ///   - ctx: Anchor上下文
    ///   - base_mint: 基础资产mint
    ///   - quote_mint: 报价资产mint
    ///   - interval: 时间区间
    /// 返回：VWAP数值（u64）
    fn get_vwap(
        &self,
        _ctx: Context<GetVwap>,
        _base_mint: Pubkey,
        _quote_mint: Pubkey,
        _interval: u64,
    ) -> Result<u64> {
        // TODO: 集成 Switchboard CPI
        Ok(0)
    }
}

impl SwitchboardAdapter {
    /// 手动注册SwitchboardAdapter到指定注册表
    /// 参数：registry 目标OracleAdapterRegistry
    pub fn register(registry: &mut crate::oracles::adapter_registry::OracleAdapterRegistry) {
        use std::sync::Arc;
        registry.register("Switchboard", Arc::new(SwitchboardAdapter));
    }
}
