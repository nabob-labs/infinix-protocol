//! 加密货币 (Crypto) 滑点保护指令
//! 
//! 本模块实现加密货币资产的滑点保护功能，支持滑点检测、保护机制、动态调整等。
//! 严格遵循最小功能单元原则，确保每个函数职责单一。
//! 
//! ## 功能特性
//! - 滑点检测：实时滑点检测和监控
//! - 保护机制：多种滑点保护策略
//! - 动态调整：根据市场情况动态调整
//! - 事件记录：完整的审计追踪
//! - 状态管理：原子性操作保证数据一致性

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams, SlippageProtectionParams};
use crate::services::crypto_service::CryptoService;
use crate::events::asset_event::AssetSlippageProtected;
use crate::validation::business::validate_slippage_protection_params;
use crate::core::security::check_authority_permission;

/// 加密货币滑点保护指令账户上下文
/// 
/// 定义滑点保护操作所需的所有账户，包括：
/// - crypto_asset: 加密货币资产账户（可变）
/// - authority: 操作权限账户（签名者）
/// - slippage_protection_program: 滑点保护程序（用于保护机制）
/// - oracle_program: 预言机程序（用于价格数据）
/// - system_program: 系统程序（用于账户管理）
#[derive(Accounts)]
#[instruction(params: SlippageProtectionParams)]
pub struct SlippageProtectionCrypto<'info> {
    /// 加密货币资产账户，需要可变权限以更新状态
    #[account(
        mut,
        seeds = [b"crypto", crypto_asset.key().as_ref()],
        bump,
        constraint = crypto_asset.asset_type == AssetType::Crypto @ crate::errors::AssetError::InvalidAssetType
    )]
    pub crypto_asset: Account<'info, crate::account_models::asset::Asset>,
    
    /// 操作权限账户，必须是签名者
    #[account(
        constraint = check_authority_permission(&authority.key(), &crypto_asset.authority) @ crate::errors::SecurityError::Unauthorized
    )]
    pub authority: Signer<'info>,
    
    /// 滑点保护程序，用于保护机制
    pub slippage_protection_program: Program<'info, crate::core::types::SlippageProtectionAdapterTrait>,
    
    /// 预言机程序，用于价格数据
    pub oracle_program: Program<'info, crate::oracles::traits::OracleAdapterTrait>,
    
    /// 系统程序，用于账户管理
    pub system_program: Program<'info, System>,
    
    /// 时钟账户，用于时间戳记录
    pub clock: Sysvar<'info, Clock>,
}

/// 加密货币滑点保护指令实现
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文，自动校验账户权限与生命周期
/// - `params`: 滑点保护参数，包含保护类型、阈值等
/// - `exec_params`: 可选算法执行参数，支持算法热插拔
/// - `strategy_params`: 可选策略参数，支持多策略扩展
/// 
/// ## 返回值
/// - `anchor_lang::Result<()>`: Anchor标准返回类型，表示指令执行成功或失败
/// 
/// ## 安全性
/// - Anchor自动校验账户权限、生命周期、PDA
/// - 业务层校验滑点保护参数合法性
/// - 滑点检测机制
/// - 完整的事件记录和审计追踪
pub fn slippage_protection_crypto(
    ctx: Context<SlippageProtectionCrypto>,
    params: SlippageProtectionParams,
    exec_params: Option<ExecutionParams>,
    strategy_params: Option<StrategyParams>
) -> anchor_lang::Result<()> {
    // === 1. 参数校验 ===
    // 校验滑点保护参数合法性
    validate_slippage_protection_params(&params)?;
    
    // === 2. 权限校验 ===
    // 检查操作权限
    require!(
        ctx.accounts.authority.key() == ctx.accounts.crypto_asset.authority,
        crate::errors::SecurityError::Unauthorized
    );
    
    // === 3. 业务逻辑执行 ===
    // 创建加密货币服务实例
    let crypto_service = CryptoService::new();
    
    // 记录滑点保护前的状态
    let balance_before = ctx.accounts.crypto_asset.balance;
    
    // 执行滑点保护操作
    let slippage_protection_result = crypto_service.slippage_protection(
        &mut ctx.accounts.crypto_asset,
        &params
    )?;
    
    // === 4. 算法执行（如果提供） ===
    if let Some(algo_params) = exec_params {
        crypto_service.execute_algorithm(&mut ctx.accounts.crypto_asset, algo_params)?;
    }
    
    // === 5. 策略执行（如果提供） ===
    if let Some(strategy) = strategy_params {
        crypto_service.execute_strategy(&mut ctx.accounts.crypto_asset, strategy)?;
    }
    
    // === 6. 事件记录 ===
    // 发出滑点保护事件，记录操作详情
    emit!(AssetSlippageProtected {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        protection_type: params.protection_type.clone(),
        protection_threshold: params.protection_threshold,
        balance_before,
        balance_after: ctx.accounts.crypto_asset.balance,
        slippage_detected: slippage_protection_result.slippage_detected,
        protection_applied: slippage_protection_result.protection_applied,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: exec_params.map(|p| p.into()),
        strategy_params: strategy_params.map(|s| s.into()),
    });
    
    // === 7. 日志记录 ===
    msg!("Crypto slippage protection executed successfully: protection_type={}, protection_threshold={}, slippage_detected={}, authority={}", 
         params.protection_type, params.protection_threshold, slippage_protection_result.slippage_detected, ctx.accounts.authority.key());
    
    Ok(())
}

