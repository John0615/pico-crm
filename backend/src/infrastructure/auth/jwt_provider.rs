use crate::domain::auth::claims::JwtClaims;
use crate::domain::auth::provider::{AuthCredential, AuthProvider};
use crate::domain::models::user::User;
use crate::domain::queries::user::UserQuery;
use crate::infrastructure::config::app::AppConfig;
use crate::infrastructure::config::jwt::JwtConfig;
use crate::infrastructure::entity::users::{Column, Entity};
use crate::infrastructure::mappers::user_mapper::UserMapper;
use crate::infrastructure::queries::user_query_impl::SeaOrmUserQuery;
use crate::infrastructure::tenant::{schema_name_from_merchant, with_tenant_txn};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use sea_orm::DatabaseConnection;

#[derive(Debug, Clone)]
pub struct JwtAuthProvider {
    db_conn: DatabaseConnection,
    jwt_config: JwtConfig,
}

impl JwtAuthProvider {
    pub fn new(db_conn: DatabaseConnection) -> Self {
        let jwt_config = if cfg!(debug_assertions) {
            JwtConfig::dev()
        } else {
            JwtConfig::from_env().expect("JWT configuration not found")
        };
        Self {
            db_conn,
            jwt_config,
        }
    }

    pub fn issue_token(
        &self,
        user: &User,
        merchant_id: String,
        role: String,
    ) -> Result<String, String> {
        self.generate_jwt(user.uuid.clone(), user.user_name.clone(), merchant_id, role)
    }

    fn generate_jwt(
        &self,
        user_uuid: String,
        user_name: String,
        merchant_id: String,
        role: String,
    ) -> Result<String, String> {
        let expiration = Utc::now() + Duration::hours(self.jwt_config.expiry_hours);
        let claims = JwtClaims {
            sub: user_uuid,
            user_name: user_name,
            merchant_id,
            role,
            exp: expiration.timestamp(),
        };

        encode(
            &Header::new(self.jwt_config.algorithm),
            &claims,
            &EncodingKey::from_secret(self.jwt_config.secret.as_bytes()),
        )
        .map_err(|err| err.to_string())
    }

    fn validate_jwt(&self, token: &str) -> Result<JwtClaims, String> {
        decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(self.jwt_config.secret.as_bytes()),
            &Validation::new(self.jwt_config.algorithm),
        )
        .map(|data| data.claims)
        .map_err(|err| err.to_string())
    }

    pub fn get_claims(&self, token: &str) -> Result<JwtClaims, String> {
        self.validate_jwt(token)
    }

    async fn get_user_by_name(&self, user_name: &str) -> Result<Option<User>, String> {
        let user_query = SeaOrmUserQuery::new(self.db_conn.clone());
        let user = user_query.get_user(user_name).await?;
        Ok(user)
    }

    async fn get_user_by_name_in_schema(
        &self,
        schema_name: &str,
        user_name: &str,
    ) -> Result<Option<User>, String> {
        let schema_name = schema_name.to_string();
        let user_name = user_name.to_string();
        with_tenant_txn(&self.db_conn, &schema_name, |txn| {
            let user_name = user_name.clone();
            Box::pin(async move {
                let user = Entity::find()
                    .filter(Column::UserName.eq(user_name))
                    .one(txn)
                    .await
                    .map_err(|e| format!("Database error: {}", e))?;
                Ok(user.map(UserMapper::to_domain))
            })
        })
        .await
    }
}

// 实现领域层的 AuthProvider 接口（用 JWT 完成认证）
#[async_trait]
impl AuthProvider for JwtAuthProvider {
    async fn authenticate(
        &self,
        user_name: &str,
        password: &str,
    ) -> Result<AuthCredential, String> {
        let result = self.get_user_by_name(&user_name).await?;

        let user: Option<User> = if let Some(u) = result {
            if u.verify_password(&password)? {
                // 检查用户状态是否为活跃
                if u.is_active() {
                    Some(u.into())
                } else {
                    return Err("用户账户已被禁用".to_string());
                }
            } else {
                None
            }
        } else {
            None
        };

        if let Some(u) = user {
            if u.is_admin() || u.role == "admin" {
                return Err("管理员请使用管理端登录".to_string());
            }
            let role = if u.is_admin() {
                "admin".to_string()
            } else {
                u.role.clone()
            };
            let merchant_id = if role == "admin" {
                "public".to_string()
            } else {
                u.merchant_uuid
                    .clone()
                    .ok_or_else(|| "用户缺少商户信息".to_string())?
            };
            let token = self
                .generate_jwt(u.uuid, u.user_name, merchant_id, role)
                .map_err(|e| format!("生成 Token 失败：{}", e))?;

            // 3. 返回抽象的认证凭证（领域层类型）
            Ok(AuthCredential(token))
        } else {
            Err("用户名或密码错误".to_string())
        }
    }

    async fn get_current_user(&self, credential: &AuthCredential) -> Result<Option<User>, String> {
        let claims = self
            .validate_jwt(&credential.0)
            .map_err(|_| format!("验证 Token 失败"))?;

        if claims.role == "admin" || claims.merchant_id == "public" {
            return Err("管理员请使用管理端登录".to_string());
        }

        let config = AppConfig::from_env()?;
        let schema_name =
            schema_name_from_merchant(&config.tenant_schema_prefix, &claims.merchant_id)?;
        let user = self
            .get_user_by_name_in_schema(&schema_name, &claims.user_name)
            .await?;
        Ok(user)
    }

    async fn invalidate_credential(&self, _credential: &AuthCredential) -> Result<(), String> {
        Ok(())
    }
}
