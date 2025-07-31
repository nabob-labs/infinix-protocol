//! 股票反洗钱(AML)检查指令
//!
//! 本模块实现了股票的反洗钱(AML)检查功能，包括客户身份核查、交易监控、风险评估等。
//!
//! ## 功能特点
//!
//! - **多种AML检查类型**: 支持身份核查、交易监控、名单筛查
//! - **灵活检查方式**: 支持自动、手动、定期检查
//! - **AML信息管理**: 完整的AML记录和追踪
//! - **事件驱动**: 完整的事件发射和审计追踪
//! - **错误处理**: 全面的错误类型和处理机制
//!
//! ## 使用场景
//!
//! - 客户身份核查
//! - 可疑交易监控
//! - 名单筛查
//! - 风险评估

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams};
use crate::services::stock_service::StockService;
use crate::events::asset_event::AssetAmlChecked;
use crate::errors::AssetError;

/// AML检查类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum AmlCheckType {
    /// 身份核查
    Identity,
    /// 交易监控
    Transaction,
    /// 名单筛查
    ListScreening,
    /// 风险评估
    RiskAssessment,
}

/// AML检查方式
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum AmlCheckMethod {
    /// 自动检查
    Automatic,
    /// 手动检查
    Manual,
    /// 定期检查
    Periodic,
    /// 事件触发
    EventTriggered,
}

/// AML检查信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AmlCheckInfo {
    /// 检查类型
    pub check_type: AmlCheckType,
    /// 检查数据
    pub check_data: Vec<String>,
    /// 检查开始时间
    pub check_start_time: i64,
    /// 检查结束时间
    pub check_end_time: i64,
    /// 检查说明
    pub description: String,
}

/// AML检查结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AmlCheckResult {
    /// 检查ID
    pub check_id: u64,
    /// 检查类型
    pub check_type: AmlCheckType,
    /// 检查方式
    pub check_method: AmlCheckMethod,
    /// 检查状态
    pub check_status: bool,
    /// 风险等级
    pub risk_level: u8,
    /// 检查时间戳
    pub check_timestamp: i64,
}

/// AML检查指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AmlCheckParams {
    /// 检查类型
    pub check_type: AmlCheckType,
    /// 检查方式
    pub check_method: AmlCheckMethod,
    /// 检查信息
    pub check_info: AmlCheckInfo,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// AML检查指令账户上下文
#[derive(Accounts)]
pub struct AmlCheck<'info> {
    /// 股票资产账户，需可变
    #[account(
        mut,
        constraint = stock.asset_type == AssetType::Stock @ AssetError::InvalidAssetType
    )]
    pub stock: Account<'info, BasketIndexState>,
    
    /// AML权限签名者
    #[account(
        constraint = authority.key() == stock.aml_authority @ AssetError::InsufficientAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 检查账户
    #[account(mut)]
    pub aml_account: Account<'info, TokenAccount>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// AML检查指令实现
pub fn aml_check(
    ctx: Context<AmlCheck>,
    params: AmlCheckParams,
) -> Result<AmlCheckResult> {
    validate_aml_check_params(&params)?;
    check_aml_authority_permission(&ctx.accounts.authority, &ctx.accounts.stock)?;
    let stock = &mut ctx.accounts.stock;
    let authority = &ctx.accounts.authority;
    let service = StockService::new();
    let result = service.aml_check(
        stock,
        &params.check_type,
        &params.check_method,
        &params.check_info,
        &params.exec_params,
    )?;
    emit!(AssetAmlChecked {
        basket_id: stock.id,
        check_id: result.check_id,
        check_type: params.check_type,
        check_status: result.check_status,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::Stock,
        exec_params: params.exec_params,
    });
    Ok(result)
}

fn validate_aml_check_params(params: &AmlCheckParams) -> Result<()> {
    require!(!params.check_info.check_data.is_empty(), AssetError::InvalidAmlInfo);
    require!(params.check_info.check_start_time > 0, AssetError::InvalidAmlTime);
    require!(params.check_info.check_end_time > 0, AssetError::InvalidAmlTime);
    require!(params.check_info.check_end_time > params.check_info.check_start_time, AssetError::InvalidAmlTime);
    validate_execution_params(&params.exec_params)?;
    Ok(())
}
fn check_aml_authority_permission(
    authority: &Signer,
    stock: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == stock.aml_authority,
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