//!
//! DEX Offchain Config Module
//!
//! 本模块定义 DEX 离线服务的配置结构体、常量与初始化逻辑，确保配置合规、可维护、可扩展。

/// DEX 离线服务配置结构体。
#[derive(Debug, Clone)]
pub struct DexOffchainConfig {
    pub api_url: String,        // DEX API 基础 URL
    pub api_key: String,       // API 密钥
    pub timeout_secs: u64,     // 请求超时时间（秒）
    pub max_retries: u8,       // 最大重试次数
}

impl DexOffchainConfig {
    /// 创建默认配置。
    pub fn default() -> Self {
        Self {
            api_url: String::from("https://api.dex.example.com"),
            api_key: String::new(),
            timeout_secs: 10,
            max_retries: 3,
        }
    }
    /// 检查配置有效性。
    pub fn is_valid(&self) -> bool {
        !self.api_url.is_empty() && self.timeout_secs > 0 && self.max_retries > 0
    }
}
