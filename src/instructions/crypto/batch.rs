//! 加密货币 (Crypto) 批量操作指令
//! 
//! 本模块实现加密货币资产的批量操作功能，支持批量交易、批量管理、批量处理等。
//! 严格遵循最小功能单元原则，确保每个函数职责单一。
//! 
//! ## 功能特性
//! - 批量交易：支持多种批量交易策略
//! - 批量管理：支持批量资产管理
//! - 批量处理：支持批量数据处理
//! - 事件记录：完整的审计追踪
//! - 状态管理：原子性操作保证数据一致性

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams, BatchParams};
use crate::services::crypto_service::CryptoService;
use crate::events::asset_event::{AssetBatchTraded, AssetBatchProcessed};
use crate::validation::business::validate_batch_params;
use crate::core::security::check_authority_permission;

/// 加密货币批量操作指令账户上下文
/// 
/// 定义批量操作所需的所有账户，包括：
/// - crypto_assets: 加密货币资产账户集合（可变）
/// - authority: 操作权限账户（签名者）
/// - dex_program: DEX程序（用于交易）
/// - oracle_program: 预言机程序（用于价格数据）
/// - system_program: 系统程序（用于账户管理）
#[derive(Accounts)]
#[instruction(params: BatchParams)]
pub struct BatchCrypto<'info> {
    /// 加密货币资产账户集合，需要可变权限以更新状态
    #[account(
        mut,
        seeds = [b"crypto_batch", crypto_assets.key().as_ref()],
        bump,
        constraint = crypto_assets.asset_type == AssetType::Crypto @ crate::errors::AssetError::InvalidAssetType
    )]
    pub crypto_assets: Account<'info, crate::account_models::asset::Asset>,
    
    /// 操作权限账户，必须是签名者
    #[account(
        constraint = check_authority_permission(&authority.key(), &crypto_assets.authority) @ crate::errors::SecurityError::Unauthorized
    )]
    pub authority: Signer<'info>,
    
    /// DEX程序，用于交易执行
    pub dex_program: Program<'info, crate::dex::traits::DexAdapterTrait>,
    
    /// 预言机程序，用于价格数据
    pub oracle_program: Program<'info, crate::oracles::traits::OracleAdapterTrait>,
    
    /// 系统程序，用于账户管理
    pub system_program: Program<'info, System>,
    
    /// 时钟账户，用于时间戳记录
    pub clock: Sysvar<'info, Clock>,
}

/// 加密货币批量交易指令实现
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文，自动校验账户权限与生命周期
/// - `params`: 批量参数，包含交易类型、数量等
/// - `exec_params`: 可选算法执行参数，支持算法热插拔
/// - `strategy_params`: 可选策略参数，支持多策略扩展
/// 
/// ## 返回值
/// - `anchor_lang::Result<()>`: Anchor标准返回类型，表示指令执行成功或失败
/// 
/// ## 安全性
/// - Anchor自动校验账户权限、生命周期、PDA
/// - 业务层校验批量参数合法性
/// - 权限验证机制
/// - 完整的事件记录和审计追踪
pub fn batch_trade_crypto(
    ctx: Context<BatchCrypto>,
    params: BatchParams,
    exec_params: Option<ExecutionParams>,
    strategy_params: Option<StrategyParams>
) -> anchor_lang::Result<()> {
    // === 1. 参数校验 ===
    // 校验批量参数合法性
    validate_batch_params(&params)?;
    
    // 检查批量操作大小限制
    require!(
        params.batch_size <= 50,
        crate::errors::AssetError::BatchTooLarge
    );
    
    // === 2. 权限校验 ===
    // 检查操作权限
    require!(
        ctx.accounts.authority.key() == ctx.accounts.crypto_assets.authority,
        crate::errors::SecurityError::Unauthorized
    );
    
    // === 3. 业务逻辑执行 ===
    // 创建加密货币服务实例
    let crypto_service = CryptoService::new();
    
    // 记录批量交易前的状态
    let total_balance_before = ctx.accounts.crypto_assets.balance;
    
    // 执行批量交易操作
    let batch_result = crypto_service.batch_trade(
        &mut ctx.accounts.crypto_assets,
        &params
    )?;
    
    // === 4. 算法执行（如果提供） ===
    if let Some(algo_params) = exec_params {
        crypto_service.execute_algorithm(&mut ctx.accounts.crypto_assets, algo_params)?;
    }
    
    // === 5. 策略执行（如果提供） ===
    if let Some(strategy) = strategy_params {
        crypto_service.execute_strategy(&mut ctx.accounts.crypto_assets, strategy)?;
    }
    
    // === 6. 事件记录 ===
    // 发出批量交易事件，记录操作详情
    emit!(AssetBatchTraded {
        asset_id: ctx.accounts.crypto_assets.key(),
        asset_type: AssetType::Crypto,
        batch_type: params.batch_type.clone(),
        batch_size: params.batch_size,
        total_balance_before,
        total_balance_after: ctx.accounts.crypto_assets.balance,
        trade_count: batch_result.trade_count,
        total_volume: batch_result.total_volume,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: exec_params.map(|p| p.into()),
        strategy_params: strategy_params.map(|s| s.into()),
    });
    
    // === 7. 日志记录 ===
    msg!("Crypto batch trade executed successfully: batch_type={}, batch_size={}, trade_count={}, total_volume={}, authority={}", 
         params.batch_type, params.batch_size, batch_result.trade_count, batch_result.total_volume, ctx.accounts.authority.key());
    
    Ok(())
}

