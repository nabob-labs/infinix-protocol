//!
//! Asset Freeze Instruction
//! 资产冻结指令实现，所有业务逻辑下沉到 service 层，指令层只做参数校验、账户校验、事件触发。

use crate::accounts::BasketIndexStateAccount; // 账户状态结构体定义
use crate::events::asset_event::*; // 资产相关事件定义（Anchor事件）
use crate::services::asset_service::AssetService; // 资产业务逻辑服务层
use crate::state::baskets::BasketIndexState; // 资产篮子状态
use crate::validation::asset_validation::AssetValidatable; // 资产校验trait
use anchor_lang::prelude::*; // Anchor预导入，提供Solana合约开发的基础类型和宏

/// 资产冻结指令账户上下文
/// - asset: 目标资产篮子账户，需可变
/// - authority: 操作人签名者
#[derive(Accounts)] // Anchor宏，自动生成账户校验与生命周期管理代码
pub struct FreezeAsset<'info> {
    /// 目标资产篮子账户，需可变，Anchor自动校验PDA
    #[account(mut)]
    pub asset: Account<'info, BasketIndexState>,
    /// 操作人签名者，需可变
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// 资产冻结指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - 返回: Anchor规范Result
pub fn freeze_asset(ctx: Context<FreezeAsset>) -> Result<()> {
    let asset = &mut ctx.accounts.asset; // 获取可变资产篮子账户
    asset.validate()?; // 校验资产篮子状态
    // 权限校验：必须是当前authority
    require_keys_eq!(asset.authority, ctx.accounts.authority.key(), crate::errors::asset_error::AssetError::AuthorizationFailed);
    // 业务逻辑：冻结资产
    AssetService::freeze(asset, ctx.accounts.authority.key())?;
    emit!(AssetFrozen {
        asset_id: asset.id, // 事件：资产ID
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，表示指令成功
} 