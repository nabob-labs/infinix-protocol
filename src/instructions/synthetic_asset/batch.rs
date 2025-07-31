//! Synthetic Asset批量操作指令模块
//! 
//! 本模块提供Synthetic Asset资产的批量操作功能，包括：
//! - 批量交易：批量交易合成资产
//! - 批量处理：批量处理合成资产
//! - 批量管理：批量管理合成资产
//! - 批量同步：批量同步合成资产
//! 
//! 设计特点：
//! - 最小功能单元：每个批量操作功能单一，职责明确
//! - 类型安全：严格的类型检查和边界验证
//! - 参数验证：全面的输入参数验证
//! - 权限控制：细粒度的权限验证
//! - 服务层抽象：核心业务逻辑委托给SyntheticAssetService
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
/// 定义批量操作的类型：
/// - Trade: 批量交易
/// - Process: 批量处理
/// - Manage: 批量管理
/// - Sync: 批量同步
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum BatchOperationType {
    /// 批量交易
    Trade,
    /// 批量处理
    Process,
    /// 批量管理
    Manage,
    /// 批量同步
    Sync,
}

/// 批量交易类型枚举
/// 
/// 定义批量交易的类型：
/// - Buy: 批量买入
/// - Sell: 批量卖出
/// - Swap: 批量兑换
/// - Arbitrage: 批量套利
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum BatchTradeType {
    /// 批量买入
    Buy,
    /// 批量卖出
    Sell,
    /// 批量兑换
    Swap,
    /// 批量套利
    Arbitrage,
}

/// 批量处理类型枚举
/// 
/// 定义批量处理的类型：
/// - Create: 批量创建
/// - Update: 批量更新
/// - Validate: 批量验证
/// - Optimize: 批量优化
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum BatchProcessType {
    /// 批量创建
    Create,
    /// 批量更新
    Update,
    /// 批量验证
    Validate,
    /// 批量优化
    Optimize,
}

/// 批量管理类型枚举
/// 
/// 定义批量管理的类型：
/// - Monitor: 批量监控
/// - Adjust: 批量调整
/// - Rebalance: 批量再平衡
/// - RiskControl: 批量风控
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum BatchManageType {
    /// 批量监控
    Monitor,
    /// 批量调整
    Adjust,
    /// 批量再平衡
    Rebalance,
    /// 批量风控
    RiskControl,
}

/// 批量同步类型枚举
/// 
/// 定义批量同步的类型：
/// - Price: 批量价格同步
/// - State: 批量状态同步
/// - Data: 批量数据同步
/// - Config: 批量配置同步
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum BatchSyncType {
    /// 批量价格同步
    Price,
    /// 批量状态同步
    State,
    /// 批量数据同步
    Data,
    /// 批量配置同步
    Config,
}

/// 批量操作结果结构体
/// 
/// 包含批量操作的结果信息：
/// - success_count: 成功数量
/// - failure_count: 失败数量
/// - total_count: 总数量
/// - operation_type: 操作类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct BatchOperationResult {
    /// 成功数量
    pub success_count: u32,
    /// 失败数量
    pub failure_count: u32,
    /// 总数量
    pub total_count: u32,
    /// 操作类型
    pub operation_type: String,
}

/// 批量交易Synthetic Asset参数结构体
/// 
/// 包含批量交易Synthetic Asset所需的所有参数：
/// - trade_type: 交易类型
/// - trade_count: 交易数量
/// - exec_params: 执行参数
/// - strategy_params: 策略参数（可选）
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct BatchTradeSyntheticParams {
    /// 交易类型
    pub trade_type: BatchTradeType,
    /// 交易数量
    pub trade_count: u32,
    /// 执行参数
    pub exec_params: ExecutionParams,
    /// 策略参数（可选）
    pub strategy_params: Option<StrategyParams>,
}

/// 批量处理Synthetic Asset参数结构体
/// 
/// 包含批量处理Synthetic Asset所需的所有参数：
/// - process_type: 处理类型
/// - process_count: 处理数量
/// - exec_params: 执行参数
/// - config_params: 配置参数（可选）
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct BatchProcessSyntheticParams {
    /// 处理类型
    pub process_type: BatchProcessType,
    /// 处理数量
    pub process_count: u32,
    /// 执行参数
    pub exec_params: ExecutionParams,
    /// 配置参数（可选）
    pub config_params: Option<ConfigParams>,
}

