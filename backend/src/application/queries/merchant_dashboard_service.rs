use std::collections::HashMap;

use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveDate, NaiveDateTime, TimeZone, Utc};
use sea_orm::{
    ColumnTrait, DatabaseBackend, DatabaseConnection, DatabaseTransaction, QueryFilter, Statement,
};
use sea_orm::entity::prelude::*;

use crate::infrastructure::entity::{contacts, orders, service_requests};
use crate::infrastructure::tenant::with_tenant_txn;
use shared::merchant_dashboard::{
    MerchantDashboardMeta, MerchantDashboardOverview, MerchantDashboardQuery,
    MerchantDashboardResponse, MerchantDashboardTodo, MerchantDashboardTrendPoint,
};

const DEFAULT_TIMEZONE: &str = "Asia/Shanghai";
const DEFAULT_GRANULARITY: &str = "day";
const MAX_RANGE_DAYS: i64 = 366;
const UPCOMING_HOURS: i64 = 24;

pub struct MerchantDashboardQueryService {
    db: DatabaseConnection,
    schema_name: String,
}

impl MerchantDashboardQueryService {
    pub fn new(db: DatabaseConnection, schema_name: String) -> Self {
        Self { db, schema_name }
    }

    pub async fn fetch_dashboard(
        &self,
        query: MerchantDashboardQuery,
    ) -> Result<MerchantDashboardResponse, String> {
        let resolved = resolve_time_range(&query)?;
        let start = resolved.start;
        let end = resolved.end;
        let meta = resolved.meta;
        let granularity = meta.granularity.clone();

        let (overview, trend, todos) =
            with_tenant_txn(&self.db, &self.schema_name, move |txn| {
                let granularity = granularity.clone();
                Box::pin(async move {
                    let new_contacts = contacts::Entity::find()
                        .filter(contacts::Column::InsertedAt.gte(start))
                        .filter(contacts::Column::InsertedAt.lte(end))
                        .count(txn)
                        .await
                        .map_err(|e| format!("query contacts count error: {}", e))?;

                    let service_requests_count = service_requests::Entity::find()
                        .filter(service_requests::Column::InsertedAt.gte(start))
                        .filter(service_requests::Column::InsertedAt.lte(end))
                        .count(txn)
                        .await
                        .map_err(|e| format!("query service requests count error: {}", e))?;

                    let orders_count = orders::Entity::find()
                        .filter(orders::Column::InsertedAt.gte(start))
                        .filter(orders::Column::InsertedAt.lte(end))
                        .count(txn)
                        .await
                        .map_err(|e| format!("query orders count error: {}", e))?;

                    let completed_orders_count = orders::Entity::find()
                        .filter(orders::Column::Status.eq("completed"))
                        .filter(orders::Column::InsertedAt.gte(start))
                        .filter(orders::Column::InsertedAt.lte(end))
                        .count(txn)
                        .await
                        .map_err(|e| format!("query completed orders error: {}", e))?;

                    let overview = MerchantDashboardOverview {
                        new_contacts,
                        service_requests: service_requests_count,
                        orders: orders_count,
                        completed_orders: completed_orders_count,
                    };

                    let contacts_map =
                        count_by_bucket(txn, "customers", "inserted_at", start, end, &granularity, None)
                            .await?;
                    let requests_map = count_by_bucket(
                        txn,
                        "service_requests",
                        "inserted_at",
                        start,
                        end,
                        &granularity,
                        None,
                    )
                    .await?;
                    let orders_map =
                        count_by_bucket(txn, "orders", "inserted_at", start, end, &granularity, None)
                            .await?;
                    let completed_map = count_by_bucket(
                        txn,
                        "orders",
                        "inserted_at",
                        start,
                        end,
                        &granularity,
                        Some("status = 'completed'"),
                    )
                    .await?;

                    let buckets = build_buckets(start, end, &granularity);
                    let trend = buckets
                        .into_iter()
                        .map(|bucket| MerchantDashboardTrendPoint {
                            bucket: bucket.clone(),
                            new_contacts: *contacts_map.get(&bucket).unwrap_or(&0),
                            service_requests: *requests_map.get(&bucket).unwrap_or(&0),
                            orders: *orders_map.get(&bucket).unwrap_or(&0),
                            completed_orders: *completed_map.get(&bucket).unwrap_or(&0),
                        })
                        .collect::<Vec<_>>();

                    let pending_requests = service_requests::Entity::find()
                        .filter(service_requests::Column::Status.eq("new"))
                        .filter(service_requests::Column::InsertedAt.gte(start))
                        .filter(service_requests::Column::InsertedAt.lte(end))
                        .count(txn)
                        .await
                        .map_err(|e| format!("query pending requests error: {}", e))?;

                    let pending_orders = orders::Entity::find()
                        .filter(orders::Column::Status.eq("pending"))
                        .filter(orders::Column::InsertedAt.gte(start))
                        .filter(orders::Column::InsertedAt.lte(end))
                        .count(txn)
                        .await
                        .map_err(|e| format!("query pending orders error: {}", e))?;

                    let now = Utc::now();
                    let upcoming_end = now + Duration::hours(UPCOMING_HOURS);
                    let upcoming_schedules = orders::Entity::find()
                        .filter(orders::Column::ScheduledStartAt.is_not_null())
                        .filter(orders::Column::ScheduledStartAt.gte(now))
                        .filter(orders::Column::ScheduledStartAt.lte(upcoming_end))
                        .filter(orders::Column::Status.is_in(vec![
                            "confirmed".to_string(),
                            "dispatching".to_string(),
                        ]))
                        .count(txn)
                        .await
                        .map_err(|e| format!("query upcoming schedules error: {}", e))?;

                    let todos = vec![
                        MerchantDashboardTodo {
                            key: "pending_requests".to_string(),
                            label: "待确认需求".to_string(),
                            count: pending_requests,
                        },
                        MerchantDashboardTodo {
                            key: "pending_orders".to_string(),
                            label: "待确认订单".to_string(),
                            count: pending_orders,
                        },
                        MerchantDashboardTodo {
                            key: "upcoming_schedules".to_string(),
                            label: "即将开始排班".to_string(),
                            count: upcoming_schedules,
                        },
                    ];

                    Ok((overview, trend, todos))
                })
            })
            .await?;

        Ok(MerchantDashboardResponse {
            meta,
            overview,
            trend,
            todos,
        })
    }
}

