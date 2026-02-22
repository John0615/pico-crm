use chrono::{DateTime, Utc};

use crate::domain::models::order::OrderStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleStatus {
    Planned,
    InService,
    Done,
    Cancelled,
}

impl ScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ScheduleStatus::Planned => "planned",
            ScheduleStatus::InService => "in_service",
            ScheduleStatus::Done => "done",
            ScheduleStatus::Cancelled => "cancelled",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "planned" => Ok(ScheduleStatus::Planned),
            "in_service" => Ok(ScheduleStatus::InService),
            "done" => Ok(ScheduleStatus::Done),
            "cancelled" => Ok(ScheduleStatus::Cancelled),
            _ => Err(format!("Invalid schedule status: {}", value)),
        }
    }

    pub fn from_order_status(status: &OrderStatus) -> Self {
        match status {
            OrderStatus::Pending | OrderStatus::Confirmed | OrderStatus::Dispatching => {
                ScheduleStatus::Planned
            }
            OrderStatus::InService => ScheduleStatus::InService,
            OrderStatus::Completed => ScheduleStatus::Done,
            OrderStatus::Cancelled => ScheduleStatus::Cancelled,
        }
    }

    pub fn order_statuses(&self) -> &'static [&'static str] {
        match self {
            ScheduleStatus::Planned => &["pending", "confirmed", "dispatching"],
            ScheduleStatus::InService => &["in_service"],
            ScheduleStatus::Done => &["completed"],
            ScheduleStatus::Cancelled => &["cancelled"],
        }
    }

    pub fn allows_assignment_update(&self) -> bool {
        matches!(self, ScheduleStatus::Planned)
    }

    pub fn allows_cancel(&self) -> bool {
        matches!(self, ScheduleStatus::Planned | ScheduleStatus::InService)
    }

    pub fn target_order_status(&self) -> Option<OrderStatus> {
        match self {
            ScheduleStatus::Planned => None,
            ScheduleStatus::InService => Some(OrderStatus::InService),
            ScheduleStatus::Done => Some(OrderStatus::Completed),
            ScheduleStatus::Cancelled => Some(OrderStatus::Cancelled),
        }
    }

    pub fn validate_transition(
        current: ScheduleStatus,
        target: ScheduleStatus,
    ) -> Result<(), String> {
        let allowed = matches!(
            (current, target),
            (ScheduleStatus::Planned, ScheduleStatus::InService)
                | (ScheduleStatus::Planned, ScheduleStatus::Cancelled)
                | (ScheduleStatus::InService, ScheduleStatus::Done)
                | (ScheduleStatus::InService, ScheduleStatus::Cancelled)
        );
        if allowed {
            Ok(())
        } else {
            Err(format!(
                "Invalid schedule status transition: {} -> {}",
                current.as_str(),
                target.as_str()
            ))
        }
    }
}

pub fn validate_time_window(start: DateTime<Utc>, end: DateTime<Utc>) -> Result<(), String> {
    if end <= start {
        return Err("scheduled end must be after start".to_string());
    }
    Ok(())
}

pub fn is_overlapping_window(
    start_a: DateTime<Utc>,
    end_a: DateTime<Utc>,
    start_b: DateTime<Utc>,
    end_b: DateTime<Utc>,
) -> bool {
    start_a < end_b && end_a > start_b
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn schedule_status_mapping_from_order() {
        assert_eq!(
            ScheduleStatus::from_order_status(&OrderStatus::Pending),
            ScheduleStatus::Planned
        );
        assert_eq!(
            ScheduleStatus::from_order_status(&OrderStatus::Confirmed),
            ScheduleStatus::Planned
        );
        assert_eq!(
            ScheduleStatus::from_order_status(&OrderStatus::Dispatching),
            ScheduleStatus::Planned
        );
        assert_eq!(
            ScheduleStatus::from_order_status(&OrderStatus::InService),
            ScheduleStatus::InService
        );
        assert_eq!(
            ScheduleStatus::from_order_status(&OrderStatus::Completed),
            ScheduleStatus::Done
        );
        assert_eq!(
            ScheduleStatus::from_order_status(&OrderStatus::Cancelled),
            ScheduleStatus::Cancelled
        );
    }

    #[test]
    fn schedule_status_transition_rules() {
        assert!(ScheduleStatus::validate_transition(
            ScheduleStatus::Planned,
            ScheduleStatus::InService
        )
        .is_ok());
        assert!(ScheduleStatus::validate_transition(
            ScheduleStatus::Planned,
            ScheduleStatus::Cancelled
        )
        .is_ok());
        assert!(ScheduleStatus::validate_transition(
            ScheduleStatus::InService,
            ScheduleStatus::Done
        )
        .is_ok());
        assert!(ScheduleStatus::validate_transition(
            ScheduleStatus::InService,
            ScheduleStatus::Cancelled
        )
        .is_ok());
        assert!(ScheduleStatus::validate_transition(
            ScheduleStatus::Done,
            ScheduleStatus::Cancelled
        )
        .is_err());
    }

    #[test]
    fn time_window_validation_requires_end_after_start() {
        let start = Utc.with_ymd_and_hms(2026, 2, 22, 9, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 2, 22, 10, 0, 0).unwrap();
        assert!(validate_time_window(start, end).is_ok());
        assert!(validate_time_window(end, end).is_err());
    }

    #[test]
    fn overlap_detection_matches_expectations() {
        let start_a = Utc.with_ymd_and_hms(2026, 2, 22, 9, 0, 0).unwrap();
        let end_a = Utc.with_ymd_and_hms(2026, 2, 22, 10, 0, 0).unwrap();
        let start_b = Utc.with_ymd_and_hms(2026, 2, 22, 9, 30, 0).unwrap();
        let end_b = Utc.with_ymd_and_hms(2026, 2, 22, 10, 30, 0).unwrap();
        let start_c = Utc.with_ymd_and_hms(2026, 2, 22, 10, 0, 0).unwrap();
        let end_c = Utc.with_ymd_and_hms(2026, 2, 22, 11, 0, 0).unwrap();

        assert!(is_overlapping_window(start_a, end_a, start_b, end_b));
        assert!(!is_overlapping_window(start_a, end_a, start_c, end_c));
    }
}
