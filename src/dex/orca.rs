//!
//! Orca DEX Adapter Module
//!
//! 本模块实现 Orca DEX 适配器，提供与 Orca AMM 的链上集成接口，确保交易路由与聚合合规、可维护。

use crate::core::adapter::AdapterTrait;
use crate::core::types::{TradeParams, BatchTradeParams, DexParams};
use crate::dex::adapter::{DexAdapter, DexSwapResult};
use anchor_lang::prelude::*;

/// Orca DEX 适配器结构体。
pub struct OrcaAdapter;

impl AdapterTrait for OrcaAdapter {
    fn name(&self) -> &str { "orca" }
    fn version(&self) -> &str { "1.0.0" }
    fn is_available(&self) -> bool { true }
    fn initialize(&mut self) -> anchor_lang::Result<()> { Ok(()) }
    fn cleanup(&mut self) -> anchor_lang::Result<()> { Ok(()) }
}

impl DexAdapter for OrcaAdapter {
    fn swap(&self, params: &TradeParams) -> anchor_lang::Result<DexSwapResult> {
        // TODO: 实现实际的 swap 逻辑
        Ok(DexSwapResult {
            executed_amount: params.amount_in,
            avg_price: 1_000_000,
            fee: 1000,
            dex_name: "orca".to_string(),
        })
    }
    
    fn batch_swap(&self, params: &BatchTradeParams) -> anchor_lang::Result<Vec<DexSwapResult>> {
        // TODO: 实现实际的批量 swap 逻辑
        Ok(vec![])
    }
    
    fn configure(&self, params: &DexParams) -> anchor_lang::Result<()> {
        // TODO: 实现配置逻辑
        Ok(())
    }
    
    fn supported_assets(&self) -> Vec<String> {
        vec!["SOL".to_string(), "USDC".to_string()]
    }
    
    fn supported_markets(&self) -> Vec<String> {
        vec!["spot".to_string()]
    }
    
    fn adapter_type(&self) -> crate::dex::adapter::DexAdapterType {
        crate::dex::adapter::DexAdapterType::AMM
    }
}

/// Orca DEX CPI账户结构声明
#[derive(Accounts)]
pub struct OrcaSwap<'info> {
    /// Orca程序
    pub orca_program: AccountInfo<'info>,
    /// Whirlpool账户
    pub whirlpool_account: AccountInfo<'info>,
    /// 输入代币账户
    pub input_token_account: AccountInfo<'info>,
    /// 输出代币账户
    pub output_token_account: AccountInfo<'info>,
    /// 用户账户
    pub user: AccountInfo<'info>,
}

/// Orca DEX错误码（Anchor错误）
/// - 用于swap等操作的输入校验和异常处理
#[error_code]
pub enum OrcaError {
    /// 金额无效
    #[msg("Invalid amount")] InvalidAmount,
    /// 代币无效
    #[msg("Invalid tokens")] InvalidTokens,
    /// 滑点过大
    #[msg("Slippage too high")] SlippageTooHigh,
    /// 流动性不足
    #[msg("Insufficient liquidity")] InsufficientLiquidity,
    /// Whirlpool账户无效
    #[msg("Invalid Whirlpool account")] InvalidWhirlpoolAccount,
    /// 操作不支持
    #[msg("Operation unsupported")] Unsupported,
}

/// 自动注册 OrcaAdapter 到工厂
// #[ctor::ctor]
fn register_orca_adapter() {
    // crate::dex::factory::DEX_FACTORY.register("orca", std::sync::Arc::new(OrcaAdapter));
}

#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::prelude::Pubkey;

    /// 测试 OrcaAdapter 名称。
    #[test]
    fn test_orca_adapter_name() {
        let adapter = OrcaAdapter;
        assert_eq!(adapter.name(), "orca");
    }

    /// 测试 OrcaAdapter swap 功能。
    #[test]
    fn test_orca_adapter_swap() {
        let adapter = OrcaAdapter;
        let params = TradeParams {
            from_token: Pubkey::default(),
            to_token: Pubkey::new_unique(), // 使用不同的token
            amount_in: 100,
            min_amount_out: 90,
            dex_name: "orca".to_string(),
        };
        let result = adapter.swap(&params);
        assert!(result.is_ok());
    }
    
    /// 测试 OrcaAdapter 支持的资产。
    #[test]
    fn test_orca_supported_assets() {
        let adapter = OrcaAdapter;
        let assets = adapter.supported_assets();
        assert!(assets.contains(&"SOL".to_string()));
        assert!(assets.contains(&"USDC".to_string()));
        assert!(assets.contains(&"ORCA".to_string()));
    }
} 