struct ResolvedRange {
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    meta: MerchantDashboardMeta,
}

fn resolve_time_range(query: &MerchantDashboardQuery) -> Result<ResolvedRange, String> {
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

    let meta = MerchantDashboardMeta {
        start: format_bucket(start, &granularity),
        end: format_bucket(end, &granularity),
        timezone,
        granularity,
    };

    Ok(ResolvedRange { start, end, meta })
}

async fn count_by_bucket(
    txn: &DatabaseTransaction,
    table: &str,
    time_column: &str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    granularity: &str,
    extra_condition: Option<&str>,
) -> Result<HashMap<String, u64>, String> {
    let granularity = normalize_granularity(granularity);
    let mut sql = format!(
        "SELECT date_trunc('{granularity}', {time_column}) AS bucket, COUNT(*) AS count
         FROM {table}
         WHERE {time_column} >= $1 AND {time_column} <= $2"
    );
    if let Some(condition) = extra_condition {
        sql.push_str(" AND ");
        sql.push_str(condition);
    }
    sql.push_str(" GROUP BY bucket ORDER BY bucket");

    let stmt = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        sql,
        vec![start.into(), end.into()],
    );
    let rows = txn
        .query_all(stmt)
        .await
        .map_err(|e| format!("query {table} trend error: {}", e))?;
    let mut map = HashMap::new();
    for row in rows {
        let bucket: DateTime<Utc> = row
            .try_get("", "bucket")
            .map_err(|e| format!("read bucket error: {}", e))?;
        let count: i64 = row
            .try_get("", "count")
            .map_err(|e| format!("read count error: {}", e))?;
        map.insert(format_bucket(bucket, &granularity), count.max(0) as u64);
    }
    Ok(map)
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
        let date =
            NaiveDate::parse_from_str(&trimmed, "%Y-%m-%d").map_err(|_| "无效日期格式".to_string())?;
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
        "Asia/Shanghai" | "UTC+8" | "UTC+08:00" => FixedOffset::east_opt(8 * 3600)
            .ok_or_else(|| "invalid timezone".to_string()),
        "UTC" | "UTC+0" => FixedOffset::east_opt(0).ok_or_else(|| "invalid timezone".to_string()),
        _ => FixedOffset::east_opt(8 * 3600).ok_or_else(|| "invalid timezone".to_string()),
    }
}