/// 批量管理Synthetic Asset参数结构体
/// 
/// 包含批量管理Synthetic Asset所需的所有参数：
/// - manage_type: 管理类型
/// - manage_count: 管理数量
/// - exec_params: 执行参数
/// - management_params: 管理参数（可选）
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct BatchManageSyntheticParams {
    /// 管理类型
    pub manage_type: BatchManageType,
    /// 管理数量
    pub manage_count: u32,
    /// 执行参数
    pub exec_params: ExecutionParams,
    /// 管理参数（可选）
    pub management_params: Option<ManagementParams>,
}

/// 批量同步Synthetic Asset参数结构体
/// 
/// 包含批量同步Synthetic Asset所需的所有参数：
/// - sync_type: 同步类型
/// - sync_count: 同步数量
/// - exec_params: 执行参数
/// - sync_params: 同步参数（可选）
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct BatchSyncSyntheticParams {
    /// 同步类型
    pub sync_type: BatchSyncType,
    /// 同步数量
    pub sync_count: u32,
    /// 执行参数
    pub exec_params: ExecutionParams,
    /// 同步参数（可选）
    pub sync_params: Option<SyncParams>,
}

/// 批量Synthetic Asset账户上下文
/// 
/// 定义批量Synthetic Asset操作指令所需的账户结构：
/// - synthetic_asset: Synthetic Asset账户（可变，Synthetic Asset类型约束）
/// - authority: 权限账户（owner约束）
/// - synthetic_pool: 合成资产池账户
/// - oracle: 预言机账户
/// - system_program: 系统程序
/// - token_program: 代币程序
#[derive(Accounts)]
pub struct BatchSynthetic<'info> {
    /// Synthetic Asset账户（可变，Synthetic Asset类型约束）
    #[account(
        mut,
        constraint = synthetic_asset.asset_type == AssetType::SyntheticAsset @ AssetError::InvalidAssetType
    )]
    pub synthetic_asset: Account<'info, Asset>,
    
    /// 权限账户（owner约束）
    #[account(
        constraint = authority.key() == synthetic_asset.owner @ AssetError::InvalidOwner
    )]
    pub authority: Signer<'info>,
    
    /// 合成资产池账户
    /// CHECK: 由程序验证
    #[account(mut)]
    pub synthetic_pool: UncheckedAccount<'info>,
    
    /// 预言机账户
    /// CHECK: 由程序验证
    pub oracle: UncheckedAccount<'info>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 验证批量交易Synthetic Asset参数
/// 
/// 检查批量交易Synthetic Asset参数的有效性和边界条件：
/// - 交易类型验证
/// - 交易数量验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: 批量交易Synthetic Asset参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_batch_trade_synthetic_params(params: &BatchTradeSyntheticParams) -> Result<()> {
    // 验证交易数量
    require!(
        params.trade_count > 0,
        AssetError::InvalidTradeCount
    );
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 验证批量处理Synthetic Asset参数
/// 
/// 检查批量处理Synthetic Asset参数的有效性和边界条件：
/// - 处理类型验证
/// - 处理数量验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: 批量处理Synthetic Asset参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_batch_process_synthetic_params(params: &BatchProcessSyntheticParams) -> Result<()> {
    // 验证处理数量
    require!(
        params.process_count > 0,
        AssetError::InvalidProcessCount
    );
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 验证批量管理Synthetic Asset参数
/// 
/// 检查批量管理Synthetic Asset参数的有效性和边界条件：
/// - 管理类型验证
/// - 管理数量验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: 批量管理Synthetic Asset参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_batch_manage_synthetic_params(params: &BatchManageSyntheticParams) -> Result<()> {
    // 验证管理数量
    require!(
        params.manage_count > 0,
        AssetError::InvalidManageCount
    );
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 验证批量同步Synthetic Asset参数
/// 
/// 检查批量同步Synthetic Asset参数的有效性和边界条件：
/// - 同步类型验证
/// - 同步数量验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: 批量同步Synthetic Asset参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_batch_sync_synthetic_params(params: &BatchSyncSyntheticParams) -> Result<()> {
    // 验证同步数量
    require!(
        params.sync_count > 0,
        AssetError::InvalidSyncCount
    );
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 检查批量操作权限
/// 
/// 验证批量操作权限和授权状态：
/// - 检查所有权
/// - 验证Synthetic Asset状态
/// - 检查批量操作权限
/// 
/// # 参数
/// - authority: 权限账户
/// - synthetic_asset: Synthetic Asset账户
/// 
/// # 返回
/// - Result<()>: 权限检查结果
pub fn check_batch_authority_permission(
    authority: &Signer,
    synthetic_asset: &Account<Asset>,
) -> Result<()> {
    // 检查所有权
    require!(
        authority.key() == synthetic_asset.owner,
        AssetError::InvalidOwner
    );
    
    // 验证Synthetic Asset状态
    require!(
        synthetic_asset.is_active(),
        AssetError::AssetNotActive
    );
    
    Ok(())
}

