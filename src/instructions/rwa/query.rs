//! RWA资产query指令
//! RWA查询指令实现，支持多种查询功能和数据聚合。

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::{AssetType, QueryParams};
use crate::services::rwa_service::RwaService;
use crate::oracles::traits::{OracleAdapter, OracleParams};
use crate::dex::adapter::DexAdapter;

/// RWA资产query指令账户上下文
#[derive(Accounts)]
pub struct QueryRwa<'info> {
    /// RWA账户，只读
    pub rwa: Account<'info, BasketIndexState>,
    
    /// Oracle程序账户（可选）
    pub oracle_program: Option<AccountInfo<'info>>,
    
    /// DEX程序账户（可选）
    pub dex_program: Option<AccountInfo<'info>>,
}

/// RWA查询参数结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct QueryRwaParams {
    /// 查询类型
    pub query_type: RwaQueryType,
    /// 是否包含价格数据
    pub include_price: bool,
    /// 是否包含流动性数据
    pub include_liquidity: bool,
    /// 是否包含历史数据
    pub include_history: bool,
    /// Oracle名称（可选）
    pub oracle_name: Option<String>,
    /// DEX名称（可选）
    pub dex_name: Option<String>,
    /// RWA类型
    pub rwa_type: String,
    /// 是否包含合规信息
    pub include_compliance: bool,
}

/// RWA查询类型枚举
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum RwaQueryType {
    /// 基本信息
    Basic,
    /// 价格信息
    Price,
    /// 流动性信息
    Liquidity,
    /// 历史数据
    History,
    /// 合规信息
    Compliance,
    /// 完整信息
    Full,
}

/// RWA查询结果结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RwaQueryResult {
    /// RWA ID
    pub rwa_id: Pubkey,
    /// RWA类型
    pub rwa_type: String,
    /// 总价值
    pub total_value: u64,
    /// 当前价格
    pub current_price: Option<u64>,
    /// 24小时价格变化
    pub price_change_24h: Option<i64>,
    /// 总流动性
    pub total_liquidity: Option<u64>,
    /// 支持的DEX列表
    pub supported_dexes: Vec<String>,
    /// 支持的Oracle列表
    pub supported_oracles: Vec<String>,
    /// 合规状态
    pub compliance_status: Option<String>,
    /// 查询时间戳
    pub timestamp: i64,
}

/// RWA资产query指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - params: 查询参数
/// - 返回: 查询结果
pub fn query_rwa(ctx: Context<QueryRwa>, params: QueryRwaParams) -> Result<RwaQueryResult> {
    let rwa = &ctx.accounts.rwa;
    
    // 验证资产类型
    require!(rwa.asset_type == AssetType::RWA, crate::errors::index_token_error::IndexTokenError::InvalidAssetType);
    
    // 验证RWA状态
    rwa.validate()?;
    
    // 创建查询参数
    let query_params = QueryParams {
        asset: rwa.id,
        include_price: params.include_price,
        include_liquidity: params.include_liquidity,
        include_history: params.include_history,
    };
    
    // 执行RWA查询逻辑
    let service = RwaService::new();
    let result = service.query_rwa(
        rwa,
        &query_params,
        params.oracle_name.as_ref(),
        params.dex_name.as_ref(),
        &params.rwa_type,
        params.include_compliance,
    )?;
    
    // 构建查询结果
    let query_result = RwaQueryResult {
        rwa_id: rwa.id,
        rwa_type: params.rwa_type,
        total_value: rwa.total_value,
        current_price: result.current_price,
        price_change_24h: result.price_change_24h,
        total_liquidity: result.total_liquidity,
        supported_dexes: result.supported_dexes,
        supported_oracles: result.supported_oracles,
        compliance_status: result.compliance_status,
        timestamp: Clock::get()?.unix_timestamp,
    };
    
    Ok(query_result)
}

/// RWA查询错误码
#[error_code]
pub enum QueryRwaError {
    /// 资产类型无效
    #[msg("Invalid asset type")] InvalidAssetType,
    /// 查询参数无效
    #[msg("Invalid query parameters")] InvalidQueryParams,
    /// Oracle不可用
    #[msg("Oracle not available")] OracleNotAvailable,
    /// DEX不可用
    #[msg("DEX not available")] DexNotAvailable,
    /// 查询失败
    #[msg("Query failed")] QueryFailed,
    /// RWA类型不支持
    #[msg("RWA type not supported")] RwaTypeNotSupported,
    /// 合规信息不可用
    #[msg("Compliance information not available")] ComplianceNotAvailable,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::prelude::Pubkey;
    
    /// 测试RWA查询参数验证
    #[test]
    fn test_query_rwa_params_validation() {
        let params = QueryRwaParams {
            query_type: RwaQueryType::Full,
            include_price: true,
            include_liquidity: true,
            include_history: false,
            oracle_name: Some("pyth".to_string()),
            dex_name: Some("jupiter".to_string()),
            rwa_type: "REAL_ESTATE".to_string(),
            include_compliance: true,
        };
        
        assert_eq!(params.query_type, RwaQueryType::Full);
        assert!(params.include_price);
        assert!(params.include_liquidity);
        assert!(!params.include_history);
        assert_eq!(params.oracle_name, Some("pyth".to_string()));
        assert_eq!(params.dex_name, Some("jupiter".to_string()));
        assert_eq!(params.rwa_type, "REAL_ESTATE");
        assert!(params.include_compliance);
    }
    
    /// 测试查询类型枚举
    #[test]
    fn test_rwa_query_type_enum() {
        assert_eq!(RwaQueryType::Basic, RwaQueryType::Basic);
        assert_ne!(RwaQueryType::Basic, RwaQueryType::Price);
        assert_ne!(RwaQueryType::Price, RwaQueryType::Liquidity);
        assert_ne!(RwaQueryType::Liquidity, RwaQueryType::Compliance);
    }
} 