#![allow(clippy::too_many_arguments)]
#![allow(unexpected_cfgs)]
#![allow(clippy::doc_overindented_list_items)]
use anchor_lang::prelude::*;

use instructions::*;
use utils::*;

pub mod events;
pub mod instructions;
pub mod state;
pub mod utils;

declare_id!("5ZyzXNgfRdCPB1PWCNjc2WrpsQbNDqQvVZ1RvYBBbBTx");

#[program]
pub mod infinix {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
