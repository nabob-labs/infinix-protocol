//! 加密货币 (Crypto) 拆分指令
//! 
//! 本模块实现加密货币资产的拆分功能，支持多种拆分策略、算法执行、事件记录等。
//! 严格遵循最小功能单元原则，确保每个函数职责单一。
//! 
//! ## 功能特性
//! - 多种拆分策略：支持等分拆分、比例拆分、自定义拆分等
//! - 算法执行：支持算法交易和智能拆分
//! - 策略集成：支持多种拆分策略
//! - 事件记录：完整的审计追踪
//! - 状态更新：原子性操作保证数据一致性

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams, SplitParams};
use crate::services::crypto_service::CryptoService;
use crate::events::asset_event::AssetSplit;
use crate::validation::business::validate_split_params;
use crate::core::security::check_authority_permission;

/// 加密货币拆分指令账户上下文
/// 
/// 定义拆分操作所需的所有账户，包括：
/// - crypto_asset: 源加密货币资产账户（可变）
/// - target_assets: 目标资产账户集合（可变）
/// - authority: 操作权限账户（签名者）
/// - system_program: 系统程序（用于账户管理）
#[derive(Accounts)]
#[instruction(params: SplitParams)]
pub struct SplitCrypto<'info> {
    /// 源加密货币资产账户，需要可变权限以扣减余额
    #[account(
        mut,
        seeds = [b"crypto", crypto_asset.key().as_ref()],
        bump,
        constraint = crypto_asset.asset_type == AssetType::Crypto @ crate::errors::AssetError::InvalidAssetType,
        constraint = crypto_asset.balance >= params.split_amount @ crate::errors::AssetError::InsufficientBalance
    )]
    pub crypto_asset: Account<'info, crate::account_models::asset::Asset>,
    
    /// 目标资产账户集合，需要可变权限以增加余额
    #[account(
        mut,
        seeds = [b"target", target_assets.key().as_ref()],
        bump,
        constraint = target_assets.asset_type == AssetType::Crypto @ crate::errors::AssetError::InvalidAssetType
    )]
    pub target_assets: Account<'info, crate::account_models::asset::Asset>,
    
    /// 操作权限账户，必须是签名者
    #[account(
        constraint = check_authority_permission(&authority.key(), &crypto_asset.authority) @ crate::errors::SecurityError::Unauthorized
    )]
    pub authority: Signer<'info>,
    
    /// 系统程序，用于账户管理
    pub system_program: Program<'info, System>,
    
    /// 时钟账户，用于时间戳记录
    pub clock: Sysvar<'info, Clock>,
}

