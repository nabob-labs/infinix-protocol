//! 加密货币 (Crypto) 转账指令
//! 
//! 本模块实现加密货币资产的转账功能，支持权限校验、数量验证、事件记录等。
//! 严格遵循最小功能单元原则，确保每个函数职责单一。
//! 
//! ## 功能特性
//! - 权限校验：确保只有授权账户可以执行转账操作
//! - 数量验证：防止转账数量超过余额
//! - 接收方验证：确保接收方账户有效
//! - 事件记录：完整的审计追踪
//! - 状态更新：原子性操作保证数据一致性

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams};
use crate::services::crypto_service::CryptoService;
use crate::events::asset_event::AssetTransferred;
use crate::validation::business::validate_transfer_amount;
use crate::core::security::check_authority_permission;

/// 加密货币转账指令账户上下文
/// 
/// 定义转账操作所需的所有账户，包括：
/// - from_crypto: 源加密货币资产账户（可变）
/// - to_crypto: 目标加密货币资产账户（可变）
/// - authority: 操作权限账户（签名者）
/// - system_program: 系统程序（用于账户管理）
#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct TransferCrypto<'info> {
    /// 源加密货币资产账户，需要可变权限以扣减余额
    #[account(
        mut,
        seeds = [b"crypto", from_crypto.key().as_ref()],
        bump,
        constraint = from_crypto.asset_type == AssetType::Crypto @ crate::errors::AssetError::InvalidAssetType,
        constraint = from_crypto.balance >= amount @ crate::errors::AssetError::InsufficientBalance
    )]
    pub from_crypto: Account<'info, crate::account_models::asset::Asset>,
    
    /// 目标加密货币资产账户，需要可变权限以增加余额
    #[account(
        mut,
        seeds = [b"crypto", to_crypto.key().as_ref()],
        bump,
        constraint = to_crypto.asset_type == AssetType::Crypto @ crate::errors::AssetError::InvalidAssetType,
        constraint = from_crypto.key() != to_crypto.key() @ crate::errors::AssetError::SelfTransfer
    )]
    pub to_crypto: Account<'info, crate::account_models::asset::Asset>,
    
    /// 操作权限账户，必须是签名者
    #[account(
        constraint = check_authority_permission(&authority.key(), &from_crypto.authority) @ crate::errors::SecurityError::Unauthorized
    )]
    pub authority: Signer<'info>,
    
    /// 系统程序，用于账户管理
    pub system_program: Program<'info, System>,
    
    /// 时钟账户，用于时间戳记录
    pub clock: Sysvar<'info, Clock>,
}

/// 加密货币转账指令实现
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文，自动校验账户权限与生命周期
/// - `amount`: 转账数量，必须为正整数且不超过源账户余额
/// - `exec_params`: 可选算法执行参数，支持算法热插拔
/// - `strategy_params`: 可选策略参数，支持多策略扩展
/// 
/// ## 返回值
/// - `anchor_lang::Result<()>`: Anchor标准返回类型，表示指令执行成功或失败
/// 
/// ## 安全性
/// - Anchor自动校验账户权限、生命周期、PDA
/// - 业务层校验转账数量合法性
/// - 余额充足性检查
/// - 防止自转账
/// - 完整的事件记录和审计追踪
pub fn transfer_crypto(
    ctx: Context<TransferCrypto>, 
    amount: u64,
    exec_params: Option<ExecutionParams>,
    strategy_params: Option<StrategyParams>
) -> anchor_lang::Result<()> {
    // === 1. 参数校验 ===
    // 校验转账数量合法性
    validate_transfer_amount(amount)?;
    
    // 检查余额充足性
    require!(
        ctx.accounts.from_crypto.balance >= amount,
        crate::errors::AssetError::InsufficientBalance
    );
    
    // 防止自转账
    require!(
        ctx.accounts.from_crypto.key() != ctx.accounts.to_crypto.key(),
        crate::errors::AssetError::SelfTransfer
    );
    
    // === 2. 权限校验 ===
    // 检查操作权限
    require!(
        ctx.accounts.authority.key() == ctx.accounts.from_crypto.authority,
        crate::errors::SecurityError::Unauthorized
    );
    
    // === 3. 业务逻辑执行 ===
    // 创建加密货币服务实例
    let crypto_service = CryptoService::new();
    
    // 记录转账前的余额
    let from_balance_before = ctx.accounts.from_crypto.balance;
    let to_balance_before = ctx.accounts.to_crypto.balance;
    
    // 执行转账操作
    crypto_service.transfer(
        &mut ctx.accounts.from_crypto,
        &mut ctx.accounts.to_crypto,
        amount
    )?;
    
    // === 4. 算法执行（如果提供） ===
    if let Some(params) = exec_params {
        crypto_service.execute_algorithm(&mut ctx.accounts.from_crypto, params.clone())?;
        crypto_service.execute_algorithm(&mut ctx.accounts.to_crypto, params)?;
    }
    
    // === 5. 策略执行（如果提供） ===
    if let Some(strategy) = strategy_params {
        crypto_service.execute_strategy(&mut ctx.accounts.from_crypto, strategy.clone())?;
        crypto_service.execute_strategy(&mut ctx.accounts.to_crypto, strategy)?;
    }
    
    // === 6. 事件记录 ===
    // 发出转账事件，记录操作详情
    emit!(AssetTransferred {
        asset_id: ctx.accounts.from_crypto.key(),
        asset_type: AssetType::Crypto,
        from_account: ctx.accounts.from_crypto.key(),
        to_account: ctx.accounts.to_crypto.key(),
        amount,
        from_balance_before,
        from_balance_after: ctx.accounts.from_crypto.balance,
        to_balance_before,
        to_balance_after: ctx.accounts.to_crypto.balance,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: exec_params.map(|p| p.into()),
        strategy_params: strategy_params.map(|s| s.into()),
    });
    
    // === 7. 日志记录 ===
    msg!("Crypto asset transferred successfully: amount={}, from={}, to={}, authority={}", 
         amount, ctx.accounts.from_crypto.key(), ctx.accounts.to_crypto.key(), ctx.accounts.authority.key());
    
    Ok(())
}

