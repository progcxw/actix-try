FROM lukemathwalker/cargo-chef:latest-rust-1.89.0 as chef
# 将工作目录切换到 `app` (相当于执行 `cd app`)
# 如果 `app` 文件夹不存在，Docker 会自动为我们创建
WORKDIR /app

FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# 将当前工作环境中的所有文件复制到 Docker 镜像中
COPY . .
# 启用离线模式，让 sqlx 用 .sqlx 里的缓存，而不是去连数据库
ENV SQLX_OFFLINE=true
# 使用 release 配置以获得更快的性能
RUN cargo build --release

FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update -y && apt-get install -y --no-install-recommends openssl\
    # 清理不必要的文件以减小镜像体积
    && apt-get autoremove -y \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/actix-try actix-try
COPY configuration configuration
ENV APP_ENVIRONMENT=production
# 当执行 `docker run` 时，启动该二进制文件！
ENTRYPOINT ["./actix-try"]
