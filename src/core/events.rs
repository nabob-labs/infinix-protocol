// ========================= 事件定义模块 =========================
// 本模块定义所有链上/链下交互、算法、DEX、预言机、指令等事件，
// 每个事件均采用 Anchor #[event] 宏声明，便于链下监听、索引、审计。
// 每个字段、用途、参数、类型、生命周期、序列化、边界均有详细注释。

use anchor_lang::prelude::*; // Anchor 预导入，包含 #[event]、Pubkey、类型等

/// 链下 DEX 请求事件结构体
/// - 用于链下服务监听 DEX swap/quote 请求，实现链上与链下 DEX 交互的异步解耦。
/// - Anchor 最佳实践：#[event] 属性，便于链下监听和索引。
#[event] // Anchor 事件宏，自动生成事件日志，便于链下监听
pub struct OffchainDexRequest {
    /// 请求唯一ID（链下响应需回填该ID）
    /// - 类型：u64，唯一标识一次链下 DEX 请求
    pub request_id: u64,
    /// DEX 名称（如Jupiter、Orca等）
    /// - 类型：String，标识目标 DEX
    pub dex_name: String,
    /// 操作类型（swap/quote）
    /// - 类型：String，标识操作类别
    pub action: String,
    /// 操作参数（序列化，便于链下解析）
    /// - 类型：Vec<u8>，bincode/JSON 序列化参数
    pub params: Vec<u8>,
    /// 请求者公钥（发起人身份）
    /// - 类型：Pubkey，链上用户身份
    pub requester: Pubkey,
    /// 请求时间戳（Unix秒）
    /// - 类型：i64，链上 Clock::get()?.unix_timestamp
    pub timestamp: i64,
}

/// 链下预言机请求事件结构体
/// - 用于链下服务监听 Oracle price/twap 请求，实现链上与链下预言机交互的异步解耦。
/// - Anchor 最佳实践：#[event] 属性，便于链下监听和索引。
#[event]
pub struct OffchainOracleRequest {
    /// 请求唯一ID（链下响应需回填该ID）
    pub request_id: u64, // 唯一标识一次链下 Oracle 请求
    /// 预言机名称（如Pyth、Chainlink等）
    pub oracle_name: String, // 目标预言机名称
    /// 操作类型（price/twap）
    pub action: String, // 操作类别
    /// 操作参数（序列化，便于链下解析）
    pub params: Vec<u8>, // bincode/JSON 序列化参数
    /// 请求者公钥（发起人身份）
    pub requester: Pubkey, // 链上用户身份
    /// 请求时间戳（Unix秒）
    pub timestamp: i64, // 链上时间戳
}

/// 链下响应事件结构体
/// - 用于链下服务返回 DEX/Oracle 结果，实现链上与链下异步通信。
/// - Anchor 最佳实践：#[event] 属性，便于链下监听和索引。
#[event]
pub struct OffchainResponse {
    /// 请求唯一ID（与请求事件对应）
    pub request_id: u64, // 对应请求事件的 request_id
    /// 是否成功（true=成功，false=失败）
    pub success: bool, // 响应是否成功
    /// 结果数据（序列化，链下返回结果）
    pub result: Vec<u8>, // bincode/JSON 序列化结果
    /// 错误信息（可选，失败时填充）
    pub error_msg: Option<String>, // 失败时的错误描述
    /// 响应者公钥（链下服务身份）
    pub responder: Pubkey, // 链下服务身份
    /// 响应时间戳（Unix秒）
    pub timestamp: i64, // 响应时间
}

/// 算法执行事件结构体
/// - 用于链上/链下观测算法执行结果，便于调试、审计、性能分析。
/// - Anchor 最佳实践：#[event] 属性，便于链下监听和索引。
#[event]
pub struct AlgorithmExecuted {
    /// 算法名称（如TWAP、VWAP等）
    pub algorithm: String, // 算法唯一名称
    /// 执行结果描述（如成功/失败/收益等）
    pub result: String, // 执行结果描述
    /// 执行时间戳（Unix秒）
    pub timestamp: i64, // 执行时间
}

/// DEX 交易执行事件结构体
/// - 用于链上/链下观测 DEX swap 结果，便于审计、性能分析。
/// - Anchor 最佳实践：#[event] 属性，便于链下监听和索引。
#[event]
pub struct DexSwapExecuted {
    /// DEX 名称（如Jupiter、Orca等）
    pub dex: String, // 目标 DEX 名称
    /// 输入数量（原始token数量）
    pub amount_in: u64, // swap 输入数量
    /// 输出数量（目标token数量）
    pub amount_out: u64, // swap 输出数量
    /// 输入代币 mint（Pubkey）
    pub token_in: Pubkey, // 输入代币 mint
    /// 输出代币 mint（Pubkey）
    pub token_out: Pubkey, // 输出代币 mint
    /// 用户公钥（发起人身份）
    pub user: Pubkey, // swap 发起人
    /// 执行时间戳（Unix秒）
    pub timestamp: i64, // swap 执行时间
}

/// 预言机价格查询事件结构体
/// - 用于链上/链下观测 Oracle 查询结果，便于审计、性能分析。
/// - Anchor 最佳实践：#[event] 属性，便于链下监听和索引。
#[event]
pub struct OraclePriceQueried {
    /// 预言机名称（如Pyth、Chainlink等）
    pub oracle: String, // 目标预言机名称
    /// 基础资产 mint（Pubkey）
    pub base_mint: Pubkey, // 基础资产 mint
    /// 报价资产 mint（Pubkey）
    pub quote_mint: Pubkey, // 报价资产 mint
    /// 查询价格（整数，单位依赖具体实现）
    pub price: u64, // 查询价格
    /// 查询时间戳（Unix秒）
    pub timestamp: i64, // 查询时间
}

/// 指令分发事件结构体
/// - 用于链上/链下观测指令调用，便于审计、权限追踪。
/// - Anchor 最佳实践：#[event] 属性，便于链下监听和索引。
#[event]
pub struct InstructionDispatched {
    /// 指令名称（如mint_asset等）
    pub instruction: String, // 指令唯一名称
    /// 相关账户列表（所有涉及账户的Pubkey）
    pub accounts: Vec<Pubkey>, // 涉及账户列表
    /// 指令参数（序列化，便于链下解析）
    pub params: Vec<u8>, // bincode/JSON 序列化参数
    /// 调用用户公钥（发起人身份）
    pub user: Pubkey, // 指令发起人
    /// 分发时间戳（Unix秒）
    pub timestamp: i64, // 分发时间
}
// ========================= 事件定义模块 END ========================= 