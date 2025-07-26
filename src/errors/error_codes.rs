//!
//! 错误码常量模块
//!
//! 定义所有错误类型的错误码基础值，确保错误码的唯一性和可追溯性。

/// 错误码基础值定义
/// 
/// 每个模块分配1000个错误码空间，确保错误码不冲突
/// 错误码范围：1000-9999

/// 资产错误码基础值 (1000-1999)
pub const ASSET_ERROR_BASE: u32 = 1000;

/// 篮子错误码基础值 (2000-2999)
pub const BASKET_ERROR_BASE: u32 = 2000;

/// 指数代币错误码基础值 (3000-3999)
pub const INDEX_TOKEN_ERROR_BASE: u32 = 3000;

/// DEX错误码基础值 (4000-4999)
pub const DEX_ERROR_BASE: u32 = 4000;

/// Oracle错误码基础值 (5000-5999)
pub const ORACLE_ERROR_BASE: u32 = 5000;

/// 算法错误码基础值 (6000-6999)
pub const ALGORITHM_ERROR_BASE: u32 = 6000;

/// 策略错误码基础值 (7000-7999)
pub const STRATEGY_ERROR_BASE: u32 = 7000;

/// 验证错误码基础值 (8000-8999)
pub const VALIDATION_ERROR_BASE: u32 = 8000;

/// 安全错误码基础值 (9000-9999)
pub const SECURITY_ERROR_BASE: u32 = 9000;

/// 错误码范围验证
pub fn validate_error_code_range(error_code: u32) -> bool {
    matches!(
        error_code,
        ASSET_ERROR_BASE..=ASSET_ERROR_BASE + 999 |
        BASKET_ERROR_BASE..=BASKET_ERROR_BASE + 999 |
        INDEX_TOKEN_ERROR_BASE..=INDEX_TOKEN_ERROR_BASE + 999 |
        DEX_ERROR_BASE..=DEX_ERROR_BASE + 999 |
        ORACLE_ERROR_BASE..=ORACLE_ERROR_BASE + 999 |
        ALGORITHM_ERROR_BASE..=ALGORITHM_ERROR_BASE + 999 |
        STRATEGY_ERROR_BASE..=STRATEGY_ERROR_BASE + 999 |
        VALIDATION_ERROR_BASE..=VALIDATION_ERROR_BASE + 999 |
        SECURITY_ERROR_BASE..=SECURITY_ERROR_BASE + 999
    )
}

/// 获取错误码所属模块
pub fn get_error_module(error_code: u32) -> &'static str {
    match error_code {
        ASSET_ERROR_BASE..=ASSET_ERROR_BASE + 999 => "Asset",
        BASKET_ERROR_BASE..=BASKET_ERROR_BASE + 999 => "Basket",
        INDEX_TOKEN_ERROR_BASE..=INDEX_TOKEN_ERROR_BASE + 999 => "IndexToken",
        DEX_ERROR_BASE..=DEX_ERROR_BASE + 999 => "DEX",
        ORACLE_ERROR_BASE..=ORACLE_ERROR_BASE + 999 => "Oracle",
        ALGORITHM_ERROR_BASE..=ALGORITHM_ERROR_BASE + 999 => "Algorithm",
        STRATEGY_ERROR_BASE..=STRATEGY_ERROR_BASE + 999 => "Strategy",
        VALIDATION_ERROR_BASE..=VALIDATION_ERROR_BASE + 999 => "Validation",
        SECURITY_ERROR_BASE..=SECURITY_ERROR_BASE + 999 => "Security",
        _ => "Unknown",
    }
}

/// 错误码统计信息
#[derive(Debug, Default)]
pub struct ErrorCodeStats {
    /// 按模块分组的错误码使用次数
    pub usage_by_module: std::collections::HashMap<String, u64>,
    /// 按错误码分组的详细使用次数
    pub usage_by_code: std::collections::HashMap<u32, u64>,
    /// 总错误码使用次数
    pub total_usage: u64,
}

impl ErrorCodeStats {
    /// 记录错误码使用
    pub fn record_error_code(&mut self, error_code: u32) {
        self.total_usage += 1;
        
        // 按模块分组
        let module = get_error_module(error_code);
        *self.usage_by_module.entry(module.to_string()).or_insert(0) += 1;
        
        // 按错误码分组
        *self.usage_by_code.entry(error_code).or_insert(0) += 1;
    }
    
    /// 获取模块使用率
    pub fn get_module_usage_rate(&self, module: &str) -> f64 {
        if self.total_usage == 0 {
            return 0.0;
        }
        let module_usage = self.usage_by_module.get(module).unwrap_or(&0);
        *module_usage as f64 / self.total_usage as f64
    }
    
    /// 获取最常使用的错误码
    pub fn get_most_used_error_code(&self) -> Option<(&u32, &u64)> {
        self.usage_by_code.iter().max_by_key(|(_, &count)| count)
    }
    
