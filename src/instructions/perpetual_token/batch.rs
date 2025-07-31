//! Perpetual Token批量操作指令模块
//! 
//! 本模块提供Perpetual Token资产的批量操作功能，包括：
//! - 批量交易：批量执行Perpetual Token交易操作
//! - 批量处理：批量处理Perpetual Token相关操作
//! - 批量管理：批量管理Perpetual Token状态
//! - 批量同步：批量同步Perpetual Token数据
//! 
//! 设计特点：
//! - 最小功能单元：专注于Perpetual Token批量操作功能
//! - 类型安全：严格的类型检查和边界验证
//! - 参数验证：全面的输入参数验证
//! - 权限控制：细粒度的权限验证
//! - 服务层抽象：核心业务逻辑委托给PerpetualTokenService
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

/// 批量交易Perpetual Token参数结构体
/// 
/// 包含批量交易Perpetual Token所需的所有参数：
/// - operations: 批量操作列表
/// - batch_size: 批量大小
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct BatchTradePerpetualParams {
    /// 批量操作列表
    pub operations: Vec<PerpetualTradeOperation>,
    /// 批量大小
    pub batch_size: u32,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 批量处理Perpetual Token参数结构体
/// 
/// 包含批量处理Perpetual Token所需的所有参数：
/// - operations: 批量处理操作列表
/// - batch_size: 批量大小
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct BatchProcessPerpetualParams {
    /// 批量处理操作列表
    pub operations: Vec<PerpetualProcessOperation>,
    /// 批量大小
    pub batch_size: u32,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 批量管理Perpetual Token参数结构体
/// 
/// 包含批量管理Perpetual Token所需的所有参数：
/// - operations: 批量管理操作列表
/// - batch_size: 批量大小
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct BatchManagePerpetualParams {
    /// 批量管理操作列表
    pub operations: Vec<PerpetualManageOperation>,
    /// 批量大小
    pub batch_size: u32,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 批量同步Perpetual Token参数结构体
/// 
/// 包含批量同步Perpetual Token所需的所有参数：
/// - operations: 批量同步操作列表
/// - batch_size: 批量大小
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct BatchSyncPerpetualParams {
    /// 批量同步操作列表
    pub operations: Vec<PerpetualSyncOperation>,
    /// 批量大小
    pub batch_size: u32,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// Perpetual Token交易操作结构体
/// 
/// 定义Perpetual Token交易操作的类型和参数：
/// - operation_type: 操作类型
/// - position_id: 仓位ID
/// - amount: 数量
/// - price: 价格
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct PerpetualTradeOperation {
    /// 操作类型
    pub operation_type: PerpetualTradeType,
    /// 仓位ID
    pub position_id: Pubkey,
    /// 数量
    pub amount: u64,
    /// 价格
    pub price: f64,
}

/// Perpetual Token处理操作结构体
/// 
/// 定义Perpetual Token处理操作的类型和参数：
/// - operation_type: 操作类型
/// - position_id: 仓位ID
/// - params: 处理参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct PerpetualProcessOperation {
    /// 操作类型
    pub operation_type: PerpetualProcessType,
    /// 仓位ID
    pub position_id: Pubkey,
    /// 处理参数
    pub params: Vec<u8>,
}

/// Perpetual Token管理操作结构体
/// 
/// 定义Perpetual Token管理操作的类型和参数：
/// - operation_type: 操作类型
/// - position_id: 仓位ID
/// - params: 管理参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct PerpetualManageOperation {
    /// 操作类型
    pub operation_type: PerpetualManageType,
    /// 仓位ID
    pub position_id: Pubkey,
    /// 管理参数
    pub params: Vec<u8>,
}

/// Perpetual Token同步操作结构体
/// 
/// 定义Perpetual Token同步操作的类型和参数：
/// - operation_type: 操作类型
/// - position_id: 仓位ID
/// - params: 同步参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct PerpetualSyncOperation {
    /// 操作类型
    pub operation_type: PerpetualSyncType,
    /// 仓位ID
    pub position_id: Pubkey,
    /// 同步参数
    pub params: Vec<u8>,
}

/// Perpetual Token交易类型枚举
/// 
/// 定义Perpetual Token交易的类型：
/// - Open: 开仓
/// - Close: 平仓
/// - Adjust: 调整
/// - Hedge: 对冲
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum PerpetualTradeType {
    /// 开仓
    Open,
    /// 平仓
    Close,
    /// 调整
    Adjust,
    /// 对冲
    Hedge,
}

/// Perpetual Token处理类型枚举
/// 
/// 定义Perpetual Token处理的类型：
/// - FundingRate: 资金费率
/// - Liquidation: 清算
/// - MarginCall: 保证金催缴
/// - Settlement: 结算
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum PerpetualProcessType {
    /// 资金费率
    FundingRate,
    /// 清算
    Liquidation,
    /// 保证金催缴
    MarginCall,
    /// 结算
    Settlement,
}

/// Perpetual Token管理类型枚举
/// 
/// 定义Perpetual Token管理的类型：
/// - PositionUpdate: 仓位更新
/// - RiskUpdate: 风险更新
/// - ConfigUpdate: 配置更新
/// - Maintenance: 维护
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum PerpetualManageType {
    /// 仓位更新
    PositionUpdate,
    /// 风险更新
    RiskUpdate,
    /// 配置更新
    ConfigUpdate,
    /// 维护
    Maintenance,
}

/// Perpetual Token同步类型枚举
/// 
/// 定义Perpetual Token同步的类型：
/// - PriceSync: 价格同步
/// - StateSync: 状态同步
/// - DataSync: 数据同步
/// - ConfigSync: 配置同步
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum PerpetualSyncType {
    /// 价格同步
    PriceSync,
    /// 状态同步
    StateSync,
    /// 数据同步
    DataSync,
    /// 配置同步
    ConfigSync,
}

/// 批量Perpetual Token操作账户上下文
/// 
/// 定义批量Perpetual Token操作指令所需的账户结构：
/// - perpetual_token: Perpetual Token账户（可变，Perpetual Token类型约束）
/// - operator: 操作者账户（owner约束）
/// - perpetual_pool: 永续池账户
/// - system_program: 系统程序
/// - token_program: 代币程序
#[derive(Accounts)]
pub struct BatchPerpetual<'info> {
    /// Perpetual Token账户（可变，Perpetual Token类型约束）
    #[account(
        mut,
        constraint = perpetual_token.asset_type == AssetType::PerpetualToken @ AssetError::InvalidAssetType
    )]
    pub perpetual_token: Account<'info, Asset>,
    
    /// 操作者账户（owner约束）
    #[account(
        constraint = operator.key() == perpetual_token.owner @ AssetError::InvalidOwner
    )]
    pub operator: Signer<'info>,
    
    /// 永续池账户
    /// CHECK: 由程序验证
    #[account(mut)]
    pub perpetual_pool: UncheckedAccount<'info>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 验证批量交易Perpetual Token参数
