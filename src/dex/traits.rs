// ========================= DEX/AMM 统一 Trait 定义 =========================
// 本模块定义所有 DEX/AMM 适配器的统一 trait、参数、结果、错误类型，
// 每个 trait、struct、enum、参数、用途、边界、Anchor 相关点均有详细注释。

use anchor_lang::prelude::*; // Anchor 预导入，包含 Context、Result、Pubkey 等

/// DEX/AMM 适配器统一 Trait 接口
/// - 统一所有DEX/AMM的swap、流动性管理、报价等操作
/// - 便于多DEX集成、策略复用、测试模拟
pub trait DexAdapter: Send + Sync {
    /// 执行 swap 操作（最小功能单元）
    /// - ctx: Anchor 上下文，包含账户信息
    /// - params: swap参数（支持多资产、多市场、多DEX）
    /// - 返回：swap结果（SwapResult）
    fn swap(&self, ctx: Context<Swap>, params: SwapParams) -> anchor_lang::Result<SwapResult>;
    /// 添加流动性（最小功能单元）
    /// - ctx: Anchor 上下文
    /// - params: 添加流动性参数
    /// - 返回：获得的流动性token数量
    fn add_liquidity(&self, ctx: Context<AddLiquidity>, params: AddLiquidityParams) -> anchor_lang::Result<u64>;
    /// 移除流动性（最小功能单元）
    /// - ctx: Anchor 上下文
    /// - params: 移除流动性参数
    /// - 返回：返还的流动性token数量
    fn remove_liquidity(&self, ctx: Context<RemoveLiquidity>, params: RemoveLiquidityParams) -> anchor_lang::Result<u64>;
    /// 获取报价（最小功能单元）
    /// - ctx: Anchor 上下文
    /// - params: 报价参数
    /// - 返回：报价结果（QuoteResult）
    fn get_quote(&self, ctx: Context<GetQuote>, params: QuoteParams) -> anchor_lang::Result<QuoteResult>;
    /// 查询支持的资产类型
    fn supported_assets(&self) -> Vec<String> { vec![] }
    /// 查询支持的市场类型
    fn supported_markets(&self) -> Vec<String> { vec![] }
    /// DEX适配器类型
    fn adapter_type(&self) -> DexAdapterType { DexAdapterType::Other }
}

/// swap 操作参数结构体
/// - 描述一次 swap 操作的所有输入参数
#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct SwapParams {
    /// 输入token mint
    pub input_mint: Pubkey, // 交易输入token
    /// 输出token mint
    pub output_mint: Pubkey, // 交易输出token
    /// 输入数量
    pub amount_in: u64, // 输入token数量
    /// 最小可接受输出数量
    pub min_amount_out: u64, // 滑点保护
    /// 用户公钥
    pub user: Pubkey, // 交易发起人
    /// DEX相关账户列表
    pub dex_accounts: Vec<Pubkey>, // 适配器所需账户
}

/// swap 操作结果结构体
/// - 记录 swap 操作的实际输出和手续费
#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct SwapResult {
    /// 实际输出数量
    pub amount_out: u64, // swap输出token数量
    /// 手续费
    pub fee: u64, // swap手续费
}

/// 添加流动性参数结构体
/// - 描述一次 add_liquidity 操作的所有输入参数
#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct AddLiquidityParams {
    /// tokenA mint
    pub token_a: Pubkey, // 资产A
    /// tokenB mint
    pub token_b: Pubkey, // 资产B
    /// tokenA数量
    pub amount_a: u64, // 资产A数量
    /// tokenB数量
    pub amount_b: u64, // 资产B数量
    /// 用户公钥
    pub user: Pubkey, // 操作发起人
    /// DEX相关账户列表
    pub dex_accounts: Vec<Pubkey>, // 适配器所需账户
}

/// 移除流动性参数结构体
/// - 描述一次 remove_liquidity 操作的所有输入参数
#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct RemoveLiquidityParams {
    /// 移除的流动性token数量
    pub liquidity: u64, // LP token数量
    /// 用户公钥
    pub user: Pubkey, // 操作发起人
    /// DEX相关账户列表
    pub dex_accounts: Vec<Pubkey>, // 适配器所需账户
}

/// 报价参数结构体
/// - 描述一次 get_quote 操作的所有输入参数
#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct QuoteParams {
    /// 输入token mint
    pub input_mint: Pubkey, // 交易输入token
    /// 输出token mint
    pub output_mint: Pubkey, // 交易输出token
    /// 输入数量
    pub amount_in: u64, // 输入token数量
}

/// 报价结果结构体
/// - 记录一次报价操作的预期输出和手续费
#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct QuoteResult {
    /// 可获得输出数量
    pub amount_out: u64, // 预期输出token数量
    /// 手续费
    pub fee: u64, // 预期手续费
}

/// DEX适配器错误类型
/// - 统一所有DEX适配器的错误类型，便于上层统一处理
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, PartialEq, Eq)]
pub enum DexError {
    /// 账户无效
    InvalidAccount,
    /// 流动性不足
    InsufficientLiquidity,
    /// 滑点超限
    SlippageExceeded,
    /// 其他错误，附带错误信息
    Other(String),
}

/// MockDexAdapter为DexAdapter trait的测试实现，所有方法返回可控的测试数据，便于单元测试和集成测试。
pub struct MockDexAdapter;

impl DexAdapter for MockDexAdapter {
    /// 模拟swap操作，直接返回输入数量和0手续费
    fn swap(&self, _ctx: Context<Swap>, params: SwapParams) -> anchor_lang::Result<SwapResult> {
        Ok(SwapResult {
            amount_out: params.amount_in,
            fee: 0,
        })
    }
    /// 模拟添加流动性，返回两种资产数量之和
    fn add_liquidity(&self, _ctx: Context<AddLiquidity>, params: AddLiquidityParams) -> anchor_lang::Result<u64> {
        Ok(params.amount_a + params.amount_b)
    }
    /// 模拟移除流动性，返回输入的流动性token数量
    fn remove_liquidity(&self, _ctx: Context<RemoveLiquidity>, params: RemoveLiquidityParams) -> anchor_lang::Result<u64> {
        Ok(params.liquidity)
    }
    /// 模拟报价，返回输入数量和0手续费
    fn get_quote(&self, _ctx: Context<GetQuote>, params: QuoteParams) -> anchor_lang::Result<QuoteResult> {
        Ok(QuoteResult {
            amount_out: params.amount_in,
            fee: 0,
        })
    }
}
// ========================= DEX/AMM 统一 Trait 定义 END ========================= 