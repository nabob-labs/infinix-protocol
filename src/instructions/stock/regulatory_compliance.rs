//! 股票监管合规指令
//!
//! 本模块实现了股票的监管合规管理功能，包括合规检查、报告生成、风险评估等。
//!
//! ## 功能特点
//!
//! - **多种合规类型**: 支持监管合规、信息披露、风险评估
//! - **灵活检查方式**: 支持自动检查、手动检查、定期检查
//! - **合规信息管理**: 完整的合规记录和追踪
//! - **事件驱动**: 完整的事件发射和审计追踪
//! - **错误处理**: 全面的错误类型和处理机制
//!
//! ## 使用场景
//!
//! - 监管合规检查
//! - 信息披露管理
//! - 风险评估
//! - 合规报告生成

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams};
use crate::services::stock_service::StockService;
use crate::events::asset_event::AssetRegulatoryComplianceUpdated;
use crate::errors::AssetError;

/// 合规类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ComplianceType {
    /// 监管合规
    Regulatory,
    /// 信息披露
    Disclosure,
    /// 风险评估
    RiskAssessment,
    /// 反洗钱
    AntiMoneyLaundering,
    /// 反恐怖融资
    AntiTerrorismFinancing,
}

/// 合规检查方式
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ComplianceCheckMethod {
    /// 自动检查
    Automatic,
    /// 手动检查
    Manual,
    /// 定期检查
    Periodic,
    /// 事件触发检查
    EventTriggered,
}

/// 合规信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ComplianceInfo {
    /// 合规类型
    pub compliance_type: ComplianceType,
    /// 合规要求
    pub compliance_requirements: Vec<String>,
    /// 检查频率
    pub check_frequency: i64,
    /// 最后检查时间
    pub last_check_time: i64,
    /// 下次检查时间
    pub next_check_time: i64,
    /// 合规说明
    pub description: String,
}

/// 监管合规结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RegulatoryComplianceResult {
    /// 合规ID
    pub compliance_id: u64,
    /// 合规类型
    pub compliance_type: ComplianceType,
    /// 检查方式
    pub check_method: ComplianceCheckMethod,
    /// 合规状态
    pub compliance_status: bool,
    /// 违规数量
    pub violation_count: u64,
    /// 检查时间戳
    pub check_timestamp: i64,
}

/// 监管合规指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RegulatoryComplianceParams {
    /// 合规类型
    pub compliance_type: ComplianceType,
    /// 检查方式
    pub check_method: ComplianceCheckMethod,
    /// 合规信息
    pub compliance_info: ComplianceInfo,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 监管合规指令账户上下文
#[derive(Accounts)]
pub struct RegulatoryCompliance<'info> {
    /// 股票资产账户，需可变
    #[account(
        mut,
        constraint = stock.asset_type == AssetType::Stock @ AssetError::InvalidAssetType
    )]
    pub stock: Account<'info, BasketIndexState>,
    
    /// 合规权限签名者
    #[account(
        constraint = authority.key() == stock.compliance_authority @ AssetError::InsufficientAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 合规检查账户
    #[account(mut)]
    pub compliance_account: Account<'info, TokenAccount>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 监管合规指令实现
///
/// ## 参数说明
/// - `ctx`: Anchor账户上下文，自动校验权限与生命周期
/// - `params`: 监管合规参数，包含合规类型、检查方式和合规信息
///
/// ## 返回值
/// - `Result<RegulatoryComplianceResult>`: 监管合规结果
///
/// ## 错误处理
/// - `InvalidAssetType`: 资产类型不匹配
/// - `InsufficientAuthority`: 权限不足
/// - `InvalidComplianceInfo`: 无效的合规信息
/// - `InvalidComplianceTime`: 无效的合规时间
/// - `InvalidParams`: 无效的参数
pub fn regulatory_compliance(
    ctx: Context<RegulatoryCompliance>,
    params: RegulatoryComplianceParams,
) -> Result<RegulatoryComplianceResult> {
    // 参数验证
    validate_regulatory_compliance_params(&params)?;
    
    // 权限检查
    check_compliance_authority_permission(&ctx.accounts.authority, &ctx.accounts.stock)?;
    
    // 获取账户引用
    let stock = &mut ctx.accounts.stock;
    let authority = &ctx.accounts.authority;
    
    // 创建服务实例
    let service = StockService::new();
    
    // 调用服务层执行监管合规操作
    let result = service.regulatory_compliance(
        stock,
        &params.compliance_type,
        &params.check_method,
        &params.compliance_info,
        &params.exec_params,
    )?;
    
    // 发射事件
    emit!(AssetRegulatoryComplianceUpdated {
        basket_id: stock.id,
        compliance_id: result.compliance_id,
        compliance_type: params.compliance_type,
        compliance_status: result.compliance_status,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::Stock,
        exec_params: params.exec_params,
    });
    
    Ok(result)
}

/// 验证监管合规参数
fn validate_regulatory_compliance_params(params: &RegulatoryComplianceParams) -> Result<()> {
    require!(!params.compliance_info.compliance_requirements.is_empty(), AssetError::InvalidComplianceInfo);
    require!(params.compliance_info.check_frequency > 0, AssetError::InvalidComplianceTime);
    require!(params.compliance_info.last_check_time > 0, AssetError::InvalidComplianceTime);
    require!(params.compliance_info.next_check_time > 0, AssetError::InvalidComplianceTime);
    require!(params.compliance_info.next_check_time > params.compliance_info.last_check_time, AssetError::InvalidComplianceTime);
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 检查合规权限
fn check_compliance_authority_permission(
    authority: &Signer,
    stock: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == stock.compliance_authority,
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