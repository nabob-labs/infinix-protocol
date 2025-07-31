//! 稳定币 (Stablecoin) 增发指令
//! 
//! 本模块实现稳定币资产的增发功能，支持抵押品管理、利率调整、风险控制等。
//! 严格遵循最小功能单元原则，确保每个函数职责单一。
//! 
//! ## 功能特性
//! - 抵押品验证：确保增发有足够的抵押品支持
//! - 利率管理：动态调整增发利率
//! - 风险控制：增发前的风险评估
//! - 治理集成：支持治理投票的增发
//! - 事件记录：完整的审计追踪

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams, MintParams};
use crate::services::stablecoin_service::StablecoinService;
use crate::events::asset_event::AssetMinted;
use crate::validation::business::validate_mint_params;
use crate::core::security::check_authority_permission;
use crate::dex::traits::DexAdapterTrait;
use crate::oracles::traits::OracleAdapterTrait;

/// 稳定币增发参数结构体
/// 
/// 定义增发操作所需的所有参数，包括：
/// - amount: 增发数量
/// - collateral_ratio: 抵押品比率要求
/// - interest_rate: 利率设置
/// - risk_params: 风险参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct MintStablecoinParams {
    /// 增发数量
    pub amount: u64,
    /// 抵押品比率要求（基点，1/10000）
    pub collateral_ratio_bps: u16,
    /// 利率设置（基点，1/10000）
    pub interest_rate_bps: u16,
    /// 风险参数
    pub risk_params: Option<crate::core::types::RiskParams>,
    /// 执行参数（可选）
    pub exec_params: Option<ExecutionParams>,
    /// 策略参数（可选）
    pub strategy_params: Option<StrategyParams>,
}

/// 稳定币增发指令账户上下文
/// 
/// 定义增发操作所需的所有账户，包括：
/// - stablecoin_asset: 稳定币资产账户（可变）
/// - authority: 操作权限账户（签名者）
/// - collateral_asset: 抵押品资产账户（可选）
/// - oracle_program: 预言机程序（用于价格验证）
/// - system_program: 系统程序
/// - clock: 时钟程序
#[derive(Accounts)]
#[instruction(params: MintStablecoinParams)]
pub struct MintStablecoin<'info> {
    /// 稳定币资产账户，需要可变权限以更新状态
    #[account(
        mut,
        seeds = [b"stablecoin", stablecoin_asset.key().as_ref()],
        bump,
        constraint = stablecoin_asset.asset_type == AssetType::Stablecoin @ crate::errors::asset_error::AssetError::InvalidAssetType,
        constraint = !stablecoin_asset.is_frozen @ crate::errors::asset_error::AssetError::AssetFrozen
    )]
    pub stablecoin_asset: Account<'info, crate::state::baskets::BasketIndexState>,
    
    /// 操作权限账户，必须是签名者
    #[account(
        constraint = authority.key() == stablecoin_asset.authority @ crate::errors::security_error::SecurityError::InvalidAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 抵押品资产账户（可选），用于抵押品验证
    /// CHECK: 由服务层验证
    pub collateral_asset: Option<UncheckedAccount<'info>>,
    
    /// 预言机程序，用于价格验证
    /// CHECK: 由预言机适配器验证
    pub oracle_program: UncheckedAccount<'info>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 时钟程序
    pub clock: Sysvar<'info, Clock>,
}

