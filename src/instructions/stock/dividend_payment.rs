//! 股票分红支付指令
//!
//! 本模块实现了股票的分红支付功能，包括现金分红、股票分红、特殊分红等。
//!
//! ## 功能特点
//!
//! - **多种分红类型**: 支持现金分红、股票分红、特殊分红
//! - **灵活支付方式**: 支持自动支付、手动支付、分批支付
//! - **分红信息管理**: 完整的分红记录和追踪
//! - **事件驱动**: 完整的事件发射和审计追踪
//! - **错误处理**: 全面的错误类型和处理机制
//!
//! ## 使用场景
//!
//! - 定期现金分红
//! - 股票分红
//! - 特殊分红
//! - 分红记录管理

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams};
use crate::services::stock_service::StockService;
use crate::events::asset_event::AssetDividendPaid;
use crate::errors::AssetError;

/// 分红类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum DividendType {
    /// 现金分红
    Cash,
    /// 股票分红
    Stock,
    /// 特殊分红
    Special,
    /// 优先股分红
    Preferred,
}

/// 分红支付方式
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum DividendPaymentMethod {
    /// 自动支付
    Automatic,
    /// 手动支付
    Manual,
    /// 分批支付
    Batch,
    /// 递延支付
    Deferred,
}

/// 分红信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct DividendInfo {
    /// 分红类型
    pub dividend_type: DividendType,
    /// 分红金额
    pub amount: u64,
    /// 每股分红
    pub dividend_per_share: f64,
    /// 分红日期
    pub dividend_date: i64,
    /// 除权日期
    pub ex_dividend_date: i64,
    /// 支付日期
    pub payment_date: i64,
    /// 分红说明
    pub description: String,
}

/// 分红支付结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct DividendPaymentResult {
    /// 分红ID
    pub dividend_id: u64,
    /// 分红类型
    pub dividend_type: DividendType,
    /// 支付方式
    pub payment_method: DividendPaymentMethod,
    /// 分红金额
    pub total_amount: u64,
    /// 支付数量
    pub payment_count: u64,
    /// 支付状态
    pub payment_status: bool,
    /// 支付时间戳
    pub payment_timestamp: i64,
}

/// 分红支付指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct DividendPaymentParams {
    /// 分红类型
    pub dividend_type: DividendType,
    /// 支付方式
    pub payment_method: DividendPaymentMethod,
    /// 分红信息
    pub dividend_info: DividendInfo,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 分红支付指令账户上下文
#[derive(Accounts)]
pub struct DividendPayment<'info> {
    /// 股票资产账户，需可变
    #[account(
        mut,
        constraint = stock.asset_type == AssetType::Stock @ AssetError::InvalidAssetType
    )]
    pub stock: Account<'info, BasketIndexState>,
    
    /// 分红权限签名者
    #[account(
        constraint = authority.key() == stock.dividend_authority @ AssetError::InsufficientAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 分红接收方账户
    #[account(mut)]
    pub recipient_account: Account<'info, TokenAccount>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 分红支付指令实现
///
/// ## 参数说明
/// - `ctx`: Anchor账户上下文，自动校验权限与生命周期
/// - `params`: 分红支付参数，包含分红类型、支付方式和分红信息
///
/// ## 返回值
/// - `Result<DividendPaymentResult>`: 分红支付结果
///
/// ## 错误处理
/// - `InvalidAssetType`: 资产类型不匹配
/// - `InsufficientAuthority`: 权限不足
/// - `InvalidDividendAmount`: 无效的分红金额
/// - `InvalidDividendDate`: 无效的分红日期
/// - `InvalidParams`: 无效的参数
pub fn dividend_payment(
    ctx: Context<DividendPayment>,
    params: DividendPaymentParams,
) -> Result<DividendPaymentResult> {
    // 参数验证
    validate_dividend_payment_params(&params)?;
    
    // 权限检查
    check_dividend_authority_permission(&ctx.accounts.authority, &ctx.accounts.stock)?;
    
    // 获取账户引用
    let stock = &mut ctx.accounts.stock;
    let authority = &ctx.accounts.authority;
    
    // 创建服务实例
    let service = StockService::new();
    
    // 调用服务层执行分红支付操作
    let result = service.dividend_payment(
        stock,
        &params.dividend_type,
        &params.payment_method,
        &params.dividend_info,
        &params.exec_params,
    )?;
    
    // 发射事件
    emit!(AssetDividendPaid {
        basket_id: stock.id,
        dividend_id: result.dividend_id,
        dividend_type: params.dividend_type,
        amount: result.total_amount,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::Stock,
        exec_params: params.exec_params,
    });
    
    Ok(result)
}

/// 验证分红支付参数
fn validate_dividend_payment_params(params: &DividendPaymentParams) -> Result<()> {
    require!(params.dividend_info.amount > 0, AssetError::InvalidDividendAmount);
    require!(params.dividend_info.dividend_per_share > 0.0, AssetError::InvalidDividendAmount);
    require!(params.dividend_info.dividend_date > 0, AssetError::InvalidDividendDate);
    require!(params.dividend_info.ex_dividend_date > 0, AssetError::InvalidDividendDate);
    require!(params.dividend_info.payment_date > 0, AssetError::InvalidDividendDate);
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 检查分红权限
fn check_dividend_authority_permission(
    authority: &Signer,
    stock: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == stock.dividend_authority,
        AssetError::InsufficientAuthority
    );
    
    Ok(())
}

/// 验证执行参数
fn validate_execution_params(exec_params: &ExecutionParams) -> Result<()> {
    require!(exec_params.slippage_tolerance > 0.0, AssetError::InvalidParams);
    require!(exec_params.slippage_tolerance <= 1.0, AssetError::InvalidParams);
    require!(exec_params.max_retries > 0, AssetError::InvalidParams);
    require!(exec_params.max_retries <= 10, AssetError::InvalidParams);
    
    Ok(())
} 