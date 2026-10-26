# 实施顺序与统一验收（总纲）

## 1. 顺序

1. **基线**：锁定源码/SBOM/active manifest，明确补丁状态。
2. **产物**：publisher 发布 `epochs.json`；按管线决定 `.chk`；补发布审计与恢复路径。
3. **入口**：公共 gRPC 补 `GetEpoch`；GraphQL surface 声明与必要路由。
4. **中间件门禁**：HBase gateway conformance 锁 + 回归；S3 能力预检。
5. **性能**：非中间件基准脚本与首轮数据。
6. **统一回归**：source verify、publication 恢复、gRPC/GraphQL 查询矩阵、失败负例。

## 2. 统一验收清单

- [ ] source.lock / active manifest / docs 同一提交，verify-source 通过。
- [ ] published 桶包含 `epochs.json`；`.chk` 策略与管线一致。
- [ ] `epochs.json` 单调、无未来项、版本不可回退。
- [ ] 公共 gRPC 14/14 方法，`GetEpoch` 正/负例通过。
- [ ] 主 GraphQL 37 个 Query 根在已配置后端上可用；Fullnode/订阅依赖明确。
- [ ] HBase gateway conformance 锁通过，未知能力失败关闭。
- [ ] publication 启动对 S3 版本化/版本列举失败关闭。
- [ ] 非中间件性能基准可重复执行，结果归档。
- [x] 零 Python 门禁通过：实现根无 `*.py`，启动/构建/运行脚本无 `python`/`pip` 调用。
- [ ] 每个专题有 workLog，记录文件、命令、结果、未覆盖项。
- [ ] 新增代码/注释/配置无上游品牌字样，遵循命名替换规则。

## 3. 发布门禁

任何一项失败都不得宣称“已对标官方”；只能标记为“部分完成/待验”，并在工作日志中保留原始失败证据。