/// 加密货币拆分指令实现
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文，自动校验账户权限与生命周期
/// - `params`: 拆分参数，包含拆分数量、策略等
/// - `exec_params`: 可选算法执行参数，支持算法热插拔
/// - `strategy_params`: 可选策略参数，支持多策略扩展
/// 
/// ## 返回值
/// - `anchor_lang::Result<()>`: Anchor标准返回类型，表示指令执行成功或失败
/// 
/// ## 安全性
/// - Anchor自动校验账户权限、生命周期、PDA
/// - 业务层校验拆分参数合法性
/// - 余额充足性检查
/// - 权限验证机制
/// - 完整的事件记录和审计追踪
pub fn split_crypto(
    ctx: Context<SplitCrypto>,
    params: SplitParams,
    exec_params: Option<ExecutionParams>,
    strategy_params: Option<StrategyParams>
) -> anchor_lang::Result<()> {
    // === 1. 参数校验 ===
    // 校验拆分参数合法性
    validate_split_params(&params)?;
    
    // 检查源资产余额充足性
    require!(
        ctx.accounts.crypto_asset.balance >= params.split_amount,
        crate::errors::AssetError::InsufficientBalance
    );
    
    // === 2. 权限校验 ===
    // 检查操作权限
    require!(
        ctx.accounts.authority.key() == ctx.accounts.crypto_asset.authority,
        crate::errors::SecurityError::Unauthorized
    );
    
    // === 3. 业务逻辑执行 ===
    // 创建加密货币服务实例
    let crypto_service = CryptoService::new();
    
    // 记录拆分前的余额
    let crypto_balance_before = ctx.accounts.crypto_asset.balance;
    let target_balance_before = ctx.accounts.target_assets.balance;
    
    // 执行拆分操作
    let actual_split_amount = crypto_service.split(
        &mut ctx.accounts.crypto_asset,
        &mut ctx.accounts.target_assets,
        &params
    )?;
    
    // === 4. 算法执行（如果提供） ===
    if let Some(algo_params) = exec_params {
        crypto_service.execute_algorithm(&mut ctx.accounts.target_assets, algo_params)?;
    }
    
    // === 5. 策略执行（如果提供） ===
    if let Some(strategy) = strategy_params {
        crypto_service.execute_strategy(&mut ctx.accounts.target_assets, strategy)?;
    }
    
    // === 6. 事件记录 ===
    // 发出拆分事件，记录操作详情
    emit!(AssetSplit {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        target_asset: ctx.accounts.target_assets.key(),
        split_amount: params.split_amount,
        actual_split_amount,
        split_strategy: params.strategy.clone(),
        crypto_balance_before,
        crypto_balance_after: ctx.accounts.crypto_asset.balance,
        target_balance_before,
        target_balance_after: ctx.accounts.target_assets.balance,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: exec_params.map(|p| p.into()),
        strategy_params: strategy_params.map(|s| s.into()),
    });
    
    // === 7. 日志记录 ===
    msg!("Crypto asset split successfully: split_amount={}, actual_split_amount={}, strategy={}, authority={}", 
         params.split_amount, actual_split_amount, params.strategy, ctx.accounts.authority.key());
    
    Ok(())
}

/// 批量拆分加密货币指令
/// 
/// 支持一次性拆分多个加密货币，提高操作效率。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `split_orders`: 拆分订单集合
/// - `exec_params`: 可选算法执行参数
/// - `strategy_params`: 可选策略参数
pub fn batch_split_crypto(
    ctx: Context<SplitCrypto>,
    split_orders: Vec<SplitOrder>,
    exec_params: Option<ExecutionParams>,
    strategy_params: Option<StrategyParams>
) -> anchor_lang::Result<()> {
    // === 1. 批量参数校验 ===
    require!(!split_orders.is_empty(), crate::errors::AssetError::EmptyBatch);
    require!(split_orders.len() <= 20, crate::errors::AssetError::BatchTooLarge);
    
    // 计算总拆分数量
    let total_split: u64 = split_orders.iter().map(|order| order.params.split_amount).sum();
    require!(
        ctx.accounts.crypto_asset.balance >= total_split,
        crate::errors::AssetError::InsufficientBalance
    );
    
    // 校验每个拆分订单
    for order in &split_orders {
        validate_split_params(&order.params)?;
    }
    
    // === 2. 批量执行 ===
    let crypto_service = CryptoService::new();
    
    // 执行批量拆分
    let results = crypto_service.batch_split(
        &mut ctx.accounts.crypto_asset,
        &mut ctx.accounts.target_assets,
        split_orders
    )?;
    
    // === 3. 算法和策略执行 ===
    if let Some(params) = exec_params {
        crypto_service.execute_algorithm(&mut ctx.accounts.target_assets, params)?;
    }
    
    if let Some(strategy) = strategy_params {
        crypto_service.execute_strategy(&mut ctx.accounts.target_assets, strategy)?;
    }
    
    // === 4. 事件记录 ===
    let total_actual_split: u64 = results.iter().map(|r| r.split_amount).sum();
    emit!(AssetSplit {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        target_asset: ctx.accounts.target_assets.key(),
        split_amount: total_split,
        actual_split_amount: total_actual_split,
        split_strategy: "batch".to_string(),
        crypto_balance_before: 0,
        crypto_balance_after: ctx.accounts.crypto_asset.balance,
        target_balance_before: 0,
        target_balance_after: ctx.accounts.target_assets.balance,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: exec_params.map(|p| p.into()),
        strategy_params: strategy_params.map(|s| s.into()),
    });
    
    msg!("Batch crypto assets split successfully: total_split={}, total_actual_split={}, batch_size={}", 
         total_split, total_actual_split, results.len());
    
    Ok(())
}

