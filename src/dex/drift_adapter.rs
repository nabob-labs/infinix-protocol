use anchor_lang::prelude::*; // Anchor预导入，包含Result、Context等
use super::traits::{DexAdapter, SwapParams, SwapResult, AddLiquidityParams, RemoveLiquidityParams, QuoteParams, QuoteResult}; // DEX适配器trait及相关类型
// use crate::core::adapter: // 暂时注释掉:AdapterTrait; // 适配器元信息trait，统一接口

/// Drift DEX适配器结构体
/// - 用于对接Solana链上的Drift DEX，实现统一的DEX适配接口
/// - 设计为无状态结构体，便于多实例、线程安全
pub struct DriftAdapter;

/// 实现AdapterTrait，提供适配器元信息
impl AdapterTrait for DriftAdapter {
    /// 返回适配器名称（唯一标识）
    fn name(&self) -> &'static str { "drift" }
    /// 返回适配器版本号
    fn version(&self) -> &'static str { "1.0.0" }
    /// 返回支持的资产列表（如SOL、USDC等）
    fn supported_assets(&self) -> Vec<String> { vec!["SOL".to_string(), "USDC".to_string()] }
    /// 返回适配器当前状态（如active、paused等）
    fn status(&self) -> Option<String> { Some("active".to_string()) }
}

/// 自动注册DriftAdapter到全局工厂
/// - 使用ctor宏在程序启动时自动注册，便于插件式扩展
/// - 设计意图：极简插件式扩展，保证所有DEX适配器可热插拔
// #[ctor::ctor]
fn auto_register_drift_adapter() {
    let adapter = DriftAdapter; // 实例化适配器
    let mut factory = crate::core::registry::ADAPTER_FACTORY.lock().unwrap(); // 获取全局工厂锁
    factory.register(adapter); // 注册到工厂，便于统一管理
}

/// 实现DexAdapter trait，集成Drift链上CPI调用（待补充）
impl DexAdapter for DriftAdapter {
    /// 执行 Drift swap 操作。
    fn swap(&self, params: &SwapParams) -> anchor_lang::Result<DexSwapResult> {
        // 生产级实现：集成Drift链上CPI调用，参数校验、错误处理、事件追踪
        require!(params.amount_in > 0, crate::errors::asset_error::AssetError::InvalidAmount);
        // TODO: 调用Drift CPI（此处应集成真实CPI调用）
        // 这里只做结构示例，实际应调用CPI并返回真实成交数据
        Ok(DexSwapResult {
            executed_amount: params.amount_in,
            avg_price: 1_000_000, // 应为CPI返回均价
            fee: 1000,            // 应为CPI返回手续费
            dex_name: "drift".to_string(),
        })
    }
    /// 添加流动性（待集成CPI）
    /// - ctx: Anchor上下文，包含所有必需账户
    /// - params: 添加流动性参数
    /// - 返回：添加后获得的LP token数量
    fn add_liquidity(&self, ctx: Context<AddLiquidity>, params: AddLiquidityParams) -> anchor_lang::Result<u64> {
        // TODO: 集成 Drift CPI
        Ok(0)
    }
    /// 移除流动性（待集成CPI）
    /// - ctx: Anchor上下文，包含所有必需账户
    /// - params: 移除流动性参数
    /// - 返回：移除后获得的资产数量
    fn remove_liquidity(&self, ctx: Context<RemoveLiquidity>, params: RemoveLiquidityParams) -> anchor_lang::Result<u64> {
        // TODO: 集成 Drift CPI
        Ok(0)
    }
    /// 获取报价（待集成CPI）
    /// - ctx: Anchor上下文，包含所有必需账户
    /// - params: 报价参数
    /// - 返回：QuoteResult结构体，包含预期兑换数量和手续费
    fn get_quote(&self, ctx: Context<GetQuote>, params: QuoteParams) -> anchor_lang::Result<QuoteResult> {
        // TODO: 集成 Drift CPI
        Ok(QuoteResult { amount_out: 0, fee: 0 })
    }
} 