use crate::domain::identity::user::{
    EmploymentStatus, HealthStatus, User as DomainUser, UserRepository,
};
use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use shared::user::{CreateUserRequest, User};

pub struct UserCommandService<R: UserRepository> {
    user_repository: R,
}

impl<R: UserRepository> UserCommandService<R> {
    pub fn new(user_repository: R) -> Self {
        Self { user_repository }
    }

    pub async fn create_user(&self, request: CreateUserRequest) -> Result<User, String> {
        if let Ok(Some(_)) = self
            .user_repository
            .find_user_by_username(&request.user_name)
            .await
        {
            return Err("用户名已存在".to_string());
        }

        if let Some(ref email) = request.email {
            if let Ok(Some(_)) = self.user_repository.find_user_by_email(email).await {
                return Err("邮箱已存在".to_string());
            }
        }

        if let Some(ref phone_number) = request.phone_number {
            if let Ok(Some(_)) = self
                .user_repository
                .find_user_by_phone_number(phone_number)
                .await
            {
                return Err("手机号码已存在".to_string());
            }
        }

        let CreateUserRequest {
            user_name,
            password,
            email,
            phone_number,
            employment_status,
            skills,
            service_areas,
            training_records,
            certificates,
            health_status,
            employee_note,
            joined_at,
            avatar_url,
            merchant_uuid,
            role,
        } = request;

        let password_hash =
            DomainUser::hash_password(&password).map_err(|e| format!("密码加密失败: {}", e))?;
        let mut user = DomainUser::new(user_name, password_hash, email, phone_number);
        if let Some(role) = role {
            user.set_role(role);
        }
        if let Some(merchant_uuid) = merchant_uuid {
            user.set_merchant_uuid(merchant_uuid);
        }
        if let Some(avatar_url) = avatar_url {
            user.avatar_url = Some(avatar_url);
        }
        user.update_employee_profile(
            Some(parse_employment_status(employment_status.as_deref())?),
            skills,
            service_areas,
            training_records,
            certificates,
            Some(parse_health_status(health_status.as_deref())?),
            employee_note,
            parse_optional_datetime(joined_at.as_deref())?,
        )?;

        let created_user = self
            .user_repository
            .create_user(user)
            .await
            .map_err(|e| format!("创建用户失败: {}", e))?;

        Ok(created_user.into())
    }

    pub async fn update_user(
        &self,
        uuid: &str,
        request: CreateUserRequest,
    ) -> Result<User, String> {
        let mut user = self
            .user_repository
            .find_user_by_uuid(uuid)
            .await
            .map_err(|e| format!("查找用户失败: {}", e))?
            .ok_or("用户不存在")?;

        if let Ok(Some(existing_user)) = self
            .user_repository
            .find_user_by_username(&request.user_name)
            .await
        {
            if existing_user.uuid != user.uuid {
                return Err("用户名已被其他用户使用".to_string());
            }
        }

        if let Some(ref email) = request.email {
            if let Ok(Some(existing_user)) = self.user_repository.find_user_by_email(email).await {
                if existing_user.uuid != user.uuid {
                    return Err("邮箱已被其他用户使用".to_string());
                }
            }
        }

        if let Some(ref phone_number) = request.phone_number {
            if let Ok(Some(existing_user)) = self
                .user_repository
                .find_user_by_phone_number(phone_number)
                .await
            {
                if existing_user.uuid != user.uuid {
                    return Err("手机号码已被其他用户使用".to_string());
                }
            }
        }

        let CreateUserRequest {
            user_name,
            password,
            email,
            phone_number,
            employment_status,
            skills,
            service_areas,
            training_records,
            certificates,
            health_status,
            employee_note,
            joined_at,
            avatar_url,
            merchant_uuid,
            role,
        } = request;

        user.update_info(Some(user_name), email, phone_number, avatar_url);

        if let Some(role) = role {
            user.set_role(role);
        }
        if let Some(merchant_uuid) = merchant_uuid {
            user.set_merchant_uuid(merchant_uuid);
        }
        user.update_employee_profile(
            Some(parse_employment_status(employment_status.as_deref())?),
            skills,
            service_areas,
            training_records,
            certificates,
            Some(parse_health_status(health_status.as_deref())?),
            employee_note,
            parse_optional_datetime(joined_at.as_deref())?,
        )?;

        if !password.is_empty() {
            let password_hash =
                DomainUser::hash_password(&password).map_err(|e| format!("密码加密失败: {}", e))?;
            user.change_password(password_hash);
        }

        let updated_user = self
            .user_repository
            .update_user(user)
            .await
            .map_err(|e| format!("更新用户失败: {}", e))?;

        Ok(updated_user.into())
    }

    pub async fn get_user_by_uuid(&self, uuid: &str) -> Result<Option<User>, String> {
        let user = self
            .user_repository
            .find_user_by_uuid(uuid)
            .await
            .map_err(|e| format!("查找用户失败: {}", e))?;

        Ok(user.map(|domain_user| domain_user.into()))
    }

    pub async fn delete_user(&self, uuid: &str) -> Result<(), String> {
        let _user = self
            .user_repository
            .find_user_by_uuid(uuid)
            .await
            .map_err(|e| format!("查找用户失败: {}", e))?
            .ok_or("用户不存在")?;

        self.user_repository
            .delete_user(uuid.to_string())
            .await
            .map_err(|e| format!("删除用户失败: {}", e))?;

        Ok(())
    }

    pub async fn activate_user(&self, uuid: &str) -> Result<User, String> {
        let mut user = self
            .user_repository
            .find_user_by_uuid(uuid)
            .await
            .map_err(|e| format!("查找用户失败: {}", e))?
            .ok_or("用户不存在")?;

        user.activate();

        let updated_user = self
            .user_repository
            .update_user(user)
            .await
            .map_err(|e| format!("激活用户失败: {}", e))?;

        Ok(updated_user.into())
    }

    pub async fn deactivate_user(&self, uuid: &str) -> Result<User, String> {
        let mut user = self
            .user_repository
            .find_user_by_uuid(uuid)
            .await
            .map_err(|e| format!("查找用户失败: {}", e))?
            .ok_or("用户不存在")?;

        user.deactivate();

        let updated_user = self
            .user_repository
            .update_user(user)
            .await
            .map_err(|e| format!("禁用用户失败: {}", e))?;

        Ok(updated_user.into())
    }
}

fn parse_employment_status(value: Option<&str>) -> Result<EmploymentStatus, String> {
    EmploymentStatus::parse(value.unwrap_or("active").trim())
}

fn parse_health_status(value: Option<&str>) -> Result<HealthStatus, String> {
    HealthStatus::parse(value.unwrap_or("healthy").trim())
}

fn parse_optional_datetime(value: Option<&str>) -> Result<Option<DateTime<Utc>>, String> {
    let Some(value) = value else { return Ok(None) };
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return Ok(Some(dt.with_timezone(&Utc)));
    }
    let normalized = value.replace('T', " ");
    if let Ok(dt) = NaiveDateTime::parse_from_str(&normalized, "%Y-%m-%d %H:%M:%S") {
        return Ok(Some(Utc.from_utc_datetime(&dt)));
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(&normalized, "%Y-%m-%d %H:%M") {
        return Ok(Some(Utc.from_utc_datetime(&dt)));
    }
    if let Ok(date) = NaiveDate::parse_from_str(&normalized, "%Y-%m-%d") {
        if let Some(dt) = date.and_hms_opt(0, 0, 0) {
            return Ok(Some(Utc.from_utc_datetime(&dt)));
        }
    }
    Err("joined_at 格式不正确".to_string())
}
