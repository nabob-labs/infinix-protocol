//! 加密货币 (Crypto) 授权指令
//! 
//! 本模块实现加密货币资产的授权功能，支持角色授权、批量授权、权限管理等。
//! 严格遵循最小功能单元原则，确保每个函数职责单一。
//! 
//! ## 功能特性
//! - 角色授权：支持多种角色和权限级别
//! - 批量授权：支持批量授权操作
//! - 权限管理：细粒度权限控制
//! - 事件记录：完整的审计追踪
//! - 状态管理：原子性操作保证数据一致性

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams, AuthorizeParams};
use crate::services::crypto_service::CryptoService;
use crate::events::asset_event::AssetAuthorized;
use crate::validation::business::validate_authorize_params;
use crate::core::security::check_authority_permission;

/// 加密货币授权指令账户上下文
/// 
/// 定义授权操作所需的所有账户，包括：
/// - crypto_asset: 加密货币资产账户（可变）
/// - authority: 操作权限账户（签名者）
/// - delegate: 被授权账户
/// - system_program: 系统程序（用于账户管理）
#[derive(Accounts)]
#[instruction(params: AuthorizeParams)]
pub struct AuthorizeCrypto<'info> {
    /// 加密货币资产账户，需要可变权限以更新授权状态
    #[account(
        mut,
        seeds = [b"crypto", crypto_asset.key().as_ref()],
        bump,
        constraint = crypto_asset.asset_type == AssetType::Crypto @ crate::errors::AssetError::InvalidAssetType
    )]
    pub crypto_asset: Account<'info, crate::account_models::asset::Asset>,
    
    /// 操作权限账户，必须是签名者
    #[account(
        constraint = check_authority_permission(&authority.key(), &crypto_asset.authority) @ crate::errors::SecurityError::Unauthorized
    )]
    pub authority: Signer<'info>,
    
    /// 被授权账户
    #[account(
        constraint = delegate.key() != authority.key() @ crate::errors::SecurityError::SelfAuthorization
    )]
    pub delegate: AccountInfo<'info>,
    
    /// 系统程序，用于账户管理
    pub system_program: Program<'info, System>,
    
    /// 时钟账户，用于时间戳记录
    pub clock: Sysvar<'info, Clock>,
}

