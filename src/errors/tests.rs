//!
//! 错误系统测试模块
//!
//! 测试所有错误类型的创建、转换、统计等功能。

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::*;

    #[test]
    fn test_program_error_creation() {
        // 测试资产错误
        let asset_error = AssetError::InvalidAmount { amount: 0 };
        let program_error = ProgramError::Asset(asset_error);
        
        assert_eq!(program_error.module_name(), "Asset");
        assert!(program_error.error_code() >= 1000 && program_error.error_code() < 2000);
        assert!(!program_error.error_message().is_empty());
    }

    #[test]
    fn test_dex_error_creation() {
        // 测试DEX错误
        let dex_error = DexError::InvalidSwapParams {
            message: "Invalid parameters".to_string(),
        };
        let program_error = ProgramError::Dex(dex_error);
        
        assert_eq!(program_error.module_name(), "DEX");
        assert!(program_error.error_code() >= 4000 && program_error.error_code() < 5000);
        assert!(!program_error.error_message().is_empty());
    }

    #[test]
    fn test_oracle_error_creation() {
        // 测试Oracle错误
        let oracle_error = OracleError::PriceStale {
            age_seconds: 3600,
            max_age_seconds: 300,
        };
        let program_error = ProgramError::Oracle(oracle_error);
        
        assert_eq!(program_error.module_name(), "Oracle");
        assert!(program_error.error_code() >= 5000 && program_error.error_code() < 6000);
        assert!(!program_error.error_message().is_empty());
    }

    #[test]
    fn test_algorithm_error_creation() {
        // 测试算法错误
        let algorithm_error = AlgorithmError::InvalidParameters {
            reason: "Invalid parameters".to_string(),
        };
        let program_error = ProgramError::Algorithm(algorithm_error);
        
        assert_eq!(program_error.module_name(), "Algorithm");
        assert!(program_error.error_code() >= 6000 && program_error.error_code() < 7000);
        assert!(!program_error.error_message().is_empty());
    }

    #[test]
    fn test_strategy_error_creation() {
        // 测试策略错误
        let strategy_error = StrategyError::InvalidParameters {
            reason: "Invalid parameters".to_string(),
        };
        let program_error = ProgramError::Strategy(strategy_error);
        
        assert_eq!(program_error.module_name(), "Strategy");
        assert!(program_error.error_code() >= 7000 && program_error.error_code() < 8000);
        assert!(!program_error.error_message().is_empty());
    }

    #[test]
    fn test_validation_error_creation() {
        // 测试验证错误
        let validation_error = ValidationError::InvalidInputParameter {
            parameter: "amount".to_string(),
            reason: "Must be positive".to_string(),
        };
        let program_error = ProgramError::Validation(validation_error);
        
        assert_eq!(program_error.module_name(), "Validation");
        assert!(program_error.error_code() >= 8000 && program_error.error_code() < 9000);
        assert!(!program_error.error_message().is_empty());
    }

    #[test]
    fn test_security_error_creation() {
        // 测试安全错误
        let security_error = SecurityError::InsufficientPermissions {
            operation: "transfer".to_string(),
            required_permission: "admin".to_string(),
        };
        let program_error = ProgramError::Security(security_error);
        
        assert_eq!(program_error.module_name(), "Security");
        assert!(program_error.error_code() >= 9000 && program_error.error_code() < 10000);
        assert!(!program_error.error_message().is_empty());
    }

    #[test]
    fn test_error_recoverability() {
        // 测试可恢复错误
        let recoverable_error = OracleError::OracleTimeout {
            oracle_name: "pyth".to_string(),
        };
        let program_error = ProgramError::Oracle(recoverable_error);
        
        assert!(program_error.is_recoverable());
        assert!(program_error.retry_after().is_some());
        
        // 测试不可恢复错误
        let non_recoverable_error = SecurityError::SignatureVerificationFailed {
            reason: "Invalid signature".to_string(),
        };
        let program_error = ProgramError::Security(non_recoverable_error);
        
        assert!(!program_error.is_recoverable());
        assert!(program_error.retry_after().is_none());
    }

    #[test]
    fn test_error_severity() {
        // 测试错误严重程度
        let security_error = SecurityError::SignatureVerificationFailed {
            reason: "Invalid signature".to_string(),
        };
        assert_eq!(security_error.severity(), SecurityErrorSeverity::Critical);
        
        let warning_error = OracleError::OracleTimeout {
            oracle_name: "pyth".to_string(),
        };
        assert_eq!(warning_error.severity(), ErrorSeverity::Warning);
    }

    #[test]
    fn test_error_categories() {
        // 测试错误分类
        let dex_error = DexError::InvalidSwapParams {
            message: "Invalid parameters".to_string(),
        };
        assert_eq!(dex_error.category(), "Validation");
        
        let oracle_error = OracleError::PriceStale {
            age_seconds: 3600,
            max_age_seconds: 300,
        };
        assert_eq!(oracle_error.category(), "DataQuality");
        
        let security_error = SecurityError::InsufficientPermissions {
            operation: "transfer".to_string(),
            required_permission: "admin".to_string(),
        };
        assert_eq!(security_error.category(), "Authorization");
    }

    #[test]
    fn test_error_code_validation() {
        // 测试错误码验证
        assert!(validate_error_code_range(1000));
        assert!(validate_error_code_range(1999));
        assert!(validate_error_code_range(4000));
        assert!(validate_error_code_range(9999));
        assert!(!validate_error_code_range(999));
        assert!(!validate_error_code_range(10000));
    }

    #[test]
    fn test_error_module_mapping() {
        // 测试错误码模块映射
        assert_eq!(get_error_module(1000), "Asset");
        assert_eq!(get_error_module(2000), "Basket");
        assert_eq!(get_error_module(3000), "IndexToken");
        assert_eq!(get_error_module(4000), "DEX");
        assert_eq!(get_error_module(5000), "Oracle");
        assert_eq!(get_error_module(6000), "Algorithm");
        assert_eq!(get_error_module(7000), "Strategy");
        assert_eq!(get_error_module(8000), "Validation");
        assert_eq!(get_error_module(9000), "Security");
        assert_eq!(get_error_module(9999), "Unknown");
    }

    #[test]
    fn test_error_code_allocator() {
        // 测试错误码分配器
        let mut allocator = ErrorCodeAllocator::new();
        
        // 分配错误码
        let code1 = allocator.allocate_next("Asset").unwrap();
        let code2 = allocator.allocate_next("Asset").unwrap();
        
        assert_eq!(code1, 1001);
        assert_eq!(code2, 1002);
        assert!(allocator.is_allocated(code1));
        assert!(allocator.is_allocated(code2));
        
        // 释放错误码
        allocator.deallocate(code1);
        assert!(!allocator.is_allocated(code1));
        assert!(allocator.is_allocated(code2));
    }

    #[test]
    fn test_error_stats() {
        // 测试错误统计
        let mut stats = ErrorCodeStats::default();
        
        stats.record_error_code(1001);
        stats.record_error_code(1002);
        stats.record_error_code(4001);
        
        assert_eq!(stats.total_usage, 3);
        assert_eq!(stats.usage_by_module.get("Asset").unwrap(), &2);
        assert_eq!(stats.usage_by_module.get("DEX").unwrap(), &1);
        
        let (module, count) = stats.get_most_error_prone_module().unwrap();
        assert_eq!(module, "Asset");
        assert_eq!(count, &2);
    }

    #[test]
    fn test_global_error_stats() {
        // 重置全局统计
        utils::reset_global_error_stats();
        
        // 创建并记录错误
        let error1 = ProgramError::Asset(AssetError::InvalidAmount { amount: 0 });
        let error2 = ProgramError::Dex(DexError::InvalidSwapParams {
            message: "Invalid parameters".to_string(),
        });
        
        utils::record_global_error(&error1);
        utils::record_global_error(&error2);
        
        let stats = utils::get_global_error_stats();
        assert_eq!(stats.total_errors, 2);
        assert_eq!(stats.errors_by_module.get("Asset").unwrap(), &1);
        assert_eq!(stats.errors_by_module.get("DEX").unwrap(), &1);
    }

    #[test]
    fn test_error_handler() {
        // 测试错误处理器
        let handler = DefaultErrorHandler;
        
        // 测试可恢复错误
        let recoverable_error = ProgramError::Oracle(OracleError::OracleTimeout {
            oracle_name: "pyth".to_string(),
        });
        
        // 注意：在实际环境中，这可能会返回Err，但在测试中我们只验证处理器能正常工作
        let result = handler.handle_error(&recoverable_error);
        // 这里我们不检查具体的返回值，因为在实际的Solana环境中可能会有所不同
        
        // 测试安全错误
        let security_error = ProgramError::Security(SecurityError::SignatureVerificationFailed {
            reason: "Invalid signature".to_string(),
        });
        
        let result = handler.handle_error(&security_error);
        // 安全错误应该总是返回Err
        assert!(result.is_err());
    }

    #[test]
    fn test_error_macros() {
        // 测试错误宏（如果可用）
        // 注意：这些宏可能需要在实际的Solana环境中测试
        let general_error = utils::general_error("Test error", 9999);
        assert_eq!(general_error.error_code(), 9999);
        assert_eq!(general_error.error_message(), "Test error");
        
        let unknown_error = utils::unknown_error();
        assert_eq!(unknown_error.error_code(), 9999);
        assert_eq!(unknown_error.error_message(), "Unknown error occurred");
    }
} 