/// 加密货币批量处理指令实现
/// 
/// 支持多种批量处理操作，如批量更新、批量同步等。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `params`: 批量参数
/// - `process_orders`: 处理订单集合
/// - `exec_params`: 可选算法执行参数
/// - `strategy_params`: 可选策略参数
pub fn batch_process_crypto(
    ctx: Context<BatchCrypto>,
    params: BatchParams,
    process_orders: Vec<BatchProcessOrder>,
    exec_params: Option<ExecutionParams>,
    strategy_params: Option<StrategyParams>
) -> anchor_lang::Result<()> {
    // === 1. 批量参数校验 ===
    validate_batch_params(&params)?;
    require!(!process_orders.is_empty(), crate::errors::AssetError::EmptyBatch);
    require!(process_orders.len() <= 100, crate::errors::AssetError::BatchTooLarge);
    
    // 校验每个处理订单
    for order in &process_orders {
        validate_batch_params(&order.params)?;
    }
    
    // === 2. 批量执行 ===
    let crypto_service = CryptoService::new();
    
    // 执行批量处理
    let results = crypto_service.batch_process(
        &mut ctx.accounts.crypto_assets,
        process_orders
    )?;
    
    // === 3. 算法和策略执行 ===
    if let Some(params) = exec_params {
        crypto_service.execute_algorithm(&mut ctx.accounts.crypto_assets, params)?;
    }
    
    if let Some(strategy) = strategy_params {
        crypto_service.execute_strategy(&mut ctx.accounts.crypto_assets, strategy)?;
    }
    
    // === 4. 事件记录 ===
    let total_processed = results.len() as u64;
    emit!(AssetBatchProcessed {
        asset_id: ctx.accounts.crypto_assets.key(),
        asset_type: AssetType::Crypto,
        process_type: params.batch_type.clone(),
        process_count: total_processed,
        total_balance_before: 0,
        total_balance_after: ctx.accounts.crypto_assets.balance,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: exec_params.map(|p| p.into()),
        strategy_params: strategy_params.map(|s| s.into()),
    });
    
    msg!("Crypto batch process executed successfully: process_type={}, process_count={}, batch_size={}", 
         params.batch_type, total_processed, results.len());
    
    Ok(())
}

