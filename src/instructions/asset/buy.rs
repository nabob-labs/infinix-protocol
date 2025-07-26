//!
//! Asset Buy Instruction
//! 资产购买指令实现，支持多种购买策略和DEX集成。

use crate::accounts::BasketIndexStateAccount;
use crate::events::asset_event::*;
use crate::services::asset_service::AssetService;
use crate::state::baskets::BasketIndexState;
use crate::validation::asset_validation::AssetValidatable;
use crate::core::types::{ExecutionParams, StrategyParams, TradeParams};
use crate::dex::adapter::DexAdapter;
use crate::algorithms::traits::AlgorithmAdapter;
use anchor_lang::prelude::*;

/// 资产购买指令账户上下文
/// - basket_index: 目标资产篮子账户，需可变
/// - user: 用户签名者
/// - dex_program: DEX程序账户
/// - input_token_account: 输入代币账户
/// - output_token_account: 输出代币账户
#[derive(Accounts)]
pub struct BuyAsset<'info> {
    /// 目标资产篮子账户，需可变
    #[account(mut)]
    pub basket_index: Account<'info, BasketIndexState>,
    
    /// 用户签名者
    #[account(mut)]
    pub user: Signer<'info>,
    
    /// DEX程序账户
    pub dex_program: AccountInfo<'info>,
    
    /// 输入代币账户（用户支付）
    #[account(mut)]
    pub input_token_account: Account<'info, spl_token::state::Account>,
    
    /// 输出代币账户（用户接收）
    #[account(mut)]
    pub output_token_account: Account<'info, spl_token::state::Account>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, spl_token::Token>,
}

/// 资产购买参数结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BuyAssetParams {
    /// 购买数量
    pub amount: u64,
    /// 最大支付金额
    pub max_payment: u64,
    /// 滑点容忍度（基点）
    pub slippage_tolerance: u16,
    /// 使用的DEX名称
    pub dex_name: String,
    /// 执行策略
    pub strategy: Option<StrategyParams>,
    /// 算法参数
    pub algorithm: Option<ExecutionParams>,
}

/// 资产购买指令实现
/// - ctx: Anchor账户上下文
/// - params: 购买参数
/// - 返回: Anchor规范Result
pub fn buy_asset(ctx: Context<BuyAsset>, params: BuyAssetParams) -> Result<()> {
    // 参数验证
    require!(params.amount > 0, crate::errors::asset_error::AssetError::InvalidAmount);
    require!(params.max_payment > 0, crate::errors::asset_error::AssetError::InvalidAmount);
    require!(params.slippage_tolerance <= 10000, crate::errors::asset_error::AssetError::InvalidSlippage);
    
    let basket_index = &mut ctx.accounts.basket_index;
    let user = &ctx.accounts.user;
    
    // 验证篮子状态
    basket_index.validate()?;
    
    // 创建交易参数
    let trade_params = TradeParams {
        from_token: ctx.accounts.input_token_account.mint,
        to_token: ctx.accounts.output_token_account.mint,
        amount_in: params.max_payment,
        min_amount_out: params.amount,
        dex_name: params.dex_name.clone(),
    };
    
    // 执行购买逻辑
    let result = AssetService::buy_asset(
        basket_index,
        &trade_params,
        params.strategy.as_ref(),
        params.algorithm.as_ref(),
    )?;
    
    // 触发购买事件
    emit!(AssetBought {
        basket_id: basket_index.id,
        user: user.key(),
        amount: params.amount,
        payment_amount: result.executed_amount,
        dex_name: params.dex_name,
        timestamp: Clock::get()?.unix_timestamp,
    });
    
    Ok(())
}

/// 资产购买错误码
#[error_code]
pub enum BuyAssetError {
    /// 金额无效
    #[msg("Invalid amount")] InvalidAmount,
    /// 滑点无效
    #[msg("Invalid slippage")] InvalidSlippage,
    /// 余额不足
    #[msg("Insufficient balance")] InsufficientBalance,
    /// DEX不支持
    #[msg("DEX not supported")] DexNotSupported,
    /// 执行失败
    #[msg("Execution failed")] ExecutionFailed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::prelude::Pubkey;
    
    /// 测试购买参数验证
    #[test]
    fn test_buy_params_validation() {
        let params = BuyAssetParams {
            amount: 1000,
            max_payment: 2000,
            slippage_tolerance: 500, // 5%
            dex_name: "jupiter".to_string(),
            strategy: None,
            algorithm: None,
        };
        
        assert_eq!(params.amount, 1000);
        assert_eq!(params.slippage_tolerance, 500);
        assert_eq!(params.dex_name, "jupiter");
    }
    
    /// 测试滑点容忍度验证
    #[test]
    fn test_slippage_validation() {
        // 有效滑点
        assert!(500 <= 10000);
        
        // 无效滑点（超过100%）
        assert!(10001 > 10000);
    }
} 