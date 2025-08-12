use chrono::prelude::{DateTime, FixedOffset, NaiveDateTime, TimeZone, Utc};

pub fn naive_to_utc(naive: NaiveDateTime) -> DateTime<Utc> {
    Utc.from_utc_datetime(&naive)
}

pub fn utc_to_naive(time: DateTime<Utc>) -> NaiveDateTime {
    let beijing_offset = FixedOffset::east_opt(8 * 3600).unwrap();
    time.with_timezone(&beijing_offset).naive_local()
}

pub fn parse_date_time_to_string(time: NaiveDateTime) -> String {
    // 1. 创建北京时间偏移量 (+08:00)
    let beijing_offset = FixedOffset::east_opt(8 * 3600).unwrap();

    // 2. 将NaiveDateTime转换为DateTime<FixedOffset>
    let beijing_time = beijing_offset.from_utc_datetime(&time);

    // 3. 格式化为字符串
    beijing_time.format("%Y-%m-%d %H:%M:%S").to_string()
}
