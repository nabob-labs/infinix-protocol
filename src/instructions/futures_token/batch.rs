//! Futures Token批量操作指令模块
//! 
//! 本模块提供Futures Token资产的批量操作功能，包括：
//! - 批量交易：批量执行Futures Token交易操作
//! - 批量处理：批量处理Futures Token相关操作
//! - 批量管理：批量管理Futures Token状态
//! - 批量同步：批量同步Futures Token数据
//! 
//! 设计特点：
//! - 最小功能单元：专注于Futures Token批量操作功能
//! - 类型安全：严格的类型检查和边界验证
//! - 参数验证：全面的输入参数验证
//! - 权限控制：细粒度的权限验证
//! - 服务层抽象：核心业务逻辑委托给FuturesTokenService
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

/// 批量交易Futures Token参数结构体
/// 
/// 包含批量交易Futures Token所需的所有参数：
/// - operations: 批量操作列表
/// - batch_size: 批量大小
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct BatchTradeFuturesParams {
    /// 批量操作列表
    pub operations: Vec<FuturesTradeOperation>,
    /// 批量大小
    pub batch_size: u32,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 批量处理Futures Token参数结构体
/// 
/// 包含批量处理Futures Token所需的所有参数：
/// - operations: 批量处理操作列表
/// - batch_size: 批量大小
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct BatchProcessFuturesParams {
    /// 批量处理操作列表
    pub operations: Vec<FuturesProcessOperation>,
    /// 批量大小
    pub batch_size: u32,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 批量管理Futures Token参数结构体
/// 
/// 包含批量管理Futures Token所需的所有参数：
/// - operations: 批量管理操作列表
/// - batch_size: 批量大小
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct BatchManageFuturesParams {
    /// 批量管理操作列表
    pub operations: Vec<FuturesManageOperation>,
    /// 批量大小
    pub batch_size: u32,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 批量同步Futures Token参数结构体
/// 
/// 包含批量同步Futures Token所需的所有参数：
/// - operations: 批量同步操作列表
/// - batch_size: 批量大小
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct BatchSyncFuturesParams {
    /// 批量同步操作列表
    pub operations: Vec<FuturesSyncOperation>,
    /// 批量大小
    pub batch_size: u32,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// Futures Token交易操作结构体
/// 
/// 定义Futures Token交易操作的类型和参数：
/// - operation_type: 操作类型
/// - futures_id: 期货ID
/// - amount: 数量
/// - price: 价格
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct FuturesTradeOperation {
    /// 操作类型
    pub operation_type: FuturesTradeType,
    /// 期货ID
    pub futures_id: Pubkey,
    /// 数量
    pub amount: u64,
    /// 价格
    pub price: f64,
}

/// Futures Token处理操作结构体
/// 
/// 定义Futures Token处理操作的类型和参数：
/// - operation_type: 操作类型
/// - futures_id: 期货ID
/// - params: 处理参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct FuturesProcessOperation {
    /// 操作类型
    pub operation_type: FuturesProcessType,
    /// 期货ID
    pub futures_id: Pubkey,
    /// 处理参数
    pub params: Vec<u8>,
}

/// Futures Token管理操作结构体
/// 
/// 定义Futures Token管理操作的类型和参数：
/// - operation_type: 操作类型
/// - futures_id: 期货ID
/// - params: 管理参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct FuturesManageOperation {
    /// 操作类型
    pub operation_type: FuturesManageType,
    /// 期货ID
    pub futures_id: Pubkey,
    /// 管理参数
    pub params: Vec<u8>,
}

/// Futures Token同步操作结构体
/// 
/// 定义Futures Token同步操作的类型和参数：
/// - operation_type: 操作类型
/// - futures_id: 期货ID
/// - params: 同步参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct FuturesSyncOperation {
    /// 操作类型
    pub operation_type: FuturesSyncType,
    /// 期货ID
    pub futures_id: Pubkey,
    /// 同步参数
    pub params: Vec<u8>,
}

/// Futures Token交易类型枚举
/// 
/// 定义Futures Token交易的类型：
/// - Buy: 买入
/// - Sell: 卖出
/// - Create: 创建
/// - Settle: 结算
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum FuturesTradeType {
    /// 买入
    Buy,
    /// 卖出
    Sell,
    /// 创建
    Create,
    /// 结算
    Settle,
}

/// Futures Token处理类型枚举
/// 
/// 定义Futures Token处理的类型：
/// - PriceUpdate: 价格更新
/// - MarginCall: 保证金催缴
/// - Liquidation: 清算
/// - Settlement: 结算
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum FuturesProcessType {
    /// 价格更新
    PriceUpdate,
    /// 保证金催缴
    MarginCall,
    /// 清算
    Liquidation,
    /// 结算
    Settlement,
}

/// Futures Token管理类型枚举
/// 
/// 定义Futures Token管理的类型：
/// - StatusUpdate: 状态更新
/// - ParameterUpdate: 参数更新
/// - ConfigurationUpdate: 配置更新
/// - Maintenance: 维护
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum FuturesManageType {
    /// 状态更新
    StatusUpdate,
    /// 参数更新
    ParameterUpdate,
    /// 配置更新
    ConfigurationUpdate,
    /// 维护
    Maintenance,
}

/// Futures Token同步类型枚举
/// 
/// 定义Futures Token同步的类型：
/// - DataSync: 数据同步
/// - StateSync: 状态同步
/// - PriceSync: 价格同步
/// - ConfigSync: 配置同步
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum FuturesSyncType {
    /// 数据同步
    DataSync,
    /// 状态同步
    StateSync,
    /// 价格同步
    PriceSync,
    /// 配置同步
    ConfigSync,
}

/// 批量Futures Token操作账户上下文
/// 
/// 定义批量Futures Token操作指令所需的账户结构：
/// - futures_token: Futures Token账户（可变，Futures Token类型约束）
/// - operator: 操作者账户（owner约束）
/// - futures_pool: 期货池账户
/// - system_program: 系统程序
/// - token_program: 代币程序
#[derive(Accounts)]
pub struct BatchFutures<'info> {
    /// Futures Token账户（可变，Futures Token类型约束）
    #[account(
        mut,
        constraint = futures_token.asset_type == AssetType::FuturesToken @ AssetError::InvalidAssetType
    )]
    pub futures_token: Account<'info, Asset>,
    
    /// 操作者账户（owner约束）
    #[account(
        constraint = operator.key() == futures_token.owner @ AssetError::InvalidOwner
    )]
    pub operator: Signer<'info>,
    
    /// 期货池账户
    /// CHECK: 由程序验证
    #[account(mut)]
    pub futures_pool: UncheckedAccount<'info>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 验证批量交易Futures Token参数
