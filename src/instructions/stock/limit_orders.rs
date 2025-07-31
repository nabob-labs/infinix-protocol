//! 股票限价单指令
//!
//! 本模块实现了股票的限价单管理功能，包括限价买入、限价卖出、订单管理等。
//!
//! ## 功能特点
//!
//! - **多种订单类型**: 支持限价买入、限价卖出、部分成交
//! - **灵活管理方式**: 支持自动、手动、定期管理
//! - **限价单信息管理**: 完整的限价单记录和追踪
//! - **事件驱动**: 完整的事件发射和审计追踪
//! - **错误处理**: 全面的错误类型和处理机制
//!
//! ## 使用场景
//!
//! - 限价买入订单
//! - 限价卖出订单
//! - 订单管理
//! - 部分成交处理

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams};
use crate::services::stock_service::StockService;
use crate::events::asset_event::AssetLimitOrderPlaced;
use crate::errors::AssetError;

/// 限价单类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum LimitOrderType {
    /// 限价买入
    Buy,
    /// 限价卖出
    Sell,
    /// 部分成交
    Partial,
}

/// 限价单管理方式
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum LimitOrderMethod {
    /// 自动管理
    Automatic,
    /// 手动管理
    Manual,
    /// 定期管理
    Periodic,
    /// 事件触发
    EventTriggered,
}

/// 限价单信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct LimitOrderInfo {
    /// 订单类型
    pub order_type: LimitOrderType,
    /// 限价
    pub limit_price: f64,
    /// 数量
    pub amount: u64,
    /// 订单开始时间
    pub start_time: i64,
    /// 订单结束时间
    pub end_time: i64,
    /// 说明
    pub description: String,
}

/// 限价单结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct LimitOrderResult {
    /// 订单ID
    pub order_id: u64,
    /// 订单类型
    pub order_type: LimitOrderType,
    /// 管理方式
    pub method: LimitOrderMethod,
    /// 状态
    pub status: bool,
    /// 时间戳
    pub timestamp: i64,
}

/// 限价单指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct LimitOrdersParams {
    /// 订单类型
    pub order_type: LimitOrderType,
    /// 管理方式
    pub method: LimitOrderMethod,
    /// 订单信息
    pub info: LimitOrderInfo,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 限价单指令账户上下文
#[derive(Accounts)]
pub struct LimitOrders<'info> {
    /// 股票资产账户，需可变
    #[account(
        mut,
        constraint = stock.asset_type == AssetType::Stock @ AssetError::InvalidAssetType
    )]
    pub stock: Account<'info, BasketIndexState>,
    
    /// 限价单权限签名者
    #[account(
        constraint = authority.key() == stock.limit_order_authority @ AssetError::InsufficientAuthority
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

/// 限价单指令实现
pub fn limit_orders(
    ctx: Context<LimitOrders>,
    params: LimitOrdersParams,
) -> Result<LimitOrderResult> {
    validate_limit_orders_params(&params)?;
    check_limit_order_authority_permission(&ctx.accounts.authority, &ctx.accounts.stock)?;
    let stock = &mut ctx.accounts.stock;
    let authority = &ctx.accounts.authority;
    let service = StockService::new();
    let result = service.limit_orders(
        stock,
        &params.order_type,
        &params.method,
        &params.info,
        &params.exec_params,
    )?;
    emit!(AssetLimitOrderPlaced {
        basket_id: stock.id,
        order_id: result.order_id,
        order_type: params.order_type,
        status: result.status,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::Stock,
        exec_params: params.exec_params,
    });
    Ok(result)
}
fn validate_limit_orders_params(params: &LimitOrdersParams) -> Result<()> {
    require!(params.info.limit_price > 0.0, AssetError::InvalidLimitOrderPrice);
    require!(params.info.amount > 0, AssetError::InvalidLimitOrderAmount);
    require!(params.info.start_time > 0, AssetError::InvalidLimitOrderTime);
    require!(params.info.end_time > 0, AssetError::InvalidLimitOrderTime);
    require!(params.info.end_time > params.info.start_time, AssetError::InvalidLimitOrderTime);
    validate_execution_params(&params.exec_params)?;
    Ok(())
}
fn check_limit_order_authority_permission(
    authority: &Signer,
    stock: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == stock.limit_order_authority,
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