/// 加密货币批量管理指令实现
/// 
/// 支持批量资产管理操作，如批量转移、批量更新等。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `params`: 批量参数
/// - `management_orders`: 管理订单集合
pub fn batch_manage_crypto(
    ctx: Context<BatchCrypto>,
    params: BatchParams,
    management_orders: Vec<BatchManagementOrder>
) -> anchor_lang::Result<()> {
    // === 1. 批量参数校验 ===
    validate_batch_params(&params)?;
    require!(!management_orders.is_empty(), crate::errors::AssetError::EmptyBatch);
    require!(management_orders.len() <= 50, crate::errors::AssetError::BatchTooLarge);
    
    // === 2. 权限校验 ===
    require!(
        ctx.accounts.authority.key() == ctx.accounts.crypto_assets.authority,
        crate::errors::SecurityError::Unauthorized
    );
    
    // === 3. 批量管理执行 ===
    let crypto_service = CryptoService::new();
    
    // 执行批量管理
    let results = crypto_service.batch_manage(
        &mut ctx.accounts.crypto_assets,
        management_orders
    )?;
    
    // === 4. 事件记录 ===
    let total_managed = results.len() as u64;
    emit!(AssetBatchProcessed {
        asset_id: ctx.accounts.crypto_assets.key(),
        asset_type: AssetType::Crypto,
        process_type: format!("MANAGE: {}", params.batch_type),
        process_count: total_managed,
        total_balance_before: 0,
        total_balance_after: ctx.accounts.crypto_assets.balance,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: None,
        strategy_params: None,
    });
    
    msg!("Crypto batch management executed successfully: management_type={}, managed_count={}, batch_size={}", 
         params.batch_type, total_managed, results.len());
    
    Ok(())
}

/// 加密货币批量同步指令实现
/// 
/// 支持批量数据同步操作，如价格同步、状态同步等。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `params`: 批量参数
/// - `sync_orders`: 同步订单集合
pub fn batch_sync_crypto(
    ctx: Context<BatchCrypto>,
    params: BatchParams,
    sync_orders: Vec<BatchSyncOrder>
) -> anchor_lang::Result<()> {
    // === 1. 批量参数校验 ===
    validate_batch_params(&params)?;
    require!(!sync_orders.is_empty(), crate::errors::AssetError::EmptyBatch);
    require!(sync_orders.len() <= 200, crate::errors::AssetError::BatchTooLarge);
    
    // === 2. 批量同步执行 ===
    let crypto_service = CryptoService::new();
    
    // 执行批量同步
    let results = crypto_service.batch_sync(
        &mut ctx.accounts.crypto_assets,
        sync_orders
    )?;
    
    // === 3. 事件记录 ===
    let total_synced = results.len() as u64;
    emit!(AssetBatchProcessed {
        asset_id: ctx.accounts.crypto_assets.key(),
        asset_type: AssetType::Crypto,
        process_type: format!("SYNC: {}", params.batch_type),
        process_count: total_synced,
        total_balance_before: 0,
        total_balance_after: ctx.accounts.crypto_assets.balance,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: None,
        strategy_params: None,
    });
    
    msg!("Crypto batch sync executed successfully: sync_type={}, synced_count={}, batch_size={}", 
         params.batch_type, total_synced, results.len());
    
    Ok(())
}

/// 批量处理订单结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct BatchProcessOrder {
    /// 批量参数
    pub params: BatchParams,
    /// 订单优先级
    pub priority: u8,
    /// 处理类型
    pub process_type: BatchProcessType,
}

/// 批量管理订单结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct BatchManagementOrder {
    /// 批量参数
    pub params: BatchParams,
    /// 管理类型
    pub management_type: BatchManagementType,
    /// 管理参数
    pub management_params: String,
}

/// 批量同步订单结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct BatchSyncOrder {
    /// 批量参数
    pub params: BatchParams,
    /// 同步类型
    pub sync_type: BatchSyncType,
    /// 同步参数
    pub sync_params: String,
}

/// 批量处理类型枚举
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub enum BatchProcessType {
    /// 批量更新
    Update,
    /// 批量验证
    Validate,
    /// 批量优化
    Optimize,
    /// 批量清理
    Cleanup,
}

/// 批量管理类型枚举
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub enum BatchManagementType {
    /// 批量转移
    Transfer,
    /// 批量更新
    Update,
    /// 批量删除
    Delete,
    /// 批量创建
    Create,
}

/// 批量同步类型枚举
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub enum BatchSyncType {
    /// 价格同步
    Price,
    /// 状态同步
    Status,
    /// 数据同步
    Data,
    /// 配置同步
    Config,
}

/// 批量结果结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct BatchResult {
    /// 批量操作是否成功
    pub success: bool,
    /// 处理数量
    pub processed_count: u64,
    /// 总交易量
    pub total_volume: u64,
    /// 交易次数
    pub trade_count: u64,
    /// 执行时间
    pub execution_time: i64,
} 