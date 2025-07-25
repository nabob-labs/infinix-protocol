//!
//! DEX服务层
//! 业务逻辑实现，供指令入口调用，封装DEX适配器注册、交易、报价、流动性管理、批量操作、权限校验等操作。

use anchor_lang::prelude::*;
use crate::dex::traits::{DexAdapter, SwapParams, SwapResult, AddLiquidityParams, RemoveLiquidityParams, QuoteParams, QuoteResult};
use crate::core::types::{BatchTradeParams};
use crate::errors::basket_error::BasketError;

// === 统一服务门面 ===
/// DEX服务门面，聚合所有操作trait，便于统一调用和扩展
///
/// 设计意图：统一对外暴露所有DEX相关操作，便于维护和扩展。
pub struct DexServiceFacade {
    /// DEX适配器注册服务
    pub register: RegisterDexAdapterService,
    /// 交易服务
    pub swap: SwapDexService,
    /// 报价服务
    pub quote: QuoteDexService,
    /// 添加流动性服务
    pub add_liquidity: AddLiquidityDexService,
    /// 移除流动性服务
    pub remove_liquidity: RemoveLiquidityDexService,
    /// 批量交易服务
    pub batch_swap: BatchSwapDexService,
    /// 权限校验服务
    pub authorize: AuthorizeDexService,
}

impl DexServiceFacade {
    /// 构造函数，初始化所有服务实现
    ///
    /// # 返回值
    /// - 返回 DexServiceFacade 实例。
    pub fn new() -> Self {
        Self {
            register: RegisterDexAdapterService,
            swap: SwapDexService,
            quote: QuoteDexService,
            add_liquidity: AddLiquidityDexService,
            remove_liquidity: RemoveLiquidityDexService,
            batch_swap: BatchSwapDexService,
            authorize: AuthorizeDexService,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::prelude::Pubkey;

    #[test]
    fn test_register_adapter() {
        let service = DexServiceFacade::new();
        let adapter: Box<dyn DexAdapter> = Box::new(MockDexAdapter {});
        let result = service.register.register_adapter(adapter);
        assert!(result.is_ok());
    }

    struct MockDexAdapter;
    impl DexAdapter for MockDexAdapter {
        fn swap(&self, _params: &SwapParams) -> Result<SwapResult> { Ok(SwapResult { amount_out: 1, fee: 0 }) }
        fn add_liquidity(&self, _params: &AddLiquidityParams) -> Result<u64> { Ok(1) }
        fn remove_liquidity(&self, _params: &RemoveLiquidityParams) -> Result<u64> { Ok(1) }
        fn quote(&self, _params: &QuoteParams) -> Result<QuoteResult> { Ok(QuoteResult { price: 1 }) }
    }

    #[test]
    fn test_swap() {
        let service = DexServiceFacade::new();
        let params = SwapParams { amount_in: 100, ..Default::default() };
        let result = service.swap.swap(&params);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().amount_out, 99);
    }

    #[test]
    fn test_quote() {
        let service = DexServiceFacade::new();
        let params = QuoteParams { amount_in: 100, ..Default::default() };
        let result = service.quote.quote(&params);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().price, 100000);
    }

    #[test]
    fn test_add_liquidity() {
        let service = DexServiceFacade::new();
        let params = AddLiquidityParams { amount_a: 50, amount_b: 50, ..Default::default() };
        let result = service.add_liquidity.add_liquidity(&params);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 100);
    }

    #[test]
    fn test_remove_liquidity() {
        let service = DexServiceFacade::new();
        let params = RemoveLiquidityParams { liquidity: 100, ..Default::default() };
        let result = service.remove_liquidity.remove_liquidity(&params);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 50);
    }

    #[test]
    fn test_batch_swap() {
        let service = DexServiceFacade::new();
        let params = BatchTradeParams { amounts: vec![100, 200, 300] };
        let result = service.batch_swap.batch_swap(&params);
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].amount_out, 99);
        assert_eq!(results[1].amount_out, 198);
        assert_eq!(results[2].amount_out, 297);
    }

    #[test]
    fn test_authorize() {
        let service = DexServiceFacade::new();
        let authority = Pubkey::default();
        let result = service.authorize.authorize(authority);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
} 