/// 
/// 检查批量交易Perpetual Token参数的有效性和边界条件：
/// - 操作列表验证
/// - 批量大小验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: 批量交易Perpetual Token参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_batch_trade_perpetual_params(params: &BatchTradePerpetualParams) -> Result<()> {
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

/// 验证批量处理Perpetual Token参数
/// 
/// 检查批量处理Perpetual Token参数的有效性和边界条件：
/// - 操作列表验证
/// - 批量大小验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: 批量处理Perpetual Token参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_batch_process_perpetual_params(params: &BatchProcessPerpetualParams) -> Result<()> {
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

/// 验证批量管理Perpetual Token参数
/// 
/// 检查批量管理Perpetual Token参数的有效性和边界条件：
/// - 操作列表验证
/// - 批量大小验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: 批量管理Perpetual Token参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_batch_manage_perpetual_params(params: &BatchManagePerpetualParams) -> Result<()> {
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

/// 验证批量同步Perpetual Token参数
/// 
/// 检查批量同步Perpetual Token参数的有效性和边界条件：
/// - 操作列表验证
/// - 批量大小验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: 批量同步Perpetual Token参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_batch_sync_perpetual_params(params: &BatchSyncPerpetualParams) -> Result<()> {
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

/// 检查批量Perpetual Token操作权限
/// 
/// 验证批量Perpetual Token操作权限和授权状态：
/// - 检查所有权
/// - 验证Perpetual Token状态
/// - 检查操作权限
/// 
/// # 参数
/// - operator: 操作者账户
/// - perpetual_token: Perpetual Token账户
/// 
/// # 返回
/// - Result<()>: 权限检查结果
pub fn check_batch_perpetual_authority_permission(
    operator: &Signer,
    perpetual_token: &Account<Asset>,
) -> Result<()> {
    // 检查所有权
    require!(
        operator.key() == perpetual_token.owner,
        AssetError::InvalidOwner
    );
    
    // 验证Perpetual Token状态
    require!(
        perpetual_token.is_active(),
        AssetError::AssetNotActive
    );
    
    Ok(())
}

