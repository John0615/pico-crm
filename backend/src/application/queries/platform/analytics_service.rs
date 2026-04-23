use std::collections::HashMap;

use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveDate, NaiveDateTime, TimeZone, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseBackend, DatabaseConnection, DatabaseTransaction,
    QueryFilter, Statement, TransactionTrait,
};

use crate::infrastructure::entity::{merchant, users};
use shared::analytics::{
    AnalyticsBreakdownItem, AnalyticsBreakdownResponse, AnalyticsMeta, AnalyticsOverview,
    AnalyticsOverviewResponse, AnalyticsQuery, AnalyticsTrendPoint, AnalyticsTrendResponse,
};

const DEFAULT_TIMEZONE: &str = "Asia/Shanghai";
const DEFAULT_GRANULARITY: &str = "day";
const MAX_RANGE_DAYS: i64 = 366;

pub struct AnalyticsQueryService {
    db: DatabaseConnection,
}

impl AnalyticsQueryService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn overview(
        &self,
        query: AnalyticsQuery,
    ) -> Result<AnalyticsOverviewResponse, String> {
        let resolved = resolve_time_range(&query)?;

        let (total_merchants, active_merchants, new_merchants) =
            self.merchant_counts(resolved.start, resolved.end).await?;

        let total_users = self.count_users(None, None, None).await?;
        let active_users = self.count_users(None, None, Some("active")).await?;
        let new_users = self
            .count_users(Some(resolved.start), Some(resolved.end), None)
            .await?;

        Ok(AnalyticsOverviewResponse {
            meta: resolved.meta,
            overview: AnalyticsOverview {
                total_merchants,
                active_merchants,
                total_users,
                active_users,
                new_merchants,
                new_users,
            },
        })
    }

    pub async fn trends(&self, query: AnalyticsQuery) -> Result<AnalyticsTrendResponse, String> {
        let resolved = resolve_time_range(&query)?;
        let buckets = build_buckets(resolved.start, resolved.end, &resolved.meta.granularity);

        let merchant_map = self
            .count_merchants_by_bucket(resolved.start, resolved.end, &resolved.meta.granularity)
            .await?;
        let user_map = self
            .count_users_by_bucket(resolved.start, resolved.end, &resolved.meta.granularity)
            .await?;

        let series = buckets
            .into_iter()
            .map(|bucket| AnalyticsTrendPoint {
                new_merchants: *merchant_map.get(&bucket).unwrap_or(&0),
                new_users: *user_map.get(&bucket).unwrap_or(&0),
                bucket,
            })
            .collect();

        Ok(AnalyticsTrendResponse {
            meta: resolved.meta,
            series,
        })
    }

    pub async fn breakdown(
        &self,
        query: AnalyticsQuery,
    ) -> Result<AnalyticsBreakdownResponse, String> {
        let resolved = resolve_time_range(&query)?;
        let dimension = query
            .dimension
            .clone()
            .unwrap_or_else(|| "merchant_status".to_string());

        let items = match dimension.as_str() {
            "plan_type" => {
                self.breakdown_merchants("plan_type", resolved.start, resolved.end)
                    .await?
            }
            "merchant_type" => {
                self.breakdown_merchants("merchant_type", resolved.start, resolved.end)
                    .await?
            }
            _ => {
                self.breakdown_merchants("status", resolved.start, resolved.end)
                    .await?
            }
        };

        Ok(AnalyticsBreakdownResponse {
            meta: resolved.meta,
            dimension,
            items,
        })
    }

    async fn merchant_counts(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<(u64, u64, u64), String> {
        let (total, active, new_merchants) = self
            .with_public_txn(|txn| {
                Box::pin(async move {
                    let total = merchant::Entity::find()
                        .count(txn)
                        .await
                        .map_err(|e| format!("count merchants error: {}", e))?;
                    let active = merchant::Entity::find()
                        .filter(merchant::Column::Status.eq("active"))
                        .count(txn)
                        .await
                        .map_err(|e| format!("count active merchants error: {}", e))?;

                    let new_merchants = merchant::Entity::find()
                        .filter(merchant::Column::CreatedAt.gte(start))
                        .filter(merchant::Column::CreatedAt.lte(end))
                        .count(txn)
                        .await
                        .map_err(|e| format!("count new merchants error: {}", e))?;

                    Ok((total, active, new_merchants))
                })
            })
            .await?;

        Ok((total, active, new_merchants))
    }

    async fn count_users(
        &self,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
        status: Option<&str>,
    ) -> Result<u64, String> {
        let status = status.map(|value| value.to_string());
        self.with_public_txn(|txn| {
            let status = status.clone();
            Box::pin(async move {
                let mut select = users::Entity::find();
                if let Some(start) = start {
                    select = select.filter(users::Column::InsertedAt.gte(start));
                }
                if let Some(end) = end {
                    select = select.filter(users::Column::InsertedAt.lte(end));
                }
                if let Some(status) = status {
                    select = select.filter(users::Column::Status.eq(status));
                }
                select
                    .count(txn)
                    .await
                    .map_err(|e| format!("count users error: {}", e))
            })
        })
        .await
    }

    async fn count_merchants_by_bucket(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        granularity: &str,
    ) -> Result<HashMap<String, u64>, String> {
        let granularity = normalize_granularity(granularity);
        let sql = format!(
            "SELECT date_trunc('{granularity}', created_at) AS bucket, COUNT(*) AS count
             FROM public.merchant
             WHERE created_at >= $1 AND created_at <= $2
             GROUP BY bucket
             ORDER BY bucket"
        );
        let stmt = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            sql,
            vec![start.into(), end.into()],
        );
        let rows = self
            .db
            .query_all(stmt)
            .await
            .map_err(|e| format!("query merchant trend error: {}", e))?;
        let mut map = HashMap::new();
        for row in rows {
            let bucket: DateTime<Utc> = row
                .try_get("", "bucket")
                .map_err(|e| format!("read bucket error: {}", e))?;
            let count: i64 = row
                .try_get("", "count")
                .map_err(|e| format!("read bucket count error: {}", e))?;
            map.insert(format_bucket(bucket, &granularity), count.max(0) as u64);
        }
        Ok(map)
    }

    async fn count_users_by_bucket(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        granularity: &str,
    ) -> Result<HashMap<String, u64>, String> {
        let granularity = normalize_granularity(granularity);
        let sql = format!(
            "SELECT date_trunc('{granularity}', inserted_at) AS bucket, COUNT(*) AS count
             FROM public.users
             WHERE inserted_at >= $1 AND inserted_at <= $2
             GROUP BY bucket"
        );
        let stmt = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            sql,
            vec![start.into(), end.into()],
        );
        let rows = self
            .db
            .query_all(stmt)
            .await
            .map_err(|e| format!("query user trend error: {}", e))?;
        let mut map: HashMap<String, u64> = HashMap::new();
        for row in rows {
            let bucket: DateTime<Utc> = row
                .try_get("", "bucket")
                .map_err(|e| format!("read user bucket error: {}", e))?;
            let count: i64 = row
                .try_get("", "count")
                .map_err(|e| format!("read user count error: {}", e))?;
            let key = format_bucket(bucket, &granularity);
            let entry = map.entry(key).or_insert(0);
            *entry += count.max(0) as u64;
        }
        Ok(map)
    }

    async fn breakdown_merchants(
        &self,
        column: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<AnalyticsBreakdownItem>, String> {
        let sql = format!(
            "SELECT COALESCE({column}, 'unknown') AS label, COUNT(*) AS count
             FROM public.merchant
             WHERE created_at >= $1 AND created_at <= $2
             GROUP BY label
             ORDER BY count DESC"
        );
        let stmt = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            sql,
            vec![start.into(), end.into()],
        );
        let rows = self
            .db
            .query_all(stmt)
            .await
            .map_err(|e| format!("query breakdown error: {}", e))?;
        let mut items = Vec::new();
        for row in rows {
            let label: String = row
                .try_get("", "label")
                .map_err(|e| format!("read breakdown label error: {}", e))?;
            let count: i64 = row
                .try_get("", "count")
                .map_err(|e| format!("read breakdown count error: {}", e))?;
            items.push(AnalyticsBreakdownItem {
                label,
                count: count.max(0) as u64,
            });
        }
        Ok(items)
    }

    async fn with_public_txn<T, F>(&self, f: F) -> Result<T, String>
    where
        F: for<'a> FnOnce(
            &'a DatabaseTransaction,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<T, String>> + Send + 'a>,
        >,
    {
        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| format!("begin transaction error: {}", e))?;
        let stmt = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT set_config('search_path', $1, true)",
            vec!["public".to_string().into()],
        );
        txn.execute(stmt)
            .await
            .map_err(|e| format!("set search_path error: {}", e))?;

        let result = f(&txn).await?;
        txn.commit()
            .await
            .map_err(|e| format!("commit transaction error: {}", e))?;
        Ok(result)
    }
}

