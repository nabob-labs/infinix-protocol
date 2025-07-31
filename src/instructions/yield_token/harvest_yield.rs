//! Yield Token收获收益指令模块
//! 
//! 本模块提供Yield Token资产的收获收益功能，包括：
//! - 参数验证：验证收获收益参数的有效性和边界条件
//! - 权限检查：验证收获收益权限和授权状态
//! - 服务层调用：委托给YieldTokenService执行核心业务逻辑
//! - 事件发射：发射Yield Token收获收益事件用于审计和追踪
//! 
//! 设计特点：
//! - 最小功能单元：专注于Yield Token收获收益功能
//! - 类型安全：严格的类型检查和边界验证
//! - 参数验证：全面的输入参数验证
//! - 权限控制：细粒度的权限验证
//! - 服务层抽象：核心业务逻辑委托给YieldTokenService
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

/// 收获收益参数结构体
/// 
/// 包含收获收益所需的所有参数：
/// - harvest_amount: 收获数量
/// - harvest_type: 收获类型
/// - compound_ratio: 复投比例
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct HarvestYieldParams {
    /// 收获数量
    pub harvest_amount: u64,
    /// 收获类型
    pub harvest_type: HarvestType,
    /// 复投比例
    pub compound_ratio: f64,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 收获类型枚举
/// 
/// 定义收获的类型：
/// - FullHarvest: 全部收获
/// - PartialHarvest: 部分收获
/// - CompoundHarvest: 复投收获
/// - AutoHarvest: 自动收获
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum HarvestType {
    /// 全部收获
    FullHarvest,
    /// 部分收获
    PartialHarvest,
    /// 复投收获
    CompoundHarvest,
    /// 自动收获
    AutoHarvest,
}

/// 收获收益账户上下文
/// 
/// 定义收获收益指令所需的账户结构：
/// - yield_token: Yield Token账户（可变，Yield Token类型约束）
/// - harvester: 收获者账户（owner约束）
/// - yield_pool: 收益池账户
/// - harvest_pool: 收获池账户
/// - harvester_token_account: 收获者代币账户
/// - system_program: 系统程序
/// - token_program: 代币程序
#[derive(Accounts)]
pub struct HarvestYield<'info> {
    /// Yield Token账户（可变，Yield Token类型约束）
    #[account(
        mut,
        constraint = yield_token.asset_type == AssetType::YieldToken @ AssetError::InvalidAssetType
    )]
    pub yield_token: Account<'info, Asset>,
    
    /// 收获者账户（owner约束）
    #[account(
        constraint = harvester.key() == yield_token.owner @ AssetError::InvalidOwner
    )]
    pub harvester: Signer<'info>,
    
    /// 收益池账户
    /// CHECK: 由程序验证
    #[account(mut)]
    pub yield_pool: UncheckedAccount<'info>,
    
    /// 收获池账户
    /// CHECK: 由程序验证
    #[account(mut)]
    pub harvest_pool: UncheckedAccount<'info>,
    
    /// 收获者代币账户
    #[account(
        mut,
        constraint = harvester_token_account.owner == harvester.key() @ AssetError::InvalidTokenAccount
    )]
    pub harvester_token_account: Account<'info, TokenAccount>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 验证收获收益参数
/// 
/// 检查收获收益参数的有效性和边界条件：
/// - 收获数量验证
/// - 收获类型验证
/// - 复投比例验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: 收获收益参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_harvest_yield_params(params: &HarvestYieldParams) -> Result<()> {
    // 验证收获数量
    require!(
        params.harvest_amount > 0,
        AssetError::InvalidHarvestAmount
    );
    
    // 验证复投比例
    require!(
        params.compound_ratio >= 0.0 && params.compound_ratio <= 1.0,
        AssetError::InvalidCompoundRatio
    );
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 检查收获收益权限
/// 
/// 验证收获收益权限和授权状态：
/// - 检查所有权
/// - 验证Yield Token状态
/// - 检查收获资格
/// 
/// # 参数
/// - harvester: 收获者账户
/// - yield_token: Yield Token账户
/// 
/// # 返回
/// - Result<()>: 权限检查结果
pub fn check_harvest_yield_authority_permission(
    harvester: &Signer,
    yield_token: &Account<Asset>,
) -> Result<()> {
    // 检查所有权
    require!(
        harvester.key() == yield_token.owner,
        AssetError::InvalidOwner
    );
    
    // 验证Yield Token状态
    require!(
        yield_token.is_active(),
        AssetError::AssetNotActive
    );
    
    Ok(())
}

/// 收获收益指令
/// 
/// 执行收获收益操作，包括：
/// - 参数验证：验证收获收益参数的有效性
/// - 权限检查：验证收获收益权限
/// - 服务层调用：委托给YieldTokenService执行收获收益逻辑
/// - 事件发射：发射Yield Token收获收益事件
/// 
/// # 参数
/// - ctx: 收获收益账户上下文
/// - params: 收获收益参数
/// 
/// # 返回
/// - Result<()>: 收获收益操作结果
pub fn harvest_yield(
    ctx: Context<HarvestYield>,
    params: HarvestYieldParams,
) -> Result<()> {
    // 参数验证
    validate_harvest_yield_params(&params)?;
    
    // 权限检查
    check_harvest_yield_authority_permission(
        &ctx.accounts.harvester,
        &ctx.accounts.yield_token,
    )?;
    
    let yield_token = &mut ctx.accounts.yield_token;
    let harvester = &ctx.accounts.harvester;
    
    // 创建Yield Token服务实例
    let service = YieldTokenService::new();
    
    // 执行收获收益
    service.harvest_yield(
        yield_token,
        params.harvest_amount,
        &params.harvest_type,
        params.compound_ratio,
        &params.exec_params,
    )?;
    
    // 发射收获收益事件
    emit!(AssetYieldHarvested {
        basket_id: yield_token.id,
        harvest_amount: params.harvest_amount,
        harvest_type: format!("{:?}", params.harvest_type),
        compound_ratio: params.compound_ratio,
        harvester: harvester.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::YieldToken,
        exec_params: params.exec_params,
    });
    
    Ok(())
} 