/// 批量交易Perpetual Token指令
/// 
/// 执行批量交易Perpetual Token操作，包括：
/// - 参数验证：验证批量交易参数的有效性
/// - 权限检查：验证批量交易权限
/// - 服务层调用：委托给PerpetualTokenService执行批量交易逻辑
/// - 事件发射：发射Perpetual Token批量交易事件
/// 
/// # 参数
/// - ctx: 批量Perpetual Token操作账户上下文
/// - params: 批量交易Perpetual Token参数
/// 
/// # 返回
/// - Result<()>: 批量交易操作结果
pub fn batch_trade_perpetual(
    ctx: Context<BatchPerpetual>,
    params: BatchTradePerpetualParams,
) -> Result<()> {
    // 参数验证
    validate_batch_trade_perpetual_params(&params)?;
    
    // 权限检查
    check_batch_perpetual_authority_permission(
        &ctx.accounts.operator,
        &ctx.accounts.perpetual_token,
    )?;
    
    let perpetual_token = &mut ctx.accounts.perpetual_token;
    let operator = &ctx.accounts.operator;
    
    // 创建Perpetual Token服务实例
    let service = PerpetualTokenService::new();
    
    // 执行批量交易
    service.batch_trade(
        perpetual_token,
        &params.operations,
        params.batch_size,
        &params.exec_params,
    )?;
    
    // 发射批量交易事件
    emit!(AssetBatchTraded {
        basket_id: perpetual_token.id,
        batch_size: params.batch_size,
        operations_count: params.operations.len() as u32,
        operator: operator.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::PerpetualToken,
        exec_params: params.exec_params,
    });
    
    Ok(())
}

/// 批量处理Perpetual Token指令
/// 
/// 执行批量处理Perpetual Token操作，包括：
/// - 参数验证：验证批量处理参数的有效性
/// - 权限检查：验证批量处理权限
/// - 服务层调用：委托给PerpetualTokenService执行批量处理逻辑
/// - 事件发射：发射Perpetual Token批量处理事件
/// 
/// # 参数
/// - ctx: 批量Perpetual Token操作账户上下文
/// - params: 批量处理Perpetual Token参数
/// 
/// # 返回
/// - Result<()>: 批量处理操作结果
pub fn batch_process_perpetual(
    ctx: Context<BatchPerpetual>,
    params: BatchProcessPerpetualParams,
) -> Result<()> {
    // 参数验证
    validate_batch_process_perpetual_params(&params)?;
    
    // 权限检查
    check_batch_perpetual_authority_permission(
        &ctx.accounts.operator,
        &ctx.accounts.perpetual_token,
    )?;
    
    let perpetual_token = &mut ctx.accounts.perpetual_token;
    let operator = &ctx.accounts.operator;
    
    // 创建Perpetual Token服务实例
    let service = PerpetualTokenService::new();
    
    // 执行批量处理
    service.batch_process(
        perpetual_token,
        &params.operations,
        params.batch_size,
        &params.exec_params,
    )?;
    
    // 发射批量处理事件
    emit!(AssetBatchProcessed {
        basket_id: perpetual_token.id,
        batch_size: params.batch_size,
        operations_count: params.operations.len() as u32,
        operator: operator.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::PerpetualToken,
        exec_params: params.exec_params,
    });
    
    Ok(())
}

