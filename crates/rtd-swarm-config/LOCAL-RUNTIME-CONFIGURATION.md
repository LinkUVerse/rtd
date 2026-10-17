# 已有本地节点配置的原生来源校验

**强制领域边界：NodeConfig、创世验证、密钥/角色语义及Rust测试属于正式rtd工程。归档admin只消费原生报告并编排固定进程；all-in-one-deploy仅承接Shell和必要配置/文档/结果，不承接这些Rust或Python业务。**

```sh
rtd-genesis-inspect --inspect-node /private/node-request.json REQUEST_SHA256
```

复用已交付的rtd-genesis-inspect角色，未增加Cargo工程或另一套NodeConfig类型；原创世、blob及部署签名者命令保持兼容。普通检查只读，不调用rtd start，不生成创世、密钥、端口、文件或数据库，不修改fullnode.yaml，也不猜测/迁移旧数据库路径。

版本1请求使用JSON，所有字段显式，拒绝重复、未知或缺失字段；Pin为`{path,sha256}`。

| 字段 | 来源及约束 |
|---|---|
| version / role | 版本1，validator或fullnode |
| environmentDirectory | 现有、规范绝对路径、当前用户私有目录 |
| chainId | 与真实genesisInput/genesisBlob验证结果相同的完整链ID |
| configuration | 原NodeConfig YAML的私有Pin，最大4MiB |
| genesisInput | 权威创世YAML私有Pin，最大1MiB |
| genesisBlob | 原生BCS创世Pin，最大128MiB，复用实际创世校验而非仅比较声明链号 |
| extraInputs | 显式额外只读文件Pin数组，空时也必须给出[] |
| extraStateDirectories | 显式额外私有状态目录数组，空时也必须给出[] |

先检查原YAML中的四类密钥、DB路径、network/RPC/metrics/admin/P2P监听、保留参数、checkpoint并发/超时及gRPC配置是否显式存在，然后才反序列化真实NodeConfig。拒绝嵌套重复键、非字符串映射键、非有限浮点预算，以及真实类型往返时丢失的未知/不支持字段。其他仍由原生实现提供的可序列化缺省字段单独列入nativeDefaults，不能假称来自原文件；effectiveConfigurationSha256绑定本次真实类型解析结果，消费者还须固定校验器/节点程序版本。

密钥仅接受单一内嵌value或规范私有path。文件密钥从已经固定的有界字节解码，校验时不调用旧懒加载器的panic路径；只在内存中建立检查快照，不写回原配置/密钥。worker/network必须为Ed25519，账户保留原生RtdKeyPair支持的全部方案；validator的协议、network、worker、账户及网络地址须匹配原生genesis验证者集合，且存在consensus配置。fullnode不能冒用验证者协议身份或携带consensus配置。节点genesis仅接受与请求Pin完全相同的显式genesis-file-location，不使用相对路径或另一份内嵌创世。

DB、consensus、checkpoint输出及额外状态目录必须预先存在、私有、位于环境目录内，彼此不包含、不重复，也不能包含固定输入。其他绝对文件路径必须被Pin固定；NodeConfig内的network multiaddr不作为文件路径。已识别的文件/目录字段不能使用相对路径。输入禁止符号链接，校验结束重新核对所有原始字节；不对运行中数据库做摘要或打开RocksDB。目录存在或路径匹配不能代替已有链数据的完整性/所有权/恢复检查。

报告包含角色、原链/创世、公开协议身份、固定监听、私有状态目录、全部inputs及每叶fieldSources，不输出配置或私钥原值。`started/databaseOpened/remoteVerified`始终false。无环境变量合并；实际部署还应由固定生命周期清空继承环境并重验全部inputs。

该接口为后续固定rtd-node阶段供给配置依据，避免rtd start的隐式创世、随机Fullnode端口及旧DB路径选择。尚未代表完整validator组、全栈阶段装配、运行健康、保留数据接管、实际重启或R0.07恢复已验收；旧启动入口退役按原工作包执行，不因这个检查器存在而删除现行依赖。
