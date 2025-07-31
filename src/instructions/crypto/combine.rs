//! 加密货币 (Crypto) 合并指令
//! 
//! 本模块实现加密货币资产的合并功能，支持多种合并策略、算法执行、事件记录等。
//! 严格遵循最小功能单元原则，确保每个函数职责单一。
//! 
//! ## 功能特性
//! - 多种合并策略：支持等权重、市值加权、自定义权重等
//! - 算法执行：支持算法交易和智能合并
//! - 策略集成：支持多种合并策略
//! - 事件记录：完整的审计追踪
//! - 状态更新：原子性操作保证数据一致性

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams, CombineParams};
use crate::services::crypto_service::CryptoService;
use crate::events::asset_event::AssetCombined;
use crate::validation::business::validate_combine_params;
use crate::core::security::check_authority_permission;

/// 加密货币合并指令账户上下文
/// 
/// 定义合并操作所需的所有账户，包括：
/// - crypto_asset: 目标加密货币资产账户（可变）
/// - source_assets: 源资产账户集合（可变）
/// - authority: 操作权限账户（签名者）
/// - system_program: 系统程序（用于账户管理）
#[derive(Accounts)]
#[instruction(params: CombineParams)]
pub struct CombineCrypto<'info> {
    /// 目标加密货币资产账户，需要可变权限以增加余额
    #[account(
        mut,
        seeds = [b"crypto", crypto_asset.key().as_ref()],
        bump,
        constraint = crypto_asset.asset_type == AssetType::Crypto @ crate::errors::AssetError::InvalidAssetType
    )]
    pub crypto_asset: Account<'info, crate::account_models::asset::Asset>,
    
    /// 源资产账户集合，需要可变权限以扣减余额
    #[account(
        mut,
        seeds = [b"source", source_assets.key().as_ref()],
        bump,
        constraint = source_assets.asset_type == AssetType::Crypto @ crate::errors::AssetError::InvalidAssetType,
        constraint = source_assets.balance >= params.source_amount @ crate::errors::AssetError::InsufficientBalance
    )]
    pub source_assets: Account<'info, crate::account_models::asset::Asset>,
    
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