/// 稳定币增发指令实现
/// 
/// 执行增发操作，包括：
/// - 参数验证和权限检查
/// - 抵押品验证和风险评估
/// - 利率计算和调整
/// - 事件记录和状态更新
pub fn mint_stablecoin(
    ctx: Context<MintStablecoin>,
    params: MintStablecoinParams
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &mut ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_mint_params(&params)?;
    require!(params.amount > 0, crate::errors::asset_error::AssetError::InvalidAmount);
    require!(params.collateral_ratio_bps > 0, crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "mint"
    )?;
    
    // 3. 调用服务层执行增发
    let stablecoin_service = StablecoinService::new();
    stablecoin_service.mint(
        stablecoin_asset,
        &params,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射增发事件
    emit!(AssetMinted {
        asset_id: stablecoin_asset.id,
        amount: params.amount,
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin mint executed successfully: asset_id={}, amount={}, collateral_ratio_bps={}", 
         stablecoin_asset.id, params.amount, params.collateral_ratio_bps);
    
    Ok(())
}

/// 批量增发稳定币指令实现
/// 
/// 执行批量增发操作，支持多个增发请求的并行处理
pub fn batch_mint_stablecoin(
    ctx: Context<MintStablecoin>,
    params: Vec<MintStablecoinParams>
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &mut ctx.accounts.stablecoin_asset;
    
    // 1. 批量参数验证
    require!(!params.is_empty(), crate::errors::asset_error::AssetError::InvalidParams);
    require!(params.len() <= 10, crate::errors::asset_error::AssetError::BatchTooLarge);
    
    for param in &params {
        validate_mint_params(param)?;
        require!(param.amount > 0, crate::errors::asset_error::AssetError::InvalidAmount);
        require!(param.collateral_ratio_bps > 0, crate::errors::asset_error::AssetError::InvalidParams);
    }
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "batch_mint"
    )?;
    
    // 3. 批量执行增发
    let stablecoin_service = StablecoinService::new();
    for param in params {
        stablecoin_service.mint(
            stablecoin_asset,
            &param,
            ctx.accounts.authority.key()
        )?;
        
        // 发射批量增发事件
        emit!(AssetMinted {
            asset_id: stablecoin_asset.id,
            amount: param.amount,
            authority: ctx.accounts.authority.key(),
            timestamp: ctx.accounts.clock.unix_timestamp,
        });
    }
    
    msg!("[INFO] Stablecoin batch mint executed successfully: asset_id={}, count={}", 
         stablecoin_asset.id, params.len());
    
    Ok(())
}

/// 抵押品增发稳定币指令实现
/// 
/// 执行基于抵押品的增发操作
pub fn collateralized_mint_stablecoin(
    ctx: Context<MintStablecoin>,
    params: MintStablecoinParams,
    collateral_amount: u64
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &mut ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_mint_params(&params)?;
    require!(params.amount > 0, crate::errors::asset_error::AssetError::InvalidAmount);
    require!(collateral_amount > 0, crate::errors::asset_error::AssetError::InvalidAmount);
    require!(params.collateral_ratio_bps > 0, crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "collateralized_mint"
    )?;
    
    // 3. 调用服务层执行抵押品增发
    let stablecoin_service = StablecoinService::new();
    stablecoin_service.collateralized_mint(
        stablecoin_asset,
        &params,
        collateral_amount,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射抵押品增发事件
    emit!(AssetMinted {
        asset_id: stablecoin_asset.id,
        amount: params.amount,
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin collateralized mint executed successfully: asset_id={}, amount={}, collateral_amount={}", 
         stablecoin_asset.id, params.amount, collateral_amount);
    
    Ok(())
}

/// 治理增发稳定币指令实现
/// 
/// 执行基于治理投票的增发操作
pub fn governance_mint_stablecoin(
    ctx: Context<MintStablecoin>,
    params: MintStablecoinParams,
    proposal_id: u64
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &mut ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_mint_params(&params)?;
    require!(params.amount > 0, crate::errors::asset_error::AssetError::InvalidAmount);
    require!(proposal_id > 0, crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "governance_mint"
    )?;
    
    // 3. 调用服务层执行治理增发
    let stablecoin_service = StablecoinService::new();
    stablecoin_service.governance_mint(
        stablecoin_asset,
        &params,
        proposal_id,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射治理增发事件
    emit!(AssetMinted {
        asset_id: stablecoin_asset.id,
        amount: params.amount,
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin governance mint executed successfully: asset_id={}, amount={}, proposal_id={}", 
         stablecoin_asset.id, params.amount, proposal_id);
    
    Ok(())
} 