struct ResolvedRange {
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    meta: AnalyticsMeta,
}

fn resolve_time_range(query: &AnalyticsQuery) -> Result<ResolvedRange, String> {
    let timezone = query
        .timezone
        .clone()
        .unwrap_or_else(|| DEFAULT_TIMEZONE.to_string());
    let granularity =
        normalize_granularity(query.granularity.as_deref().unwrap_or(DEFAULT_GRANULARITY));
    let now = Utc::now();

    let (start, end) = if let Some(preset) = query.preset.as_deref() {
        match preset {
            "today" => {
                let start = start_of_day(now, &timezone)?;
                (start, now)
            }
            "last_30_days" => (now - Duration::days(30), now),
            "last_7_days" | _ => (now - Duration::days(7), now),
        }
    } else if query.start.is_some() || query.end.is_some() {
        let start = query
            .start
            .as_deref()
            .map(|raw| parse_to_utc(raw, false, &timezone))
            .transpose()?
            .unwrap_or_else(|| now - Duration::days(7));
        let end = query
            .end
            .as_deref()
            .map(|raw| parse_to_utc(raw, true, &timezone))
            .transpose()?
            .unwrap_or(now);
        (start, end)
    } else {
        (now - Duration::days(7), now)
    };

    if (end - start).num_days() > MAX_RANGE_DAYS {
        return Err(format!("时间范围不能超过 {} 天", MAX_RANGE_DAYS));
    }

    let meta = AnalyticsMeta {
        start: format_bucket(start, &granularity),
        end: format_bucket(end, &granularity),
        timezone,
        granularity,
    };

    Ok(ResolvedRange { start, end, meta })
}

