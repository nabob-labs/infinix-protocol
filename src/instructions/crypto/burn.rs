//! 加密货币 (Crypto) 销毁指令
//! 
//! 本模块实现加密货币资产的销毁功能，支持权限校验、数量验证、事件记录等。
//! 严格遵循最小功能单元原则，确保每个函数职责单一。
//! 
//! ## 功能特性
//! - 权限校验：确保只有授权账户可以执行销毁操作
//! - 数量验证：防止销毁数量超过余额
//! - 事件记录：完整的审计追踪
//! - 状态更新：原子性操作保证数据一致性

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams};
use crate::services::crypto_service::CryptoService;
use crate::events::asset_event::AssetBurned;
use crate::validation::business::validate_burn_amount;
use crate::core::security::check_authority_permission;

/// 加密货币销毁指令账户上下文
/// 
/// 定义销毁操作所需的所有账户，包括：
/// - crypto_asset: 加密货币资产账户（可变）
/// - authority: 操作权限账户（签名者）
/// - burn_authority: 销毁权限账户（可选）
/// - system_program: 系统程序（用于账户管理）
#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct BurnCrypto<'info> {
    /// 加密货币资产账户，需要可变权限以更新余额
    #[account(
        mut,
        seeds = [b"crypto", crypto_asset.key().as_ref()],
        bump,
        constraint = crypto_asset.asset_type == AssetType::Crypto @ crate::errors::AssetError::InvalidAssetType,
        constraint = crypto_asset.balance >= amount @ crate::errors::AssetError::InsufficientBalance
    )]
    pub crypto_asset: Account<'info, crate::account_models::asset::Asset>,
    
    /// 操作权限账户，必须是签名者
    #[account(
        constraint = check_authority_permission(&authority.key(), &crypto_asset.authority) @ crate::errors::SecurityError::Unauthorized
    )]
    pub authority: Signer<'info>,
    
    /// 销毁权限账户（可选），用于双重权限控制
    #[account(
        constraint = burn_authority.key() == crypto_asset.burn_authority @ crate::errors::SecurityError::InvalidBurnAuthority,
        required = false
    )]
    pub burn_authority: Option<Signer<'info>>,
    
    /// 系统程序，用于账户管理
    pub system_program: Program<'info, System>,
    
    /// 时钟账户，用于时间戳记录
    pub clock: Sysvar<'info, Clock>,
}

/// 加密货币销毁指令实现
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文，自动校验账户权限与生命周期
/// - `amount`: 销毁数量，必须为正整数且不超过当前余额
/// - `exec_params`: 可选算法执行参数，支持算法热插拔
/// - `strategy_params`: 可选策略参数，支持多策略扩展
/// 
/// ## 返回值
/// - `anchor_lang::Result<()>`: Anchor标准返回类型，表示指令执行成功或失败
/// 
/// ## 安全性
/// - Anchor自动校验账户权限、生命周期、PDA
/// - 业务层校验销毁数量合法性
/// - 余额充足性检查
/// - 权限双重校验机制
/// - 完整的事件记录和审计追踪
pub fn burn_crypto(
    ctx: Context<BurnCrypto>, 
    amount: u64,
    exec_params: Option<ExecutionParams>,
    strategy_params: Option<StrategyParams>
) -> anchor_lang::Result<()> {
    // === 1. 参数校验 ===
    // 校验销毁数量合法性
    validate_burn_amount(amount)?;
    
    // 检查余额充足性
    require!(
        ctx.accounts.crypto_asset.balance >= amount,
        crate::errors::AssetError::InsufficientBalance
    );
    
    // === 2. 权限校验 ===
    // 检查操作权限
    require!(
        ctx.accounts.authority.key() == ctx.accounts.crypto_asset.authority,
        crate::errors::SecurityError::Unauthorized
    );
    
    // 如果提供了销毁权限账户，进行双重校验
    if let Some(burn_auth) = &ctx.accounts.burn_authority {
        require!(
            burn_auth.key() == ctx.accounts.crypto_asset.burn_authority,
            crate::errors::SecurityError::InvalidBurnAuthority
        );
    }
    
    // === 3. 业务逻辑执行 ===
    // 创建加密货币服务实例
    let crypto_service = CryptoService::new();
    
    // 执行销毁操作
    crypto_service.burn(&mut ctx.accounts.crypto_asset, amount)?;
    
    // === 4. 算法执行（如果提供） ===
    if let Some(params) = exec_params {
        crypto_service.execute_algorithm(&mut ctx.accounts.crypto_asset, params)?;
    }
    
    // === 5. 策略执行（如果提供） ===
    if let Some(strategy) = strategy_params {
        crypto_service.execute_strategy(&mut ctx.accounts.crypto_asset, strategy)?;
    }
    
    // === 6. 事件记录 ===
    // 发出销毁事件，记录操作详情
    emit!(AssetBurned {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        amount,
        authority: ctx.accounts.authority.key(),
        burn_authority: ctx.accounts.burn_authority.as_ref().map(|auth| auth.key()),
        remaining_balance: ctx.accounts.crypto_asset.balance,
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: exec_params.map(|p| p.into()),
        strategy_params: strategy_params.map(|s| s.into()),
    });
    
    // === 7. 日志记录 ===
    msg!("Crypto asset burned successfully: amount={}, authority={}, remaining_balance={}", 
         amount, ctx.accounts.authority.key(), ctx.accounts.crypto_asset.balance);
    
    Ok(())
}

