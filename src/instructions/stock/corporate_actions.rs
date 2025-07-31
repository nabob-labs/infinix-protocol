//! 股票公司行为指令
//!
//! 本模块实现了股票的公司行为管理功能，包括股票分割、合并、回购、增发等。
//!
//! ## 功能特点
//!
//! - **多种行为类型**: 支持股票分割、合并、回购、增发
//! - **灵活执行方式**: 支持自动执行、手动执行、分批执行
//! - **行为信息管理**: 完整的公司行为记录和追踪
//! - **事件驱动**: 完整的事件发射和审计追踪
//! - **错误处理**: 全面的错误类型和处理机制
//!
//! ## 使用场景
//!
//! - 股票分割
//! - 股票合并
//! - 股票回购
//! - 股票增发

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams};
use crate::services::stock_service::StockService;
use crate::events::asset_event::AssetCorporateActionExecuted;
use crate::errors::AssetError;

/// 公司行为类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum CorporateActionType {
    /// 股票分割
    StockSplit,
    /// 股票合并
    StockMerge,
    /// 股票回购
    StockRepurchase,
    /// 股票增发
    StockIssuance,
    /// 股票注销
    StockCancellation,
    /// 股票转换
    StockConversion,
}

/// 公司行为执行方式
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum CorporateActionMethod {
    /// 自动执行
    Automatic,
    /// 手动执行
    Manual,
    /// 分批执行
    Batch,
    /// 条件执行
    Conditional,
}

/// 公司行为信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CorporateActionInfo {
    /// 行为类型
    pub action_type: CorporateActionType,
    /// 行为比例
    pub action_ratio: f64,
    /// 行为数量
    pub action_amount: u64,
    /// 行为开始时间
    pub action_start_time: i64,
    /// 行为结束时间
    pub action_end_time: i64,
    /// 行为说明
    pub description: String,
}

/// 公司行为结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CorporateActionResult {
    /// 行为ID
    pub action_id: u64,
    /// 行为类型
    pub action_type: CorporateActionType,
    /// 执行方式
    pub execution_method: CorporateActionMethod,
    /// 执行数量
    pub execution_amount: u64,
    /// 执行状态
    pub execution_status: bool,
    /// 执行时间戳
    pub execution_timestamp: i64,
}

/// 公司行为指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CorporateActionsParams {
    /// 行为类型
    pub action_type: CorporateActionType,
    /// 执行方式
    pub execution_method: CorporateActionMethod,
    /// 行为信息
    pub action_info: CorporateActionInfo,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 公司行为指令账户上下文
#[derive(Accounts)]
pub struct CorporateActions<'info> {
    /// 股票资产账户，需可变
    #[account(
        mut,
        constraint = stock.asset_type == AssetType::Stock @ AssetError::InvalidAssetType
    )]
    pub stock: Account<'info, BasketIndexState>,
    
    /// 公司行为权限签名者
    #[account(
        constraint = authority.key() == stock.corporate_action_authority @ AssetError::InsufficientAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 目标账户
    #[account(mut)]
    pub target_account: Account<'info, TokenAccount>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 公司行为指令实现
///
/// ## 参数说明
/// - `ctx`: Anchor账户上下文，自动校验权限与生命周期
/// - `params`: 公司行为参数，包含行为类型、执行方式和行为信息
///
/// ## 返回值
/// - `Result<CorporateActionResult>`: 公司行为结果
///
/// ## 错误处理
/// - `InvalidAssetType`: 资产类型不匹配
/// - `InsufficientAuthority`: 权限不足
/// - `InvalidActionInfo`: 无效的行为信息
/// - `InvalidActionTime`: 无效的行为时间
/// - `InvalidParams`: 无效的参数
pub fn corporate_actions(
    ctx: Context<CorporateActions>,
    params: CorporateActionsParams,
) -> Result<CorporateActionResult> {
    // 参数验证
    validate_corporate_actions_params(&params)?;
    
    // 权限检查
    check_corporate_action_authority_permission(&ctx.accounts.authority, &ctx.accounts.stock)?;
    
    // 获取账户引用
    let stock = &mut ctx.accounts.stock;
    let authority = &ctx.accounts.authority;
    
    // 创建服务实例
    let service = StockService::new();
    
    // 调用服务层执行公司行为操作
    let result = service.corporate_actions(
        stock,
        &params.action_type,
        &params.execution_method,
        &params.action_info,
        &params.exec_params,
    )?;
    
    // 发射事件
    emit!(AssetCorporateActionExecuted {
        basket_id: stock.id,
        action_id: result.action_id,
        action_type: params.action_type,
        execution_amount: result.execution_amount,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::Stock,
        exec_params: params.exec_params,
    });
    
    Ok(result)
}

/// 验证公司行为参数
fn validate_corporate_actions_params(params: &CorporateActionsParams) -> Result<()> {
    require!(params.action_info.action_ratio > 0.0, AssetError::InvalidActionInfo);
    require!(params.action_info.action_amount > 0, AssetError::InvalidActionInfo);
    require!(params.action_info.action_start_time > 0, AssetError::InvalidActionTime);
    require!(params.action_info.action_end_time > 0, AssetError::InvalidActionTime);
    require!(params.action_info.action_end_time > params.action_info.action_start_time, AssetError::InvalidActionTime);
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 检查公司行为权限
fn check_corporate_action_authority_permission(
    authority: &Signer,
    stock: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == stock.corporate_action_authority,
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