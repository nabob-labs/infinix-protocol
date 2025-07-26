use anchor_lang::prelude::*; // Anchor预导入，包含Result、Context等
use super::traits::{DexAdapter, SwapParams, SwapResult, AddLiquidityParams, RemoveLiquidityParams, QuoteParams, QuoteResult}; // DEX适配器trait及相关类型
use crate::core::adapter::AdapterTrait; // 适配器元信息trait，统一接口

/// Mango DEX适配器结构体
/// - 用于对接Solana链上的Mango DEX，实现统一的DEX适配接口
/// - 设计为无状态结构体，便于多实例、线程安全
pub struct MangoAdapter;

/// 实现AdapterTrait，提供适配器元信息
impl AdapterTrait for MangoAdapter {
    /// 返回适配器名称（唯一标识）
    fn name(&self) -> &'static str { "mango" }
    /// 返回适配器版本号
    fn version(&self) -> &'static str { "1.0.0" }
    /// 返回支持的资产列表（如SOL、USDC等）
    fn supported_assets(&self) -> Vec<String> { vec!["SOL".to_string(), "USDC".to_string()] }
    /// 返回适配器当前状态（如active、paused等）
    fn status(&self) -> Option<String> { Some("active".to_string()) }
}

/// 自动注册MangoAdapter到全局工厂
/// - 使用ctor宏在程序启动时自动注册，便于插件式扩展
/// - 设计意图：极简插件式扩展，保证所有DEX适配器可热插拔
// #[ctor::ctor]
fn auto_register_mango_adapter() {
    let adapter = MangoAdapter; // 实例化适配器
    let mut factory = crate::core::registry::ADAPTER_FACTORY.lock().unwrap(); // 获取全局工厂锁
    factory.register(adapter); // 注册到工厂，便于统一管理
}

// === Mango CPI账户结构声明 ===
#[derive(Accounts)]
pub struct Swap<'info> {
    /// 用户主账户
    #[account(mut)]
    pub user: Signer<'info>,
    /// Mango池账户
    #[account(mut)]
    pub pool: AccountInfo<'info>,
    /// 输入token的vault
    #[account(mut)]
    pub input_vault: AccountInfo<'info>,
    /// 输出token的vault
    #[account(mut)]
    pub output_vault: AccountInfo<'info>,
    /// Mango池权限账户
    pub authority: AccountInfo<'info>,
    /// SPL Token程序
    pub token_program: AccountInfo<'info>,
    /// Mango主程序
    pub mango_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(mut)] pub user: Signer<'info>,
    #[account(mut)] pub pool: AccountInfo<'info>,
    #[account(mut)] pub vault_a: AccountInfo<'info>,
    #[account(mut)] pub vault_b: AccountInfo<'info>,
    pub authority: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub mango_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    #[account(mut)] pub user: Signer<'info>,
    #[account(mut)] pub pool: AccountInfo<'info>,
    #[account(mut)] pub lp_vault: AccountInfo<'info>,
    pub authority: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub mango_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct GetQuote<'info> {
    pub pool: AccountInfo<'info>,
    pub input_vault: AccountInfo<'info>,
    pub output_vault: AccountInfo<'info>,
    pub mango_program: AccountInfo<'info>,
}

/// 实现 DexAdapter trait，集成 Mango 链上 CPI 调用。
impl DexAdapter for MangoAdapter {
    /// 执行 Mango swap 操作。
    fn swap(&self, ctx: Context<Swap>, params: SwapParams) -> Result<SwapResult> {
        // 校验输入数量必须大于0。
        require!(params.amount_in > 0, ErrorCode::InvalidAmount);
        // 校验输入输出token不可相同。
        require!(params.token_in != params.token_out, ErrorCode::InvalidAccount);
        // 构造CPI账户。
        let cpi_accounts = mango::cpi::accounts::Swap {
            user: ctx.accounts.user.to_account_info(),
            pool: ctx.accounts.pool.to_account_info(),
            input_vault: ctx.accounts.input_vault.to_account_info(),
            output_vault: ctx.accounts.output_vault.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        };
        // 构造CPI上下文。
        let cpi_ctx = CpiContext::new(ctx.accounts.mango_program.to_account_info(), cpi_accounts);
        // 调用Mango CPI swap。
        let result = mango::cpi::swap(cpi_ctx, params.amount_in, params.min_amount_out)?;
        // 返回swap结果。
        Ok(SwapResult { amount_out: result.amount_out, fee: result.fee })
    }
    /// 添加流动性。
    fn add_liquidity(&self, ctx: Context<AddLiquidity>, params: AddLiquidityParams) -> Result<u64> {
        let cpi_accounts = mango::cpi::accounts::AddLiquidity {
            user: ctx.accounts.user.to_account_info(),
            pool: ctx.accounts.pool.to_account_info(),
            vault_a: ctx.accounts.vault_a.to_account_info(),
            vault_b: ctx.accounts.vault_b.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.mango_program.to_account_info(), cpi_accounts);
        let result = mango::cpi::add_liquidity(cpi_ctx, params.amount_a, params.amount_b)?;
        Ok(result.liquidity)
    }
    /// 移除流动性。
    fn remove_liquidity(&self, ctx: Context<RemoveLiquidity>, params: RemoveLiquidityParams) -> Result<u64> {
        let cpi_accounts = mango::cpi::accounts::RemoveLiquidity {
            user: ctx.accounts.user.to_account_info(),
            pool: ctx.accounts.pool.to_account_info(),
            lp_vault: ctx.accounts.lp_vault.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.mango_program.to_account_info(), cpi_accounts);
        let result = mango::cpi::remove_liquidity(cpi_ctx, params.liquidity)?;
        Ok(result.amount_out)
    }
    /// 获取报价。
    fn get_quote(&self, ctx: Context<GetQuote>, params: QuoteParams) -> Result<QuoteResult> {
        let cpi_accounts = mango::cpi::accounts::GetQuote {
            pool: ctx.accounts.pool.to_account_info(),
            input_vault: ctx.accounts.input_vault.to_account_info(),
            output_vault: ctx.accounts.output_vault.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.mango_program.to_account_info(), cpi_accounts);
        let quote = mango::cpi::get_quote(cpi_ctx, params.input_mint, params.output_mint, params.amount_in)?;
        Ok(QuoteResult { amount_out: quote.amount_out, fee: quote.fee })
    }
}

/// Mango适配器错误码（Anchor错误）。
#[error_code]
pub enum ErrorCode {
    #[msg("Invalid amount")] InvalidAmount,      // 输入数量无效（如为0）
    #[msg("Invalid account")] InvalidAccount,    // 账户参数无效（如token_in=token_out）
    #[msg("Operation unsupported")] Unsupported, // 操作不支持
} 