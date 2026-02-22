use crate::domain::queries::schedule::ScheduleQuery as DomainScheduleQuery;
use shared::schedule::{Schedule, ScheduleQuery};
use shared::ListResult;

pub struct ScheduleQueryService<R: DomainScheduleQuery> {
    query: R,
}

impl<R: DomainScheduleQuery<Result = Schedule>> ScheduleQueryService<R> {
    pub fn new(query: R) -> Self {
        Self { query }
    }

    pub async fn fetch_schedules(
        &self,
        params: ScheduleQuery,
    ) -> Result<ListResult<Schedule>, String> {
        let (items, total) = self.query.list_schedules(params).await?;
        Ok(ListResult { items, total })
    }

    pub async fn fetch_schedule(&self, uuid: String) -> Result<Option<Schedule>, String> {
        self.query.get_schedule(uuid).await
    }
}
