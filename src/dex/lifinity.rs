//!
//! Lifinity DEX Adapter Module
//!
//! 本模块实现 Lifinity DEX 适配器，集成 Anchor CPI 调用，支持流动性管理、报价、异常处理等，确保链上集成合规、可维护。

use anchor_lang::prelude::*;            // Anchor 预导入，包含 Result、Context、CpiContext 等
use crate::dex::adapter::*;             // DexAdapter trait 及相关类型

/// Lifinity DEX 适配器结构体。
/// - 用于对接 Solana 链上的 Lifinity DEX，实现统一的 DEX 适配接口。
/// - 设计为无状态结构体，便于多实例、线程安全。
pub struct LifinityAdapter;

/// 实现 DexAdapter trait，集成 Lifinity 链上 CPI 调用。
impl DexAdapter for LifinityAdapter {
    /// 执行 Lifinity swap 操作。
    fn swap(&self, ctx: Context<Swap>, params: SwapParams) -> Result<SwapResult> {
        // 校验输入数量必须大于 0。
        require!(params.amount_in > 0, ErrorCode::InvalidAmount);
        // 校验输入输出 token 不可相同。
        require!(params.token_in != params.token_out, ErrorCode::InvalidAccount);
        // 构造 CPI 所需账户。
        let cpi_accounts = lifinity::cpi::accounts::Swap {
            user: ctx.accounts.user.to_account_info(),
            pool: ctx.accounts.pool.to_account_info(),
            input_vault: ctx.accounts.input_vault.to_account_info(),
            output_vault: ctx.accounts.output_vault.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        };
        // 构造 CPI 上下文。
        let cpi_ctx = CpiContext::new(ctx.accounts.lifinity_program.to_account_info(), cpi_accounts);
        // 调用 Lifinity CPI swap。
        let result = lifinity::cpi::swap(cpi_ctx, params.amount_in, params.min_amount_out)?;
        // 返回 swap 结果。
        Ok(SwapResult {
            amount_out: result.amount_out,
            fee: result.fee,
        })
    }
    /// 添加流动性。
    fn add_liquidity(&self, ctx: Context<AddLiquidity>, params: AddLiquidityParams) -> Result<u64> {
        let cpi_accounts = lifinity::cpi::accounts::AddLiquidity {
            user: ctx.accounts.user.to_account_info(),
            pool: ctx.accounts.pool.to_account_info(),
            vault_a: ctx.accounts.vault_a.to_account_info(),
            vault_b: ctx.accounts.vault_b.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.lifinity_program.to_account_info(), cpi_accounts);
        let result = lifinity::cpi::add_liquidity(cpi_ctx, params.amount_a, params.amount_b)?;
        Ok(result.liquidity)
    }
    /// 移除流动性。
    fn remove_liquidity(&self, ctx: Context<RemoveLiquidity>, params: RemoveLiquidityParams) -> Result<u64> {
        let cpi_accounts = lifinity::cpi::accounts::RemoveLiquidity {
            user: ctx.accounts.user.to_account_info(),
            pool: ctx.accounts.pool.to_account_info(),
            lp_vault: ctx.accounts.lp_vault.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.lifinity_program.to_account_info(), cpi_accounts);
        let result = lifinity::cpi::remove_liquidity(cpi_ctx, params.liquidity)?;
        Ok(result.amount_out)
    }
    /// 获取报价。
    fn get_quote(&self, ctx: Context<GetQuote>, params: QuoteParams) -> Result<QuoteResult> {
        let cpi_accounts = lifinity::cpi::accounts::GetQuote {
            pool: ctx.accounts.pool.to_account_info(),
            input_vault: ctx.accounts.input_vault.to_account_info(),
            output_vault: ctx.accounts.output_vault.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.lifinity_program.to_account_info(), cpi_accounts);
        let quote = lifinity::cpi::get_quote(cpi_ctx, params.input_mint, params.output_mint, params.amount_in)?;
        Ok(QuoteResult {
            amount_out: quote.amount_out,
            fee: quote.fee,
        })
    }
}

/// Lifinity 适配器错误码（Anchor 错误）。
#[error_code]
pub enum ErrorCode {
    #[msg("Invalid amount")] InvalidAmount,      // 输入数量无效（如为0）
    #[msg("Invalid account")] InvalidAccount,    // 账户参数无效（如token_in=token_out）
    #[msg("Operation unsupported")] Unsupported, // 操作不支持
} 