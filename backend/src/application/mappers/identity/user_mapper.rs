use crate::application::utils::parse_utc_time_to_string;
use crate::domain::identity::user::{EmploymentStatus, HealthStatus, Status, User as DomainUser};
use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use shared::user::{CreateUserRequest, User};

impl From<DomainUser> for User {
    fn from(user: DomainUser) -> Self {
        let status = match user.status {
            Status::Inactive => "inactive".to_string(),
            Status::Active => "active".to_string(),
        };

        Self {
            uuid: user.uuid,
            user_name: user.user_name,
            email: user.email,
            phone_number: user.phone_number,
            merchant_uuid: user.merchant_uuid,
            role: user.role,
            is_admin: user.is_admin,
            status,
            employment_status: user.employment_status.as_str().to_string(),
            skills: user.skills,
            service_areas: user.service_areas,
            training_records: user.training_records,
            certificates: user.certificates,
            health_status: user.health_status.as_str().to_string(),
            employee_note: user.employee_note,
            joined_at: user.joined_at.map(parse_utc_time_to_string),
            completed_service_count: user.completed_service_count,
            feedback_count: user.feedback_count,
            average_rating: user.average_rating,
            after_sales_case_count: user.after_sales_case_count,
            complaint_case_count: user.complaint_case_count,
            refund_case_count: user.refund_case_count,
            rework_count: user.rework_count,
            avatar_url: user.avatar_url,
            last_login_at: user.last_login_at.map(parse_utc_time_to_string),
            email_verified_at: user.email_verified_at.map(parse_utc_time_to_string),
            inserted_at: parse_utc_time_to_string(user.inserted_at),
            updated_at: parse_utc_time_to_string(user.updated_at),
        }
    }
}

impl From<CreateUserRequest> for DomainUser {
    fn from(request: CreateUserRequest) -> Self {
        let mut user = DomainUser::new(
            request.user_name,
            request.password,
            request.email,
            request.phone_number,
        );
        if let Some(role) = request.role {
            user.set_role(role);
        }
        if let Some(merchant_uuid) = request.merchant_uuid {
            user.set_merchant_uuid(merchant_uuid);
        }
        if let Some(avatar_url) = request.avatar_url {
            user.avatar_url = Some(avatar_url);
        }
        let employment_status = request
            .employment_status
            .as_deref()
            .and_then(|value| EmploymentStatus::parse(value).ok())
            .unwrap_or(EmploymentStatus::Active);
        let health_status = request
            .health_status
            .as_deref()
            .and_then(|value| HealthStatus::parse(value).ok())
            .unwrap_or(HealthStatus::Healthy);
        let _ = user.update_employee_profile(
            Some(employment_status),
            request.skills,
            request.service_areas,
            request.training_records,
            request.certificates,
            Some(health_status),
            request.employee_note,
            parse_datetime(request.joined_at.as_deref()),
        );
        user
    }
}

fn parse_datetime(value: Option<&str>) -> Option<DateTime<Utc>> {
    let value = value?.trim();
    if value.is_empty() {
        return None;
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return Some(dt.with_timezone(&Utc));
    }

    let normalized = value.replace('T', " ");
    if let Ok(dt) = NaiveDateTime::parse_from_str(&normalized, "%Y-%m-%d %H:%M:%S") {
        return Some(Utc.from_utc_datetime(&dt));
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(&normalized, "%Y-%m-%d %H:%M") {
        return Some(Utc.from_utc_datetime(&dt));
    }
    if let Ok(date) = NaiveDate::parse_from_str(&normalized, "%Y-%m-%d") {
        if let Some(dt) = date.and_hms_opt(0, 0, 0) {
            return Some(Utc.from_utc_datetime(&dt));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_employee_extension_fields_to_shared_user() {
        let mut user = DomainUser::new(
            "阿姨".to_string(),
            "hashed".to_string(),
            Some("worker@example.com".to_string()),
            Some("13800138000".to_string()),
        );
        user.set_role("user".to_string());
        user.update_employee_profile(
            Some(EmploymentStatus::OnLeave),
            vec!["保洁".to_string(), "深度保洁".to_string()],
            vec!["朝阳".to_string()],
            vec!["岗前培训".to_string()],
            vec!["养老护理证".to_string()],
            Some(HealthStatus::Attention),
            Some("临时请假".to_string()),
            Some(Utc.with_ymd_and_hms(2026, 4, 1, 9, 0, 0).unwrap()),
        )
        .expect("employee profile should update");

        let shared: User = user.into();
        assert_eq!(shared.employment_status, "on_leave");
        assert_eq!(
            shared.skills,
            vec!["保洁".to_string(), "深度保洁".to_string()]
        );
        assert_eq!(shared.service_areas, vec!["朝阳".to_string()]);
        assert_eq!(shared.training_records, vec!["岗前培训".to_string()]);
        assert_eq!(shared.certificates, vec!["养老护理证".to_string()]);
        assert_eq!(shared.health_status, "attention");
        assert_eq!(shared.employee_note.as_deref(), Some("临时请假"));
        assert!(shared.joined_at.is_some());
    }
}
