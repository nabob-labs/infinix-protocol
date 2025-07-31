//! 加密货币 (Crypto) 冻结指令
//! 
//! 本模块实现加密货币资产的冻结功能，支持紧急冻结、批量冻结、权限管理等。
//! 严格遵循最小功能单元原则，确保每个函数职责单一。
//! 
//! ## 功能特性
//! - 紧急冻结：支持紧急情况下的资产冻结
//! - 批量冻结：支持批量冻结操作
//! - 权限管理：多级权限控制
//! - 事件记录：完整的审计追踪
//! - 状态管理：原子性操作保证数据一致性

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams, FreezeParams};
use crate::services::crypto_service::CryptoService;
use crate::events::asset_event::AssetFrozen;
use crate::validation::business::validate_freeze_params;
use crate::core::security::check_authority_permission;

/// 加密货币冻结指令账户上下文
/// 
/// 定义冻结操作所需的所有账户，包括：
/// - crypto_asset: 加密货币资产账户（可变）
/// - authority: 操作权限账户（签名者）
/// - optional_freeze_authority: 可选冻结权限账户（用于双重控制）
/// - system_program: 系统程序（用于账户管理）
#[derive(Accounts)]
#[instruction(params: FreezeParams)]
pub struct FreezeCrypto<'info> {
    /// 加密货币资产账户，需要可变权限以更新冻结状态
    #[account(
        mut,
        seeds = [b"crypto", crypto_asset.key().as_ref()],
        bump,
        constraint = crypto_asset.asset_type == AssetType::Crypto @ crate::errors::AssetError::InvalidAssetType,
        constraint = !crypto_asset.is_frozen @ crate::errors::AssetError::AlreadyFrozen
    )]
    pub crypto_asset: Account<'info, crate::account_models::asset::Asset>,
    
    /// 操作权限账户，必须是签名者
    #[account(
        constraint = check_authority_permission(&authority.key(), &crypto_asset.authority) @ crate::errors::SecurityError::Unauthorized
    )]
    pub authority: Signer<'info>,
    
    /// 可选冻结权限账户，用于双重控制机制
    #[account(
        constraint = freeze_authority.key() == crypto_asset.freeze_authority.unwrap_or(authority.key()) @ crate::errors::SecurityError::Unauthorized
    )]
    pub freeze_authority: Option<Signer<'info>>,
    
    /// 系统程序，用于账户管理
    pub system_program: Program<'info, System>,
    
    /// 时钟账户，用于时间戳记录
    pub clock: Sysvar<'info, Clock>,
}

