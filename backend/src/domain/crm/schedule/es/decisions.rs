use chrono::{DateTime, Utc};
use disintegrate::Decision;

use super::events::{ScheduleEventEnvelope, seed_created_event};
use super::state::ScheduleState;
use crate::domain::crm::schedule::{ScheduleAssignment, ScheduleStatus, validate_time_window};

pub struct CreateScheduleAssignmentDecision {
    merchant_id: String,
    assignment: ScheduleAssignment,
}

impl CreateScheduleAssignmentDecision {
    pub fn new(merchant_id: impl Into<String>, assignment: ScheduleAssignment) -> Self {
        Self {
            merchant_id: merchant_id.into(),
            assignment,
        }
    }
}

impl Decision for CreateScheduleAssignmentDecision {
    type Event = ScheduleEventEnvelope;
    type StateQuery = ScheduleState;
    type Error = String;

    fn state_query(&self) -> Self::StateQuery {
        ScheduleState::new(&self.merchant_id, &self.assignment.order_uuid)
    }

    fn process(&self, state: &Self::StateQuery) -> Result<Vec<Self::Event>, Self::Error> {
        if state.exists {
            return Err(format!(
                "schedule for order {} already exists",
                self.assignment.order_uuid
            ));
        }

        if self.assignment.assigned_user_uuid.trim().is_empty() {
            return Err("assigned user is required".to_string());
        }
        validate_time_window(self.assignment.start_at, self.assignment.end_at)?;

        Ok(vec![seed_created_event(
            &self.merchant_id,
            &self.assignment,
        )])
    }
}

pub struct UpdateScheduleAssignmentDecision {
    merchant_id: String,
    order_uuid: String,
    assigned_user_uuid: String,
    start_at: DateTime<Utc>,
    end_at: DateTime<Utc>,
    notes: Option<String>,
    updated_at: DateTime<Utc>,
}

impl UpdateScheduleAssignmentDecision {
    pub fn new(
        merchant_id: impl Into<String>,
        order_uuid: impl Into<String>,
        assigned_user_uuid: impl Into<String>,
        start_at: DateTime<Utc>,
        end_at: DateTime<Utc>,
        notes: Option<String>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            merchant_id: merchant_id.into(),
            order_uuid: order_uuid.into(),
            assigned_user_uuid: assigned_user_uuid.into(),
            start_at,
            end_at,
            notes,
            updated_at,
        }
    }
}

impl Decision for UpdateScheduleAssignmentDecision {
    type Event = ScheduleEventEnvelope;
    type StateQuery = ScheduleState;
    type Error = String;

    fn state_query(&self) -> Self::StateQuery {
        ScheduleState::new(&self.merchant_id, &self.order_uuid)
    }

    fn process(&self, state: &Self::StateQuery) -> Result<Vec<Self::Event>, Self::Error> {
        if !state.exists {
            return Err(format!("schedule for order {} not found", self.order_uuid));
        }

        let current_status = ScheduleStatus::parse(
            state
                .status
                .as_deref()
                .unwrap_or(ScheduleStatus::Planned.as_str()),
        )?;
        if !current_status.allows_assignment_update() {
            return Err(format!(
                "schedule assignment can only be updated in planned status (current: {})",
                current_status.as_str()
            ));
        }
        if self.assigned_user_uuid.trim().is_empty() {
            return Err("assigned user is required".to_string());
        }
        validate_time_window(self.start_at, self.end_at)?;

        Ok(vec![ScheduleEventEnvelope::ScheduleAssignmentUpdated {
            merchant_id: self.merchant_id.clone(),
            order_uuid: self.order_uuid.clone(),
            assigned_user_uuid: self.assigned_user_uuid.clone(),
            start_at: self.start_at,
            end_at: self.end_at,
            notes: self.notes.clone(),
            updated_at: self.updated_at,
        }])
    }
}

pub struct UpdateScheduleStatusDecision {
    merchant_id: String,
    order_uuid: String,
    next_status: ScheduleStatus,
    updated_at: DateTime<Utc>,
}

impl UpdateScheduleStatusDecision {
    pub fn new(
        merchant_id: impl Into<String>,
        order_uuid: impl Into<String>,
        next_status: ScheduleStatus,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            merchant_id: merchant_id.into(),
            order_uuid: order_uuid.into(),
            next_status,
            updated_at,
        }
    }
}

impl Decision for UpdateScheduleStatusDecision {
    type Event = ScheduleEventEnvelope;
    type StateQuery = ScheduleState;
    type Error = String;

    fn state_query(&self) -> Self::StateQuery {
        ScheduleState::new(&self.merchant_id, &self.order_uuid)
    }

