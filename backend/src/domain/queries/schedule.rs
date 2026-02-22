use shared::schedule::ScheduleQuery as ScheduleQueryParams;

pub trait ScheduleQuery: Send + Sync {
    type Result: std::fmt::Debug + Send + Sync;

    fn list_schedules(
        &self,
        query: ScheduleQueryParams,
    ) -> impl std::future::Future<Output = Result<(Vec<Self::Result>, u64), String>> + Send;

    fn get_schedule(
        &self,
        uuid: String,
    ) -> impl std::future::Future<Output = Result<Option<Self::Result>, String>> + Send;
}
