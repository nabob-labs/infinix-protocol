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

    // TwapAlgorithm 单元测试 - 覆盖所有最小功能单元，逐行专业注释

    use super::twap::{TwapAlgorithm, TwapAlgorithmTrait};

    #[test]
    fn test_twap_initialize_and_reset() {
        // 初始化TWAP算法，窗口10秒
        let mut twap = TwapAlgorithm::new(10);
        // 检查初始样本数量为0
        assert_eq!(twap.sample_count(), 0);
        // 添加样本
        twap.update(100, 1);
        twap.update(200, 2);
        // 检查样本数量为2
        assert_eq!(twap.sample_count(), 2);
        // 重置样本
        twap.reset();
        // 检查样本数量为0
        assert_eq!(twap.sample_count(), 0);
    }

    #[test]
    fn test_twap_update_and_window() {
        // 初始化TWAP算法，窗口5秒
        let mut twap = TwapAlgorithm::new(5);
        // 添加多个样本，部分超出窗口
        twap.update(100, 1);
        twap.update(200, 3);
        twap.update(300, 10); // 1,3应被移除
        // 检查只剩下最后一个样本
        assert_eq!(twap.sample_count(), 1);
        // 检查样本内容
        assert_eq!(twap.compute_twap(), 300);
    }

    #[test]
    fn test_twap_compute_weighted_average() {
        // 初始化TWAP算法，窗口100秒
        let mut twap = TwapAlgorithm::new(100);
        // 添加样本 (价格, 时间戳)
        twap.update(100, 1);   // 0-1: 100
        twap.update(200, 11);  // 1-11: 100, 11-21: 200
        twap.update(300, 21);
        // 计算TWAP
        let twap_value = twap.compute_twap();
        // 计算加权平均：(100*10 + 200*10 + 300*当前权重) / 总权重
        // 由于最后一个样本到当前时间的权重依赖于Clock::get()，这里只能保证不为0
        assert!(twap_value > 0);
    }

    // VwapAlgorithm 单元测试 - 覆盖所有最小功能单元，逐行专业注释

    use super::vwap::{VwapAlgorithm, VwapAlgorithmTrait};

    #[test]
    fn test_vwap_initialize_and_reset() {
        // 初始化VWAP算法，窗口10秒
        let mut vwap = VwapAlgorithm::new(10);
        // 检查初始样本数量为0
        assert_eq!(vwap.sample_count(), 0);
        // 添加样本
        vwap.update(100, 10, 1);
        vwap.update(200, 20, 2);
        // 检查样本数量为2
        assert_eq!(vwap.sample_count(), 2);
        // 重置样本
        vwap.reset();
        // 检查样本数量为0
        assert_eq!(vwap.sample_count(), 0);
    }

    #[test]
    fn test_vwap_update_and_window() {
        // 初始化VWAP算法，窗口5秒
        let mut vwap = VwapAlgorithm::new(5);
        // 添加多个样本，部分超出窗口
        vwap.update(100, 10, 1);
        vwap.update(200, 20, 3);
        vwap.update(300, 30, 10); // 1,3应被移除
        // 检查只剩下最后一个样本
        assert_eq!(vwap.sample_count(), 1);
        // 检查样本内容
        assert_eq!(vwap.compute_vwap(), 300);
    }

    #[test]
    fn test_vwap_compute_weighted_average() {
        // 初始化VWAP算法，窗口100秒
        let mut vwap = VwapAlgorithm::new(100);
        // 添加样本 (价格, 成交量, 时间戳)
        vwap.update(100, 10, 1);   // 100*10
        vwap.update(200, 20, 11);  // 200*20
        vwap.update(300, 30, 21);  // 300*30
        // 计算VWAP
        let vwap_value = vwap.compute_vwap();
        // 计算加权平均：(100*10 + 200*20 + 300*30) / (10+20+30) = (1000+4000+9000)/60 = 14000/60 = 233
        assert_eq!(vwap_value, 233);
    }
} 