/// 批量交易Synthetic Asset指令
/// 
/// 执行批量交易Synthetic Asset操作，包括：
/// - 参数验证：验证批量交易参数的有效性
/// - 权限检查：验证批量交易权限
/// - 服务层调用：委托给SyntheticAssetService执行批量交易逻辑
/// - 事件发射：发射Synthetic Asset批量交易事件
/// 
/// # 参数
/// - ctx: 批量Synthetic Asset账户上下文
/// - params: 批量交易Synthetic Asset参数
/// 
/// # 返回
/// - Result<BatchOperationResult>: 批量交易操作结果
pub fn batch_trade_synthetic_asset(
    ctx: Context<BatchSynthetic>,
    params: BatchTradeSyntheticParams,
) -> Result<BatchOperationResult> {
    // 参数验证
    validate_batch_trade_synthetic_params(&params)?;
    
    // 权限检查
    check_batch_authority_permission(&ctx.accounts.authority, &ctx.accounts.synthetic_asset)?;
    
    let synthetic_asset = &mut ctx.accounts.synthetic_asset;
    let authority = &ctx.accounts.authority;
    
    // 创建Synthetic Asset服务实例
    let service = SyntheticAssetService::new();
    
    // 执行Synthetic Asset批量交易
    let result = service.batch_trade_synthetic_asset(
        synthetic_asset,
        &params.trade_type,
        params.trade_count,
        &params.exec_params,
        params.strategy_params.as_ref(),
    )?;
    
    // 发射Synthetic Asset批量交易事件
    emit!(AssetBatchTraded {
        basket_id: synthetic_asset.id,
        trade_type: format!("{:?}", params.trade_type),
        trade_count: params.trade_count,
        success_count: result.success_count,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::SyntheticAsset,
        exec_params: params.exec_params,
    });
    
    Ok(result)
}

/// 批量处理Synthetic Asset指令
/// 
/// 执行批量处理Synthetic Asset操作，包括：
/// - 参数验证：验证批量处理参数的有效性
/// - 权限检查：验证批量处理权限
/// - 服务层调用：委托给SyntheticAssetService执行批量处理逻辑
/// - 事件发射：发射Synthetic Asset批量处理事件
/// 
/// # 参数
/// - ctx: 批量Synthetic Asset账户上下文
/// - params: 批量处理Synthetic Asset参数
/// 
/// # 返回
/// - Result<BatchOperationResult>: 批量处理操作结果
pub fn batch_process_synthetic_asset(
    ctx: Context<BatchSynthetic>,
    params: BatchProcessSyntheticParams,
) -> Result<BatchOperationResult> {
    // 参数验证
    validate_batch_process_synthetic_params(&params)?;
    
    // 权限检查
    check_batch_authority_permission(&ctx.accounts.authority, &ctx.accounts.synthetic_asset)?;
    
    let synthetic_asset = &mut ctx.accounts.synthetic_asset;
    let authority = &ctx.accounts.authority;
    
    // 创建Synthetic Asset服务实例
    let service = SyntheticAssetService::new();
    
    // 执行Synthetic Asset批量处理
    let result = service.batch_process_synthetic_asset(
        synthetic_asset,
        &params.process_type,
        params.process_count,
        &params.exec_params,
        params.config_params.as_ref(),
    )?;
    
    // 发射Synthetic Asset批量处理事件
    emit!(AssetBatchProcessed {
        basket_id: synthetic_asset.id,
        process_type: format!("{:?}", params.process_type),
        process_count: params.process_count,
        success_count: result.success_count,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::SyntheticAsset,
        exec_params: params.exec_params,
    });
    
    Ok(result)
}

