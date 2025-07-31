//! 现实世界资产报价指令
//!
//! 本模块实现了现实世界资产的报价功能，包括参数验证、权限检查、服务层调用和事件发射。
//!
//! ## 功能特点
//!
//! - **参数验证**: 严格的输入参数验证和边界检查
//! - **权限控制**: 细粒度的权限验证和管理
//! - **服务层抽象**: 核心业务逻辑委托给RwaService
//! - **事件驱动**: 完整的事件发射和审计追踪
//! - **错误处理**: 全面的错误类型和处理机制
//!
//! ## 使用场景
//!
//! - 现实世界资产报价操作
//! - 投资组合估值
//! - 自动化交易

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, QuoteParams};
use crate::services::rwa_service::RwaService;
use crate::events::asset_event::AssetQuoted;
use crate::errors::AssetError;

/// 现实世界资产报价指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct QuoteRwaParams {
    /// 报价参数
    pub quote_params: QuoteParams,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 现实世界资产报价指令账户上下文
#[derive(Accounts)]
pub struct QuoteRwa<'info> {
    /// 现实世界资产账户，需可变
    #[account(
        mut,
        constraint = rwa.asset_type == AssetType::RWA @ AssetError::InvalidAssetType
    )]
    pub rwa: Account<'info, BasketIndexState>,
    
    /// 报价权限签名者
    #[account(
        constraint = authority.key() == rwa.quote_authority @ AssetError::InsufficientAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 资金账户
    #[account(mut)]
    pub fund_account: Account<'info, TokenAccount>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 现实世界资产报价指令实现
pub fn quote_rwa(
    ctx: Context<QuoteRwa>,
    params: QuoteRwaParams,
) -> Result<()> {
    validate_quote_rwa_params(&params)?;
    check_quote_authority_permission(&ctx.accounts.authority, &ctx.accounts.rwa)?;
    let rwa = &mut ctx.accounts.rwa;
    let authority = &ctx.accounts.authority;
    let service = RwaService::new();
    service.quote(rwa, &params.quote_params, &params.exec_params)?;
    emit!(AssetQuoted {
        basket_id: rwa.id,
        quote_params: params.quote_params.clone(),
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::RWA,
        exec_params: params.exec_params,
    });
    Ok(())
}

fn validate_quote_rwa_params(params: &QuoteRwaParams) -> Result<()> {
    // 这里假设QuoteParams有自己的校验逻辑
    require!(params.exec_params.slippage_tolerance > 0.0, AssetError::InvalidParams);
    require!(params.exec_params.slippage_tolerance <= 1.0, AssetError::InvalidParams);
    require!(params.exec_params.max_retries > 0, AssetError::InvalidParams);
    require!(params.exec_params.max_retries <= 10, AssetError::InvalidParams);
    Ok(())
}

fn check_quote_authority_permission(
    authority: &Signer,
    rwa: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == rwa.quote_authority,
        AssetError::InsufficientAuthority
    );
    Ok(())
} 