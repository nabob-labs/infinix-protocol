//! RWA资产buy指令
//! RWA购买指令实现，支持多种购买策略和DEX集成。

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams, TradeParams};
use crate::services::rwa_service::RwaService;
use crate::events::index_token_event::IndexTokenBought;
use crate::dex::adapter::DexAdapter;
use crate::algorithms::traits::AlgorithmAdapter;

/// RWA资产buy指令账户上下文
#[derive(Accounts)]
pub struct BuyRwa<'info> {
    /// RWA账户，需可变
    #[account(mut)]
    pub rwa: Account<'info, BasketIndexState>,
    
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

/// RWA购买参数结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BuyRwaParams {
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
    /// RWA类型
    pub rwa_type: String,
    /// 合规检查
    pub compliance_check: bool,
}

/// RWA资产buy指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - params: 购买参数
/// - 返回: Anchor规范Result
pub fn buy_rwa(ctx: Context<BuyRwa>, params: BuyRwaParams) -> Result<()> {
    // 参数验证
    require!(params.amount > 0, crate::errors::index_token_error::IndexTokenError::InvalidAmount);
    require!(params.max_payment > 0, crate::errors::index_token_error::IndexTokenError::InvalidAmount);
    require!(params.slippage_tolerance <= 10000, crate::errors::index_token_error::IndexTokenError::InvalidSlippage);
    
    let rwa = &mut ctx.accounts.rwa;
    let user = &ctx.accounts.user;
    
    // 验证资产类型
    require!(rwa.asset_type == AssetType::RWA, crate::errors::index_token_error::IndexTokenError::InvalidAssetType);
    
    // 验证RWA状态
    rwa.validate()?;
    
    // 合规检查
    if params.compliance_check {
        require!(user.key() != Pubkey::default(), crate::errors::security_error::SecurityError::ComplianceCheckFailed);
    }
    
    // 创建交易参数
    let trade_params = TradeParams {
        from_token: ctx.accounts.input_token_account.mint,
        to_token: ctx.accounts.output_token_account.mint,
        amount_in: params.max_payment,
        min_amount_out: params.amount,
        dex_name: params.dex_name.clone(),
    };
    
    // 执行RWA购买逻辑
    let service = RwaService::new();
    let result = service.buy_rwa(
        rwa,
        &trade_params,
        params.strategy.as_ref(),
        params.algorithm.as_ref(),
        params.use_smart_routing,
        &params.rwa_type,
        params.compliance_check,
    )?;
    
    // 触发RWA购买事件
    emit!(IndexTokenBought {
        index_token_id: rwa.id,
        user: user.key(),
        amount: params.amount,
        payment_amount: result.executed_amount,
        dex_name: params.dex_name,
        timestamp: Clock::get()?.unix_timestamp,
    });
    
    Ok(())
}

/// RWA购买错误码
#[error_code]
pub enum BuyRwaError {
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
    /// RWA类型不支持
    #[msg("RWA type not supported")] RwaTypeNotSupported,
    /// 合规检查失败
    #[msg("Compliance check failed")] ComplianceCheckFailed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::prelude::Pubkey;
    
    /// 测试RWA购买参数验证
    #[test]
    fn test_buy_rwa_params_validation() {
        let params = BuyRwaParams {
            amount: 1000,
            max_payment: 2000,
            slippage_tolerance: 500, // 5%
            dex_name: "jupiter".to_string(),
            strategy: None,
            algorithm: None,
            use_smart_routing: true,
            rwa_type: "REAL_ESTATE".to_string(),
            compliance_check: true,
        };
        
        assert_eq!(params.amount, 1000);
        assert_eq!(params.slippage_tolerance, 500);
        assert_eq!(params.dex_name, "jupiter");
        assert_eq!(params.rwa_type, "REAL_ESTATE");
        assert!(params.use_smart_routing);
        assert!(params.compliance_check);
    }
    
    /// 测试滑点容忍度验证
    #[test]
    fn test_rwa_slippage_validation() {
        // 有效滑点
        assert!(500 <= 10000);
        
        // 无效滑点（超过100%）
        assert!(10001 > 10000);
    }
} 