/// 
/// 检查批量交易Futures Token参数的有效性和边界条件：
/// - 操作列表验证
/// - 批量大小验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: 批量交易Futures Token参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_batch_trade_futures_params(params: &BatchTradeFuturesParams) -> Result<()> {
    // 验证操作列表
    require!(
        !params.operations.is_empty(),
        AssetError::InvalidOperations
    );
    
    // 验证批量大小
    require!(
        params.batch_size > 0 && params.batch_size <= MAX_BATCH_SIZE,
        AssetError::InvalidBatchSize
    );
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 验证批量处理Futures Token参数
/// 
/// 检查批量处理Futures Token参数的有效性和边界条件：
/// - 操作列表验证
/// - 批量大小验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: 批量处理Futures Token参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_batch_process_futures_params(params: &BatchProcessFuturesParams) -> Result<()> {
    // 验证操作列表
    require!(
        !params.operations.is_empty(),
        AssetError::InvalidOperations
    );
    
    // 验证批量大小
    require!(
        params.batch_size > 0 && params.batch_size <= MAX_BATCH_SIZE,
        AssetError::InvalidBatchSize
    );
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 验证批量管理Futures Token参数
/// 
/// 检查批量管理Futures Token参数的有效性和边界条件：
/// - 操作列表验证
/// - 批量大小验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: 批量管理Futures Token参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_batch_manage_futures_params(params: &BatchManageFuturesParams) -> Result<()> {
    // 验证操作列表
    require!(
        !params.operations.is_empty(),
        AssetError::InvalidOperations
    );
    
    // 验证批量大小
    require!(
        params.batch_size > 0 && params.batch_size <= MAX_BATCH_SIZE,
        AssetError::InvalidBatchSize
    );
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 验证批量同步Futures Token参数
/// 
/// 检查批量同步Futures Token参数的有效性和边界条件：
/// - 操作列表验证
/// - 批量大小验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: 批量同步Futures Token参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_batch_sync_futures_params(params: &BatchSyncFuturesParams) -> Result<()> {
    // 验证操作列表
    require!(
        !params.operations.is_empty(),
        AssetError::InvalidOperations
    );
    
    // 验证批量大小
    require!(
        params.batch_size > 0 && params.batch_size <= MAX_BATCH_SIZE,
        AssetError::InvalidBatchSize
    );
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 检查批量Futures Token操作权限
/// 
/// 验证批量Futures Token操作权限和授权状态：
/// - 检查所有权
/// - 验证Futures Token状态
/// - 检查操作权限
/// 
/// # 参数
/// - operator: 操作者账户
/// - futures_token: Futures Token账户
/// 
/// # 返回
/// - Result<()>: 权限检查结果
pub fn check_batch_futures_authority_permission(
    operator: &Signer,
    futures_token: &Account<Asset>,
) -> Result<()> {
    // 检查所有权
    require!(
        operator.key() == futures_token.owner,
        AssetError::InvalidOwner
    );
    
    // 验证Futures Token状态
    require!(
        futures_token.is_active(),
        AssetError::AssetNotActive
    );
    
    Ok(())
}

