//! 加密货币 (Crypto) 解冻指令
//! 
//! 本模块实现加密货币资产的解冻功能，支持批量解冻、条件解冻、权限管理等。
//! 严格遵循最小功能单元原则，确保每个函数职责单一。
//! 
//! ## 功能特性
//! - 批量解冻：支持批量解冻操作
//! - 条件解冻：基于特定条件执行解冻
//! - 权限管理：多级权限控制
//! - 事件记录：完整的审计追踪
//! - 状态管理：原子性操作保证数据一致性

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams, UnfreezeParams};
use crate::services::crypto_service::CryptoService;
use crate::events::asset_event::AssetUnfrozen;
use crate::validation::business::validate_unfreeze_params;
use crate::core::security::check_authority_permission;

/// 加密货币解冻指令账户上下文
/// 
/// 定义解冻操作所需的所有账户，包括：
/// - crypto_asset: 加密货币资产账户（可变）
/// - authority: 操作权限账户（签名者）
/// - optional_unfreeze_authority: 可选解冻权限账户（用于双重控制）
/// - system_program: 系统程序（用于账户管理）
#[derive(Accounts)]
#[instruction(params: UnfreezeParams)]
pub struct UnfreezeCrypto<'info> {
    /// 加密货币资产账户，需要可变权限以更新解冻状态
    #[account(
        mut,
        seeds = [b"crypto", crypto_asset.key().as_ref()],
        bump,
        constraint = crypto_asset.asset_type == AssetType::Crypto @ crate::errors::AssetError::InvalidAssetType,
        constraint = crypto_asset.is_frozen @ crate::errors::AssetError::NotFrozen
    )]
    pub crypto_asset: Account<'info, crate::account_models::asset::Asset>,
    
    /// 操作权限账户，必须是签名者
    #[account(
        constraint = check_authority_permission(&authority.key(), &crypto_asset.authority) @ crate::errors::SecurityError::Unauthorized
    )]
    pub authority: Signer<'info>,
    
    /// 可选解冻权限账户，用于双重控制机制
    #[account(
        constraint = unfreeze_authority.key() == crypto_asset.unfreeze_authority.unwrap_or(authority.key()) @ crate::errors::SecurityError::Unauthorized
    )]
    pub unfreeze_authority: Option<Signer<'info>>,
    
    /// 系统程序，用于账户管理
    pub system_program: Program<'info, System>,
    
    /// 时钟账户，用于时间戳记录
    pub clock: Sysvar<'info, Clock>,
}

