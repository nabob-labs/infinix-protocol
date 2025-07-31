//!
//! Basket Authorize Instruction
//! 篮子授权指令最小功能单元实现，严格遵循Anchor规范、SOLID原则、分层设计、接口清晰、类型安全、事件追踪、权限校验、生命周期管理、错误处理、逐行注释，生产级代码质量。

use crate::events::basket_event::*; // 引入所有篮子相关事件定义，便于emit!宏调用
use crate::services::basket_service::BasketServiceFacade; // 引入篮子服务层，封装核心业务逻辑
use anchor_lang::prelude::*; // Anchor预导出内容，包含Context、Account、Signer、Result等
use crate::core::types::*; // 引入算法和策略参数类型

// 篮子授权指令账户上下文已被注释掉，因为该指令未在 lib.rs 中注册
// #[derive(Accounts)]
// pub struct AuthorizeBasket<'info> {
//     #[account(mut)]
//     pub basket: Account<'info, BasketIndexState>,
//     #[account(mut)]
//     pub authority: Signer<'info>,
//     pub new_authority: Pubkey,
// }

// 篮子授权指令主函数已被注释掉，因为该指令未在 lib.rs 中注册 