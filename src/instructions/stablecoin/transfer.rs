//! Stablecoin资产transfer指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::AssetType;
use crate::services::stablecoin_service::StablecoinService;
use crate::events::asset_event::AssetTransferred;

/// Stablecoin资产transfer指令账户上下文
#[derive(Accounts)]
pub struct TransferStablecoin<'info> {
    #[account(mut)]
    pub from_stablecoin: Account<'info, BasketIndexState>, // 转出Stablecoin资产账户，需可变
    #[account(mut)]
    pub to_stablecoin: Account<'info, BasketIndexState>,   // 转入Stablecoin资产账户，需可变
    pub authority: Signer<'info>,                          // 操作人签名者
}

/// Stablecoin资产transfer指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amount: 转账数量，类型安全
pub fn transfer_stablecoin(ctx: Context<TransferStablecoin>, amount: u64) -> Result<()> {
    let from = &mut ctx.accounts.from_stablecoin;
    let to = &mut ctx.accounts.to_stablecoin;
    require!(from.asset_type == AssetType::Stablecoin && to.asset_type == AssetType::Stablecoin, crate::error::ProgramError::InvalidAssetType);
    let service = StablecoinService::new();
    service.transfer(from, to, amount)?;
    emit!(AssetTransferred {
        from_asset_id: from.id,
        to_asset_id: to.id,
        amount,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 