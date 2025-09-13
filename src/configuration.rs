use crate::domain::SubscriberEmail;

/// 应用整体配置项
/// 从配置文件与环境变量反序列化而来
#[derive(serde::Deserialize, Debug, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,       // 数据库配置
    pub application: ApplicationSettings, // 应用运行时配置（主机/端口等）
    pub email: EmailClientSettings,       // 邮件客户端配置
}


#[derive(serde::Deserialize, Debug, Clone)]
pub struct EmailClientSettings {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub use_starttls: bool,
    pub sender_email: String,
}

impl EmailClientSettings {
    pub fn sender(&self) -> Result<SubscriberEmail, String> {
        SubscriberEmail::parse(self.sender_email.clone())
    }
}

/// 数据库连接配置
#[derive(serde::Deserialize, Debug, Clone)]
pub struct DatabaseSettings {
    pub username: String,      // 数据库用户名
    pub password: String,      // 数据库密码
    pub port: u16,             // 数据库端口
    pub host: String,          // 数据库主机名/IP
    pub database_name: String, // 数据库名称
}

/// 应用层配置（Host/Port）
#[derive(serde::Deserialize, Debug, Clone)]
pub struct ApplicationSettings {
    pub port: u16,    // HTTP 监听端口
    pub host: String, // 监听地址（如 127.0.0.1 或 0.0.0.0）
}

impl DatabaseSettings {
    /// 生成完整的 Postgres 连接串（包含数据库名）
    /// 形如：postgres://user:pwd@host:port/db
        pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }

    /// 生成不包含数据库名的连接串（用于建库等场景）
    /// 形如：postgres://user:pwd@host:port/
    pub fn connection_string_without_db(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/",
            self.username, self.password, self.host, self.port
        )
    }
}

/// 读取并合并配置：
/// 1) configuration/base.yaml 基础配置
/// 2) configuration/{ENV}.yaml 环境配置（ENV 来自 APP_ENVIRONMENT，默认 local）
/// 3) 环境变量覆盖（前缀 APP，层级分隔符 `__`，前缀与键名之间 `_`）
///    例如：`APP__APPLICATION__HOST=0.0.0.0` 会覆盖 `application.host`
pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    const CONFIG_DIR: &str = "configuration";
    const BASE_FILE: &str = "base.yaml";
    const ENV_KEY: &str = "APP_ENVIRONMENT";

    // 工程根目录
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join(CONFIG_DIR);

    // 侦测运行环境，默认 local
    let environment: Environment = std::env::var(ENV_KEY)
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");
    let environment_filename = format!("{}.yaml", environment.as_str());
    let settings = config::Config::builder()
        // 基础配置
        .add_source(config::File::from(configuration_directory.join(BASE_FILE)))
        // 环境配置
        .add_source(config::File::from(
            configuration_directory.join(environment_filename),
        ))
        // 环境变量覆盖
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    settings.try_deserialize()
}

/// 可选的运行环境
pub enum Environment {
    Local,
    Production,
}

impl Environment {
    /// 以字符串形式返回环境名（用于拼接文件名等）
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        // 仅有 ASCII 名称，使用 to_ascii_lowercase 更高效
        let s = s.to_ascii_lowercase();
        match s.as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `production`.",
                other
            )),
        }
    }
}
