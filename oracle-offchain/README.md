# oracle-offchain 模块文档

## 模块简介
`oracle-offchain` 提供链下预言机集成服务，支持Pyth、Switchboard、Chainlink等主流Solana预言机，REST接口对接链上/前端/自动化服务。

## 架构说明
- 统一trait抽象（`traits.rs`），便于扩展新预言机
- 工厂模式（`factory.rs`），支持动态适配
- 每个预言机独立适配器（如`pyth.rs`、`switchboard.rs`、`chainlink.rs`）
- REST服务（`rest.rs`），自动路由、详细日志、错误处理

## 主要接口
- `POST /get_price`：参数`{oracle_type, symbol}`
- `POST /get_twap`：参数`{oracle_type, symbol, interval}`
- `POST /get_vwap`：参数`{oracle_type, symbol, interval}`

## 用法示例
```bash
curl -X POST http://localhost:8081/get_price -H 'Content-Type: application/json' -d '{"oracle_type":"Pyth","symbol":"BTC/USD"}'
```

## 扩展点
- 新增预言机：实现`OffchainOracle` trait并注册到factory
- 支持gRPC、WebSocket等接口扩展
- 日志、监控、风控可按需增强 