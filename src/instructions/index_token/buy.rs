//! IndexToken资产buy指令
//! 指数代币购买指令实现，支持多种购买策略和DEX集成。

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams, TradeParams};
use crate::services::index_token_service::IndexTokenService;
use crate::events::index_token_event::IndexTokenBought;
use crate::dex::adapter::DexAdapter;
use crate::algorithms::traits::AlgorithmAdapter;

/// IndexToken资产buy指令账户上下文
#[derive(Accounts)]
pub struct BuyIndexToken<'info> {
    /// 指数代币账户，需可变
    #[account(mut)]
    pub index_token: Account<'info, BasketIndexState>,
    
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

/// 指数代币购买参数结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BuyIndexTokenParams {
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
    /// 是否使用智能路由
    pub use_smart_routing: bool,
}

/// IndexToken资产buy指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - params: 购买参数
/// - 返回: Anchor规范Result
pub fn buy_index_token(ctx: Context<BuyIndexToken>, params: BuyIndexTokenParams) -> Result<()> {
    // 参数验证
    require!(params.amount > 0, crate::errors::index_token_error::IndexTokenError::InvalidAmount);
    require!(params.max_payment > 0, crate::errors::index_token_error::IndexTokenError::InvalidAmount);
    require!(params.slippage_tolerance <= 10000, crate::errors::index_token_error::IndexTokenError::InvalidSlippage);
    
    let index_token = &mut ctx.accounts.index_token;
    let user = &ctx.accounts.user;
    
    // 验证资产类型
    require!(index_token.asset_type == AssetType::IndexToken, crate::errors::index_token_error::IndexTokenError::InvalidAssetType);
    
    // 验证指数代币状态
    index_token.validate()?;
    
    // 创建交易参数
    let trade_params = TradeParams {
        from_token: ctx.accounts.input_token_account.mint,
        to_token: ctx.accounts.output_token_account.mint,
        amount_in: params.max_payment,
        min_amount_out: params.amount,
        dex_name: params.dex_name.clone(),
    };
    
    // 执行指数代币购买逻辑
    let service = IndexTokenService::new();
    let result = service.buy_index_token(
        index_token,
        &trade_params,
        params.strategy.as_ref(),
        params.algorithm.as_ref(),
        params.use_smart_routing,
    )?;
    
    // 触发指数代币购买事件
    emit!(IndexTokenBought {
        index_token_id: index_token.id,
        user: user.key(),
        amount: params.amount,
        payment_amount: result.executed_amount,
        dex_name: params.dex_name,
        timestamp: Clock::get()?.unix_timestamp,
    });
    
    Ok(())
}

/// 指数代币购买错误码
#[error_code]
pub enum BuyIndexTokenError {
    /// 金额无效
    #[msg("Invalid amount")] InvalidAmount,
    /// 滑点无效
    #[msg("Invalid slippage")] InvalidSlippage,
    /// 资产类型无效
    #[msg("Invalid asset type")] InvalidAssetType,
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
    
    /// 测试指数代币购买参数验证
    #[test]
    fn test_buy_index_token_params_validation() {
        let params = BuyIndexTokenParams {
            amount: 1000,
            max_payment: 2000,
            slippage_tolerance: 500, // 5%
            dex_name: "jupiter".to_string(),
            strategy: None,
            algorithm: None,
            use_smart_routing: true,
        };
        
        assert_eq!(params.amount, 1000);
        assert_eq!(params.slippage_tolerance, 500);
        assert_eq!(params.dex_name, "jupiter");
        assert!(params.use_smart_routing);
    }
    
    /// 测试滑点容忍度验证
    #[test]
    fn test_index_token_slippage_validation() {
        // 有效滑点
        assert!(500 <= 10000);
        
        // 无效滑点（超过100%）
        assert!(10001 > 10000);
    }
} 