/// 加密货币合并指令实现
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文，自动校验账户权限与生命周期
/// - `params`: 合并参数，包含源资产、数量、策略等
/// - `exec_params`: 可选算法执行参数，支持算法热插拔
/// - `strategy_params`: 可选策略参数，支持多策略扩展
/// 
/// ## 返回值
/// - `anchor_lang::Result<()>`: Anchor标准返回类型，表示指令执行成功或失败
/// 
/// ## 安全性
/// - Anchor自动校验账户权限、生命周期、PDA
/// - 业务层校验合并参数合法性
/// - 余额充足性检查
/// - 权限验证机制
/// - 完整的事件记录和审计追踪
pub fn combine_crypto(
    ctx: Context<CombineCrypto>,
    params: CombineParams,
    exec_params: Option<ExecutionParams>,
    strategy_params: Option<StrategyParams>
) -> anchor_lang::Result<()> {
    // === 1. 参数校验 ===
    // 校验合并参数合法性
    validate_combine_params(&params)?;
    
    // 检查源资产余额充足性
    require!(
        ctx.accounts.source_assets.balance >= params.source_amount,
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
    
    // 记录合并前的余额
    let crypto_balance_before = ctx.accounts.crypto_asset.balance;
    let source_balance_before = ctx.accounts.source_assets.balance;
    
    // 执行合并操作
    let actual_combined_amount = crypto_service.combine(
        &mut ctx.accounts.crypto_asset,
        &mut ctx.accounts.source_assets,
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
    // 发出合并事件，记录操作详情
    emit!(AssetCombined {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        source_asset: ctx.accounts.source_assets.key(),
        source_amount: params.source_amount,
        combined_amount: actual_combined_amount,
        combine_strategy: params.strategy.clone(),
        crypto_balance_before,
        crypto_balance_after: ctx.accounts.crypto_asset.balance,
        source_balance_before,
        source_balance_after: ctx.accounts.source_assets.balance,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: exec_params.map(|p| p.into()),
        strategy_params: strategy_params.map(|s| s.into()),
    });
    
    // === 7. 日志记录 ===
    msg!("Crypto asset combined successfully: source_amount={}, combined_amount={}, strategy={}, authority={}", 
         params.source_amount, actual_combined_amount, params.strategy, ctx.accounts.authority.key());
    
    Ok(())
}

/// 批量合并加密货币指令
/// 
/// 支持一次性合并多个加密货币，提高操作效率。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `combine_orders`: 合并订单集合
/// - `exec_params`: 可选算法执行参数
/// - `strategy_params`: 可选策略参数
pub fn batch_combine_crypto(
    ctx: Context<CombineCrypto>,
    combine_orders: Vec<CombineOrder>,
    exec_params: Option<ExecutionParams>,
    strategy_params: Option<StrategyParams>
) -> anchor_lang::Result<()> {
    // === 1. 批量参数校验 ===
    require!(!combine_orders.is_empty(), crate::errors::AssetError::EmptyBatch);
    require!(combine_orders.len() <= 20, crate::errors::AssetError::BatchTooLarge);
    
    // 计算总源资产数量
    let total_source: u64 = combine_orders.iter().map(|order| order.params.source_amount).sum();
    require!(
        ctx.accounts.source_assets.balance >= total_source,
        crate::errors::AssetError::InsufficientBalance
    );
    
    // 校验每个合并订单
    for order in &combine_orders {
        validate_combine_params(&order.params)?;
    }
    
    // === 2. 批量执行 ===
    let crypto_service = CryptoService::new();
    
    // 执行批量合并
    let results = crypto_service.batch_combine(
        &mut ctx.accounts.crypto_asset,
        &mut ctx.accounts.source_assets,
        combine_orders
    )?;
    
    // === 3. 算法和策略执行 ===
    if let Some(params) = exec_params {
        crypto_service.execute_algorithm(&mut ctx.accounts.crypto_asset, params)?;
    }
    
    if let Some(strategy) = strategy_params {
        crypto_service.execute_strategy(&mut ctx.accounts.crypto_asset, strategy)?;
    }
    
    // === 4. 事件记录 ===
    let total_combined: u64 = results.iter().map(|r| r.combined_amount).sum();
    emit!(AssetCombined {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        source_asset: ctx.accounts.source_assets.key(),
        source_amount: total_source,
        combined_amount: total_combined,
        combine_strategy: "batch".to_string(),
        crypto_balance_before: 0,
        crypto_balance_after: ctx.accounts.crypto_asset.balance,
        source_balance_before: 0,
        source_balance_after: ctx.accounts.source_assets.balance,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: exec_params.map(|p| p.into()),
        strategy_params: strategy_params.map(|s| s.into()),
    });
    
    msg!("Batch crypto assets combined successfully: total_source={}, total_combined={}, batch_size={}", 
         total_source, total_combined, results.len());
    
    Ok(())
}

/// 算法合并加密货币指令
/// 
/// 使用指定算法执行合并操作，支持智能合并策略。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `params`: 合并参数
/// - `algo_params`: 算法参数
pub fn algo_combine_crypto(
    ctx: Context<CombineCrypto>,
    params: CombineParams,
    algo_params: ExecutionParams
) -> anchor_lang::Result<()> {
    // === 1. 参数校验 ===
    validate_combine_params(&params)?;
    require!(
        ctx.accounts.source_assets.balance >= params.source_amount,
        crate::errors::AssetError::InsufficientBalance
    );
    
    // === 2. 算法执行 ===
    let crypto_service = CryptoService::new();
    
    // 使用算法执行合并
    let result = crypto_service.algo_combine(
        &mut ctx.accounts.crypto_asset,
        &mut ctx.accounts.source_assets,
        &params,
        &algo_params
    )?;
    
    // === 3. 事件记录 ===
    emit!(AssetCombined {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        source_asset: ctx.accounts.source_assets.key(),
        source_amount: params.source_amount,
        combined_amount: result.combined_amount,
        combine_strategy: params.strategy,
        crypto_balance_before: 0,
        crypto_balance_after: ctx.accounts.crypto_asset.balance,
        source_balance_before: 0,
        source_balance_after: ctx.accounts.source_assets.balance,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: Some(algo_params.into()),
        strategy_params: None,
    });
    
    msg!("Algorithmic crypto combine executed: algorithm={}, source_amount={}, combined_amount={}", 
         algo_params.algorithm_name, params.source_amount, result.combined_amount);
    
    Ok(())
}

/// 策略合并加密货币指令
/// 
/// 使用指定策略执行合并操作，支持多种合并策略。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `params`: 合并参数
/// - `strategy_params`: 策略参数
pub fn strategy_combine_crypto(
    ctx: Context<CombineCrypto>,
    params: CombineParams,
    strategy_params: StrategyParams
) -> anchor_lang::Result<()> {
    // === 1. 参数校验 ===
    validate_combine_params(&params)?;
    require!(
        ctx.accounts.source_assets.balance >= params.source_amount,
        crate::errors::AssetError::InsufficientBalance
    );
    
    // === 2. 策略执行 ===
    let crypto_service = CryptoService::new();
    
    // 使用策略执行合并
    let result = crypto_service.strategy_combine(
        &mut ctx.accounts.crypto_asset,
        &mut ctx.accounts.source_assets,
        &params,
        &strategy_params
    )?;
    
    // === 3. 事件记录 ===
    emit!(AssetCombined {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        source_asset: ctx.accounts.source_assets.key(),
        source_amount: params.source_amount,
        combined_amount: result.combined_amount,
        combine_strategy: params.strategy,
        crypto_balance_before: 0,
        crypto_balance_after: ctx.accounts.crypto_asset.balance,
        source_balance_before: 0,
        source_balance_after: ctx.accounts.source_assets.balance,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: None,
        strategy_params: Some(strategy_params.into()),
    });
    
    msg!("Strategy crypto combine executed: strategy={}, source_amount={}, combined_amount={}", 
         strategy_params.strategy_name, params.source_amount, result.combined_amount);
    
    Ok(())
}

/// 合并订单结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct CombineOrder {
    /// 合并参数
    pub params: CombineParams,
    /// 订单优先级
    pub priority: u8,
}

/// 合并结果结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct CombineResult {
    /// 实际合并数量
    pub combined_amount: u64,
    /// 合并效率
    pub efficiency: u64,
    /// 执行时间
    pub execution_time: i64,
    /// 策略执行结果
    pub strategy_result: String,
} 