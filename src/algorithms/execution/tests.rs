//! 执行算法相关单元测试模块
//! 覆盖 TWAP、VWAP、Genetic、ML 等算法的基础行为和边界条件。

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithms::traits::{ExecutionInput, OrderInput, MarketDataInput}; // 引入通用输入类型

    /// 测试：TWAP 算法基础行为
    #[test]
    fn test_twap_algorithm_basic() {
        let algo = super::TwapExecutionAlgorithm; // TWAP 算法实例
        let input = ExecutionInput {
            orders: vec![OrderInput { token_in: "A".to_string(), token_out: "B".to_string(), amount: 100 }],
            market_data: MarketDataInput { prices: vec![("A".to_string(), 100)], liquidity: vec![("A".to_string(), 1000)] },
        };
        let result = algo.execute(&input);
        // 目前为 NotSupported，预期返回错误
        assert!(result.is_err());
    }

    /// 测试：VWAP 算法基础行为
    #[test]
    fn test_vwap_algorithm_basic() {
        let algo = super::VwapExecutionAlgorithm; // VWAP 算法实例
        let input = ExecutionInput {
            orders: vec![OrderInput { token_in: "A".to_string(), token_out: "B".to_string(), amount: 100 }],
            market_data: MarketDataInput { prices: vec![("A".to_string(), 100)], liquidity: vec![("A".to_string(), 1000)] },
        };
        let result = algo.execute(&input);
        assert!(result.is_err());
    }

    /// 测试：Genetic 算法基础行为
    #[test]
    fn test_genetic_algorithm_basic() {
        let algo = super::GeneticExecutionAlgorithm; // Genetic 算法实例
        let input = ExecutionInput {
            orders: vec![OrderInput { token_in: "A".to_string(), token_out: "B".to_string(), amount: 100 }],
            market_data: MarketDataInput { prices: vec![("A".to_string(), 100)], liquidity: vec![("A".to_string(), 1000)] },
        };
        let result = algo.execute(&input);
        assert!(result.is_err());
    }

    /// 测试：ML 算法基础行为
    #[test]
    fn test_ml_algorithm_basic() {
        let algo = super::MlExecutionAlgorithm; // ML 算法实例
        let input = ExecutionInput {
            orders: vec![OrderInput { token_in: "A".to_string(), token_out: "B".to_string(), amount: 100 }],
            market_data: MarketDataInput { prices: vec![("A".to_string(), 100)], liquidity: vec![("A".to_string(), 1000)] },
        };
        let result = algo.execute(&input);
        assert!(result.is_err());
    }
} 