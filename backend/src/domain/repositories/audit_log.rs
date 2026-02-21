use crate::domain::models::audit_log::AuditLogCreate;

pub trait AuditLogRepository: Send + Sync {
    fn create_log(
        &self,
        log: AuditLogCreate,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send;
}
