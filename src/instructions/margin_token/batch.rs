//! Margin Token批量操作指令模块
//! 
//! 本模块提供Margin Token资产的批量操作功能，包括：
//! - 批量交易：批量执行Margin Token交易操作
//! - 批量处理：批量处理Margin Token相关操作
//! - 批量管理：批量管理Margin Token状态
//! - 批量同步：批量同步Margin Token数据
//! 
//! 设计特点：
//! - 最小功能单元：专注于Margin Token批量操作功能
//! - 类型安全：严格的类型检查和边界验证
//! - 参数验证：全面的输入参数验证
//! - 权限控制：细粒度的权限验证
//! - 服务层抽象：核心业务逻辑委托给MarginTokenService
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

/// 批量交易Margin Token参数结构体
/// 
/// 包含批量交易Margin Token所需的所有参数：
/// - operations: 批量操作列表
/// - batch_size: 批量大小
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct BatchTradeMarginParams {
    /// 批量操作列表
    pub operations: Vec<MarginTradeOperation>,
    /// 批量大小
    pub batch_size: u32,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 批量处理Margin Token参数结构体
/// 
/// 包含批量处理Margin Token所需的所有参数：
/// - operations: 批量处理操作列表
/// - batch_size: 批量大小
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct BatchProcessMarginParams {
    /// 批量处理操作列表
    pub operations: Vec<MarginProcessOperation>,
    /// 批量大小
    pub batch_size: u32,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 批量管理Margin Token参数结构体
/// 
/// 包含批量管理Margin Token所需的所有参数：
/// - operations: 批量管理操作列表
/// - batch_size: 批量大小
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct BatchManageMarginParams {
    /// 批量管理操作列表
    pub operations: Vec<MarginManageOperation>,
    /// 批量大小
    pub batch_size: u32,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 批量同步Margin Token参数结构体
/// 
/// 包含批量同步Margin Token所需的所有参数：
/// - operations: 批量同步操作列表
/// - batch_size: 批量大小
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct BatchSyncMarginParams {
    /// 批量同步操作列表
    pub operations: Vec<MarginSyncOperation>,
    /// 批量大小
    pub batch_size: u32,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// Margin Token交易操作结构体
/// 
/// 定义Margin Token交易操作的类型和参数：
/// - operation_type: 操作类型
/// - position_id: 仓位ID
/// - amount: 数量
/// - rate: 比率
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct MarginTradeOperation {
    /// 操作类型
    pub operation_type: MarginTradeType,
    /// 仓位ID
    pub position_id: Pubkey,
    /// 数量
    pub amount: u64,
    /// 比率
    pub rate: f64,
}

/// Margin Token处理操作结构体
/// 
/// 定义Margin Token处理操作的类型和参数：
/// - operation_type: 操作类型
/// - position_id: 仓位ID
/// - params: 处理参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct MarginProcessOperation {
    /// 操作类型
    pub operation_type: MarginProcessType,
    /// 仓位ID
    pub position_id: Pubkey,
    /// 处理参数
    pub params: Vec<u8>,
}

/// Margin Token管理操作结构体
/// 
/// 定义Margin Token管理操作的类型和参数：
/// - operation_type: 操作类型
/// - position_id: 仓位ID
/// - params: 管理参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct MarginManageOperation {
    /// 操作类型
    pub operation_type: MarginManageType,
    /// 仓位ID
    pub position_id: Pubkey,
    /// 管理参数
    pub params: Vec<u8>,
}

/// Margin Token同步操作结构体
/// 
/// 定义Margin Token同步操作的类型和参数：
/// - operation_type: 操作类型
/// - position_id: 仓位ID
/// - params: 同步参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct MarginSyncOperation {
    /// 操作类型
    pub operation_type: MarginSyncType,
    /// 仓位ID
    pub position_id: Pubkey,
    /// 同步参数
    pub params: Vec<u8>,
}

/// Margin Token交易类型枚举
/// 
/// 定义Margin Token交易的类型：
/// - Borrow: 借入
/// - Repay: 偿还
/// - Liquidate: 清算
/// - Adjust: 调整
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum MarginTradeType {
    /// 借入
    Borrow,
    /// 偿还
    Repay,
    /// 清算
    Liquidate,
    /// 调整
    Adjust,
}

/// Margin Token处理类型枚举
/// 
/// 定义Margin Token处理的类型：
/// - InterestAccrual: 利息累积
/// - MarginCall: 保证金催缴
/// - Liquidation: 清算
/// - Settlement: 结算
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum MarginProcessType {
    /// 利息累积
    InterestAccrual,
    /// 保证金催缴
    MarginCall,
    /// 清算
    Liquidation,
    /// 结算
    Settlement,
}

/// Margin Token管理类型枚举
/// 
/// 定义Margin Token管理的类型：
/// - PositionUpdate: 仓位更新
/// - RiskUpdate: 风险更新
/// - ConfigUpdate: 配置更新
/// - Maintenance: 维护
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum MarginManageType {
    /// 仓位更新
    PositionUpdate,
    /// 风险更新
    RiskUpdate,
    /// 配置更新
    ConfigUpdate,
    /// 维护
    Maintenance,
}

/// Margin Token同步类型枚举
/// 
/// 定义Margin Token同步的类型：
/// - PriceSync: 价格同步
/// - StateSync: 状态同步
/// - DataSync: 数据同步
/// - ConfigSync: 配置同步
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum MarginSyncType {
    /// 价格同步
    PriceSync,
    /// 状态同步
    StateSync,
    /// 数据同步
    DataSync,
    /// 配置同步
    ConfigSync,
}

/// 批量Margin Token操作账户上下文
/// 
/// 定义批量Margin Token操作指令所需的账户结构：
/// - margin_token: Margin Token账户（可变，Margin Token类型约束）
/// - operator: 操作者账户（owner约束）
/// - margin_pool: 保证金池账户
/// - system_program: 系统程序
/// - token_program: 代币程序
#[derive(Accounts)]
pub struct BatchMargin<'info> {
    /// Margin Token账户（可变，Margin Token类型约束）
    #[account(
        mut,
        constraint = margin_token.asset_type == AssetType::MarginToken @ AssetError::InvalidAssetType
    )]
    pub margin_token: Account<'info, Asset>,
    
    /// 操作者账户（owner约束）
    #[account(
        constraint = operator.key() == margin_token.owner @ AssetError::InvalidOwner
    )]
    pub operator: Signer<'info>,
    
    /// 保证金池账户
    /// CHECK: 由程序验证
    #[account(mut)]
    pub margin_pool: UncheckedAccount<'info>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 验证批量交易Margin Token参数
