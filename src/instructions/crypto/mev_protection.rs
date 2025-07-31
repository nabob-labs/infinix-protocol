//! 加密货币 (Crypto) MEV保护指令
//! 
//! 本模块实现加密货币资产的MEV保护功能，支持MEV检测、保护机制、反套利等。
//! 严格遵循最小功能单元原则，确保每个函数职责单一。
//! 
//! ## 功能特性
//! - MEV检测：实时MEV攻击检测
//! - 保护机制：多种MEV保护策略
//! - 反套利：防止套利攻击
//! - 事件记录：完整的审计追踪
//! - 状态管理：原子性操作保证数据一致性

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams, MevProtectionParams};
use crate::services::crypto_service::CryptoService;
use crate::events::asset_event::AssetMevProtected;
use crate::validation::business::validate_mev_protection_params;
use crate::core::security::check_authority_permission;

/// 加密货币MEV保护指令账户上下文
/// 
/// 定义MEV保护操作所需的所有账户，包括：
/// - crypto_asset: 加密货币资产账户（可变）
/// - authority: 操作权限账户（签名者）
/// - mev_protection_program: MEV保护程序（用于保护机制）
/// - oracle_program: 预言机程序（用于价格数据）
/// - system_program: 系统程序（用于账户管理）
#[derive(Accounts)]
#[instruction(params: MevProtectionParams)]
pub struct MevProtectionCrypto<'info> {
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
    
    /// MEV保护程序，用于保护机制
    pub mev_protection_program: Program<'info, crate::core::types::MevProtectionAdapterTrait>,
    
    /// 预言机程序，用于价格数据
    pub oracle_program: Program<'info, crate::oracles::traits::OracleAdapterTrait>,
    
    /// 系统程序，用于账户管理
    pub system_program: Program<'info, System>,
    
    /// 时钟账户，用于时间戳记录
    pub clock: Sysvar<'info, Clock>,
}