/// 批量交易Futures Token指令
/// 
/// 执行批量交易Futures Token操作，包括：
/// - 参数验证：验证批量交易参数的有效性
/// - 权限检查：验证批量交易权限
/// - 服务层调用：委托给FuturesTokenService执行批量交易逻辑
/// - 事件发射：发射Futures Token批量交易事件
/// 
/// # 参数
/// - ctx: 批量Futures Token操作账户上下文
/// - params: 批量交易Futures Token参数
/// 
/// # 返回
/// - Result<()>: 批量交易操作结果
pub fn batch_trade_futures(
    ctx: Context<BatchFutures>,
    params: BatchTradeFuturesParams,
) -> Result<()> {
    // 参数验证
    validate_batch_trade_futures_params(&params)?;
    
    // 权限检查
    check_batch_futures_authority_permission(
        &ctx.accounts.operator,
        &ctx.accounts.futures_token,
    )?;
    
    let futures_token = &mut ctx.accounts.futures_token;
    let operator = &ctx.accounts.operator;
    
    // 创建Futures Token服务实例
    let service = FuturesTokenService::new();
    
    // 执行批量交易
    service.batch_trade(
        futures_token,
        &params.operations,
        params.batch_size,
        &params.exec_params,
    )?;
    
    // 发射批量交易事件
    emit!(AssetBatchTraded {
        basket_id: futures_token.id,
        batch_size: params.batch_size,
        operations_count: params.operations.len() as u32,
        operator: operator.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::FuturesToken,
        exec_params: params.exec_params,
    });
    
    Ok(())
}

