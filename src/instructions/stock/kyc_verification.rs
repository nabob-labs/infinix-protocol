//! 股票KYC验证指令
//!
//! 本模块实现了股票的KYC验证管理功能，包括身份验证、风险评估、合规检查等。
//!
//! ## 功能特点
//!
//! - **多种验证类型**: 支持身份验证、地址验证、风险评估
//! - **灵活验证方式**: 支持自动验证、手动验证、第三方验证
//! - **验证信息管理**: 完整的KYC记录和追踪
//! - **事件驱动**: 完整的事件发射和审计追踪
//! - **错误处理**: 全面的错误类型和处理机制
//!
//! ## 使用场景
//!
//! - 客户身份验证
//! - 地址验证
//! - 风险评估
//! - 合规检查

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams};
use crate::services::stock_service::StockService;
use crate::events::asset_event::AssetKycVerified;
use crate::errors::AssetError;

/// KYC验证类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum KycVerificationType {
    /// 身份验证
    Identity,
    /// 地址验证
    Address,
    /// 风险评估
    RiskAssessment,
    /// 政治敏感人物检查
    PepCheck,
    /// 制裁名单检查
    SanctionsCheck,
}

/// KYC验证方式
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum KycVerificationMethod {
    /// 自动验证
    Automatic,
    /// 手动验证
    Manual,
    /// 第三方验证
    ThirdParty,
    /// 区块链验证
    Blockchain,
}

/// KYC验证信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct KycVerificationInfo {
    /// 验证类型
    pub verification_type: KycVerificationType,
    /// 验证数据
    pub verification_data: Vec<String>,
    /// 验证开始时间
    pub verification_start_time: i64,
    /// 验证结束时间
    pub verification_end_time: i64,
    /// 验证说明
    pub description: String,
}

/// KYC验证结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct KycVerificationResult {
    /// 验证ID
    pub verification_id: u64,
    /// 验证类型
    pub verification_type: KycVerificationType,
    /// 验证方式
    pub verification_method: KycVerificationMethod,
    /// 验证状态
    pub verification_status: bool,
    /// 风险等级
    pub risk_level: u8,
    /// 验证时间戳
    pub verification_timestamp: i64,
}

/// KYC验证指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct KycVerificationParams {
    /// 验证类型
    pub verification_type: KycVerificationType,
    /// 验证方式
    pub verification_method: KycVerificationMethod,
    /// 验证信息
    pub verification_info: KycVerificationInfo,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// KYC验证指令账户上下文
#[derive(Accounts)]
pub struct KycVerification<'info> {
    /// 股票资产账户，需可变
    #[account(
        mut,
        constraint = stock.asset_type == AssetType::Stock @ AssetError::InvalidAssetType
    )]
    pub stock: Account<'info, BasketIndexState>,
    
    /// KYC验证权限签名者
    #[account(
        constraint = authority.key() == stock.kyc_authority @ AssetError::InsufficientAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 验证者账户
    #[account(mut)]
    pub verifier_account: Account<'info, TokenAccount>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// KYC验证指令实现
///
/// ## 参数说明
/// - `ctx`: Anchor账户上下文，自动校验权限与生命周期
/// - `params`: KYC验证参数，包含验证类型、验证方式和验证信息
///
/// ## 返回值
/// - `Result<KycVerificationResult>`: KYC验证结果
///
/// ## 错误处理
/// - `InvalidAssetType`: 资产类型不匹配
/// - `InsufficientAuthority`: 权限不足
/// - `InvalidVerificationInfo`: 无效的验证信息
/// - `InvalidVerificationTime`: 无效的验证时间
/// - `InvalidParams`: 无效的参数
pub fn kyc_verification(
    ctx: Context<KycVerification>,
    params: KycVerificationParams,
) -> Result<KycVerificationResult> {
    // 参数验证
    validate_kyc_verification_params(&params)?;
    
    // 权限检查
    check_kyc_authority_permission(&ctx.accounts.authority, &ctx.accounts.stock)?;
    
    // 获取账户引用
    let stock = &mut ctx.accounts.stock;
    let authority = &ctx.accounts.authority;
    
    // 创建服务实例
    let service = StockService::new();
    
    // 调用服务层执行KYC验证操作
    let result = service.kyc_verification(
        stock,
        &params.verification_type,
        &params.verification_method,
        &params.verification_info,
        &params.exec_params,
    )?;
    
    // 发射事件
    emit!(AssetKycVerified {
        basket_id: stock.id,
        verification_id: result.verification_id,
        verification_type: params.verification_type,
        verification_status: result.verification_status,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::Stock,
        exec_params: params.exec_params,
    });
    
    Ok(result)
}

/// 验证KYC验证参数
fn validate_kyc_verification_params(params: &KycVerificationParams) -> Result<()> {
    require!(!params.verification_info.verification_data.is_empty(), AssetError::InvalidVerificationInfo);
    require!(params.verification_info.verification_start_time > 0, AssetError::InvalidVerificationTime);
    require!(params.verification_info.verification_end_time > 0, AssetError::InvalidVerificationTime);
    require!(params.verification_info.verification_end_time > params.verification_info.verification_start_time, AssetError::InvalidVerificationTime);
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 检查KYC权限
fn check_kyc_authority_permission(
    authority: &Signer,
    stock: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == stock.kyc_authority,
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