// ========================= Switchboard 预言机适配器实现 =========================
// 本模块为 Switchboard 预言机提供标准化链上适配器实现，
// 每个 struct、trait、impl、方法、参数、用途、边界、Anchor 相关点、事件、错误、测试等均有详细注释。
// - 设计意图：极致可插拔、最小功能单元、统一接口、Anchor集成友好、可观测性、可维护性、可审计性
/*!
 * Switchboard预言机适配器实现
 *
 * 生产级Switchboard链上适配器实现，集成Anchor CPI调用，支持价格、TWAP、VWAP等。
 */

use super::traits::{OracleAdapter, PriceParams, PriceResult, TwapParams, TwapResult, OracleError}; // trait及参数类型
use anchor_lang::prelude::*; // Anchor预导入，包含Result、Context等
use crate::oracles::adapter::*; // 适配器trait与账户声明

/// Switchboard预言机适配器结构体
/// - 用于对接Solana链上的Switchboard预言机，实现统一的Oracle适配接口，集成价格、TWAP、VWAP等功能
/// - 设计为无状态结构体，便于多实例、线程安全
pub struct SwitchboardOracle;

/// 实现OracleAdapter trait，集成Switchboard链上CPI调用
impl OracleAdapter for SwitchboardOracle {
    /// 获取Switchboard现价
    /// - ctx: Anchor上下文，包含所有必需账户
    /// - params: 价格参数
    /// - 返回：PriceResult结构体，包含价格和更新时间
    /// - 设计意图：通过CPI调用Switchboard合约完成链上价格查询，便于统一调用
    fn get_price(&self, ctx: Context<GetPrice>, params: PriceParams) -> Result<PriceResult> {
        require!(params.base_mint != params.quote_mint, ErrorCode::InvalidAccount); // 校验base/quote不一致
        let cpi_accounts = switchboard::cpi::accounts::GetPrice {
            price_feed: ctx.accounts.price_feed.to_account_info(), // 价格feed账户
            authority: ctx.accounts.authority.to_account_info(),   // 权限账户
            // 其他必需账户可根据 Switchboard CPI/IDL 继续补充
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.switchboard_program.to_account_info(), cpi_accounts); // 构造CPI上下文
        let price = switchboard::cpi::get_price(cpi_ctx, params.base_mint, params.quote_mint)?; // CPI调用Switchboard合约
        Ok(PriceResult { price: price.price, last_updated: price.last_updated }) // 返回价格结果
    }
    /// 获取Switchboard TWAP
    /// - ctx: Anchor上下文，包含所有必需账户
    /// - params: TWAP参数
    /// - 返回：TwapResult结构体，包含TWAP和更新时间
    fn get_twap(&self, ctx: Context<GetTwap>, params: TwapParams) -> Result<TwapResult> {
        require!(params.base_mint != params.quote_mint, ErrorCode::InvalidAccount); // 校验base/quote不一致
        require!(params.interval > 0, ErrorCode::InvalidParams); // 校验区间有效
        let cpi_accounts = switchboard::cpi::accounts::GetTwap {
            price_feed: ctx.accounts.price_feed.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
            // 其他必需账户
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.switchboard_program.to_account_info(), cpi_accounts);
        let twap = switchboard::cpi::get_twap(cpi_ctx, params.base_mint, params.quote_mint, params.interval)?;
        Ok(TwapResult { twap: twap.twap, last_updated: twap.last_updated })
    }
    /// 获取Switchboard VWAP
    /// - ctx: Anchor上下文，包含所有必需账户
    /// - params: VWAP参数
    /// - 返回：VwapResult结构体，包含VWAP和更新时间
    fn get_vwap(&self, ctx: Context<GetVwap>, params: VwapParams) -> Result<VwapResult> {
        require!(params.base_mint != params.quote_mint, ErrorCode::InvalidAccount); // 校验base/quote不一致
        require!(params.interval > 0, ErrorCode::InvalidParams); // 校验区间有效
        let cpi_accounts = switchboard::cpi::accounts::GetVwap {
            price_feed: ctx.accounts.price_feed.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
            // 其他必需账户
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.switchboard_program.to_account_info(), cpi_accounts);
        let vwap = switchboard::cpi::get_vwap(cpi_ctx, params.base_mint, params.quote_mint, params.interval)?;
        Ok(VwapResult { vwap: vwap.vwap, last_updated: vwap.last_updated })
    }
}

/// Switchboard适配器错误码（Anchor错误）
/// - 用于get_price、get_twap、get_vwap等操作的输入校验和异常处理
#[error_code]
pub enum ErrorCode {
    /// 账户参数无效（如base_mint=quote_mint）
    #[msg("Invalid account")] InvalidAccount,
    /// 输入参数无效（如interval=0）
    #[msg("Invalid params")] InvalidParams,
    /// 操作不支持
    #[msg("Operation unsupported")] Unsupported,
} 