/// 批量处理Futures Token指令
/// 
/// 执行批量处理Futures Token操作，包括：
/// - 参数验证：验证批量处理参数的有效性
/// - 权限检查：验证批量处理权限
/// - 服务层调用：委托给FuturesTokenService执行批量处理逻辑
/// - 事件发射：发射Futures Token批量处理事件
/// 
/// # 参数
/// - ctx: 批量Futures Token操作账户上下文
/// - params: 批量处理Futures Token参数
/// 
/// # 返回
/// - Result<()>: 批量处理操作结果
pub fn batch_process_futures(
    ctx: Context<BatchFutures>,
    params: BatchProcessFuturesParams,
) -> Result<()> {
    // 参数验证
    validate_batch_process_futures_params(&params)?;
    
    // 权限检查
    check_batch_futures_authority_permission(
        &ctx.accounts.operator,
        &ctx.accounts.futures_token,
    )?;
    
    let futures_token = &mut ctx.accounts.futures_token;
    let operator = &ctx.accounts.operator;
    
    // 创建Futures Token服务实例
    let service = FuturesTokenService::new();
    
    // 执行批量处理
    service.batch_process(
        futures_token,
        &params.operations,
        params.batch_size,
        &params.exec_params,
    )?;
    
    // 发射批量处理事件
    emit!(AssetBatchProcessed {
        basket_id: futures_token.id,
        batch_size: params.batch_size,
        operations_count: params.operations.len() as u32,
        operator: operator.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::FuturesToken,
        exec_params: params.exec_params,
    });
    
    Ok(())
}

/// 批量管理Futures Token指令
/// 
/// 执行批量管理Futures Token操作，包括：
/// - 参数验证：验证批量管理参数的有效性
/// - 权限检查：验证批量管理权限
/// - 服务层调用：委托给FuturesTokenService执行批量管理逻辑
/// - 事件发射：发射Futures Token批量管理事件
/// 
/// # 参数
/// - ctx: 批量Futures Token操作账户上下文
/// - params: 批量管理Futures Token参数
/// 
/// # 返回
/// - Result<()>: 批量管理操作结果
pub fn batch_manage_futures(
    ctx: Context<BatchFutures>,
    params: BatchManageFuturesParams,
) -> Result<()> {
    // 参数验证
    validate_batch_manage_futures_params(&params)?;
    
    // 权限检查
    check_batch_futures_authority_permission(
        &ctx.accounts.operator,
        &ctx.accounts.futures_token,
    )?;
    
    let futures_token = &mut ctx.accounts.futures_token;
    let operator = &ctx.accounts.operator;
    
    // 创建Futures Token服务实例
    let service = FuturesTokenService::new();
    
    // 执行批量管理
    service.batch_manage(
        futures_token,
        &params.operations,
        params.batch_size,
        &params.exec_params,
    )?;
    
    // 发射批量管理事件
    emit!(AssetBatchManaged {
        basket_id: futures_token.id,
        batch_size: params.batch_size,
        operations_count: params.operations.len() as u32,
        operator: operator.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::FuturesToken,
        exec_params: params.exec_params,
    });
    
    Ok(())
}

/// 批量同步Futures Token指令
/// 
/// 执行批量同步Futures Token操作，包括：
/// - 参数验证：验证批量同步参数的有效性
/// - 权限检查：验证批量同步权限
/// - 服务层调用：委托给FuturesTokenService执行批量同步逻辑
/// - 事件发射：发射Futures Token批量同步事件
/// 
/// # 参数
/// - ctx: 批量Futures Token操作账户上下文
/// - params: 批量同步Futures Token参数
/// 
/// # 返回
/// - Result<()>: 批量同步操作结果
pub fn batch_sync_futures(
    ctx: Context<BatchFutures>,
    params: BatchSyncFuturesParams,
) -> Result<()> {
    // 参数验证
    validate_batch_sync_futures_params(&params)?;
    
    // 权限检查
    check_batch_futures_authority_permission(
        &ctx.accounts.operator,
        &ctx.accounts.futures_token,
    )?;
    
    let futures_token = &mut ctx.accounts.futures_token;
    let operator = &ctx.accounts.operator;
    
    // 创建Futures Token服务实例
    let service = FuturesTokenService::new();
    
    // 执行批量同步
    service.batch_sync(
        futures_token,
        &params.operations,
        params.batch_size,
        &params.exec_params,
    )?;
    
    // 发射批量同步事件
    emit!(AssetBatchSynced {
        basket_id: futures_token.id,
        batch_size: params.batch_size,
        operations_count: params.operations.len() as u32,
        operator: operator.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::FuturesToken,
        exec_params: params.exec_params,
    });
    
    Ok(())
} 