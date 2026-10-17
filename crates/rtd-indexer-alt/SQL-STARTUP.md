# Alt SQL初始化与普通启动

`rtd-indexer-alt indexer`只接受已安装的当前SQL。它在推进writer epoch之前读取完整Diesel版本集合，并核对已安装的fence函数、表和27个statement触发器；缺失、未来版本、坏结构或关闭的guard均拒绝，不自动升级、降级或修复。继承的非public `search_path`覆盖也拒绝。取得admission锁后，使用同一会话再次验证迁移/guard，然后才推进epoch；没有新增连接预算，也不会在预检查与正式接管之间留下升级窗口。`--require-exclusive-writer`仍持有同一数据库的session锁，运行身份无需schema CREATE、迁移历史写权限或表所有权。

明确的安装/升级入口：

```sh
rtd-indexer-alt initialize-schema --database-url-file /private/admin.url
rtd-indexer-alt initialize-schema --database-url-file /private/admin.url --apply
rtd-indexer-alt initialize-schema --database-url-file /private/admin.url --apply --restrict-runtime-access
```

第一条只读验证；第二条才执行合并后的原生迁移、准备空fence表并安装guard。URL仅从原有owner-only普通文件读取，拒绝链接或多行。应用需要至少两个明确的池连接，并持有与writer相同的admission锁；运行中的writer阻止初始化。调用者仍须先停写并准备备份，版本不兼容时按恢复计划处理，不自动逆向执行down.sql。

第三条另明确收紧当前数据库PUBLIC CREATE/TEMP与public schema PUBLIC CREATE，供[固定运行身份供应](./POSTGRES-IDENTITIES.md)使用。该开关必须与apply同时指定；普通启动和身份plan/verify均不代为修改PUBLIC权限。权限供应、同身份续接、已供应角色OID/数据库/集群绑定与历史身份转换分别按该合同执行。

普通启动调用`setup_indexer_existing_schema`；原有`setup_indexer`保留为显式测试/benchmark初始化帮助函数。`Db::require_current_migrations`直接只读查询已有历史，不调用可能创建元数据的MigrationHarness。正常writer取新epoch使用已存在的fence表，epoch是写入代际，不属于schema迁移。初始化失败可能停在已提交的迁移版本；再次只读核验后显式续跑，普通启动始终拒绝不完整版本。

验收由正式归档工程的原生SQL/运维集成测试联合执行：空库的检查及普通启动零建表、未知/缺失版本和关闭guard拒绝且零epoch提升、运行锁阻止升级、显式初始化后以非owner且无DDL/迁移元数据写权限身份，在DDL事件拦截器开启时真实摄取36检查点/22管线。随后验证KILL恢复、修剪事务原子性、固定范围读取和原生备份恢复。测试使用隔离PG，未替换现行共享binary。
