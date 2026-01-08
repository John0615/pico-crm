use jsonwebtoken::Algorithm;
use std::env;

#[derive(Debug, Clone)]
pub struct JwtConfig {
    /// JWT 加密/解密密钥（生产环境从环境变量读取）
    pub secret: String,
    /// JWT 加密算法（如 HS256/HS512）
    pub algorithm: Algorithm,
    /// Token 过期时间（小时）
    pub expiry_hours: i64,
    /// 可选：Refresh Token 过期时间（小时）
    pub refresh_expiry_days: i64,
}

impl JwtConfig {
    /// 从环境变量加载配置（最常用的初始化方式）
    pub fn from_env() -> Result<Self, String> {
        // 从环境变量读取密钥（必须设置）
        let secret = env::var("JWT_SECRET")
            .map_err(|_| format!("JWT_SECRET environment variable is missing"))?;

        // 从环境变量读取过期时间（有默认值）
        let expiry_hours = env::var("JWT_EXPIRY_HOURS")
            .unwrap_or_else(|_| "24".into())
            .parse::<i64>()
            .map_err(|e| format!("JWT_EXPIRY_HOURS environment variable parse error: {}", e))?;

        // Refresh Token 过期时间（默认7天）
        let refresh_expiry_days = env::var("JWT_REFRESH_EXPIRY_DAYS")
            .unwrap_or_else(|_| "7".into())
            .parse::<i64>()
            .map_err(|e| {
                format!(
                    "JWT_REFRESH_EXPIRY_DAYS environment variable parse error: {}",
                    e
                )
            })?;

        Ok(Self {
            secret,
            algorithm: Algorithm::HS256, // 固定算法，也可从环境变量读取
            expiry_hours,
            refresh_expiry_days,
        })
    }

    /// 开发环境快捷初始化（仅用于本地开发，不要在生产环境使用）
    pub fn dev() -> Self {
        Self {
            secret: "dev_secure_secret_key_32_bytes_long!!!".into(),
            algorithm: Algorithm::HS256,
            expiry_hours: 24,
            refresh_expiry_days: 7,
        }
    }
}
