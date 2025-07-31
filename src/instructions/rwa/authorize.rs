//! RWA资产authorize指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
// use crate::core::types::AssetType; // 暂时注释掉
// use crate::services::rwa_service::RwaService; // 暂时注释掉
// use crate::events::asset_event::AssetAuthorized; // 暂时注释掉

// RWA资产authorize指令账户上下文已被注释掉，因为该指令未在 lib.rs 中注册
// #[derive(Accounts)]
// pub struct AuthorizeRwa<'info> {
//     #[account(mut)]
//     pub rwa: Account<'info, BasketIndexState>, // RWA资产账户，需可变
//     pub authority: Signer<'info>,             // 当前授权人
//     pub new_authority: Pubkey,                // 新授权人公钥
// }

// RWA资产authorize指令实现已被注释掉，因为该指令未在 lib.rs 中注册 