/// 加密货币解冻指令实现
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文，自动校验账户权限与生命周期
/// - `params`: 解冻参数，包含解冻原因、条件等
/// - `exec_params`: 可选算法执行参数，支持算法热插拔
/// - `strategy_params`: 可选策略参数，支持多策略扩展
/// 
/// ## 返回值
/// - `anchor_lang::Result<()>`: Anchor标准返回类型，表示指令执行成功或失败
/// 
/// ## 安全性
/// - Anchor自动校验账户权限、生命周期、PDA
/// - 业务层校验解冻参数合法性
/// - 双重权限验证机制
/// - 完整的事件记录和审计追踪
pub fn unfreeze_crypto(
    ctx: Context<UnfreezeCrypto>,
    params: UnfreezeParams,
    exec_params: Option<ExecutionParams>,
    strategy_params: Option<StrategyParams>
) -> anchor_lang::Result<()> {
    // === 1. 参数校验 ===
    // 校验解冻参数合法性
    validate_unfreeze_params(&params)?;
    
    // 检查资产是否已经冻结
    require!(
        ctx.accounts.crypto_asset.is_frozen,
        crate::errors::AssetError::NotFrozen
    );
    
    // === 2. 权限校验 ===
    // 检查操作权限
    require!(
        ctx.accounts.authority.key() == ctx.accounts.crypto_asset.authority,
        crate::errors::SecurityError::Unauthorized
    );
    
    // 检查解冻权限（如果设置了双重控制）
    if let Some(unfreeze_auth) = &ctx.accounts.unfreeze_authority {
        require!(
            unfreeze_auth.key() == ctx.accounts.crypto_asset.unfreeze_authority.unwrap_or(ctx.accounts.authority.key()),
            crate::errors::SecurityError::Unauthorized
        );
    }
    
    // === 3. 业务逻辑执行 ===
    // 创建加密货币服务实例
    let crypto_service = CryptoService::new();
    
    // 记录解冻前的状态
    let balance_before = ctx.accounts.crypto_asset.balance;
    let is_frozen_before = ctx.accounts.crypto_asset.is_frozen;
    
    // 执行解冻操作
    let unfreeze_result = crypto_service.unfreeze(
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
    // 发出解冻事件，记录操作详情
    emit!(AssetUnfrozen {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        unfreeze_reason: params.reason.clone(),
        balance_before,
        balance_after: ctx.accounts.crypto_asset.balance,
        is_frozen_before,
        is_frozen_after: ctx.accounts.crypto_asset.is_frozen,
        authority: ctx.accounts.authority.key(),
        unfreeze_authority: ctx.accounts.unfreeze_authority.as_ref().map(|auth| auth.key()),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: exec_params.map(|p| p.into()),
        strategy_params: strategy_params.map(|s| s.into()),
    });
    
    // === 7. 日志记录 ===
    msg!("Crypto asset unfrozen successfully: reason={}, authority={}", 
         params.reason, ctx.accounts.authority.key());
    
    Ok(())
}

/// 批量解冻加密货币指令
/// 
/// 支持一次性解冻多个加密货币，提高操作效率。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `unfreeze_orders`: 解冻订单集合
/// - `exec_params`: 可选算法执行参数
/// - `strategy_params`: 可选策略参数
pub fn batch_unfreeze_crypto(
    ctx: Context<UnfreezeCrypto>,
    unfreeze_orders: Vec<UnfreezeOrder>,
    exec_params: Option<ExecutionParams>,
    strategy_params: Option<StrategyParams>
) -> anchor_lang::Result<()> {
    // === 1. 批量参数校验 ===
    require!(!unfreeze_orders.is_empty(), crate::errors::AssetError::EmptyBatch);
    require!(unfreeze_orders.len() <= 10, crate::errors::AssetError::BatchTooLarge);
    
    // 校验每个解冻订单
    for order in &unfreeze_orders {
        validate_unfreeze_params(&order.params)?;
    }
    
    // === 2. 批量执行 ===
    let crypto_service = CryptoService::new();
    
    // 执行批量解冻
    let results = crypto_service.batch_unfreeze(
        &mut ctx.accounts.crypto_asset,
        unfreeze_orders
    )?;
    
    // === 3. 算法和策略执行 ===
    if let Some(params) = exec_params {
        crypto_service.execute_algorithm(&mut ctx.accounts.crypto_asset, params)?;
    }
    
    if let Some(strategy) = strategy_params {
        crypto_service.execute_strategy(&mut ctx.accounts.crypto_asset, strategy)?;
    }
    
    // === 4. 事件记录 ===
    let total_unfrozen = results.len() as u64;
    emit!(AssetUnfrozen {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        unfreeze_reason: "batch_unfreeze".to_string(),
        balance_before: 0,
        balance_after: ctx.accounts.crypto_asset.balance,
        is_frozen_before: true,
        is_frozen_after: ctx.accounts.crypto_asset.is_frozen,
        authority: ctx.accounts.authority.key(),
        unfreeze_authority: ctx.accounts.unfreeze_authority.as_ref().map(|auth| auth.key()),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: exec_params.map(|p| p.into()),
        strategy_params: strategy_params.map(|s| s.into()),
    });
    
    msg!("Batch crypto assets unfrozen successfully: total_unfrozen={}, batch_size={}", 
         total_unfrozen, results.len());
    
    Ok(())
}

/// 条件解冻加密货币指令
/// 
/// 基于特定条件执行解冻操作，支持智能解冻策略。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `params`: 解冻参数
/// - `condition_params`: 条件参数
pub fn conditional_unfreeze_crypto(
    ctx: Context<UnfreezeCrypto>,
    params: UnfreezeParams,
    condition_params: UnfreezeCondition
) -> anchor_lang::Result<()> {
    // === 1. 参数校验 ===
    validate_unfreeze_params(&params)?;
    require!(
        ctx.accounts.crypto_asset.is_frozen,
        crate::errors::AssetError::NotFrozen
    );
    
    // === 2. 条件检查 ===
    let crypto_service = CryptoService::new();
    
    // 检查解冻条件
    let should_unfreeze = crypto_service.check_unfreeze_condition(
        &ctx.accounts.crypto_asset,
        &condition_params
    )?;
    
    require!(should_unfreeze, crate::errors::AssetError::UnfreezeConditionNotMet);
    
    // === 3. 条件解冻执行 ===
    let unfreeze_result = crypto_service.conditional_unfreeze(
        &mut ctx.accounts.crypto_asset,
        &params,
        &condition_params
    )?;
    
    // === 4. 事件记录 ===
    emit!(AssetUnfrozen {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        unfreeze_reason: format!("CONDITIONAL: {}", params.reason),
        balance_before: 0,
        balance_after: ctx.accounts.crypto_asset.balance,
        is_frozen_before: true,
        is_frozen_after: ctx.accounts.crypto_asset.is_frozen,
        authority: ctx.accounts.authority.key(),
        unfreeze_authority: ctx.accounts.unfreeze_authority.as_ref().map(|auth| auth.key()),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: None,
        strategy_params: None,
    });
    
    msg!("Conditional crypto asset unfreeze executed: reason={}, condition={}, authority={}", 
         params.reason, condition_params.condition_type, ctx.accounts.authority.key());
    
    Ok(())
}

/// 自动解冻加密货币指令
/// 
/// 基于时间或其他条件自动执行解冻操作。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `params`: 解冻参数
pub fn auto_unfreeze_crypto(
    ctx: Context<UnfreezeCrypto>,
    params: UnfreezeParams
) -> anchor_lang::Result<()> {
    // === 1. 参数校验 ===
    validate_unfreeze_params(&params)?;
    require!(
        ctx.accounts.crypto_asset.is_frozen,
        crate::errors::AssetError::NotFrozen
    );
    
    // === 2. 自动解冻检查 ===
    let crypto_service = CryptoService::new();
    
    // 检查是否可以自动解冻
    let can_auto_unfreeze = crypto_service.check_auto_unfreeze_eligibility(
        &ctx.accounts.crypto_asset
    )?;
    
    require!(can_auto_unfreeze, crate::errors::AssetError::AutoUnfreezeNotEligible);
    
    // === 3. 自动解冻执行 ===
    let unfreeze_result = crypto_service.auto_unfreeze(
        &mut ctx.accounts.crypto_asset,
        &params
    )?;
    
    // === 4. 事件记录 ===
    emit!(AssetUnfrozen {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        unfreeze_reason: format!("AUTO: {}", params.reason),
        balance_before: 0,
        balance_after: ctx.accounts.crypto_asset.balance,
        is_frozen_before: true,
        is_frozen_after: ctx.accounts.crypto_asset.is_frozen,
        authority: ctx.accounts.authority.key(),
        unfreeze_authority: ctx.accounts.unfreeze_authority.as_ref().map(|auth| auth.key()),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: None,
        strategy_params: None,
    });
    
    msg!("Auto crypto asset unfreeze executed: reason={}, authority={}", 
         params.reason, ctx.accounts.authority.key());
    
    Ok(())
}

/// 解冻订单结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct UnfreezeOrder {
    /// 解冻参数
    pub params: UnfreezeParams,
    /// 订单优先级
    pub priority: u8,
    /// 解冻类型
    pub unfreeze_type: UnfreezeType,
}

/// 解冻条件结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct UnfreezeCondition {
    /// 条件类型
    pub condition_type: String,
    /// 条件参数
    pub condition_params: String,
    /// 条件阈值
    pub threshold: u64,
    /// 条件操作符
    pub operator: String,
}

/// 解冻类型枚举
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub enum UnfreezeType {
    /// 正常解冻
    Normal,
    /// 条件解冻
    Conditional,
    /// 自动解冻
    Auto,
    /// 紧急解冻
    Emergency,
}

/// 解冻结果结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct UnfreezeResult {
    /// 解冻是否成功
    pub success: bool,
    /// 解冻原因
    pub reason: String,
    /// 解冻时间
    pub unfreeze_time: i64,
    /// 冻结持续时间
    pub freeze_duration: u64,
} 