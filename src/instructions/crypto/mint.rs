//! 加密货币 (Crypto) 增发指令
//! 
//! 本模块实现加密货币资产的增发功能，支持权限校验、数量验证、事件记录等。
//! 严格遵循最小功能单元原则，确保每个函数职责单一。
//! 
//! ## 功能特性
//! - 权限校验：确保只有授权账户可以执行增发操作
//! - 数量验证：防止溢出和非法数量
//! - 事件记录：完整的审计追踪
//! - 状态更新：原子性操作保证数据一致性

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams};
use crate::services::crypto_service::CryptoService;
use crate::events::asset_event::AssetMinted;
use crate::validation::business::validate_mint_amount;
use crate::core::security::check_authority_permission;

/// 加密货币增发指令账户上下文
/// 
/// 定义增发操作所需的所有账户，包括：
/// - crypto: 加密货币资产账户（可变）
/// - authority: 操作权限账户（签名者）
/// - mint_authority: 增发权限账户（可选）
/// - system_program: 系统程序（用于账户创建）
#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct MintCrypto<'info> {
    /// 加密货币资产账户，需要可变权限以更新余额
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
    
    /// 增发权限账户（可选），用于双重权限控制
    #[account(
        constraint = mint_authority.key() == crypto_asset.mint_authority @ crate::errors::SecurityError::InvalidMintAuthority,
        required = false
    )]
    pub mint_authority: Option<Signer<'info>>,
    
    /// 系统程序，用于账户创建和租金支付
    pub system_program: Program<'info, System>,
    
    /// 时钟账户，用于时间戳记录
    pub clock: Sysvar<'info, Clock>,
}

/// 加密货币增发指令实现
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文，自动校验账户权限与生命周期
/// - `amount`: 增发数量，必须为正整数且不超过最大限制
/// - `exec_params`: 可选算法执行参数，支持算法热插拔
/// - `strategy_params`: 可选策略参数，支持多策略扩展
/// 
/// ## 返回值
/// - `anchor_lang::Result<()>`: Anchor标准返回类型，表示指令执行成功或失败
/// 
/// ## 安全性
/// - Anchor自动校验账户权限、生命周期、PDA
/// - 业务层校验增发数量合法性
/// - 权限双重校验机制
/// - 完整的事件记录和审计追踪
pub fn mint_crypto(
    ctx: Context<MintCrypto>, 
    amount: u64,
    exec_params: Option<ExecutionParams>,
    strategy_params: Option<StrategyParams>
) -> anchor_lang::Result<()> {
    // === 1. 参数校验 ===
    // 校验增发数量合法性
    validate_mint_amount(amount)?;
    
    // === 2. 权限校验 ===
    // 检查操作权限
    require!(
        ctx.accounts.authority.key() == ctx.accounts.crypto_asset.authority,
        crate::errors::SecurityError::Unauthorized
    );
    
    // 如果提供了增发权限账户，进行双重校验
    if let Some(mint_auth) = &ctx.accounts.mint_authority {
        require!(
            mint_auth.key() == ctx.accounts.crypto_asset.mint_authority,
            crate::errors::SecurityError::InvalidMintAuthority
        );
    }
    
    // === 3. 业务逻辑执行 ===
    // 创建加密货币服务实例
    let crypto_service = CryptoService::new();
    
    // 执行增发操作
    crypto_service.mint(&mut ctx.accounts.crypto_asset, amount)?;
    
    // === 4. 算法执行（如果提供） ===
    if let Some(params) = exec_params {
        crypto_service.execute_algorithm(&mut ctx.accounts.crypto_asset, params)?;
    }
    
    // === 5. 策略执行（如果提供） ===
    if let Some(strategy) = strategy_params {
        crypto_service.execute_strategy(&mut ctx.accounts.crypto_asset, strategy)?;
    }
    
    // === 6. 事件记录 ===
    // 发出增发事件，记录操作详情
    emit!(AssetMinted {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        amount,
        authority: ctx.accounts.authority.key(),
        mint_authority: ctx.accounts.mint_authority.as_ref().map(|auth| auth.key()),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: exec_params.map(|p| p.into()),
        strategy_params: strategy_params.map(|s| s.into()),
    });
    
    // === 7. 日志记录 ===
    msg!("Crypto asset minted successfully: amount={}, authority={}", 
         amount, ctx.accounts.authority.key());
    
    Ok(())
}

/// 批量增发加密货币指令
/// 
/// 支持一次性增发多个加密货币资产，提高操作效率。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `amounts`: 增发数量集合
/// - `exec_params`: 可选算法执行参数
/// - `strategy_params`: 可选策略参数
pub fn batch_mint_crypto(
    ctx: Context<MintCrypto>,
    amounts: Vec<u64>,
    exec_params: Option<ExecutionParams>,
    strategy_params: Option<StrategyParams>
) -> anchor_lang::Result<()> {
    // === 1. 批量参数校验 ===
    require!(!amounts.is_empty(), crate::errors::AssetError::EmptyBatch);
    require!(amounts.len() <= 100, crate::errors::AssetError::BatchTooLarge);
    
    // 校验每个增发数量
    for amount in &amounts {
        validate_mint_amount(*amount)?;
    }
    
    // === 2. 批量执行 ===
    let crypto_service = CryptoService::new();
    let total_amount: u64 = amounts.iter().sum();
    
    // 执行批量增发
    crypto_service.batch_mint(&mut ctx.accounts.crypto_asset, amounts)?;
    
    // === 3. 算法和策略执行 ===
    if let Some(params) = exec_params {
        crypto_service.execute_algorithm(&mut ctx.accounts.crypto_asset, params)?;
    }
    
    if let Some(strategy) = strategy_params {
        crypto_service.execute_strategy(&mut ctx.accounts.crypto_asset, strategy)?;
    }
    
    // === 4. 事件记录 ===
    emit!(AssetMinted {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        amount: total_amount,
        authority: ctx.accounts.authority.key(),
        mint_authority: ctx.accounts.mint_authority.as_ref().map(|auth| auth.key()),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: exec_params.map(|p| p.into()),
        strategy_params: strategy_params.map(|s| s.into()),
    });
    
    msg!("Batch crypto assets minted successfully: total_amount={}, batch_size={}", 
         total_amount, amounts.len());
    
    Ok(())
} 