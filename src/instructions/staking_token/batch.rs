//! Staking Token批量操作指令模块
//! 
//! 本模块提供Staking Token资产的批量操作功能，包括：
//! - 批量交易：批量质押、解质押、领取奖励
//! - 批量处理：批量铸造、销毁、转账
//! - 批量管理：批量锁定质押、提前解质押、惩罚机制
//! - 批量同步：批量状态更新、数据同步
//! 
//! 设计特点：
//! - 最小功能单元：专注于Staking Token批量操作功能
//! - 类型安全：严格的类型检查和边界验证
//! - 参数验证：全面的输入参数验证
//! - 权限控制：细粒度的权限验证
//! - 服务层抽象：核心业务逻辑委托给StakingTokenService
//! - 事件驱动：完整的事件发射和审计追踪

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::{
    core::{
        constants::*,
        events::*,
        types::*,
        validation::*,
    },
    errors::*,
    services::*,
    utils::*,
};

/// 批量操作类型枚举
/// 
/// 定义Staking Token批量操作的类型：
/// - Trade: 批量交易操作
/// - Process: 批量处理操作
/// - Manage: 批量管理操作
/// - Sync: 批量同步操作
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum BatchOperationType {
    /// 批量交易操作
    Trade,
    /// 批量处理操作
    Process,
    /// 批量管理操作
    Manage,
    /// 批量同步操作
    Sync,
}

/// 批量交易类型枚举
/// 
/// 定义Staking Token批量交易的类型：
/// - Stake: 批量质押
/// - Unstake: 批量解质押
/// - ClaimRewards: 批量领取奖励
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum BatchTradeType {
    /// 批量质押
    Stake,
    /// 批量解质押
    Unstake,
    /// 批量领取奖励
    ClaimRewards,
}

/// 批量处理类型枚举
/// 
/// 定义Staking Token批量处理的类型：
/// - Mint: 批量铸造
/// - Burn: 批量销毁
/// - Transfer: 批量转账
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum BatchProcessType {
    /// 批量铸造
    Mint,
    /// 批量销毁
    Burn,
    /// 批量转账
    Transfer,
}

/// 批量管理类型枚举
/// 
/// 定义Staking Token批量管理的类型：
/// - LockStaking: 批量锁定质押
/// - EarlyUnstake: 批量提前解质押
/// - Slashing: 批量惩罚机制
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum BatchManageType {
    /// 批量锁定质押
    LockStaking,
    /// 批量提前解质押
    EarlyUnstake,
    /// 批量惩罚机制
    Slashing,
}

/// 批量同步类型枚举
/// 
/// 定义Staking Token批量同步的类型：
/// - Status: 批量状态更新
/// - Data: 批量数据同步
/// - Metrics: 批量指标更新
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum BatchSyncType {
    /// 批量状态更新
    Status,
    /// 批量数据同步
    Data,
    /// 批量指标更新
    Metrics,
}

/// 批量操作结果结构体
/// 
/// 包含批量操作的结果信息：
/// - success_count: 成功操作数量
/// - failure_count: 失败操作数量
/// - total_count: 总操作数量
/// - operation_type: 操作类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct BatchOperationResult {
    /// 成功操作数量
    pub success_count: u32,
    /// 失败操作数量
    pub failure_count: u32,
    /// 总操作数量
    pub total_count: u32,
    /// 操作类型
    pub operation_type: BatchOperationType,
}

/// 批量交易Staking Token参数结构体
/// 
/// 包含批量交易Staking Token所需的所有参数：
/// - trade_type: 交易类型
/// - trade_count: 交易数量
/// - exec_params: 执行参数
/// - strategy_params: 策略参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct BatchTradeStakingParams {
    /// 交易类型
    pub trade_type: BatchTradeType,
    /// 交易数量
    pub trade_count: u32,
    /// 执行参数
    pub exec_params: ExecutionParams,
    /// 策略参数
    pub strategy_params: Option<StrategyParams>,
}