    /// 获取最常出错的模块
    pub fn get_most_error_prone_module(&self) -> Option<(&String, &u64)> {
        self.usage_by_module.iter().max_by_key(|(_, &count)| count)
    }
    
    /// 重置统计
    pub fn reset(&mut self) {
        self.total_usage = 0;
        self.usage_by_module.clear();
        self.usage_by_code.clear();
    }
}

/// 错误码范围检查
pub fn is_valid_error_code_range(start: u32, end: u32) -> bool {
    // 检查范围是否在允许的范围内
    if start < 1000 || end > 9999 {
        return false;
    }
    
    // 检查范围是否跨越多个模块
    let start_module = get_error_module(start);
    let end_module = get_error_module(end);
    
    start_module == end_module
}

/// 错误码分配器
/// 
/// 用于动态分配错误码，确保不冲突
#[derive(Debug)]
pub struct ErrorCodeAllocator {
    /// 已分配的错误码
    allocated_codes: std::collections::HashSet<u32>,
    /// 模块分配器
    module_allocators: std::collections::HashMap<String, u32>,
}

impl ErrorCodeAllocator {
    /// 创建新的错误码分配器
    pub fn new() -> Self {
        let mut allocator = Self {
            allocated_codes: std::collections::HashSet::new(),
            module_allocators: std::collections::HashMap::new(),
        };
        
        // 初始化模块分配器
        allocator.module_allocators.insert("Asset".to_string(), ASSET_ERROR_BASE);
        allocator.module_allocators.insert("Basket".to_string(), BASKET_ERROR_BASE);
        allocator.module_allocators.insert("IndexToken".to_string(), INDEX_TOKEN_ERROR_BASE);
        allocator.module_allocators.insert("DEX".to_string(), DEX_ERROR_BASE);
        allocator.module_allocators.insert("Oracle".to_string(), ORACLE_ERROR_BASE);
        allocator.module_allocators.insert("Algorithm".to_string(), ALGORITHM_ERROR_BASE);
        allocator.module_allocators.insert("Strategy".to_string(), STRATEGY_ERROR_BASE);
        allocator.module_allocators.insert("Validation".to_string(), VALIDATION_ERROR_BASE);
        allocator.module_allocators.insert("Security".to_string(), SECURITY_ERROR_BASE);
        
        allocator
    }
    
    /// 为指定模块分配下一个错误码
    pub fn allocate_next(&mut self, module: &str) -> Option<u32> {
        let base = self.module_allocators.get(module)?;
        let next_code = *base + self.get_module_usage_count(module) as u32 + 1;
        
        // 检查是否超出模块范围
        let max_code = match module {
            "Asset" => ASSET_ERROR_BASE + 999,
            "Basket" => BASKET_ERROR_BASE + 999,
            "IndexToken" => INDEX_TOKEN_ERROR_BASE + 999,
            "DEX" => DEX_ERROR_BASE + 999,
            "Oracle" => ORACLE_ERROR_BASE + 999,
            "Algorithm" => ALGORITHM_ERROR_BASE + 999,
            "Strategy" => STRATEGY_ERROR_BASE + 999,
            "Validation" => VALIDATION_ERROR_BASE + 999,
            "Security" => SECURITY_ERROR_BASE + 999,
            _ => return None,
        };
        
        if next_code > max_code {
            return None; // 模块错误码已用完
        }
        
        // 检查是否已被分配
        if self.allocated_codes.contains(&next_code) {
            return None;
        }
        
        self.allocated_codes.insert(next_code);
        Some(next_code)
    }
    
    /// 获取模块已使用的错误码数量
    pub fn get_module_usage_count(&self, module: &str) -> usize {
        let base = match module {
            "Asset" => ASSET_ERROR_BASE,
            "Basket" => BASKET_ERROR_BASE,
            "IndexToken" => INDEX_TOKEN_ERROR_BASE,
            "DEX" => DEX_ERROR_BASE,
            "Oracle" => ORACLE_ERROR_BASE,
            "Algorithm" => ALGORITHM_ERROR_BASE,
            "Strategy" => STRATEGY_ERROR_BASE,
            "Validation" => VALIDATION_ERROR_BASE,
            "Security" => SECURITY_ERROR_BASE,
            _ => return 0,
        };
        
        self.allocated_codes
            .iter()
            .filter(|&&code| code >= base && code < base + 1000)
            .count()
    }
    
    /// 检查错误码是否已被分配
    pub fn is_allocated(&self, error_code: u32) -> bool {
        self.allocated_codes.contains(&error_code)
    }
    
    /// 释放错误码
    pub fn deallocate(&mut self, error_code: u32) {
        self.allocated_codes.remove(&error_code);
    }
    
    /// 获取所有已分配的错误码
    pub fn get_allocated_codes(&self) -> &std::collections::HashSet<u32> {
        &self.allocated_codes
    }
    
    /// 重置分配器
    pub fn reset(&mut self) {
        self.allocated_codes.clear();
    }
}

impl Default for ErrorCodeAllocator {
    fn default() -> Self {
        Self::new()
    }
} 