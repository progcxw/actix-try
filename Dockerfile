# 使用最新的稳定版 Rust 作为基础镜像
FROM rust:1.89.0
# 将工作目录切换到 `app` (相当于执行 `cd app`)
# 如果 `app` 文件夹不存在，Docker 会自动为我们创建
WORKDIR /app
# 将当前工作环境中的所有文件复制到 Docker 镜像中
COPY . .
# 启用离线模式，让 sqlx 用 .sqlx 里的缓存，而不是去连数据库
ENV SQLX_OFFLINE=true
# 构建我们的二进制文件！
# 使用 release 配置以获得更快的性能
RUN cargo build --release
# 当执行 `docker run` 时，启动该二进制文件！
ENTRYPOINT ["./target/release/actix-try"]