/// 批量销毁加密货币指令
/// 
/// 支持一次性销毁多个加密货币资产，提高操作效率。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `amounts`: 销毁数量集合
/// - `exec_params`: 可选算法执行参数
/// - `strategy_params`: 可选策略参数
pub fn batch_burn_crypto(
    ctx: Context<BurnCrypto>,
    amounts: Vec<u64>,
    exec_params: Option<ExecutionParams>,
    strategy_params: Option<StrategyParams>
) -> anchor_lang::Result<()> {
    // === 1. 批量参数校验 ===
    require!(!amounts.is_empty(), crate::errors::AssetError::EmptyBatch);
    require!(amounts.len() <= 100, crate::errors::AssetError::BatchTooLarge);
    
    // 校验每个销毁数量
    for amount in &amounts {
        validate_burn_amount(*amount)?;
    }
    
    // 检查总余额充足性
    let total_amount: u64 = amounts.iter().sum();
    require!(
        ctx.accounts.crypto_asset.balance >= total_amount,
        crate::errors::AssetError::InsufficientBalance
    );
    
    // === 2. 批量执行 ===
    let crypto_service = CryptoService::new();
    
    // 执行批量销毁
    crypto_service.batch_burn(&mut ctx.accounts.crypto_asset, amounts)?;
    
    // === 3. 算法和策略执行 ===
    if let Some(params) = exec_params {
        crypto_service.execute_algorithm(&mut ctx.accounts.crypto_asset, params)?;
    }
    
    if let Some(strategy) = strategy_params {
        crypto_service.execute_strategy(&mut ctx.accounts.crypto_asset, strategy)?;
    }
    
    // === 4. 事件记录 ===
    emit!(AssetBurned {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        amount: total_amount,
        authority: ctx.accounts.authority.key(),
        burn_authority: ctx.accounts.burn_authority.as_ref().map(|auth| auth.key()),
        remaining_balance: ctx.accounts.crypto_asset.balance,
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: exec_params.map(|p| p.into()),
        strategy_params: strategy_params.map(|s| s.into()),
    });
    
    msg!("Batch crypto assets burned successfully: total_amount={}, batch_size={}, remaining_balance={}", 
         total_amount, amounts.len(), ctx.accounts.crypto_asset.balance);
    
    Ok(())
}

/// 紧急销毁加密货币指令
/// 
/// 在紧急情况下，管理员可以销毁指定数量的加密货币。
/// 需要特殊的紧急权限。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `amount`: 销毁数量
/// - `reason`: 紧急销毁原因
pub fn emergency_burn_crypto(
    ctx: Context<BurnCrypto>,
    amount: u64,
    reason: String
) -> anchor_lang::Result<()> {
    // === 1. 紧急权限校验 ===
    require!(
        ctx.accounts.authority.key() == ctx.accounts.crypto_asset.emergency_authority,
        crate::errors::SecurityError::Unauthorized
    );
    
    // === 2. 参数校验 ===
    validate_burn_amount(amount)?;
    require!(
        ctx.accounts.crypto_asset.balance >= amount,
        crate::errors::AssetError::InsufficientBalance
    );
    
    // === 3. 执行销毁 ===
    let crypto_service = CryptoService::new();
    crypto_service.burn(&mut ctx.accounts.crypto_asset, amount)?;
    
    // === 4. 事件记录 ===
    emit!(AssetBurned {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        amount,
        authority: ctx.accounts.authority.key(),
        burn_authority: None,
        remaining_balance: ctx.accounts.crypto_asset.balance,
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: None,
        strategy_params: None,
    });
    
    msg!("Emergency crypto asset burn: amount={}, reason={}, authority={}", 
         amount, reason, ctx.accounts.authority.key());
    
    Ok(())
} 