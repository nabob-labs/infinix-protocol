//!
//! Asset Authorize Instruction
//! 资产授权指令实现，所有业务逻辑下沉到 service 层，指令层只做参数校验、账户校验、事件触发。

use crate::accounts::BasketIndexStateAccount; // 账户状态结构体定义
use crate::events::asset_event::*; // 资产相关事件定义（Anchor事件）
use crate::services::asset_service::AssetService; // 资产业务逻辑服务层
use crate::state::baskets::BasketIndexState; // 资产篮子状态
use crate::validation::asset_validation::AssetValidatable; // 资产校验trait
use anchor_lang::prelude::*; // Anchor预导入，提供Solana合约开发的基础类型和宏

/// 资产授权指令账户上下文
/// - asset: 目标资产篮子账户，需可变
/// - authority: 当前操作人签名者
/// - new_authority: 新授权公钥
#[derive(Accounts)] // Anchor宏，自动生成账户校验与生命周期管理代码
pub struct AuthorizeAsset<'info> {
    /// 目标资产篮子账户，需可变，Anchor自动校验PDA
    #[account(mut)]
    pub asset: Account<'info, BasketIndexState>,
    /// 当前操作人签名者，需可变
    #[account(mut)]
    pub authority: Signer<'info>,
    /// 新授权公钥
    pub new_authority: UncheckedAccount<'info>,
}

/// 资产授权指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - 返回: Anchor规范Result
pub fn authorize_asset(ctx: Context<AuthorizeAsset>) -> Result<()> {
    let asset = &mut ctx.accounts.asset; // 获取可变资产篮子账户
    asset.validate()?; // 校验资产篮子状态
    // 权限校验：必须是当前authority
    require_keys_eq!(asset.authority, ctx.accounts.authority.key(), crate::errors::asset_error::AssetError::AuthorizationFailed);
    // 业务逻辑：更新authority
    AssetService::authorize(asset, ctx.accounts.new_authority.key(), ctx.accounts.authority.key())?;
    emit!(AssetAuthorized {
        asset_id: asset.id, // 事件：资产ID
        old_authority: ctx.accounts.authority.key(), // 事件：原授权
        new_authority: ctx.accounts.new_authority.key(), // 事件：新授权
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，表示指令成功
} 