/// 批量管理Perpetual Token指令
/// 
/// 执行批量管理Perpetual Token操作，包括：
/// - 参数验证：验证批量管理参数的有效性
/// - 权限检查：验证批量管理权限
/// - 服务层调用：委托给PerpetualTokenService执行批量管理逻辑
/// - 事件发射：发射Perpetual Token批量管理事件
/// 
/// # 参数
/// - ctx: 批量Perpetual Token操作账户上下文
/// - params: 批量管理Perpetual Token参数
/// 
/// # 返回
/// - Result<()>: 批量管理操作结果
pub fn batch_manage_perpetual(
    ctx: Context<BatchPerpetual>,
    params: BatchManagePerpetualParams,
) -> Result<()> {
    // 参数验证
    validate_batch_manage_perpetual_params(&params)?;
    
    // 权限检查
    check_batch_perpetual_authority_permission(
        &ctx.accounts.operator,
        &ctx.accounts.perpetual_token,
    )?;
    
    let perpetual_token = &mut ctx.accounts.perpetual_token;
    let operator = &ctx.accounts.operator;
    
    // 创建Perpetual Token服务实例
    let service = PerpetualTokenService::new();
    
    // 执行批量管理
    service.batch_manage(
        perpetual_token,
        &params.operations,
        params.batch_size,
        &params.exec_params,
    )?;
    
    // 发射批量管理事件
    emit!(AssetBatchManaged {
        basket_id: perpetual_token.id,
        batch_size: params.batch_size,
        operations_count: params.operations.len() as u32,
        operator: operator.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::PerpetualToken,
        exec_params: params.exec_params,
    });
    
    Ok(())
}

/// 批量同步Perpetual Token指令
/// 
/// 执行批量同步Perpetual Token操作，包括：
/// - 参数验证：验证批量同步参数的有效性
/// - 权限检查：验证批量同步权限
/// - 服务层调用：委托给PerpetualTokenService执行批量同步逻辑
/// - 事件发射：发射Perpetual Token批量同步事件
/// 
/// # 参数
/// - ctx: 批量Perpetual Token操作账户上下文
/// - params: 批量同步Perpetual Token参数
/// 
/// # 返回
/// - Result<()>: 批量同步操作结果
pub fn batch_sync_perpetual(
    ctx: Context<BatchPerpetual>,
    params: BatchSyncPerpetualParams,
) -> Result<()> {
    // 参数验证
    validate_batch_sync_perpetual_params(&params)?;
    
    // 权限检查
    check_batch_perpetual_authority_permission(
        &ctx.accounts.operator,
        &ctx.accounts.perpetual_token,
    )?;
    
    let perpetual_token = &mut ctx.accounts.perpetual_token;
    let operator = &ctx.accounts.operator;
    
    // 创建Perpetual Token服务实例
    let service = PerpetualTokenService::new();
    
    // 执行批量同步
    service.batch_sync(
        perpetual_token,
        &params.operations,
        params.batch_size,
        &params.exec_params,
    )?;
    
    // 发射批量同步事件
    emit!(AssetBatchSynced {
        basket_id: perpetual_token.id,
        batch_size: params.batch_size,
        operations_count: params.operations.len() as u32,
        operator: operator.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::PerpetualToken,
        exec_params: params.exec_params,
    });
    
    Ok(())
} 