/// 算法拆分加密货币指令
/// 
/// 使用指定算法执行拆分操作，支持智能拆分策略。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `params`: 拆分参数
/// - `algo_params`: 算法参数
pub fn algo_split_crypto(
    ctx: Context<SplitCrypto>,
    params: SplitParams,
    algo_params: ExecutionParams
) -> anchor_lang::Result<()> {
    // === 1. 参数校验 ===
    validate_split_params(&params)?;
    require!(
        ctx.accounts.crypto_asset.balance >= params.split_amount,
        crate::errors::AssetError::InsufficientBalance
    );
    
    // === 2. 算法执行 ===
    let crypto_service = CryptoService::new();
    
    // 使用算法执行拆分
    let result = crypto_service.algo_split(
        &mut ctx.accounts.crypto_asset,
        &mut ctx.accounts.target_assets,
        &params,
        &algo_params
    )?;
    
    // === 3. 事件记录 ===
    emit!(AssetSplit {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        target_asset: ctx.accounts.target_assets.key(),
        split_amount: params.split_amount,
        actual_split_amount: result.split_amount,
        split_strategy: params.strategy,
        crypto_balance_before: 0,
        crypto_balance_after: ctx.accounts.crypto_asset.balance,
        target_balance_before: 0,
        target_balance_after: ctx.accounts.target_assets.balance,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: Some(algo_params.into()),
        strategy_params: None,
    });
    
    msg!("Algorithmic crypto split executed: algorithm={}, split_amount={}, actual_split_amount={}", 
         algo_params.algorithm_name, params.split_amount, result.split_amount);
    
    Ok(())
}

/// 策略拆分加密货币指令
/// 
/// 使用指定策略执行拆分操作，支持多种拆分策略。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `params`: 拆分参数
/// - `strategy_params`: 策略参数
pub fn strategy_split_crypto(
    ctx: Context<SplitCrypto>,
    params: SplitParams,
    strategy_params: StrategyParams
) -> anchor_lang::Result<()> {
    // === 1. 参数校验 ===
    validate_split_params(&params)?;
    require!(
        ctx.accounts.crypto_asset.balance >= params.split_amount,
        crate::errors::AssetError::InsufficientBalance
    );
    
    // === 2. 策略执行 ===
    let crypto_service = CryptoService::new();
    
    // 使用策略执行拆分
    let result = crypto_service.strategy_split(
        &mut ctx.accounts.crypto_asset,
        &mut ctx.accounts.target_assets,
        &params,
        &strategy_params
    )?;
    
    // === 3. 事件记录 ===
    emit!(AssetSplit {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        target_asset: ctx.accounts.target_assets.key(),
        split_amount: params.split_amount,
        actual_split_amount: result.split_amount,
        split_strategy: params.strategy,
        crypto_balance_before: 0,
        crypto_balance_after: ctx.accounts.crypto_asset.balance,
        target_balance_before: 0,
        target_balance_after: ctx.accounts.target_assets.balance,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: None,
        strategy_params: Some(strategy_params.into()),
    });
    
    msg!("Strategy crypto split executed: strategy={}, split_amount={}, actual_split_amount={}", 
         strategy_params.strategy_name, params.split_amount, result.split_amount);
    
    Ok(())
}

/// 拆分订单结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct SplitOrder {
    /// 拆分参数
    pub params: SplitParams,
    /// 订单优先级
    pub priority: u8,
}

/// 拆分结果结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct SplitResult {
    /// 实际拆分数量
    pub split_amount: u64,
    /// 拆分效率
    pub efficiency: u64,
    /// 执行时间
    pub execution_time: i64,
    /// 策略执行结果
    pub strategy_result: String,
} 