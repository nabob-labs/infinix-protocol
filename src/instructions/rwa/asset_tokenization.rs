//! 现实世界资产代币化指令
//!
//! 本模块实现了现实世界资产的代币化功能，包括资产数字化、代币创建、法律文件管理等。
//!
//! ## 功能特点
//!
//! - **多种代币化类型**: 支持房地产、债券、商品、艺术品等
//! - **灵活代币化方式**: 支持自动、手动、定期代币化
//! - **代币化信息管理**: 完整的代币化记录和追踪
//! - **事件驱动**: 完整的事件发射和审计追踪
//! - **错误处理**: 全面的错误类型和处理机制
//!
//! ## 使用场景
//!
//! - 房地产代币化
//! - 债券代币化
//! - 商品代币化
//! - 艺术品代币化

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams};
use crate::services::rwa_service::RwaService;
use crate::events::asset_event::AssetTokenized;
use crate::errors::AssetError;

/// 代币化类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum TokenizationType {
    /// 房地产
    RealEstate,
    /// 债券
    Bond,
    /// 商品
    Commodity,
    /// 艺术品
    Artwork,
    /// 知识产权
    IntellectualProperty,
    /// 基础设施
    Infrastructure,
}

/// 代币化方式
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum TokenizationMethod {
    /// 自动代币化
    Automatic,
    /// 手动代币化
    Manual,
    /// 定期代币化
    Periodic,
    /// 事件触发
    EventTriggered,
}

/// 代币化信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct TokenizationInfo {
    /// 代币化类型
    pub tokenization_type: TokenizationType,
    /// 资产价值
    pub asset_value: f64,
    /// 代币数量
    pub token_amount: u64,
    /// 代币化开始时间
    pub start_time: i64,
    /// 代币化结束时间
    pub end_time: i64,
    /// 说明
    pub description: String,
}

/// 代币化结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct TokenizationResult {
    /// 代币化ID
    pub tokenization_id: u64,
    /// 代币化类型
    pub tokenization_type: TokenizationType,
    /// 代币化方式
    pub method: TokenizationMethod,
    /// 状态
    pub status: bool,
    /// 时间戳
    pub timestamp: i64,
}

/// 资产代币化指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AssetTokenizationParams {
    /// 代币化类型
    pub tokenization_type: TokenizationType,
    /// 代币化方式
    pub method: TokenizationMethod,
    /// 代币化信息
    pub info: TokenizationInfo,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 资产代币化指令账户上下文
#[derive(Accounts)]
pub struct AssetTokenization<'info> {
    /// 现实世界资产账户，需可变
    #[account(
        mut,
        constraint = rwa.asset_type == AssetType::RWA @ AssetError::InvalidAssetType
    )]
    pub rwa: Account<'info, BasketIndexState>,
    
    /// 代币化权限签名者
    #[account(
        constraint = authority.key() == rwa.tokenization_authority @ AssetError::InsufficientAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 代币化账户
    #[account(mut)]
    pub tokenization_account: Account<'info, TokenAccount>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 资产代币化指令实现
pub fn asset_tokenization(
    ctx: Context<AssetTokenization>,
    params: AssetTokenizationParams,
) -> Result<TokenizationResult> {
    validate_asset_tokenization_params(&params)?;
    check_tokenization_authority_permission(&ctx.accounts.authority, &ctx.accounts.rwa)?;
    let rwa = &mut ctx.accounts.rwa;
    let authority = &ctx.accounts.authority;
    let service = RwaService::new();
    let result = service.asset_tokenization(
        rwa,
        &params.tokenization_type,
        &params.method,
        &params.info,
        &params.exec_params,
    )?;
    emit!(AssetTokenized {
        basket_id: rwa.id,
        tokenization_id: result.tokenization_id,
        tokenization_type: params.tokenization_type,
        status: result.status,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::RWA,
        exec_params: params.exec_params,
    });
    Ok(result)
}

fn validate_asset_tokenization_params(params: &AssetTokenizationParams) -> Result<()> {
    require!(params.info.asset_value > 0.0, AssetError::InvalidTokenizationValue);
    require!(params.info.token_amount > 0, AssetError::InvalidTokenizationAmount);
    require!(params.info.start_time > 0, AssetError::InvalidTokenizationTime);
    require!(params.info.end_time > 0, AssetError::InvalidTokenizationTime);
    require!(params.info.end_time > params.info.start_time, AssetError::InvalidTokenizationTime);
    validate_execution_params(&params.exec_params)?;
    Ok(())
}

fn check_tokenization_authority_permission(
    authority: &Signer,
    rwa: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == rwa.tokenization_authority,
        AssetError::InsufficientAuthority
    );
    Ok(())
}

fn validate_execution_params(exec_params: &ExecutionParams) -> Result<()> {
    require!(exec_params.slippage_tolerance > 0.0, AssetError::InvalidParams);
    require!(exec_params.slippage_tolerance <= 1.0, AssetError::InvalidParams);
    require!(exec_params.max_retries > 0, AssetError::InvalidParams);
    require!(exec_params.max_retries <= 10, AssetError::InvalidParams);
    Ok(())
} 