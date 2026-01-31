use crate::domain::models::user::User as DomainUser;
use crate::domain::repositories::user::UserRepository;
use shared::user::{CreateUserRequest, User};

pub struct UserCommandService<R: UserRepository> {
    user_repository: R,
}

impl<R: UserRepository> UserCommandService<R> {
    pub fn new(user_repository: R) -> Self {
        Self { user_repository }
    }

    /// 创建新用户
    pub async fn create_user(&self, request: CreateUserRequest) -> Result<User, String> {
        // 检查用户名是否已存在
        if let Ok(Some(_)) = self
            .user_repository
            .find_user_by_username(&request.user_name)
            .await
        {
            return Err("用户名已存在".to_string());
        }

        // 检查邮箱是否已存在（如果提供了邮箱）
        if let Some(ref email) = request.email {
            if let Ok(Some(_)) = self.user_repository.find_user_by_email(email).await {
                return Err("邮箱已存在".to_string());
            }
        }

        // 检查手机号码是否已存在（如果提供了手机号码）
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
            avatar_url,
        } = request;

        // 创建用户实体并生成密码哈希
        let password_hash =
            DomainUser::hash_password(&password).map_err(|e| format!("密码加密失败: {}", e))?;
        let mut user = DomainUser::new(user_name, password_hash, email, phone_number);
        if let Some(avatar_url) = avatar_url {
            user.avatar_url = Some(avatar_url);
        }

        // 保存到数据库
        let created_user = self
            .user_repository
            .create_user(user)
            .await
            .map_err(|e| format!("创建用户失败: {}", e))?;

        Ok(created_user.into())
    }

    /// 更新用户信息
    pub async fn update_user(
        &self,
        uuid: &str,
        request: CreateUserRequest,
    ) -> Result<User, String> {
        // 查找现有用户
        let mut user = self
            .user_repository
            .find_user_by_uuid(uuid)
            .await
            .map_err(|e| format!("查找用户失败: {}", e))?
            .ok_or("用户不存在")?;

        // 检查用户名是否被其他用户使用
        if let Ok(Some(existing_user)) = self
            .user_repository
            .find_user_by_username(&request.user_name)
            .await
        {
            if existing_user.uuid != user.uuid {
                return Err("用户名已被其他用户使用".to_string());
            }
        }

        // 检查邮箱是否被其他用户使用（如果提供了邮箱）
        if let Some(ref email) = request.email {
            if let Ok(Some(existing_user)) = self.user_repository.find_user_by_email(email).await {
                if existing_user.uuid != user.uuid {
                    return Err("邮箱已被其他用户使用".to_string());
                }
            }
        }

        // 检查手机号码是否被其他用户使用（如果提供了手机号码）
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

        // 更新用户信息
        user.update_info(
            Some(request.user_name),
            request.email,
            request.phone_number,
            request.avatar_url, // 更新头像URL
        );

        // 如果提供了新密码，更新密码
        if !request.password.is_empty() {
            let password_hash = DomainUser::hash_password(&request.password)
                .map_err(|e| format!("密码加密失败: {}", e))?;
            user.change_password(password_hash);
        }

        // 保存到数据库
        let updated_user = self
            .user_repository
            .update_user(user)
            .await
            .map_err(|e| format!("更新用户失败: {}", e))?;

        Ok(updated_user.into())
    }

    /// 根据UUID获取用户
    pub async fn get_user_by_uuid(&self, uuid: &str) -> Result<Option<User>, String> {
        let user = self
            .user_repository
            .find_user_by_uuid(uuid)
            .await
            .map_err(|e| format!("查找用户失败: {}", e))?;

        Ok(user.map(|domain_user| domain_user.into()))
    }

    /// 删除用户
    pub async fn delete_user(&self, uuid: &str) -> Result<(), String> {
        // 检查用户是否存在
        let _user = self
            .user_repository
            .find_user_by_uuid(uuid)
            .await
            .map_err(|e| format!("查找用户失败: {}", e))?
            .ok_or("用户不存在")?;

        // 删除用户
        self.user_repository
            .delete_user(uuid.to_string())
            .await
            .map_err(|e| format!("删除用户失败: {}", e))?;

        Ok(())
    }

    /// 激活用户
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

    /// 禁用用户
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