fn build_buckets(start: DateTime<Utc>, end: DateTime<Utc>, granularity: &str) -> Vec<String> {
    let granularity = normalize_granularity(granularity);
    let mut buckets = Vec::new();
    let mut cursor = truncate_to_granularity(start, &granularity);
    let end_cursor = truncate_to_granularity(end, &granularity);
    while cursor <= end_cursor {
        buckets.push(format_bucket(cursor, &granularity));
        cursor = match granularity.as_str() {
            "week" => cursor + Duration::weeks(1),
            "month" => add_month(cursor),
            _ => cursor + Duration::days(1),
        };
    }
    buckets
}

fn normalize_granularity(input: &str) -> String {
    match input {
        "week" | "month" | "day" => input.to_string(),
        _ => DEFAULT_GRANULARITY.to_string(),
    }
}

fn format_bucket(dt: DateTime<Utc>, granularity: &str) -> String {
    match granularity {
        "month" => dt.format("%Y-%m").to_string(),
        _ => dt.format("%Y-%m-%d").to_string(),
    }
}

fn truncate_to_granularity(dt: DateTime<Utc>, granularity: &str) -> DateTime<Utc> {
    match granularity {
        "month" => {
            let date = NaiveDate::from_ymd_opt(dt.year(), dt.month(), 1)
                .unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
            Utc.from_utc_datetime(&date.and_hms_opt(0, 0, 0).unwrap())
        }
        "week" => {
            let weekday = dt.weekday().num_days_from_monday() as i64;
            let date = dt.date_naive() - Duration::days(weekday);
            Utc.from_utc_datetime(&date.and_hms_opt(0, 0, 0).unwrap())
        }
        _ => Utc.from_utc_datetime(&dt.date_naive().and_hms_opt(0, 0, 0).unwrap()),
    }
}

fn add_month(dt: DateTime<Utc>) -> DateTime<Utc> {
    let mut year = dt.year();
    let mut month = dt.month() + 1;
    if month == 13 {
        month = 1;
        year += 1;
    }
    let date = NaiveDate::from_ymd_opt(year, month, 1)
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
    Utc.from_utc_datetime(&date.and_hms_opt(0, 0, 0).unwrap())
}

fn start_of_day(now: DateTime<Utc>, timezone: &str) -> Result<DateTime<Utc>, String> {
    let offset = timezone_offset(timezone)?;
    let local = now.with_timezone(&offset);
    let date = local.date_naive();
    let local_start = offset
        .from_local_datetime(&date.and_hms_opt(0, 0, 0).unwrap())
        .single()
        .ok_or_else(|| "invalid start of day".to_string())?;
    Ok(local_start.with_timezone(&Utc))
}

fn parse_to_utc(raw: &str, is_end: bool, timezone: &str) -> Result<DateTime<Utc>, String> {
    let offset = timezone_offset(timezone)?;
    let trimmed = raw.trim().replace('T', " ");
    if trimmed.len() == 10 {
        let date = NaiveDate::parse_from_str(&trimmed, "%Y-%m-%d")
            .map_err(|_| "无效日期格式".to_string())?;
        let time = if is_end { (23, 59, 59) } else { (0, 0, 0) };
        let local = offset
            .from_local_datetime(&date.and_hms_opt(time.0, time.1, time.2).unwrap())
            .single()
            .ok_or_else(|| "invalid date".to_string())?;
        return Ok(local.with_timezone(&Utc));
    }

    let naive = NaiveDateTime::parse_from_str(&trimmed, "%Y-%m-%d %H:%M:%S")
        .map_err(|_| "无效日期时间格式".to_string())?;
    let local = offset
        .from_local_datetime(&naive)
        .single()
        .ok_or_else(|| "invalid date time".to_string())?;
    Ok(local.with_timezone(&Utc))
}

fn timezone_offset(timezone: &str) -> Result<FixedOffset, String> {
    match timezone {
        "Asia/Shanghai" | "UTC+8" | "UTC+08:00" => {
            FixedOffset::east_opt(8 * 3600).ok_or_else(|| "invalid timezone".to_string())
        }
        "UTC" | "UTC+0" => FixedOffset::east_opt(0).ok_or_else(|| "invalid timezone".to_string()),
        _ => FixedOffset::east_opt(8 * 3600).ok_or_else(|| "invalid timezone".to_string()),
    }
}
