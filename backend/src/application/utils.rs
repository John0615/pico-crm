use chrono::prelude::{DateTime, FixedOffset, NaiveDateTime, TimeZone, Utc};

pub fn naive_to_utc(naive: NaiveDateTime) -> DateTime<Utc> {
    Utc.from_utc_datetime(&naive)
}

pub fn utc_to_naive(time: DateTime<Utc>) -> NaiveDateTime {
    let beijing_offset = FixedOffset::east_opt(8 * 3600)
        .expect("创建北京时间偏移失败: east_opt(8 * 3600) 返回 None");
    time.with_timezone(&beijing_offset).naive_local()
}

pub fn parse_utc_time_to_string(time: DateTime<Utc>) -> String {
    let beijing_offset = FixedOffset::east_opt(8 * 3600)
        .expect("创建北京时间偏移失败: east_opt(8 * 3600) 返回 None");
    let beijing_time = time.with_timezone(&beijing_offset);

    beijing_time.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn parse_string_to_utc_time(time_str: &str) -> DateTime<Utc> {
    let beijing_offset = FixedOffset::east_opt(8 * 3600).unwrap();
    DateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M:%S")
        .map(|dt| dt.with_timezone(&beijing_offset).with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}
