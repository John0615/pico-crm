use crate::domain::platform::merchant::MerchantRepository;
use crate::infrastructure::auth::jwt_provider::JwtAuthProvider;
use crate::infrastructure::entity::users::{Column as UserColumn, Entity as UserEntity};
use crate::infrastructure::mappers::identity::user_mapper::UserMapper;
use crate::infrastructure::tenant::with_tenant_txn;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

pub struct MerchantAuthAppService<R: MerchantRepository> {
    repo: R,
    db: DatabaseConnection,
    jwt: JwtAuthProvider,
}

impl<R: MerchantRepository> MerchantAuthAppService<R> {
    pub fn new(repo: R, db: DatabaseConnection, jwt: JwtAuthProvider) -> Self {
        Self { repo, db, jwt }
    }

    pub async fn authenticate_by_contact_phone(
        &self,
        contact_phone: &str,
        user_name: &str,
        password: &str,
    ) -> Result<String, String> {
        let merchant = self
            .repo
            .find_by_contact_phone(contact_phone)
            .await?
            .ok_or_else(|| "商户不存在或未开通".to_string())?;

        if merchant.status != "active" {
            return Err("商户已停用".to_string());
        }
        if let Some(expired_at) = merchant.expired_at {
            if expired_at <= Utc::now() {
                return Err("商户已过期".to_string());
            }
        }

        let schema_name = merchant.schema_name.clone();
        let login_user_name = user_name.to_string();
        let login_password = password.to_string();
        let merchant_uuid = merchant.uuid.clone();

        let user = with_tenant_txn(&self.db, &schema_name, |txn| {
            let login_user_name = login_user_name.clone();
            let login_password = login_password.clone();
            let merchant_uuid = merchant_uuid.clone();
            Box::pin(async move {
                let user_model = UserEntity::find()
                    .filter(UserColumn::UserName.eq(login_user_name))
                    .one(txn)
                    .await
                    .map_err(|e| format!("查询用户失败: {}", e))?
                    .ok_or_else(|| "用户名或密码错误".to_string())?;

                let mut user = UserMapper::to_domain(user_model.clone());

                if !user.verify_password(&login_password)? {
                    return Err("用户名或密码错误".to_string());
                }
                if !user.is_active() {
                    return Err("用户账户已被禁用".to_string());
                }
                if user.is_admin() || user.role == "admin" {
                    return Err("管理员请使用管理端登录".to_string());
                }
                if user.merchant_uuid.as_deref() != Some(merchant_uuid.as_str()) {
                    return Err("账号不属于该商户".to_string());
                }

                user.record_login();
                let active_user = UserMapper::to_update_active_entity(user.clone(), user_model);
                let _ = active_user
                    .update(txn)
                    .await
                    .map_err(|e| format!("更新登录时间失败: {}", e))?;

                Ok(user)
            })
        })
        .await?;

        let role = user.role.clone();
        let merchant_id = user
            .merchant_uuid
            .clone()
            .ok_or_else(|| "用户缺少商户信息".to_string())?;

        self.jwt
            .issue_token(&user, merchant_id, role)
            .map_err(|e| format!("生成 Token 失败：{}", e))
    }
}
