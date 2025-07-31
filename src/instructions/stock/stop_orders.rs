//! 股票止损单指令
//!
//! 本模块实现了股票的止损单管理功能，包括止损买入、止损卖出、订单管理等。
//!
//! ## 功能特点
//!
//! - **多种订单类型**: 支持止损买入、止损卖出、跟踪止损
//! - **灵活管理方式**: 支持自动、手动、定期管理
//! - **止损单信息管理**: 完整的止损单记录和追踪
//! - **事件驱动**: 完整的事件发射和审计追踪
//! - **错误处理**: 全面的错误类型和处理机制
//!
//! ## 使用场景
//!
//! - 止损买入订单
//! - 止损卖出订单
//! - 跟踪止损
//! - 风险管理

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams};
use crate::services::stock_service::StockService;
use crate::events::asset_event::AssetStopOrderPlaced;
use crate::errors::AssetError;

/// 止损单类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum StopOrderType {
    /// 止损买入
    Buy,
    /// 止损卖出
    Sell,
    /// 跟踪止损
    Trailing,
}

/// 止损单管理方式
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum StopOrderMethod {
    /// 自动管理
    Automatic,
    /// 手动管理
    Manual,
    /// 定期管理
    Periodic,
    /// 事件触发
    EventTriggered,
}

/// 止损单信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct StopOrderInfo {
    /// 订单类型
    pub order_type: StopOrderType,
    /// 止损价格
    pub stop_price: f64,
    /// 数量
    pub amount: u64,
    /// 订单开始时间
    pub start_time: i64,
    /// 订单结束时间
    pub end_time: i64,
    /// 说明
    pub description: String,
}

/// 止损单结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct StopOrderResult {
    /// 订单ID
    pub order_id: u64,
    /// 订单类型
    pub order_type: StopOrderType,
    /// 管理方式
    pub method: StopOrderMethod,
    /// 状态
    pub status: bool,
    /// 时间戳
    pub timestamp: i64,
}

/// 止损单指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct StopOrdersParams {
    /// 订单类型
    pub order_type: StopOrderType,
    /// 管理方式
    pub method: StopOrderMethod,
    /// 订单信息
    pub info: StopOrderInfo,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 止损单指令账户上下文
#[derive(Accounts)]
pub struct StopOrders<'info> {
    /// 股票资产账户，需可变
    #[account(
        mut,
        constraint = stock.asset_type == AssetType::Stock @ AssetError::InvalidAssetType
    )]
    pub stock: Account<'info, BasketIndexState>,
    
    /// 止损单权限签名者
    #[account(
        constraint = authority.key() == stock.stop_order_authority @ AssetError::InsufficientAuthority
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

/// 止损单指令实现
pub fn stop_orders(
    ctx: Context<StopOrders>,
    params: StopOrdersParams,
) -> Result<StopOrderResult> {
    validate_stop_orders_params(&params)?;
    check_stop_order_authority_permission(&ctx.accounts.authority, &ctx.accounts.stock)?;
    let stock = &mut ctx.accounts.stock;
    let authority = &ctx.accounts.authority;
    let service = StockService::new();
    let result = service.stop_orders(
        stock,
        &params.order_type,
        &params.method,
        &params.info,
        &params.exec_params,
    )?;
    emit!(AssetStopOrderPlaced {
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
fn validate_stop_orders_params(params: &StopOrdersParams) -> Result<()> {
    require!(params.info.stop_price > 0.0, AssetError::InvalidStopOrderPrice);
    require!(params.info.amount > 0, AssetError::InvalidStopOrderAmount);
    require!(params.info.start_time > 0, AssetError::InvalidStopOrderTime);
    require!(params.info.end_time > 0, AssetError::InvalidStopOrderTime);
    require!(params.info.end_time > params.info.start_time, AssetError::InvalidStopOrderTime);
    validate_execution_params(&params.exec_params)?;
    Ok(())
}
fn check_stop_order_authority_permission(
    authority: &Signer,
    stock: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == stock.stop_order_authority,
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