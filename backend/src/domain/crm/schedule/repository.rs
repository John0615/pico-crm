use chrono::{DateTime, Utc};

use super::model::{ScheduleAssignment, ScheduleStatus};

pub trait ScheduleRepository: Send + Sync {
    fn find_by_order(
        &self,
        order_id: String,
    ) -> impl std::future::Future<Output = Result<Option<ScheduleAssignment>, String>> + Send;

    fn create_assignment(
        &self,
        assignment: ScheduleAssignment,
    ) -> impl std::future::Future<Output = Result<ScheduleAssignment, String>> + Send;

    fn update_assignment(
        &self,
        order_id: String,
        assigned_user_uuid: String,
        start_at: DateTime<Utc>,
        end_at: DateTime<Utc>,
        notes: Option<String>,
    ) -> impl std::future::Future<Output = Result<ScheduleAssignment, String>> + Send;

    fn delete_by_order(
        &self,
        order_id: String,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send;

    fn update_status(
        &self,
        order_id: String,
        status: ScheduleStatus,
    ) -> impl std::future::Future<Output = Result<Option<ScheduleAssignment>, String>> + Send;

    fn find_conflict(
        &self,
        assigned_user_uuid: String,
        start_at: DateTime<Utc>,
        end_at: DateTime<Utc>,
        exclude_order_id: Option<String>,
    ) -> impl std::future::Future<Output = Result<Option<ScheduleAssignment>, String>> + Send;
}