/// 加密货币MEV保护指令实现
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文，自动校验账户权限与生命周期
/// - `params`: MEV保护参数，包含保护类型、阈值等
/// - `exec_params`: 可选算法执行参数，支持算法热插拔
/// - `strategy_params`: 可选策略参数，支持多策略扩展
/// 
/// ## 返回值
/// - `anchor_lang::Result<()>`: Anchor标准返回类型，表示指令执行成功或失败
/// 
/// ## 安全性
/// - Anchor自动校验账户权限、生命周期、PDA
/// - 业务层校验MEV保护参数合法性
/// - MEV检测机制
/// - 完整的事件记录和审计追踪
pub fn mev_protection_crypto(
    ctx: Context<MevProtectionCrypto>,
    params: MevProtectionParams,
    exec_params: Option<ExecutionParams>,
    strategy_params: Option<StrategyParams>
) -> anchor_lang::Result<()> {
    // === 1. 参数校验 ===
    // 校验MEV保护参数合法性
    validate_mev_protection_params(&params)?;
    
    // === 2. 权限校验 ===
    // 检查操作权限
    require!(
        ctx.accounts.authority.key() == ctx.accounts.crypto_asset.authority,
        crate::errors::SecurityError::Unauthorized
    );
    
    // === 3. 业务逻辑执行 ===
    // 创建加密货币服务实例
    let crypto_service = CryptoService::new();
    
    // 记录MEV保护前的状态
    let balance_before = ctx.accounts.crypto_asset.balance;
    
    // 执行MEV保护操作
    let mev_protection_result = crypto_service.mev_protection(
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
    // 发出MEV保护事件，记录操作详情
    emit!(AssetMevProtected {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        protection_type: params.protection_type.clone(),
        protection_threshold: params.protection_threshold,
        balance_before,
        balance_after: ctx.accounts.crypto_asset.balance,
        mev_detected: mev_protection_result.mev_detected,
        protection_applied: mev_protection_result.protection_applied,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: exec_params.map(|p| p.into()),
        strategy_params: strategy_params.map(|s| s.into()),
    });
    
    // === 7. 日志记录 ===
    msg!("Crypto MEV protection executed successfully: protection_type={}, protection_threshold={}, mev_detected={}, authority={}", 
         params.protection_type, params.protection_threshold, mev_protection_result.mev_detected, ctx.accounts.authority.key());
    
    Ok(())
}

/// MEV检测指令
/// 
/// 检测潜在的MEV攻击。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `detection_params`: 检测参数
pub fn mev_detection_crypto(
    ctx: Context<MevProtectionCrypto>,
    detection_params: MevDetection
) -> anchor_lang::Result<()> {
    // === 1. 参数校验 ===
    require!(!detection_params.detection_type.is_empty(), crate::errors::AssetError::InvalidDetectionType);
    
    // === 2. MEV检测执行 ===
    let crypto_service = CryptoService::new();
    
    // 执行MEV检测
    let detection_result = crypto_service.mev_detection(
        &mut ctx.accounts.crypto_asset,
        &detection_params
    )?;
    
    // === 3. 事件记录 ===
    emit!(AssetMevProtected {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        protection_type: "detection".to_string(),
        protection_threshold: 0,
        balance_before: 0,
        balance_after: ctx.accounts.crypto_asset.balance,
        mev_detected: detection_result.mev_detected,
        protection_applied: detection_result.protection_applied,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: None,
        strategy_params: None,
    });
    
    msg!("Crypto MEV detection completed: detection_type={}, mev_detected={}, authority={}", 
         detection_params.detection_type, detection_result.mev_detected, ctx.accounts.authority.key());
    
    Ok(())
}

/// 反套利保护指令
/// 
/// 执行反套利保护机制。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `anti_arbitrage_params`: 反套利参数
pub fn anti_arbitrage_protection_crypto(
    ctx: Context<MevProtectionCrypto>,
    anti_arbitrage_params: AntiArbitrageProtection
) -> anchor_lang::Result<()> {
    // === 1. 参数校验 ===
    require!(!anti_arbitrage_params.protection_strategy.is_empty(), crate::errors::AssetError::InvalidProtectionStrategy);
    
    // === 2. 反套利保护执行 ===
    let crypto_service = CryptoService::new();
    
    // 执行反套利保护
    let protection_result = crypto_service.anti_arbitrage_protection(
        &mut ctx.accounts.crypto_asset,
        &anti_arbitrage_params
    )?;
    
    // === 3. 事件记录 ===
    emit!(AssetMevProtected {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        protection_type: "anti_arbitrage".to_string(),
        protection_threshold: anti_arbitrage_params.protection_threshold,
        balance_before: 0,
        balance_after: ctx.accounts.crypto_asset.balance,
        mev_detected: protection_result.mev_detected,
        protection_applied: protection_result.protection_applied,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: None,
        strategy_params: None,
    });
    
    msg!("Crypto anti-arbitrage protection executed: strategy={}, threshold={}, protection_applied={}, authority={}", 
         anti_arbitrage_params.protection_strategy, anti_arbitrage_params.protection_threshold, protection_result.protection_applied, ctx.accounts.authority.key());
    
    Ok(())
}

/// MEV保护参数结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct MevProtectionParams {
    /// 保护类型
    pub protection_type: String,
    /// 保护阈值
    pub protection_threshold: u64,
    /// 保护策略
    pub protection_strategy: String,
    /// 保护参数
    pub protection_params: String,
}

/// MEV检测结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct MevDetection {
    /// 检测类型
    pub detection_type: String,
    /// 检测参数
    pub detection_params: String,
    /// 检测阈值
    pub detection_threshold: u64,
}

/// 反套利保护结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct AntiArbitrageProtection {
    /// 保护策略
    pub protection_strategy: String,
    /// 保护阈值
    pub protection_threshold: u64,
    /// 保护参数
    pub protection_params: String,
}

/// MEV保护类型枚举
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub enum MevProtectionType {
    /// 时间保护
    TimeProtection,
    /// 价格保护
    PriceProtection,
    /// 流动性保护
    LiquidityProtection,
    /// 组合保护
    CombinedProtection,
}

/// MEV保护结果结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct MevProtectionResult {
    /// MEV是否被检测到
    pub mev_detected: bool,
    /// 保护是否被应用
    pub protection_applied: bool,
    /// 保护时间
    pub protection_time: i64,
    /// 保护费用
    pub protection_fee: u64,
} 