    fn process(&self, state: &Self::StateQuery) -> Result<Vec<Self::Event>, Self::Error> {
        if !state.exists {
            return Err(format!("schedule for order {} not found", self.order_uuid));
        }

        let current_status = ScheduleStatus::parse(
            state
                .status
                .as_deref()
                .unwrap_or(ScheduleStatus::Planned.as_str()),
        )?;
        ScheduleStatus::validate_transition(current_status, self.next_status)?;

        Ok(vec![ScheduleEventEnvelope::ScheduleStatusChanged {
            merchant_id: self.merchant_id.clone(),
            order_uuid: self.order_uuid.clone(),
            status: self.next_status.as_str().to_string(),
            updated_at: self.updated_at,
        }])
    }
}

pub struct DeleteScheduleDecision {
    merchant_id: String,
    order_uuid: String,
    deleted_at: DateTime<Utc>,
}

impl DeleteScheduleDecision {
    pub fn new(
        merchant_id: impl Into<String>,
        order_uuid: impl Into<String>,
        deleted_at: DateTime<Utc>,
    ) -> Self {
        Self {
            merchant_id: merchant_id.into(),
            order_uuid: order_uuid.into(),
            deleted_at,
        }
    }
}

impl Decision for DeleteScheduleDecision {
    type Event = ScheduleEventEnvelope;
    type StateQuery = ScheduleState;
    type Error = String;

    fn state_query(&self) -> Self::StateQuery {
        ScheduleState::new(&self.merchant_id, &self.order_uuid)
    }

    fn process(&self, state: &Self::StateQuery) -> Result<Vec<Self::Event>, Self::Error> {
        if !state.exists {
            return Ok(Vec::new());
        }

        Ok(vec![ScheduleEventEnvelope::ScheduleDeleted {
            merchant_id: self.merchant_id.clone(),
            order_uuid: self.order_uuid.clone(),
            deleted_at: self.deleted_at,
        }])
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use disintegrate::TestHarness;

    use super::*;

    fn ts(day: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 3, day, 10, 0, 0)
            .single()
            .expect("valid timestamp")
    }

    fn sample_assignment() -> ScheduleAssignment {
        ScheduleAssignment {
            uuid: "schedule-1".to_string(),
            order_uuid: "order-1".to_string(),
            assigned_user_uuid: "user-1".to_string(),
            start_at: ts(1),
            end_at: ts(1) + chrono::Duration::hours(1),
            status: ScheduleStatus::Planned,
            notes: Some("dispatch".to_string()),
            inserted_at: ts(1),
            updated_at: ts(1),
        }
    }

    #[test]
    fn it_creates_a_schedule_assignment() {
        let assignment = sample_assignment();

        TestHarness::given([])
            .when(CreateScheduleAssignmentDecision::new(
                "11111111-1111-1111-1111-111111111111",
                assignment.clone(),
            ))
            .then([seed_created_event(
                "11111111-1111-1111-1111-111111111111",
                &assignment,
            )]);
    }

    #[test]
    fn it_updates_schedule_assignment() {
        TestHarness::given([seed_created_event(
            "11111111-1111-1111-1111-111111111111",
            &sample_assignment(),
        )])
        .when(UpdateScheduleAssignmentDecision::new(
            "11111111-1111-1111-1111-111111111111",
            "order-1",
            "user-2",
            ts(2),
            ts(2) + chrono::Duration::hours(2),
            Some("updated".to_string()),
            ts(2),
        ))
        .then([ScheduleEventEnvelope::ScheduleAssignmentUpdated {
            merchant_id: "11111111-1111-1111-1111-111111111111".to_string(),
            order_uuid: "order-1".to_string(),
            assigned_user_uuid: "user-2".to_string(),
            start_at: ts(2),
            end_at: ts(2) + chrono::Duration::hours(2),
            notes: Some("updated".to_string()),
            updated_at: ts(2),
        }]);
    }

    #[test]
    fn it_rejects_invalid_schedule_status_transition() {
        let in_service = ScheduleEventEnvelope::ScheduleStatusChanged {
            merchant_id: "11111111-1111-1111-1111-111111111111".to_string(),
            order_uuid: "order-1".to_string(),
            status: ScheduleStatus::InService.as_str().to_string(),
            updated_at: ts(2),
        };

        TestHarness::given([
            seed_created_event("11111111-1111-1111-1111-111111111111", &sample_assignment()),
            in_service,
        ])
        .when(UpdateScheduleStatusDecision::new(
            "11111111-1111-1111-1111-111111111111",
            "order-1",
            ScheduleStatus::Planned,
            ts(3),
        ))
        .then_err("Invalid schedule status transition: in_service -> planned".to_string());
    }
}
