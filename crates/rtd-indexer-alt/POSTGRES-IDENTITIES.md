# Alt 数据库运行身份

职责、实现和Rust测试属于正式RTD的`rtd-indexer-alt::postgres_identity`。归档工程复用导出的运行连接信息，部署工程只保留`manage_alt_postgres.sh`薄Shell。此功能不接管现行身份、不启动服务、不重建业务表，也不证明全栈ready。

| profile | SELECT | INSERT / UPDATE / DELETE |
|---|---|---|
| writer | 27张受fence保护的原生业务表、writer fence和Diesel版本表 | 27张业务表可INSERT/UPDATE/DELETE；fence仅INSERT/UPDATE；版本表不可写 |
| query-reader | 上述29张表 | 全部拒绝 |
| coverage-reader | watermarks、kv_checkpoints | 全部拒绝 |

不授予TRUNCATE、REFERENCES、TRIGGER、MAINTAIN、grant option、对象所有权、角色成员、schema/database CREATE、TEMP、外部数据或危险SECURITY DEFINER函数权限。逐表、逐列核对有效权限，包括PUBLIC来源。只读默认设置仅为辅助，关闭后仍不能写。写者仍须使用原有独占锁、任期fence和已认证输入；本供应器不代替停旧写者、撤权或交接协议。

## 显式初始化与供应

先通过既有`initialize-schema --apply`安装完整43个合并版本及27个guard。新增的`--restrict-runtime-access`只能与`--apply`同时使用，在同一admission锁保护范围内明确撤销当前数据库PUBLIC CREATE/TEMP及public schema PUBLIC CREATE。重复执行保留业务数据和writer epoch。默认schema检查、普通indexer和身份plan/verify均不执行该步骤。

身份供应本身要求这些PUBLIC权限已经收紧；缺失时在CREATE ROLE前拒绝，不隐式修改其他身份或PUBLIC权限。

```text
rtd-indexer-alt postgres-identity --profile writer|query-reader|coverage-reader \
  --action prepare|plan|apply|verify|binding \
  --config /absolute/private/config.json --expected-sha256 SHA256 \
  --state-directory /absolute/private/identity-state \
  [--identity-sha256 IDENTITY_SHA] [--expected-plan-sha256 PLAN_SHA]
```

prepare不接受identity/plan参数；其余四个动作必须指定identity摘要；只有apply接受并要求当前plan摘要。Shell使用位置参数`PROFILE ACTION CONFIG SHA STATE [IDENTITY_SHA [PLAN_SHA]]`，从已固定摘要的RTD产物解析器选择binary，不构建、下载或回退Python。

输入是严格camelCase、拒绝重复/未知字段的私有JSON：

```json
{
  "version": 1,
  "environmentId": "reviewed-environment",
  "chainId": "FULL_CANONICAL_BASE58_CHAIN_ID",
  "dataGeneration": "71000000-0000-4000-8000-000000000001",
  "requestId": "72000000-0000-4000-8000-000000000001",
  "administratorUrl": {"path": "/private/administrator.url", "sha256": "SHA256"},
  "tlsCa": {"path": "/private/ca.crt", "sha256": "SHA256"},
  "runtimeMaxConnections": 4,
  "operationTimeoutMs": 30000
}
```

administrator.url只包含一个完整PostgreSQL URL，明确数据库、账号、密码和唯一的`sslmode=require`。不继承PG环境变量或URL options；数据库名仅接受1～63位字母、数字、下划线。默认验证服务端证书/SAN并核对实际TLS。只有测试CLI显式`--allow-loopback-development`允许数字loopback与`sslmode=disable`，部署Shell不开放该选项。管理接口要求PG17管理身份；导出的运行身份永远不是该管理身份。

## 状态与验证边界

prepare离线生成独立UUID和随机口令，重复调用不换身份。state目录0700，identity.json、database.url、postgres.json和lock为0600。源文件/CA、配置、身份和运行投影均绑定摘要；禁止链接、宽权限或普通verify修复丢失文件。运行投影不含管理员凭据。调用方须同时把CA和`--tls-verify-cert --tls-ca-cert-path`接入RTD服务，不能只设置URL的sslmode就当作证书验证已接通。

plan在只读快照中检查完整Diesel版本、fence ABI、29表目录、ACL、默认权限、schema/函数、集群标识、库OID、登录属性和所有非空watermarks链标识。链/环境/代际绑定配置及当前已存在的链事实；空库检查不证明检查点认证、数据代际迁移或行级隔离。

apply在原有admission锁下重验plan，写持久intent，再用SCRAM verifier创建专用角色并授固定权限，SQL不包含明文口令。提交后重新认证和核验；旧回执重放也重新检查。成功记录角色OID、数据库OID及集群system identifier。已供应角色被删除、同名替换、数据库换代，或回执存在而供应记录丢失时拒绝继续，不能通过再次prepare/初始化复活。角色的数据库专属设置也必须为空，防止覆盖固定运行设置。

未知提交后保留同一身份：角色仍存在且认证/权限有效时，可以重新plan后显式对账；已存在intent但角色缺失时拒绝重新创建，交由显式状态恢复处理。不清空journal、不分配第二个UUID掩盖未知结果。普通indexer、GraphQL和readiness仍沿用各自的认证、链与水位检查。

## 普通启动阶段

`--action binding`只检查本地已有身份、供应marker及投影，导出`alt-postgres-verification-binding` v1 JSON：环境、完整链ID、配置代际、固定profile、操作预算、productionTls和全部来源路径/SHA。不输出连接URL或口令，不连接PG，不创建文件；缺少有效supplied.json时拒绝。此结果是本地材料摘要，不证明当前远端身份有效，也没有额外的签名认证。

将输出作为0600不可变文件，与当前固定RTD binary的路径/SHA一并交给归档工程的storage-verification-plan；三个kind为alt-writer-postgres、alt-query-postgres、alt-coverage-postgres。归档只负责固定阶段编排，完整SQL/身份语义仍由本crate执行。阶段argv固定为`postgres-identity --profile PROFILE --action verify`及已封存输入，不提供开发明文覆盖。生成期间不启动RTD或查询PG；运行阶段才验证真实TLS、当前角色OID、权限、密码、表结构和水位链身份，失败阻止后续阶段，无授权修复/身份重发。

完整请求合同见正式归档工程rtd-archive-admin/STORAGE-PREFLIGHT.md。管理预检材料含管理员URL来源，不能把整个inputs投影给业务进程。配置代际仍仅是本地供应scope，不能将它描述为PG行级代际认证、已导入旧身份或全栈恢复完成。
