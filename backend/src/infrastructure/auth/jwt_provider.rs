use crate::domain::identity::auth::{AuthCredential, AuthProvider, JwtClaims};
use crate::domain::identity::user::User;
use crate::infrastructure::config::jwt::JwtConfig;
use crate::infrastructure::entity::users::{Column, Entity};
use crate::infrastructure::mappers::identity::user_mapper::UserMapper;
use async_trait::async_trait;
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use sea_orm::DatabaseConnection;
use sea_orm::prelude::Uuid;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};

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
        let user_name = user_name.to_string();
        let users = Entity::find()
            .filter(Column::UserName.eq(user_name))
            .all(&self.db_conn)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        if users.len() > 1 {
            return Err("用户名存在重复，请联系管理员处理".to_string());
        }

        Ok(users.into_iter().next().map(UserMapper::to_domain))
    }

    async fn get_user_by_name_for_merchant(
        &self,
        merchant_id: &str,
        user_name: &str,
    ) -> Result<Option<User>, String> {
        let merchant_uuid =
            Uuid::parse_str(merchant_id).map_err(|e| format!("invalid merchant uuid: {}", e))?;
        let user_name = user_name.to_string();
        let user = Entity::find()
            .filter(Column::MerchantUuid.eq(merchant_uuid))
            .filter(Column::UserName.eq(user_name))
            .one(&self.db_conn)
            .await
            .map_err(|e| format!("Database error: {}", e))?;
        Ok(user.map(UserMapper::to_domain))
    }

    async fn record_login_for_user(&self, user: &User) -> Result<(), String> {
        let user_uuid =
            Uuid::parse_str(&user.uuid).map_err(|e| format!("invalid user uuid: {}", e))?;
        let Some(original_user) = Entity::find_by_id(user_uuid)
            .one(&self.db_conn)
            .await
            .map_err(|e| format!("Database error: {}", e))?
        else {
            return Err("用户不存在".to_string());
        };

        let mut updated_user = user.clone();
        updated_user.record_login();
        let active_user = UserMapper::to_update_active_entity(updated_user, original_user);
        let _ = active_user
            .update(&self.db_conn)
            .await
            .map_err(|e| format!("Database error: {}", e))?;
        Ok(())
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
            let role = u.role.clone();
            let merchant_id = u
                .merchant_uuid
                .clone()
                .ok_or_else(|| "用户缺少商户信息".to_string())?;
            let _ = self.record_login_for_user(&u).await;
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

        let user = self
            .get_user_by_name_for_merchant(&claims.merchant_id, &claims.user_name)
            .await?;
        Ok(user)
    }

    async fn invalidate_credential(&self, _credential: &AuthCredential) -> Result<(), String> {
        Ok(())
    }
}