/// 批量处理Staking Token参数结构体
/// 
/// 包含批量处理Staking Token所需的所有参数：
/// - process_type: 处理类型
/// - process_count: 处理数量
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct BatchProcessStakingParams {
    /// 处理类型
    pub process_type: BatchProcessType,
    /// 处理数量
    pub process_count: u32,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 批量管理Staking Token参数结构体
/// 
/// 包含批量管理Staking Token所需的所有参数：
/// - manage_type: 管理类型
/// - manage_count: 管理数量
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct BatchManageStakingParams {
    /// 管理类型
    pub manage_type: BatchManageType,
    /// 管理数量
    pub manage_count: u32,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 批量同步Staking Token参数结构体
/// 
/// 包含批量同步Staking Token所需的所有参数：
/// - sync_type: 同步类型
/// - sync_count: 同步数量
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct BatchSyncStakingParams {
    /// 同步类型
    pub sync_type: BatchSyncType,
    /// 同步数量
    pub sync_count: u32,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// Staking Token批量操作账户上下文
/// 
/// 定义Staking Token批量操作指令所需的账户结构：
/// - staking_token: Staking Token账户（可变，Staking Token类型约束）
/// - authority: 批量操作权限账户
/// - system_program: 系统程序
/// - token_program: 代币程序
#[derive(Accounts)]
pub struct BatchStaking<'info> {
    /// Staking Token账户（可变，Staking Token类型约束）
    #[account(
        mut,
        constraint = staking_token.asset_type == AssetType::StakingToken @ AssetError::InvalidAssetType
    )]
    pub staking_token: Account<'info, Asset>,
    
    /// 批量操作权限账户
    pub authority: Signer<'info>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 验证批量交易Staking Token参数
/// 
/// 检查批量交易Staking Token参数的有效性和边界条件：
/// - 交易数量验证
/// - 执行参数验证
/// - 策略参数验证
/// 
/// # 参数
/// - params: 批量交易Staking Token参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_batch_trade_staking_params(params: &BatchTradeStakingParams) -> Result<()> {
    // 验证交易数量
    require!(
        params.trade_count > 0,
        AssetError::InvalidTradeCount
    );
    
    require!(
        params.trade_count <= MAX_BATCH_TRADE_COUNT,
        AssetError::TradeCountTooLarge
    );
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    // 验证策略参数（如果提供）
    if let Some(ref strategy_params) = params.strategy_params {
        validate_strategy_params(strategy_params)?;
    }
    
    Ok(())
}

/// 验证批量处理Staking Token参数
/// 
/// 检查批量处理Staking Token参数的有效性和边界条件：
/// - 处理数量验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: 批量处理Staking Token参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_batch_process_staking_params(params: &BatchProcessStakingParams) -> Result<()> {
    // 验证处理数量
    require!(
        params.process_count > 0,
        AssetError::InvalidProcessCount
    );
    
    require!(
        params.process_count <= MAX_BATCH_PROCESS_COUNT,
        AssetError::ProcessCountTooLarge
    );
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 验证批量管理Staking Token参数
/// 
/// 检查批量管理Staking Token参数的有效性和边界条件：
/// - 管理数量验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: 批量管理Staking Token参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_batch_manage_staking_params(params: &BatchManageStakingParams) -> Result<()> {
    // 验证管理数量
    require!(
        params.manage_count > 0,
        AssetError::InvalidManageCount
    );
    
    require!(
        params.manage_count <= MAX_BATCH_MANAGE_COUNT,
        AssetError::ManageCountTooLarge
    );
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 验证批量同步Staking Token参数
/// 
/// 检查批量同步Staking Token参数的有效性和边界条件：
/// - 同步数量验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: 批量同步Staking Token参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_batch_sync_staking_params(params: &BatchSyncStakingParams) -> Result<()> {
    // 验证同步数量
    require!(
        params.sync_count > 0,
        AssetError::InvalidSyncCount
    );
    
    require!(
        params.sync_count <= MAX_BATCH_SYNC_COUNT,
        AssetError::SyncCountTooLarge
    );
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 检查批量操作权限
/// 
/// 验证批量操作权限和授权状态：
/// - 检查批量操作权限
/// - 验证Staking Token状态
/// 
/// # 参数
/// - authority: 权限账户
/// - staking_token: Staking Token账户
/// 
/// # 返回
/// - Result<()>: 权限检查结果
pub fn check_batch_authority_permission(
    authority: &Signer,
    staking_token: &Account<Asset>,
) -> Result<()> {
    // 检查批量操作权限
    require!(
        authority.key() == staking_token.owner || authority.key() == staking_token.mint_authority,
        AssetError::InvalidBatchAuthority
    );
    
    // 验证Staking Token状态
    require!(
        staking_token.is_active(),
        AssetError::AssetNotActive
    );
    
    Ok(())
}

/// 批量交易Staking Token指令
/// 
/// 执行Staking Token批量交易操作，包括：
/// - 参数验证：验证批量交易参数的有效性
/// - 权限检查：验证批量操作权限
/// - 服务层调用：委托给StakingTokenService执行批量交易逻辑
/// - 事件发射：发射Staking Token批量交易事件
/// 
/// # 参数
/// - ctx: Staking Token批量操作账户上下文
/// - params: 批量交易Staking Token参数
/// 
/// # 返回
/// - Result<BatchOperationResult>: 批量交易操作结果
pub fn batch_trade_staking_token(
    ctx: Context<BatchStaking>,
    params: BatchTradeStakingParams,
) -> Result<BatchOperationResult> {
    // 参数验证
    validate_batch_trade_staking_params(&params)?;
    
    // 权限检查
    check_batch_authority_permission(&ctx.accounts.authority, &ctx.accounts.staking_token)?;
    
    let staking_token = &mut ctx.accounts.staking_token;
    let authority = &ctx.accounts.authority;
    
    // 创建Staking Token服务实例
    let service = StakingTokenService::new();
    
    // 执行Staking Token批量交易
    let result = service.batch_trade_staking_token(
        staking_token,
        &params.trade_type,
        params.trade_count,
        &params.exec_params,
        params.strategy_params.as_ref(),
    )?;
    
    // 发射Staking Token批量交易事件
    emit!(AssetBatchTraded {
        basket_id: staking_token.id,
        trade_type: format!("{:?}", params.trade_type),
        trade_count: params.trade_count,
        success_count: result.success_count,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::StakingToken,
        exec_params: params.exec_params,
    });
    
    Ok(result)
}

