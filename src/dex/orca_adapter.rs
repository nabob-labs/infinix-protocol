//!
//! Orca DEX Adapter Bridge Module
//!
//! 本模块为 Orca DEX 提供 Anchor 兼容的桥接适配器，实现统一接口、自动注册、CPI集成（预留），确保可插拔、合规、可维护。

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, DexAdapterTrait};
use crate::dex::traits::*;

/// OrcaAdapter结构体，代表Orca DEX/AMM适配器
pub struct OrcaAdapter {
    /// Orca支持的资产类型集合
    supported: Vec<String>,
}

impl OrcaAdapter {
    /// 构造函数，初始化OrcaAdapter，注册支持的资产类型
    pub fn new() -> Self {
        Self {
            supported: vec![
                "Crypto".to_string(),
                "Stablecoin".to_string(),
            ],
        }
    }
}

impl DexAdapter for OrcaAdapter {
    /// 执行swap（mock实现，实际应调用Orca CPI）
    fn swap(&self, _ctx: Context<Swap>, params: SwapParams) -> Result<SwapResult> {
        let amount_out = params.amount_in; // 假定1:1兑换
        Ok(SwapResult { amount_out, fee: 0 })
    }
    /// 添加流动性（mock实现）
    fn add_liquidity(&self, _ctx: Context<AddLiquidity>, params: AddLiquidityParams) -> Result<u64> {
        Ok(params.amount)
    }
    /// 移除流动性（mock实现）
    fn remove_liquidity(&self, _ctx: Context<RemoveLiquidity>, params: RemoveLiquidityParams) -> Result<u64> {
        Ok(params.amount)
    }
    /// 获取报价（mock实现）
    fn get_quote(&self, _ctx: Context<GetQuote>, params: QuoteParams) -> Result<QuoteResult> {
        Ok(QuoteResult { amount_out: params.amount_in, fee: 0 })
    }
    /// 查询支持的资产类型
    fn supported_assets(&self) -> Vec<String> {
        self.supported.clone()
    }
    /// 查询支持的市场类型
    fn supported_markets(&self) -> Vec<String> {
        vec!["Spot".to_string(), "AMM".to_string()]
    }
    /// DEX适配器类型
    fn adapter_type(&self) -> DexAdapterType {
        DexAdapterType::AMM
    }
}

/// 错误码定义，便于合规和可维护性
#[error_code]
pub enum ErrorCode {
    #[msg("不支持的资产类型")] 
    UnsupportedAsset,
} 