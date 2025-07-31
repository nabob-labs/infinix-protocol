//! ETF资产query指令
//! ETF查询指令实现，支持多种查询功能和数据聚合。

use anchor_lang::prelude::*;
use crate::core::types::*;
use crate::services::etf_service::EtfService;
use crate::oracles::traits::{OracleAdapter, OracleParams};
use crate::dex::adapter::DexAdapter;

/// ETF资产query指令账户上下文
#[derive(Accounts)]
pub struct QueryEtf<'info> {
    /// ETF账户，只读
    pub etf: Account<'info, BasketIndexState>,
    
    /// Oracle程序账户（可选）
    pub oracle_program: Option<AccountInfo<'info>>,
    
    /// DEX程序账户（可选）
    pub dex_program: Option<AccountInfo<'info>>,
}

/// ETF查询参数结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct QueryEtfParams {
    /// 查询类型
    pub query_type: EtfQueryType,
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
    /// ETF类型
    pub etf_type: String,
}

/// ETF查询类型枚举
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum EtfQueryType {
    /// 基本信息
    Basic,
    /// 价格信息
    Price,
    /// 流动性信息
    Liquidity,
    /// 历史数据
    History,
    /// 完整信息
    Full,
}

/// ETF查询结果结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct EtfQueryResult {
    /// ETF ID
    pub etf_id: Pubkey,
    /// ETF类型
    pub etf_type: String,
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
    /// 查询时间戳
    pub timestamp: i64,
}

/// ETF资产query指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - params: 查询参数
/// - 返回: 查询结果
pub fn query_etf(ctx: Context<QueryEtf>, params: QueryEtfParams) -> anchor_lang::Result<EtfQueryResult> {
    let etf = &ctx.accounts.etf;
    
    // 验证资产类型
    require!(etf.asset_type == AssetType::ETF, crate::errors::index_token_error::IndexTokenError::InvalidAssetType);
    
    // 验证ETF状态
    etf.validate()?;
    
    // 创建查询参数
    let query_params = QueryParams {
        asset: etf.id,
        include_price: params.include_price,
        include_liquidity: params.include_liquidity,
        include_history: params.include_history,
    };
    
    // 执行ETF查询逻辑
    let service = EtfService::new();
    let result = service.query_etf(
        etf,
        &query_params,
        params.oracle_name.as_ref(),
        params.dex_name.as_ref(),
        &params.etf_type,
    )?;
    
    // 构建查询结果
    let query_result = EtfQueryResult {
        etf_id: etf.id,
        etf_type: params.etf_type,
        total_value: etf.total_value,
        current_price: result.current_price,
        price_change_24h: result.price_change_24h,
        total_liquidity: result.total_liquidity,
        supported_dexes: result.supported_dexes,
        supported_oracles: result.supported_oracles,
        timestamp: Clock::get()?.unix_timestamp,
    };
    
    Ok(query_result)
}

/// ETF查询错误码
#[error_code]
pub enum QueryEtfError {
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
    /// ETF类型不支持
    #[msg("ETF type not supported")] EtfTypeNotSupported,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::prelude::Pubkey;
    
    /// 测试ETF查询参数验证
    #[test]
    fn test_query_etf_params_validation() {
        let params = QueryEtfParams {
            query_type: EtfQueryType::Full,
            include_price: true,
            include_liquidity: true,
            include_history: false,
            oracle_name: Some("pyth".to_string()),
            dex_name: Some("jupiter".to_string()),
            etf_type: "SPY".to_string(),
        };
        
        assert_eq!(params.query_type, EtfQueryType::Full);
        assert!(params.include_price);
        assert!(params.include_liquidity);
        assert!(!params.include_history);
        assert_eq!(params.oracle_name, Some("pyth".to_string()));
        assert_eq!(params.dex_name, Some("jupiter".to_string()));
        assert_eq!(params.etf_type, "SPY");
    }
    
    /// 测试查询类型枚举
    #[test]
    fn test_etf_query_type_enum() {
        assert_eq!(EtfQueryType::Basic, EtfQueryType::Basic);
        assert_ne!(EtfQueryType::Basic, EtfQueryType::Price);
        assert_ne!(EtfQueryType::Price, EtfQueryType::Liquidity);
    }
} 