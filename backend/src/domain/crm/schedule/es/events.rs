use chrono::{DateTime, Utc};
use disintegrate::Event;
use serde::{Deserialize, Serialize};

use crate::domain::crm::schedule::ScheduleAssignment;

#[derive(Debug, Clone, PartialEq, Eq, Event, Serialize, Deserialize)]
#[stream(
    ScheduleEvent,
    [
        ScheduleAssignmentCreated,
        ScheduleAssignmentUpdated,
        ScheduleStatusChanged,
        ScheduleDeleted
    ]
)]
pub enum ScheduleEventEnvelope {
    ScheduleAssignmentCreated {
        #[id]
        tenant_schema: String,
        #[id]
        order_uuid: String,
        schedule_uuid: String,
        assigned_user_uuid: String,
        start_at: DateTime<Utc>,
        end_at: DateTime<Utc>,
        status: String,
        notes: Option<String>,
        inserted_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    },
    ScheduleAssignmentUpdated {
        #[id]
        tenant_schema: String,
        #[id]
        order_uuid: String,
        assigned_user_uuid: String,
        start_at: DateTime<Utc>,
        end_at: DateTime<Utc>,
        notes: Option<String>,
        updated_at: DateTime<Utc>,
    },
    ScheduleStatusChanged {
        #[id]
        tenant_schema: String,
        #[id]
        order_uuid: String,
        status: String,
        updated_at: DateTime<Utc>,
    },
    ScheduleDeleted {
        #[id]
        tenant_schema: String,
        #[id]
        order_uuid: String,
        deleted_at: DateTime<Utc>,
    },
}

pub fn seed_created_event(
    tenant_schema: &str,
    assignment: &ScheduleAssignment,
) -> ScheduleEventEnvelope {
    ScheduleEventEnvelope::ScheduleAssignmentCreated {
        tenant_schema: tenant_schema.to_string(),
        order_uuid: assignment.order_uuid.clone(),
        schedule_uuid: assignment.uuid.clone(),
        assigned_user_uuid: assignment.assigned_user_uuid.clone(),
        start_at: assignment.start_at,
        end_at: assignment.end_at,
        status: assignment.status.as_str().to_string(),
        notes: assignment.notes.clone(),
        inserted_at: assignment.inserted_at,
        updated_at: assignment.updated_at,
    }
}
