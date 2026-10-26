# 专题 E：任意 S3 兼容存储的能力合同

## 1. 目标

归档服务不绑定具体对象存储厂商。publication、HBase S3A、checkpoint 读取都只使用标准 S3 API。

## 2. 当前 publication 实际依赖

发布层不是简单的对象 PUT，必须使用：

- 桶版本化（Versioning = Enabled）；
- 固定 VersionId 读取（GET `versionId=...`）；
- 单 key 版本列举（`ListObjectVersions`）；
- 版本化 COPY 与 COPY 结果/ETag 校验；
- 条件写/`If-None-Match`（取决于写入路径）；
- 用户元数据（发布 epoch 溯源）；
- 固定对象版本读回、latest 复核。

因此，“任意 S3 兼容”必须解释为：**支持上述标准 S3 能力子集的实现**。不支持版本化或版本列举的存储不能承载 publication published 桶。

HBase 的 S3A 根目录另有 rename、list、multipart 等语义要求，必须单独按目标存储版本验证，不能复用 checkpoint 存储的兼容结论。

## 3. 必须补齐

### 3.1 启动预检

publication 启动/发布前对 staging 和 published 桶执行：

1. `GET ?versioning` 必须返回 `Enabled`（已有）；
2. 对已知元数据 key 执行一次有界 `ListObjectVersions`，验证：
   - 有版本时返回固定版本列表；
   - 无版本时返回空而不是挂起；
   - 分页字段符合合同；
3. 验证 `HEAD` 返回 `x-amz-version-id`（版本化对象）；
4. 失败则拒绝发布，不自动降级到“无版本模式”。

### 3.2 文档和拒绝路径

- 在部署文档明确最小能力矩阵；
- 在启动错误中明确缺少哪项能力；
- 不在运行时静默切换 path/virtual style 或跳过版本校验；
- HBase S3A 的兼容性验证单独列入中间件验收，不与 publication 混同。

## 4. 验收

1. 版本化开启 + 版本列举可用的存储：预检通过。
2. 未开启版本化：启动失败，错误包含 capability 类别。
3. 版本列举不支持/超时：启动失败，不进入发布循环。
4. 固定版本读、COPY、读回、latest 复核在隔离存储通过。
5. 文档不再宣称“任意 S3 兼容”无条件可用，而是列出能力前提。
6. HBase S3A 在目标存储版本上完成独立读写/恢复验证。