/// 
/// 检查批量交易Margin Token参数的有效性和边界条件：
/// - 操作列表验证
/// - 批量大小验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: 批量交易Margin Token参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_batch_trade_margin_params(params: &BatchTradeMarginParams) -> Result<()> {
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

/// 验证批量处理Margin Token参数
/// 
/// 检查批量处理Margin Token参数的有效性和边界条件：
/// - 操作列表验证
/// - 批量大小验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: 批量处理Margin Token参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_batch_process_margin_params(params: &BatchProcessMarginParams) -> Result<()> {
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

/// 验证批量管理Margin Token参数
/// 
/// 检查批量管理Margin Token参数的有效性和边界条件：
/// - 操作列表验证
/// - 批量大小验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: 批量管理Margin Token参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_batch_manage_margin_params(params: &BatchManageMarginParams) -> Result<()> {
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

/// 验证批量同步Margin Token参数
/// 
/// 检查批量同步Margin Token参数的有效性和边界条件：
/// - 操作列表验证
/// - 批量大小验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: 批量同步Margin Token参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_batch_sync_margin_params(params: &BatchSyncMarginParams) -> Result<()> {
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

/// 检查批量Margin Token操作权限
/// 
/// 验证批量Margin Token操作权限和授权状态：
/// - 检查所有权
/// - 验证Margin Token状态
/// - 检查操作权限
/// 
/// # 参数
/// - operator: 操作者账户
/// - margin_token: Margin Token账户
/// 
/// # 返回
/// - Result<()>: 权限检查结果
pub fn check_batch_margin_authority_permission(
    operator: &Signer,
    margin_token: &Account<Asset>,
) -> Result<()> {
    // 检查所有权
    require!(
        operator.key() == margin_token.owner,
        AssetError::InvalidOwner
    );
    
    // 验证Margin Token状态
    require!(
        margin_token.is_active(),
        AssetError::AssetNotActive
    );
    
    Ok(())
}

/// 批量交易Margin Token指令
/// 
/// 执行批量交易Margin Token操作，包括：
/// - 参数验证：验证批量交易参数的有效性
/// - 权限检查：验证批量交易权限
/// - 服务层调用：委托给MarginTokenService执行批量交易逻辑
/// - 事件发射：发射Margin Token批量交易事件
/// 
/// # 参数
/// - ctx: 批量Margin Token操作账户上下文
/// - params: 批量交易Margin Token参数
/// 
/// # 返回
/// - Result<()>: 批量交易操作结果
pub fn batch_trade_margin(
    ctx: Context<BatchMargin>,
    params: BatchTradeMarginParams,
) -> Result<()> {
    // 参数验证
    validate_batch_trade_margin_params(&params)?;
    
    // 权限检查
    check_batch_margin_authority_permission(
        &ctx.accounts.operator,
        &ctx.accounts.margin_token,
    )?;
    
    let margin_token = &mut ctx.accounts.margin_token;
    let operator = &ctx.accounts.operator;
    
    // 创建Margin Token服务实例
    let service = MarginTokenService::new();
    
    // 执行批量交易
    service.batch_trade(
        margin_token,
        &params.operations,
        params.batch_size,
        &params.exec_params,
    )?;
    
    // 发射批量交易事件
    emit!(AssetBatchTraded {
        basket_id: margin_token.id,
        batch_size: params.batch_size,
        operations_count: params.operations.len() as u32,
        operator: operator.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::MarginToken,
        exec_params: params.exec_params,
    });
    
    Ok(())
}