/// 加密货币冻结指令实现
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文，自动校验账户权限与生命周期
/// - `params`: 冻结参数，包含冻结原因、期限等
/// - `exec_params`: 可选算法执行参数，支持算法热插拔
/// - `strategy_params`: 可选策略参数，支持多策略扩展
/// 
/// ## 返回值
/// - `anchor_lang::Result<()>`: Anchor标准返回类型，表示指令执行成功或失败
/// 
/// ## 安全性
/// - Anchor自动校验账户权限、生命周期、PDA
/// - 业务层校验冻结参数合法性
/// - 双重权限验证机制
/// - 完整的事件记录和审计追踪
pub fn freeze_crypto(
    ctx: Context<FreezeCrypto>,
    params: FreezeParams,
    exec_params: Option<ExecutionParams>,
    strategy_params: Option<StrategyParams>
) -> anchor_lang::Result<()> {
    // === 1. 参数校验 ===
    // 校验冻结参数合法性
    validate_freeze_params(&params)?;
    
    // 检查资产是否已经冻结
    require!(
        !ctx.accounts.crypto_asset.is_frozen,
        crate::errors::AssetError::AlreadyFrozen
    );
    
    // === 2. 权限校验 ===
    // 检查操作权限
    require!(
        ctx.accounts.authority.key() == ctx.accounts.crypto_asset.authority,
        crate::errors::SecurityError::Unauthorized
    );
    
    // 检查冻结权限（如果设置了双重控制）
    if let Some(freeze_auth) = &ctx.accounts.freeze_authority {
        require!(
            freeze_auth.key() == ctx.accounts.crypto_asset.freeze_authority.unwrap_or(ctx.accounts.authority.key()),
            crate::errors::SecurityError::Unauthorized
        );
    }
    
    // === 3. 业务逻辑执行 ===
    // 创建加密货币服务实例
    let crypto_service = CryptoService::new();
    
    // 记录冻结前的状态
    let balance_before = ctx.accounts.crypto_asset.balance;
    let is_frozen_before = ctx.accounts.crypto_asset.is_frozen;
    
    // 执行冻结操作
    let freeze_result = crypto_service.freeze(
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
    // 发出冻结事件，记录操作详情
    emit!(AssetFrozen {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        freeze_reason: params.reason.clone(),
        freeze_duration: params.duration,
        balance_before,
        balance_after: ctx.accounts.crypto_asset.balance,
        is_frozen_before,
        is_frozen_after: ctx.accounts.crypto_asset.is_frozen,
        authority: ctx.accounts.authority.key(),
        freeze_authority: ctx.accounts.freeze_authority.as_ref().map(|auth| auth.key()),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: exec_params.map(|p| p.into()),
        strategy_params: strategy_params.map(|s| s.into()),
    });
    
    // === 7. 日志记录 ===
    msg!("Crypto asset frozen successfully: reason={}, duration={}, authority={}", 
         params.reason, params.duration, ctx.accounts.authority.key());
    
    Ok(())
}

/// 紧急冻结加密货币指令
/// 
/// 支持紧急情况下的资产冻结，具有更高的权限要求。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `params`: 冻结参数
/// - `emergency_code`: 紧急代码
pub fn emergency_freeze_crypto(
    ctx: Context<FreezeCrypto>,
    params: FreezeParams,
    emergency_code: String
) -> anchor_lang::Result<()> {
    // === 1. 参数校验 ===
    validate_freeze_params(&params)?;
    require!(
        !ctx.accounts.crypto_asset.is_frozen,
        crate::errors::AssetError::AlreadyFrozen
    );
    
    // === 2. 紧急权限校验 ===
    // 检查紧急权限
    require!(
        ctx.accounts.authority.key() == ctx.accounts.crypto_asset.emergency_authority.unwrap_or(ctx.accounts.crypto_asset.authority),
        crate::errors::SecurityError::Unauthorized
    );
    
    // 验证紧急代码
    require!(
        emergency_code == ctx.accounts.crypto_asset.emergency_code.unwrap_or_default(),
        crate::errors::SecurityError::InvalidEmergencyCode
    );
    
    // === 3. 紧急冻结执行 ===
    let crypto_service = CryptoService::new();
    
    // 执行紧急冻结
    let freeze_result = crypto_service.emergency_freeze(
        &mut ctx.accounts.crypto_asset,
        &params,
        &emergency_code
    )?;
    
    // === 4. 事件记录 ===
    emit!(AssetFrozen {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        freeze_reason: format!("EMERGENCY: {}", params.reason),
        freeze_duration: params.duration,
        balance_before: 0,
        balance_after: ctx.accounts.crypto_asset.balance,
        is_frozen_before: false,
        is_frozen_after: ctx.accounts.crypto_asset.is_frozen,
        authority: ctx.accounts.authority.key(),
        freeze_authority: ctx.accounts.freeze_authority.as_ref().map(|auth| auth.key()),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: None,
        strategy_params: None,
    });
    
    msg!("Emergency crypto asset freeze executed: reason={}, emergency_code={}, authority={}", 
         params.reason, emergency_code, ctx.accounts.authority.key());
    
    Ok(())
}

/// 批量冻结加密货币指令
/// 
/// 支持一次性冻结多个加密货币，提高操作效率。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `freeze_orders`: 冻结订单集合
/// - `exec_params`: 可选算法执行参数
/// - `strategy_params`: 可选策略参数
pub fn batch_freeze_crypto(
    ctx: Context<FreezeCrypto>,
    freeze_orders: Vec<FreezeOrder>,
    exec_params: Option<ExecutionParams>,
    strategy_params: Option<StrategyParams>
) -> anchor_lang::Result<()> {
    // === 1. 批量参数校验 ===
    require!(!freeze_orders.is_empty(), crate::errors::AssetError::EmptyBatch);
    require!(freeze_orders.len() <= 10, crate::errors::AssetError::BatchTooLarge);
    
    // 校验每个冻结订单
    for order in &freeze_orders {
        validate_freeze_params(&order.params)?;
    }
    
    // === 2. 批量执行 ===
    let crypto_service = CryptoService::new();
    
    // 执行批量冻结
    let results = crypto_service.batch_freeze(
        &mut ctx.accounts.crypto_asset,
        freeze_orders
    )?;
    
    // === 3. 算法和策略执行 ===
    if let Some(params) = exec_params {
        crypto_service.execute_algorithm(&mut ctx.accounts.crypto_asset, params)?;
    }
    
    if let Some(strategy) = strategy_params {
        crypto_service.execute_strategy(&mut ctx.accounts.crypto_asset, strategy)?;
    }
    
    // === 4. 事件记录 ===
    let total_frozen = results.len() as u64;
    emit!(AssetFrozen {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        freeze_reason: "batch_freeze".to_string(),
        freeze_duration: 0,
        balance_before: 0,
        balance_after: ctx.accounts.crypto_asset.balance,
        is_frozen_before: false,
        is_frozen_after: ctx.accounts.crypto_asset.is_frozen,
        authority: ctx.accounts.authority.key(),
        freeze_authority: ctx.accounts.freeze_authority.as_ref().map(|auth| auth.key()),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: exec_params.map(|p| p.into()),
        strategy_params: strategy_params.map(|s| s.into()),
    });
    
    msg!("Batch crypto assets frozen successfully: total_frozen={}, batch_size={}", 
         total_frozen, results.len());
    
    Ok(())
}

/// 条件冻结加密货币指令
/// 
/// 基于特定条件执行冻结操作，支持智能冻结策略。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `params`: 冻结参数
/// - `condition_params`: 条件参数
pub fn conditional_freeze_crypto(
    ctx: Context<FreezeCrypto>,
    params: FreezeParams,
    condition_params: FreezeCondition
) -> anchor_lang::Result<()> {
    // === 1. 参数校验 ===
    validate_freeze_params(&params)?;
    require!(
        !ctx.accounts.crypto_asset.is_frozen,
        crate::errors::AssetError::AlreadyFrozen
    );
    
    // === 2. 条件检查 ===
    let crypto_service = CryptoService::new();
    
    // 检查冻结条件
    let should_freeze = crypto_service.check_freeze_condition(
        &ctx.accounts.crypto_asset,
        &condition_params
    )?;
    
    require!(should_freeze, crate::errors::AssetError::FreezeConditionNotMet);
    
    // === 3. 条件冻结执行 ===
    let freeze_result = crypto_service.conditional_freeze(
        &mut ctx.accounts.crypto_asset,
        &params,
        &condition_params
    )?;
    
    // === 4. 事件记录 ===
    emit!(AssetFrozen {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        freeze_reason: format!("CONDITIONAL: {}", params.reason),
        freeze_duration: params.duration,
        balance_before: 0,
        balance_after: ctx.accounts.crypto_asset.balance,
        is_frozen_before: false,
        is_frozen_after: ctx.accounts.crypto_asset.is_frozen,
        authority: ctx.accounts.authority.key(),
        freeze_authority: ctx.accounts.freeze_authority.as_ref().map(|auth| auth.key()),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: None,
        strategy_params: None,
    });
    
    msg!("Conditional crypto asset freeze executed: reason={}, condition={}, authority={}", 
         params.reason, condition_params.condition_type, ctx.accounts.authority.key());
    
    Ok(())
}

/// 冻结订单结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct FreezeOrder {
    /// 冻结参数
    pub params: FreezeParams,
    /// 订单优先级
    pub priority: u8,
    /// 冻结类型
    pub freeze_type: FreezeType,
}

/// 冻结条件结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct FreezeCondition {
    /// 条件类型
    pub condition_type: String,
    /// 条件参数
    pub condition_params: String,
    /// 条件阈值
    pub threshold: u64,
    /// 条件操作符
    pub operator: String,
}

/// 冻结类型枚举
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub enum FreezeType {
    /// 正常冻结
    Normal,
    /// 紧急冻结
    Emergency,
    /// 条件冻结
    Conditional,
    /// 临时冻结
    Temporary,
}

/// 冻结结果结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct FreezeResult {
    /// 冻结是否成功
    pub success: bool,
    /// 冻结原因
    pub reason: String,
    /// 冻结时间
    pub freeze_time: i64,
    /// 冻结期限
    pub duration: u64,
} 