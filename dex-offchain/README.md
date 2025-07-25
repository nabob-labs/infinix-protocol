# dex-offchain 模块文档

## 模块简介
`dex-offchain` 提供链下DEX/AMM集成服务，支持Jupiter、Orca等主流Solana DEX，REST接口对接链上/前端/自动化服务。

## 架构说明
- 统一trait抽象（`traits.rs`），便于扩展新DEX
- 工厂模式（`factory.rs`），支持动态适配
- 每个DEX独立适配器（如`jupiter.rs`、`orca.rs`）
- REST服务（`rest.rs`），自动路由、详细日志、错误处理

## 主要接口
- `POST /swap`：参数`{dex_type, input_token, output_token, amount_in, min_out}`
- `POST /quote`：参数`{dex_type, input_token, output_token, amount_in}`

## 用法示例
```bash
curl -X POST http://localhost:8080/swap -H 'Content-Type: application/json' -d '{"dex_type":"Jupiter","input_token":"USDC","output_token":"SOL","amount_in":1000000,"min_out":990000}'
```

## 扩展点
- 新增DEX：实现`OffchainDex` trait并注册到factory
- 支持gRPC、WebSocket等接口扩展
- 日志、监控、风控可按需增强 