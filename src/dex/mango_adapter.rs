use anchor_lang::prelude::*; // Anchor预导入，包含Result、Context等
use super::traits::{DexAdapter, SwapParams, SwapResult, AddLiquidityParams, RemoveLiquidityParams, QuoteParams, QuoteResult}; // DEX适配器trait及相关类型
use crate::core::adapter::AdapterTrait; // 适配器元信息trait，统一接口

/// Mango DEX适配器结构体
/// - 用于对接Solana链上的Mango DEX，实现统一的DEX适配接口
/// - 设计为无状态结构体，便于多实例、线程安全
pub struct MangoAdapter;

/// 实现AdapterTrait，提供适配器元信息
impl AdapterTrait for MangoAdapter {
    fn name(&self) -> &str { "mango" }
    fn version(&self) -> &str { "1.0.0" }
    fn is_available(&self) -> bool { true }
    fn initialize(&mut self) -> anchor_lang::Result<()> { Ok(()) }
    fn cleanup(&mut self) -> anchor_lang::Result<()> { Ok(()) }
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
    fn swap(&self, params: SwapParams) -> anchor_lang::Result<SwapResult> {
        // TODO: 实现 Mango DEX 交换逻辑
        // 由于 mango 依赖不可用，暂时返回错误
        Err(anchor_lang::error::Error::from(anchor_lang::error::ErrorCode::AccountNotInitialized))
        
        // let cpi_accounts = mango::cpi::accounts::Swap {
        //     // 账户配置
        // };
        // let cpi_ctx = CpiContext::new(self.program_id, cpi_accounts);
        // let result = mango::cpi::swap(cpi_ctx, params.amount_in, params.min_amount_out)?;
        // Ok(SwapResult {
        //     amount_out: result.amount_out,
        //     fee: result.fee,
        // })
    }
    /// 添加流动性。
    fn add_liquidity(&self, params: AddLiquidityParams) -> anchor_lang::Result<u64> {
        // TODO: 实现 Mango DEX 添加流动性逻辑
        // 由于 mango 依赖不可用，暂时返回错误
        Err(anchor_lang::error::Error::from(anchor_lang::error::ErrorCode::AccountNotInitialized))
        
        // let cpi_accounts = mango::cpi::accounts::AddLiquidity {
        //     // 账户配置
        // };
        // let cpi_ctx = CpiContext::new(self.program_id, cpi_accounts);
        // let result = mango::cpi::add_liquidity(cpi_ctx, params.amount_a, params.amount_b)?;
        // Ok(AddLiquidityResult {
        //     lp_tokens: result.lp_tokens,
        //     fee: result.fee,
        // })
    }
    /// 移除流动性。
    fn remove_liquidity(&self, params: RemoveLiquidityParams) -> anchor_lang::Result<u64> {
        // TODO: 实现 Mango DEX 移除流动性逻辑
        // 由于 mango 依赖不可用，暂时返回错误
        Err(anchor_lang::error::Error::from(anchor_lang::error::ErrorCode::AccountNotInitialized))
        
        // let cpi_accounts = mango::cpi::accounts::RemoveLiquidity {
        //     // 账户配置
        // };
        // let cpi_ctx = CpiContext::new(self.program_id, cpi_accounts);
        // let result = mango::cpi::remove_liquidity(cpi_ctx, params.liquidity)?;
        // Ok(result.amount_out)
    }
    /// 获取报价。
    fn get_quote(&self, params: QuoteParams) -> anchor_lang::Result<QuoteResult> {
        // TODO: 实现 Mango DEX 获取报价逻辑
        // 由于 mango 依赖不可用，暂时返回错误
        Err(anchor_lang::error::Error::from(anchor_lang::error::ErrorCode::AccountNotInitialized))
        
        // let cpi_accounts = mango::cpi::accounts::GetQuote {
        //     // 账户配置
        // };
        // let cpi_ctx = CpiContext::new(self.program_id, cpi_accounts);
        // let quote = mango::cpi::get_quote(cpi_ctx, params.input_mint, params.output_mint, params.amount_in)?;
        // Ok(QuoteResult { amount_out: quote.amount_out, fee: quote.fee })
    }
}

/// Mango适配器错误码（Anchor错误）。
#[error_code]
pub enum ErrorCode {
    #[msg("Invalid amount")] InvalidAmount,      // 输入数量无效（如为0）
    #[msg("Invalid account")] InvalidAccount,    // 账户参数无效（如token_in=token_out）
    #[msg("Operation unsupported")] Unsupported, // 操作不支持
} 