/// 批量处理Staking Token指令
/// 
/// 执行Staking Token批量处理操作，包括：
/// - 参数验证：验证批量处理参数的有效性
/// - 权限检查：验证批量操作权限
/// - 服务层调用：委托给StakingTokenService执行批量处理逻辑
/// - 事件发射：发射Staking Token批量处理事件
/// 
/// # 参数
/// - ctx: Staking Token批量操作账户上下文
/// - params: 批量处理Staking Token参数
/// 
/// # 返回
/// - Result<BatchOperationResult>: 批量处理操作结果
pub fn batch_process_staking_token(
    ctx: Context<BatchStaking>,
    params: BatchProcessStakingParams,
) -> Result<BatchOperationResult> {
    // 参数验证
    validate_batch_process_staking_params(&params)?;
    
    // 权限检查
    check_batch_authority_permission(&ctx.accounts.authority, &ctx.accounts.staking_token)?;
    
    let staking_token = &mut ctx.accounts.staking_token;
    let authority = &ctx.accounts.authority;
    
    // 创建Staking Token服务实例
    let service = StakingTokenService::new();
    
    // 执行Staking Token批量处理
    let result = service.batch_process_staking_token(
        staking_token,
        &params.process_type,
        params.process_count,
        &params.exec_params,
    )?;
    
    // 发射Staking Token批量处理事件
    emit!(AssetBatchProcessed {
        basket_id: staking_token.id,
        process_type: format!("{:?}", params.process_type),
        process_count: params.process_count,
        success_count: result.success_count,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::StakingToken,
        exec_params: params.exec_params,
    });
    
    Ok(result)
}

/// 批量管理Staking Token指令
/// 
/// 执行Staking Token批量管理操作，包括：
/// - 参数验证：验证批量管理参数的有效性
/// - 权限检查：验证批量操作权限
/// - 服务层调用：委托给StakingTokenService执行批量管理逻辑
/// - 事件发射：发射Staking Token批量管理事件
/// 
/// # 参数
/// - ctx: Staking Token批量操作账户上下文
/// - params: 批量管理Staking Token参数
/// 
/// # 返回
/// - Result<BatchOperationResult>: 批量管理操作结果
pub fn batch_manage_staking_token(
    ctx: Context<BatchStaking>,
    params: BatchManageStakingParams,
) -> Result<BatchOperationResult> {
    // 参数验证
    validate_batch_manage_staking_params(&params)?;
    
    // 权限检查
    check_batch_authority_permission(&ctx.accounts.authority, &ctx.accounts.staking_token)?;
    
    let staking_token = &mut ctx.accounts.staking_token;
    let authority = &ctx.accounts.authority;
    
    // 创建Staking Token服务实例
    let service = StakingTokenService::new();
    
    // 执行Staking Token批量管理
    let result = service.batch_manage_staking_token(
        staking_token,
        &params.manage_type,
        params.manage_count,
        &params.exec_params,
    )?;
    
    // 发射Staking Token批量管理事件
    emit!(AssetBatchManaged {
        basket_id: staking_token.id,
        manage_type: format!("{:?}", params.manage_type),
        manage_count: params.manage_count,
        success_count: result.success_count,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::StakingToken,
        exec_params: params.exec_params,
    });
    
    Ok(result)
}

/// 批量同步Staking Token指令
/// 
/// 执行Staking Token批量同步操作，包括：
/// - 参数验证：验证批量同步参数的有效性
/// - 权限检查：验证批量操作权限
/// - 服务层调用：委托给StakingTokenService执行批量同步逻辑
/// - 事件发射：发射Staking Token批量同步事件
/// 
/// # 参数
/// - ctx: Staking Token批量操作账户上下文
/// - params: 批量同步Staking Token参数
/// 
/// # 返回
/// - Result<BatchOperationResult>: 批量同步操作结果
pub fn batch_sync_staking_token(
    ctx: Context<BatchStaking>,
    params: BatchSyncStakingParams,
) -> Result<BatchOperationResult> {
    // 参数验证
    validate_batch_sync_staking_params(&params)?;
    
    // 权限检查
    check_batch_authority_permission(&ctx.accounts.authority, &ctx.accounts.staking_token)?;
    
    let staking_token = &mut ctx.accounts.staking_token;
    let authority = &ctx.accounts.authority;
    
    // 创建Staking Token服务实例
    let service = StakingTokenService::new();
    
    // 执行Staking Token批量同步
    let result = service.batch_sync_staking_token(
        staking_token,
        &params.sync_type,
        params.sync_count,
        &params.exec_params,
    )?;
    
    // 发射Staking Token批量同步事件
    emit!(AssetBatchSynced {
        basket_id: staking_token.id,
        sync_type: format!("{:?}", params.sync_type),
        sync_count: params.sync_count,
        success_count: result.success_count,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::StakingToken,
        exec_params: params.exec_params,
    });
    
    Ok(result)
} 