/// 批量处理Margin Token指令
/// 
/// 执行批量处理Margin Token操作，包括：
/// - 参数验证：验证批量处理参数的有效性
/// - 权限检查：验证批量处理权限
/// - 服务层调用：委托给MarginTokenService执行批量处理逻辑
/// - 事件发射：发射Margin Token批量处理事件
/// 
/// # 参数
/// - ctx: 批量Margin Token操作账户上下文
/// - params: 批量处理Margin Token参数
/// 
/// # 返回
/// - Result<()>: 批量处理操作结果
pub fn batch_process_margin(
    ctx: Context<BatchMargin>,
    params: BatchProcessMarginParams,
) -> Result<()> {
    // 参数验证
    validate_batch_process_margin_params(&params)?;
    
    // 权限检查
    check_batch_margin_authority_permission(
        &ctx.accounts.operator,
        &ctx.accounts.margin_token,
    )?;
    
    let margin_token = &mut ctx.accounts.margin_token;
    let operator = &ctx.accounts.operator;
    
    // 创建Margin Token服务实例
    let service = MarginTokenService::new();
    
    // 执行批量处理
    service.batch_process(
        margin_token,
        &params.operations,
        params.batch_size,
        &params.exec_params,
    )?;
    
    // 发射批量处理事件
    emit!(AssetBatchProcessed {
        basket_id: margin_token.id,
        batch_size: params.batch_size,
        operations_count: params.operations.len() as u32,
        operator: operator.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::MarginToken,
        exec_params: params.exec_params,
    });
    
    Ok(())
}

/// 批量管理Margin Token指令
/// 
/// 执行批量管理Margin Token操作，包括：
/// - 参数验证：验证批量管理参数的有效性
/// - 权限检查：验证批量管理权限
/// - 服务层调用：委托给MarginTokenService执行批量管理逻辑
/// - 事件发射：发射Margin Token批量管理事件
/// 
/// # 参数
/// - ctx: 批量Margin Token操作账户上下文
/// - params: 批量管理Margin Token参数
/// 
/// # 返回
/// - Result<()>: 批量管理操作结果
pub fn batch_manage_margin(
    ctx: Context<BatchMargin>,
    params: BatchManageMarginParams,
) -> Result<()> {
    // 参数验证
    validate_batch_manage_margin_params(&params)?;
    
    // 权限检查
    check_batch_margin_authority_permission(
        &ctx.accounts.operator,
        &ctx.accounts.margin_token,
    )?;
    
    let margin_token = &mut ctx.accounts.margin_token;
    let operator = &ctx.accounts.operator;
    
    // 创建Margin Token服务实例
    let service = MarginTokenService::new();
    
    // 执行批量管理
    service.batch_manage(
        margin_token,
        &params.operations,
        params.batch_size,
        &params.exec_params,
    )?;
    
    // 发射批量管理事件
    emit!(AssetBatchManaged {
        basket_id: margin_token.id,
        batch_size: params.batch_size,
        operations_count: params.operations.len() as u32,
        operator: operator.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::MarginToken,
        exec_params: params.exec_params,
    });
    
    Ok(())
}

/// 批量同步Margin Token指令
/// 
/// 执行批量同步Margin Token操作，包括：
/// - 参数验证：验证批量同步参数的有效性
/// - 权限检查：验证批量同步权限
/// - 服务层调用：委托给MarginTokenService执行批量同步逻辑
/// - 事件发射：发射Margin Token批量同步事件
/// 
/// # 参数
/// - ctx: 批量Margin Token操作账户上下文
/// - params: 批量同步Margin Token参数
/// 
/// # 返回
/// - Result<()>: 批量同步操作结果
pub fn batch_sync_margin(
    ctx: Context<BatchMargin>,
    params: BatchSyncMarginParams,
) -> Result<()> {
    // 参数验证
    validate_batch_sync_margin_params(&params)?;
    
    // 权限检查
    check_batch_margin_authority_permission(
        &ctx.accounts.operator,
        &ctx.accounts.margin_token,
    )?;
    
    let margin_token = &mut ctx.accounts.margin_token;
    let operator = &ctx.accounts.operator;
    
    // 创建Margin Token服务实例
    let service = MarginTokenService::new();
    
    // 执行批量同步
    service.batch_sync(
        margin_token,
        &params.operations,
        params.batch_size,
        &params.exec_params,
    )?;
    
    // 发射批量同步事件
    emit!(AssetBatchSynced {
        basket_id: margin_token.id,
        batch_size: params.batch_size,
        operations_count: params.operations.len() as u32,
        operator: operator.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::MarginToken,
        exec_params: params.exec_params,
    });
    
    Ok(())
} 