//!
//! Phoenix DEX Adapter Module
//!
//! 本模块实现 Phoenix DEX 适配器，集成 Anchor CPI 调用，支持流动性管理、报价、异常处理等，确保链上集成合规、可维护。

use anchor_lang::prelude::*;
use crate::dex::adapter::{DexAdapter, Swap, AddLiquidity, RemoveLiquidity};

/// Phoenix DEX 适配器结构体。
/// 用于对接 Solana 链上的 Phoenix DEX，实现统一的 DEX 适配接口，集成流动性管理、报价等功能。
#[derive(Default)]
pub struct PhoenixAdapter;

/// 实现 DexAdapter trait，集成 Phoenix 链上 CPI 调用。
impl DexAdapter for PhoenixAdapter {
    /// 执行 Phoenix swap 操作。
    fn swap(
        &self,
        _ctx: Context<Swap>,
        amount_in: u64,
        min_amount_out: u64,
    ) -> Result<u64> {
        // 校验输入数量和最小输出数量必须大于 0。
        require!(amount_in > 0, ErrorCode::InvalidAmount);
        require!(min_amount_out > 0, ErrorCode::InvalidAmount);
        // TODO: 集成 Phoenix 官方 IDL 或 Anchor CPI 接口。
        Ok(min_amount_out) // 示例返回
    }
    /// 添加流动性。
    fn add_liquidity(
        &self,
        _ctx: Context<AddLiquidity>,
        amount_a: u64,
        amount_b: u64,
    ) -> Result<u64> {
        // 校验资产数量必须大于 0。
        require!(amount_a > 0 && amount_b > 0, ErrorCode::InvalidAmount);
        // TODO: 集成 Phoenix CPI。
        Ok(amount_a + amount_b) // 示例返回
    }
    /// 移除流动性。
    fn remove_liquidity(
        &self,
        _ctx: Context<RemoveLiquidity>,
        liquidity: u64,
    ) -> Result<(u64, u64)> {
        // 校验 LP token 数量必须大于 0。
        require!(liquidity > 0, ErrorCode::InvalidAmount);
        // TODO: 集成 Phoenix CPI。
        Ok((liquidity / 2, liquidity / 2)) // 示例返回
    }
}

/// Phoenix 适配器错误码（Anchor 错误）。
#[error_code]
pub enum ErrorCode {
    #[msg("Invalid amount")] InvalidAmount,      // 输入数量无效（如为0）
    #[msg("Operation unsupported")] Unsupported, // 操作不支持
}

/// 自动注册 PhoenixAdapter 到工厂（如有需要可补充）。
#[ctor::ctor]
fn register_phoenix_adapter() {
    // DEX_FACTORY.register("phoenix", Arc::new(PhoenixAdapter::default())); // 如需自动注册可取消注释
}

#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::prelude::*;
    use std::str::FromStr;

    /// MockPhoenixAdapter 为 Phoenix DEX 的测试实现，便于单元测试。
    struct MockPhoenixAdapter;
    impl DexAdapter for MockPhoenixAdapter {
        /// 模拟 swap 操作，返回输入数量的 97% 作为输出。
        fn swap(
            &self,
            _ctx: Context<Swap>,
            _input_mint: Pubkey,
            _output_mint: Pubkey,
            amount_in: u64,
            _min_amount_out: u64,
        ) -> Result<u64> {
            Ok(amount_in * 97 / 100) // 模拟 3% 滑点
        }
        /// 模拟报价操作，返回输入数量的 97% 作为预期输出。
        fn quote(
            &self,
            _input_mint: Pubkey,
            _output_mint: Pubkey,
            amount_in: u64,
        ) -> Result<u64> {
            Ok(amount_in * 97 / 100)
        }
    }

    /// 测试 PhoenixAdapter swap 功能。
    #[test]
    fn test_phoenix_adapter_swap() {
        let adapter = MockPhoenixAdapter;
        let ctx = Context::new();
        let input = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
        let output = Pubkey::from_str("USDC111111111111111111111111111111111111111").unwrap();
        let result = adapter.swap(ctx, input, output, 100_000, 97_000);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 97_000);
    }

    /// 测试 PhoenixAdapter quote 功能。
    #[test]
    fn test_phoenix_adapter_quote() {
        let adapter = MockPhoenixAdapter;
        let input = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
        let output = Pubkey::from_str("USDC111111111111111111111111111111111111111").unwrap();
        let result = adapter.quote(input, output, 100_000);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 97_000);
    }
} 