/// 滑点检测指令
/// 
/// 检测潜在的滑点风险。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `detection_params`: 检测参数
pub fn slippage_detection_crypto(
    ctx: Context<SlippageProtectionCrypto>,
    detection_params: SlippageDetection
) -> anchor_lang::Result<()> {
    // === 1. 参数校验 ===
    require!(!detection_params.detection_type.is_empty(), crate::errors::AssetError::InvalidDetectionType);
    
    // === 2. 滑点检测执行 ===
    let crypto_service = CryptoService::new();
    
    // 执行滑点检测
    let detection_result = crypto_service.slippage_detection(
        &mut ctx.accounts.crypto_asset,
        &detection_params
    )?;
    
    // === 3. 事件记录 ===
    emit!(AssetSlippageProtected {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        protection_type: "detection".to_string(),
        protection_threshold: 0,
        balance_before: 0,
        balance_after: ctx.accounts.crypto_asset.balance,
        slippage_detected: detection_result.slippage_detected,
        protection_applied: detection_result.protection_applied,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: None,
        strategy_params: None,
    });
    
    msg!("Crypto slippage detection completed: detection_type={}, slippage_detected={}, authority={}", 
         detection_params.detection_type, detection_result.slippage_detected, ctx.accounts.authority.key());
    
    Ok(())
}

/// 动态滑点调整指令
/// 
/// 根据市场情况动态调整滑点保护参数。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `adjustment_params`: 调整参数
pub fn dynamic_slippage_adjustment_crypto(
    ctx: Context<SlippageProtectionCrypto>,
    adjustment_params: DynamicSlippageAdjustment
) -> anchor_lang::Result<()> {
    // === 1. 参数校验 ===
    require!(!adjustment_params.adjustment_strategy.is_empty(), crate::errors::AssetError::InvalidAdjustmentStrategy);
    
    // === 2. 动态调整执行 ===
    let crypto_service = CryptoService::new();
    
    // 执行动态调整
    let adjustment_result = crypto_service.dynamic_slippage_adjustment(
        &mut ctx.accounts.crypto_asset,
        &adjustment_params
    )?;
    
    // === 3. 事件记录 ===
    emit!(AssetSlippageProtected {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        protection_type: "dynamic_adjustment".to_string(),
        protection_threshold: adjustment_params.new_threshold,
        balance_before: 0,
        balance_after: ctx.accounts.crypto_asset.balance,
        slippage_detected: adjustment_result.slippage_detected,
        protection_applied: adjustment_result.protection_applied,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: None,
        strategy_params: None,
    });
    
    msg!("Crypto dynamic slippage adjustment executed: strategy={}, new_threshold={}, adjustment_applied={}, authority={}", 
         adjustment_params.adjustment_strategy, adjustment_params.new_threshold, adjustment_result.protection_applied, ctx.accounts.authority.key());
    
    Ok(())
}

/// 滑点保护参数结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct SlippageProtectionParams {
    /// 保护类型
    pub protection_type: String,
    /// 保护阈值
    pub protection_threshold: u64,
    /// 保护策略
    pub protection_strategy: String,
    /// 保护参数
    pub protection_params: String,
}

/// 滑点检测结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct SlippageDetection {
    /// 检测类型
    pub detection_type: String,
    /// 检测参数
    pub detection_params: String,
    /// 检测阈值
    pub detection_threshold: u64,
}

/// 动态滑点调整结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct DynamicSlippageAdjustment {
    /// 调整策略
    pub adjustment_strategy: String,
    /// 新阈值
    pub new_threshold: u64,
    /// 调整参数
    pub adjustment_params: String,
}

/// 滑点保护类型枚举
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub enum SlippageProtectionType {
    /// 固定滑点保护
    FixedProtection,
    /// 动态滑点保护
    DynamicProtection,
    /// 自适应滑点保护
    AdaptiveProtection,
    /// 组合滑点保护
    CombinedProtection,
}

/// 滑点保护结果结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct SlippageProtectionResult {
    /// 滑点是否被检测到
    pub slippage_detected: bool,
    /// 保护是否被应用
    pub protection_applied: bool,
    /// 保护时间
    pub protection_time: i64,
    /// 保护费用
    pub protection_fee: u64,
} 