//! 股票卖空交易指令
//!
//! 本模块实现了股票的卖空交易管理功能，包括卖空、回补、风险管理等。
//!
//! ## 功能特点
//!
//! - **多种交易类型**: 支持卖空、回补、风险管理
//! - **灵活管理方式**: 支持自动、手动、定期管理
//! - **卖空交易信息管理**: 完整的卖空交易记录和追踪
//! - **事件驱动**: 完整的事件发射和审计追踪
//! - **错误处理**: 全面的错误类型和处理机制
//!
//! ## 使用场景
//!
//! - 卖空交易
//! - 回补操作
//! - 风险管理
//! - 对冲策略

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams};
use crate::services::stock_service::StockService;
use crate::events::asset_event::AssetShortSellingExecuted;
use crate::errors::AssetError;

/// 卖空交易类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ShortSellingType {
    /// 卖空
    Short,
    /// 回补
    Cover,
    /// 风险管理
    RiskManagement,
}

/// 卖空交易管理方式
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ShortSellingMethod {
    /// 自动管理
    Automatic,
    /// 手动管理
    Manual,
    /// 定期管理
    Periodic,
    /// 事件触发
    EventTriggered,
}

/// 卖空交易信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ShortSellingInfo {
    /// 交易类型
    pub trading_type: ShortSellingType,
    /// 卖空价格
    pub short_price: f64,
    /// 回补价格
    pub cover_price: Option<f64>,
    /// 交易数量
    pub amount: u64,
    /// 交易开始时间
    pub start_time: i64,
    /// 交易结束时间
    pub end_time: i64,
    /// 说明
    pub description: String,
}

/// 卖空交易结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ShortSellingResult {
    /// 交易ID
    pub trading_id: u64,
    /// 交易类型
    pub trading_type: ShortSellingType,
    /// 管理方式
    pub method: ShortSellingMethod,
    /// 状态
    pub status: bool,
    /// 时间戳
    pub timestamp: i64,
}

/// 卖空交易指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ShortSellingParams {
    /// 交易类型
    pub trading_type: ShortSellingType,
    /// 管理方式
    pub method: ShortSellingMethod,
    /// 交易信息
    pub info: ShortSellingInfo,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 卖空交易指令账户上下文
#[derive(Accounts)]
pub struct ShortSelling<'info> {
    /// 股票资产账户，需可变
    #[account(
        mut,
        constraint = stock.asset_type == AssetType::Stock @ AssetError::InvalidAssetType
    )]
    pub stock: Account<'info, BasketIndexState>,
    
    /// 卖空交易权限签名者
    #[account(
        constraint = authority.key() == stock.short_selling_authority @ AssetError::InsufficientAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 管理账户
    #[account(mut)]
    pub manage_account: Account<'info, TokenAccount>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 卖空交易指令实现
pub fn short_selling(
    ctx: Context<ShortSelling>,
    params: ShortSellingParams,
) -> Result<ShortSellingResult> {
    validate_short_selling_params(&params)?;
    check_short_selling_authority_permission(&ctx.accounts.authority, &ctx.accounts.stock)?;
    let stock = &mut ctx.accounts.stock;
    let authority = &ctx.accounts.authority;
    let service = StockService::new();
    let result = service.short_selling(
        stock,
        &params.trading_type,
        &params.method,
        &params.info,
        &params.exec_params,
    )?;
    emit!(AssetShortSellingExecuted {
        basket_id: stock.id,
        trading_id: result.trading_id,
        trading_type: params.trading_type,
        status: result.status,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::Stock,
        exec_params: params.exec_params,
    });
    Ok(result)
}
fn validate_short_selling_params(params: &ShortSellingParams) -> Result<()> {
    require!(params.info.short_price > 0.0, AssetError::InvalidShortSellingPrice);
    require!(params.info.amount > 0, AssetError::InvalidShortSellingAmount);
    require!(params.info.start_time > 0, AssetError::InvalidShortSellingTime);
    require!(params.info.end_time > 0, AssetError::InvalidShortSellingTime);
    require!(params.info.end_time > params.info.start_time, AssetError::InvalidShortSellingTime);
    validate_execution_params(&params.exec_params)?;
    Ok(())
}
fn check_short_selling_authority_permission(
    authority: &Signer,
    stock: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == stock.short_selling_authority,
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