/// 批量管理Synthetic Asset指令
/// 
/// 执行批量管理Synthetic Asset操作，包括：
/// - 参数验证：验证批量管理参数的有效性
/// - 权限检查：验证批量管理权限
/// - 服务层调用：委托给SyntheticAssetService执行批量管理逻辑
/// - 事件发射：发射Synthetic Asset批量管理事件
/// 
/// # 参数
/// - ctx: 批量Synthetic Asset账户上下文
/// - params: 批量管理Synthetic Asset参数
/// 
/// # 返回
/// - Result<BatchOperationResult>: 批量管理操作结果
pub fn batch_manage_synthetic_asset(
    ctx: Context<BatchSynthetic>,
    params: BatchManageSyntheticParams,
) -> Result<BatchOperationResult> {
    // 参数验证
    validate_batch_manage_synthetic_params(&params)?;
    
    // 权限检查
    check_batch_authority_permission(&ctx.accounts.authority, &ctx.accounts.synthetic_asset)?;
    
    let synthetic_asset = &mut ctx.accounts.synthetic_asset;
    let authority = &ctx.accounts.authority;
    
    // 创建Synthetic Asset服务实例
    let service = SyntheticAssetService::new();
    
    // 执行Synthetic Asset批量管理
    let result = service.batch_manage_synthetic_asset(
        synthetic_asset,
        &params.manage_type,
        params.manage_count,
        &params.exec_params,
        params.management_params.as_ref(),
    )?;
    
    // 发射Synthetic Asset批量管理事件
    emit!(AssetBatchManaged {
        basket_id: synthetic_asset.id,
        manage_type: format!("{:?}", params.manage_type),
        manage_count: params.manage_count,
        success_count: result.success_count,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::SyntheticAsset,
        exec_params: params.exec_params,
    });
    
    Ok(result)
}

/// 批量同步Synthetic Asset指令
/// 
/// 执行批量同步Synthetic Asset操作，包括：
/// - 参数验证：验证批量同步参数的有效性
/// - 权限检查：验证批量同步权限
/// - 服务层调用：委托给SyntheticAssetService执行批量同步逻辑
/// - 事件发射：发射Synthetic Asset批量同步事件
/// 
/// # 参数
/// - ctx: 批量Synthetic Asset账户上下文
/// - params: 批量同步Synthetic Asset参数
/// 
/// # 返回
/// - Result<BatchOperationResult>: 批量同步操作结果
pub fn batch_sync_synthetic_asset(
    ctx: Context<BatchSynthetic>,
    params: BatchSyncSyntheticParams,
) -> Result<BatchOperationResult> {
    // 参数验证
    validate_batch_sync_synthetic_params(&params)?;
    
    // 权限检查
    check_batch_authority_permission(&ctx.accounts.authority, &ctx.accounts.synthetic_asset)?;
    
    let synthetic_asset = &mut ctx.accounts.synthetic_asset;
    let authority = &ctx.accounts.authority;
    
    // 创建Synthetic Asset服务实例
    let service = SyntheticAssetService::new();
    
    // 执行Synthetic Asset批量同步
    let result = service.batch_sync_synthetic_asset(
        synthetic_asset,
        &params.sync_type,
        params.sync_count,
        &params.exec_params,
        params.sync_params.as_ref(),
    )?;
    
    // 发射Synthetic Asset批量同步事件
    emit!(AssetBatchSynced {
        basket_id: synthetic_asset.id,
        sync_type: format!("{:?}", params.sync_type),
        sync_count: params.sync_count,
        success_count: result.success_count,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::SyntheticAsset,
        exec_params: params.exec_params,
    });
    
    Ok(result)
} 