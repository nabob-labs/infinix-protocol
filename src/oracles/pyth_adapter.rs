use super::traits::{
    OracleAdapter, PriceParams, PriceResult, TwapParams, TwapResult, VwapParams, VwapResult,
};
use super::{GetPrice, GetTwap, GetVwap};
use anchor_lang::prelude::*;
use crate::core::adapter::AdapterTrait;

// ========================= Pyth 预言机适配器桥接实现 =========================
// 本模块为 Pyth 预言机提供 Anchor 兼容的桥接适配器，
// 每个 struct、trait、impl、方法、参数、用途、边界、Anchor 相关点、事件、注册、测试等均有详细注释。
/// Pyth预言机适配器结构体
/// 用于对接Solana链上的Pyth预言机，实现统一的Oracle适配接口
pub struct PythAdapter;

/// 实现AdapterTrait，提供适配器元信息
impl AdapterTrait for PythAdapter {
    /// 返回适配器名称
    fn name(&self) -> &'static str { "pyth" }
    /// 返回适配器版本号
    fn version(&self) -> &'static str { "1.0.0" }
    /// 返回支持的资产列表
    fn supported_assets(&self) -> Vec<String> { vec!["SOL".to_string(), "USDC".to_string()] }
    /// 返回适配器当前状态
    fn status(&self) -> Option<String> { Some("active".to_string()) }
}

/// 实现OracleAdapter trait，集成Pyth链上CPI调用（待补充）
impl OracleAdapter for PythAdapter {
    /// 获取现价（待集成CPI）
    /// 参数：
    ///   - ctx: Anchor上下文
    ///   - params: 价格参数
    /// 返回：PriceResult结构体，包含价格和更新时间
    fn get_price(&self, ctx: Context<GetPrice>, params: PriceParams) -> Result<PriceResult> {
        // TODO: 集成 Pyth CPI
        Ok(PriceResult {
            price: 0,
            last_updated: 0,
        })
    }
    /// 获取TWAP（待集成CPI）
    /// 参数：
    ///   - ctx: Anchor上下文
    ///   - params: TWAP参数
    /// 返回：TwapResult结构体，包含TWAP和更新时间
    fn get_twap(&self, ctx: Context<GetTwap>, params: TwapParams) -> Result<TwapResult> {
        // TODO: 集成 Pyth CPI
        Ok(TwapResult {
            twap: 0,
            last_updated: 0,
        })
    }
    /// 获取VWAP（待集成CPI）
    /// 参数：
    ///   - ctx: Anchor上下文
    ///   - params: VWAP参数
    /// 返回：VwapResult结构体，包含VWAP和更新时间
    fn get_vwap(&self, ctx: Context<GetVwap>, params: VwapParams) -> Result<VwapResult> {
        // TODO: 集成 Pyth CPI
        Ok(VwapResult {
            vwap: 0,
            last_updated: 0,
        })
    }
}

impl PythAdapter {
    /// 手动注册PythAdapter到指定注册表
    /// 参数：registry 目标OracleAdapterRegistry
    pub fn register(registry: &mut crate::oracles::adapter_registry::OracleAdapterRegistry) {
        use std::sync::Arc;
        registry.register("Pyth", Arc::new(PythAdapter));
    }
}

/// 自动注册PythAdapter到全局工厂
/// - 使用ctor宏在程序启动时自动注册，便于插件式扩展
#[ctor::ctor]
fn auto_register_pyth_adapter() {
    let adapter = PythAdapter;
    let mut factory = crate::core::registry::ADAPTER_FACTORY.lock().unwrap();
    factory.register(adapter);
}
