//! Stablecoin资产query指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::AssetType;
use crate::services::stablecoin_service::StablecoinService;
use crate::events::asset_event::AssetQueried;

/// Stablecoin资产query指令账户上下文
#[derive(Accounts)]
pub struct QueryStablecoin<'info> {
    pub stablecoin: Account<'info, BasketIndexState>, // Stablecoin资产账户
}

/// Stablecoin资产query指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
pub fn query_stablecoin(ctx: Context<QueryStablecoin>) -> Result<()> {
    let stablecoin = &ctx.accounts.stablecoin;
    require!(stablecoin.asset_type == AssetType::Stablecoin, crate::error::ProgramError::InvalidAssetType);
    let service = StablecoinService::new();
    let info = service.query(stablecoin)?;
    emit!(AssetQueried {
        asset_id: stablecoin.id,
        total_value: info.total_value,
        is_active: info.is_active,
        authority: stablecoin.authority,
        asset_type: stablecoin.asset_type as u8,
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 