/// 加密货币授权指令实现
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文，自动校验账户权限与生命周期
/// - `params`: 授权参数，包含权限类型、期限等
/// - `exec_params`: 可选算法执行参数，支持算法热插拔
/// - `strategy_params`: 可选策略参数，支持多策略扩展
/// 
/// ## 返回值
/// - `anchor_lang::Result<()>`: Anchor标准返回类型，表示指令执行成功或失败
/// 
/// ## 安全性
/// - Anchor自动校验账户权限、生命周期、PDA
/// - 业务层校验授权参数合法性
/// - 权限验证机制
/// - 完整的事件记录和审计追踪
pub fn authorize_crypto(
    ctx: Context<AuthorizeCrypto>,
    params: AuthorizeParams,
    exec_params: Option<ExecutionParams>,
    strategy_params: Option<StrategyParams>
) -> anchor_lang::Result<()> {
    // === 1. 参数校验 ===
    // 校验授权参数合法性
    validate_authorize_params(&params)?;
    
    // 检查是否自我授权
    require!(
        ctx.accounts.delegate.key() != ctx.accounts.authority.key(),
        crate::errors::SecurityError::SelfAuthorization
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
    
    // 记录授权前的状态
    let authorized_before = ctx.accounts.crypto_asset.authorized_delegates.len();
    
    // 执行授权操作
    let authorize_result = crypto_service.authorize(
        &mut ctx.accounts.crypto_asset,
        &ctx.accounts.delegate.key(),
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
    // 发出授权事件，记录操作详情
    emit!(AssetAuthorized {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        delegate: ctx.accounts.delegate.key(),
        permission_type: params.permission_type.clone(),
        permission_level: params.permission_level,
        authorized_before,
        authorized_after: ctx.accounts.crypto_asset.authorized_delegates.len(),
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: exec_params.map(|p| p.into()),
        strategy_params: strategy_params.map(|s| s.into()),
    });
    
    // === 7. 日志记录 ===
    msg!("Crypto asset authorized successfully: delegate={}, permission_type={}, permission_level={}, authority={}", 
         ctx.accounts.delegate.key(), params.permission_type, params.permission_level, ctx.accounts.authority.key());
    
    Ok(())
}

/// 批量授权加密货币指令
/// 
/// 支持一次性授权多个账户，提高操作效率。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `authorize_orders`: 授权订单集合
/// - `exec_params`: 可选算法执行参数
/// - `strategy_params`: 可选策略参数
pub fn batch_authorize_crypto(
    ctx: Context<AuthorizeCrypto>,
    authorize_orders: Vec<AuthorizeOrder>,
    exec_params: Option<ExecutionParams>,
    strategy_params: Option<StrategyParams>
) -> anchor_lang::Result<()> {
    // === 1. 批量参数校验 ===
    require!(!authorize_orders.is_empty(), crate::errors::AssetError::EmptyBatch);
    require!(authorize_orders.len() <= 20, crate::errors::AssetError::BatchTooLarge);
    
    // 校验每个授权订单
    for order in &authorize_orders {
        validate_authorize_params(&order.params)?;
    }
    
    // === 2. 批量执行 ===
    let crypto_service = CryptoService::new();
    
    // 执行批量授权
    let results = crypto_service.batch_authorize(
        &mut ctx.accounts.crypto_asset,
        authorize_orders
    )?;
    
    // === 3. 算法和策略执行 ===
    if let Some(params) = exec_params {
        crypto_service.execute_algorithm(&mut ctx.accounts.crypto_asset, params)?;
    }
    
    if let Some(strategy) = strategy_params {
        crypto_service.execute_strategy(&mut ctx.accounts.crypto_asset, strategy)?;
    }
    
    // === 4. 事件记录 ===
    let total_authorized = results.len() as u64;
    emit!(AssetAuthorized {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        delegate: ctx.accounts.delegate.key(),
        permission_type: "batch_authorize".to_string(),
        permission_level: 0,
        authorized_before: 0,
        authorized_after: ctx.accounts.crypto_asset.authorized_delegates.len(),
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: exec_params.map(|p| p.into()),
        strategy_params: strategy_params.map(|s| s.into()),
    });
    
    msg!("Batch crypto assets authorized successfully: total_authorized={}, batch_size={}", 
         total_authorized, results.len());
    
    Ok(())
}

/// 角色授权加密货币指令
/// 
/// 基于角色执行授权操作，支持角色权限管理。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `params`: 授权参数
/// - `role_params`: 角色参数
pub fn role_authorize_crypto(
    ctx: Context<AuthorizeCrypto>,
    params: AuthorizeParams,
    role_params: RoleAuthorization
) -> anchor_lang::Result<()> {
    // === 1. 参数校验 ===
    validate_authorize_params(&params)?;
    require!(
        ctx.accounts.delegate.key() != ctx.accounts.authority.key(),
        crate::errors::SecurityError::SelfAuthorization
    );
    
    // === 2. 角色检查 ===
    let crypto_service = CryptoService::new();
    
    // 检查角色权限
    let can_assign_role = crypto_service.check_role_permission(
        &ctx.accounts.crypto_asset,
        &role_params
    )?;
    
    require!(can_assign_role, crate::errors::SecurityError::InsufficientRolePermission);
    
    // === 3. 角色授权执行 ===
    let authorize_result = crypto_service.role_authorize(
        &mut ctx.accounts.crypto_asset,
        &ctx.accounts.delegate.key(),
        &params,
        &role_params
    )?;
    
    // === 4. 事件记录 ===
    emit!(AssetAuthorized {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        delegate: ctx.accounts.delegate.key(),
        permission_type: format!("ROLE: {}", role_params.role_name),
        permission_level: params.permission_level,
        authorized_before: 0,
        authorized_after: ctx.accounts.crypto_asset.authorized_delegates.len(),
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: None,
        strategy_params: None,
    });
    
    msg!("Role crypto asset authorization executed: role={}, delegate={}, authority={}", 
         role_params.role_name, ctx.accounts.delegate.key(), ctx.accounts.authority.key());
    
    Ok(())
}

/// 临时授权加密货币指令
/// 
/// 设置临时授权，支持时间限制的权限管理。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `params`: 授权参数
/// - `temporary_params`: 临时授权参数
pub fn temporary_authorize_crypto(
    ctx: Context<AuthorizeCrypto>,
    params: AuthorizeParams,
    temporary_params: TemporaryAuthorization
) -> anchor_lang::Result<()> {
    // === 1. 参数校验 ===
    validate_authorize_params(&params)?;
    require!(
        ctx.accounts.delegate.key() != ctx.accounts.authority.key(),
        crate::errors::SecurityError::SelfAuthorization
    );
    
    // 检查临时授权时间
    require!(
        temporary_params.expiry_time > Clock::get()?.unix_timestamp,
        crate::errors::SecurityError::InvalidExpiryTime
    );
    
    // === 2. 临时授权执行 ===
    let crypto_service = CryptoService::new();
    
    // 执行临时授权
    let authorize_result = crypto_service.temporary_authorize(
        &mut ctx.accounts.crypto_asset,
        &ctx.accounts.delegate.key(),
        &params,
        &temporary_params
    )?;
    
    // === 3. 事件记录 ===
    emit!(AssetAuthorized {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        delegate: ctx.accounts.delegate.key(),
        permission_type: format!("TEMPORARY: {}", params.permission_type),
        permission_level: params.permission_level,
        authorized_before: 0,
        authorized_after: ctx.accounts.crypto_asset.authorized_delegates.len(),
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: None,
        strategy_params: None,
    });
    
    msg!("Temporary crypto asset authorization executed: delegate={}, expiry_time={}, authority={}", 
         ctx.accounts.delegate.key(), temporary_params.expiry_time, ctx.accounts.authority.key());
    
    Ok(())
}

/// 授权订单结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct AuthorizeOrder {
    /// 授权参数
    pub params: AuthorizeParams,
    /// 订单优先级
    pub priority: u8,
    /// 授权类型
    pub authorize_type: AuthorizeType,
}

/// 角色授权结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct RoleAuthorization {
    /// 角色名称
    pub role_name: String,
    /// 角色权限
    pub role_permissions: Vec<String>,
    /// 角色级别
    pub role_level: u8,
}

/// 临时授权结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct TemporaryAuthorization {
    /// 过期时间
    pub expiry_time: i64,
    /// 最大使用次数
    pub max_usage: u32,
    /// 当前使用次数
    pub current_usage: u32,
}

/// 授权类型枚举
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub enum AuthorizeType {
    /// 正常授权
    Normal,
    /// 角色授权
    Role,
    /// 临时授权
    Temporary,
    /// 紧急授权
    Emergency,
}

/// 授权结果结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct AuthorizeResult {
    /// 授权是否成功
    pub success: bool,
    /// 授权原因
    pub reason: String,
    /// 授权时间
    pub authorize_time: i64,
    /// 授权期限
    pub duration: u64,
} 