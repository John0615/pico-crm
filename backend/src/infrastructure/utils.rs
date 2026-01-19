use chrono::prelude::{DateTime, FixedOffset, Utc};

/// 将 UTC DateTime 转换为本地时间字符串显示
pub fn parse_date_time_to_string(time: DateTime<Utc>) -> String {
    // 1. 创建北京时间偏移量 (+08:00)
    let beijing_offset = FixedOffset::east_opt(8 * 3600).unwrap();

    // 2. 将UTC时间转换为北京时间
    let beijing_time = time.with_timezone(&beijing_offset);

    // 3. 格式化为字符串
    beijing_time.format("%Y-%m-%d %H:%M:%S").to_string()
}