/// 批量转账加密货币指令
/// 
/// 支持一次性向多个账户转账加密货币，提高操作效率。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `transfers`: 转账信息集合
/// - `exec_params`: 可选算法执行参数
/// - `strategy_params`: 可选策略参数
pub fn batch_transfer_crypto(
    ctx: Context<TransferCrypto>,
    transfers: Vec<TransferInfo>,
    exec_params: Option<ExecutionParams>,
    strategy_params: Option<StrategyParams>
) -> anchor_lang::Result<()> {
    // === 1. 批量参数校验 ===
    require!(!transfers.is_empty(), crate::errors::AssetError::EmptyBatch);
    require!(transfers.len() <= 50, crate::errors::AssetError::BatchTooLarge);
    
    // 校验每个转账信息
    let total_amount: u64 = transfers.iter().map(|t| t.amount).sum();
    require!(
        ctx.accounts.from_crypto.balance >= total_amount,
        crate::errors::AssetError::InsufficientBalance
    );
    
    for transfer in &transfers {
        validate_transfer_amount(transfer.amount)?;
    }
    
    // === 2. 批量执行 ===
    let crypto_service = CryptoService::new();
    
    // 执行批量转账
    crypto_service.batch_transfer(&mut ctx.accounts.from_crypto, transfers)?;
    
    // === 3. 算法和策略执行 ===
    if let Some(params) = exec_params {
        crypto_service.execute_algorithm(&mut ctx.accounts.from_crypto, params)?;
    }
    
    if let Some(strategy) = strategy_params {
        crypto_service.execute_strategy(&mut ctx.accounts.from_crypto, strategy)?;
    }
    
    // === 4. 事件记录 ===
    emit!(AssetTransferred {
        asset_id: ctx.accounts.from_crypto.key(),
        asset_type: AssetType::Crypto,
        from_account: ctx.accounts.from_crypto.key(),
        to_account: ctx.accounts.to_crypto.key(),
        amount: total_amount,
        from_balance_before: 0, // 批量操作中不记录详细余额
        from_balance_after: ctx.accounts.from_crypto.balance,
        to_balance_before: 0,
        to_balance_after: ctx.accounts.to_crypto.balance,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: exec_params.map(|p| p.into()),
        strategy_params: strategy_params.map(|s| s.into()),
    });
    
    msg!("Batch crypto assets transferred successfully: total_amount={}, batch_size={}", 
         total_amount, transfers.len());
    
    Ok(())
}

/// 转账信息结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct TransferInfo {
    /// 目标账户公钥
    pub to_account: Pubkey,
    /// 转账数量
    pub amount: u64,
    /// 转账备注（可选）
    pub memo: Option<String>,
}

/// 跨链转账加密货币指令
/// 
/// 支持跨链转账加密货币，需要特殊的跨链权限。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `amount`: 转账数量
/// - `target_chain`: 目标链标识
/// - `target_address`: 目标地址
pub fn cross_chain_transfer_crypto(
    ctx: Context<TransferCrypto>,
    amount: u64,
    target_chain: String,
    target_address: String
) -> anchor_lang::Result<()> {
    // === 1. 跨链权限校验 ===
    require!(
        ctx.accounts.authority.key() == ctx.accounts.from_crypto.cross_chain_authority,
        crate::errors::SecurityError::Unauthorized
    );
    
    // === 2. 参数校验 ===
    validate_transfer_amount(amount)?;
    require!(
        ctx.accounts.from_crypto.balance >= amount,
        crate::errors::AssetError::InsufficientBalance
    );
    
    // === 3. 执行跨链转账 ===
    let crypto_service = CryptoService::new();
    crypto_service.cross_chain_transfer(
        &mut ctx.accounts.from_crypto,
        amount,
        &target_chain,
        &target_address
    )?;
    
    // === 4. 事件记录 ===
    emit!(AssetTransferred {
        asset_id: ctx.accounts.from_crypto.key(),
        asset_type: AssetType::Crypto,
        from_account: ctx.accounts.from_crypto.key(),
        to_account: ctx.accounts.to_crypto.key(),
        amount,
        from_balance_before: 0,
        from_balance_after: ctx.accounts.from_crypto.balance,
        to_balance_before: 0,
        to_balance_after: 0, // 跨链转账中目标余额未知
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: None,
        strategy_params: None,
    });
    
    msg!("Cross-chain crypto asset transfer: amount={}, target_chain={}, target_address={}, authority={}", 
         amount, target_chain, target_address, ctx.accounts.authority.key());
    
    Ok(())
} 