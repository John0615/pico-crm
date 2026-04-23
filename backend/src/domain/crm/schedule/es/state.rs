use chrono::{DateTime, Utc};
use disintegrate::{StateMutate, StateQuery};
use serde::{Deserialize, Serialize};

use super::events::ScheduleEvent;
use crate::domain::crm::schedule::{ScheduleAssignment, ScheduleStatus};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, StateQuery)]
#[state_query(ScheduleEvent)]
pub struct ScheduleState {
    #[id]
    pub merchant_id: String,
    #[id]
    pub order_uuid: String,
    pub exists: bool,
    pub schedule_uuid: Option<String>,
    pub assigned_user_uuid: Option<String>,
    pub start_at: Option<DateTime<Utc>>,
    pub end_at: Option<DateTime<Utc>>,
    pub status: Option<String>,
    pub notes: Option<String>,
    pub inserted_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl ScheduleState {
    pub fn new(merchant_id: impl Into<String>, order_uuid: impl Into<String>) -> Self {
        Self {
            merchant_id: merchant_id.into(),
            order_uuid: order_uuid.into(),
            ..Default::default()
        }
    }

    pub fn to_domain(&self) -> Result<ScheduleAssignment, String> {
        if !self.exists {
            return Err(format!("schedule for order {} not found", self.order_uuid));
        }

        let uuid = self
            .schedule_uuid
            .clone()
            .ok_or_else(|| "schedule uuid is missing".to_string())?;
        let assigned_user_uuid = self
            .assigned_user_uuid
            .clone()
            .ok_or_else(|| "schedule assigned user is missing".to_string())?;
        let start_at = self
            .start_at
            .ok_or_else(|| "schedule start_at is missing".to_string())?;
        let end_at = self
            .end_at
            .ok_or_else(|| "schedule end_at is missing".to_string())?;
        let status = ScheduleStatus::parse(
            self.status
                .as_deref()
                .unwrap_or(ScheduleStatus::Planned.as_str()),
        )?;
        let inserted_at = self
            .inserted_at
            .ok_or_else(|| "schedule inserted_at is missing".to_string())?;
        let updated_at = self
            .updated_at
            .ok_or_else(|| "schedule updated_at is missing".to_string())?;

        Ok(ScheduleAssignment {
            uuid,
            order_uuid: self.order_uuid.clone(),
            assigned_user_uuid,
            start_at,
            end_at,
            status,
            notes: self.notes.clone(),
            inserted_at,
            updated_at,
        })
    }
}

impl StateMutate for ScheduleState {
    fn mutate(&mut self, event: Self::Event) {
        match event {
            ScheduleEvent::ScheduleAssignmentCreated {
                merchant_id,
                order_uuid,
                schedule_uuid,
                assigned_user_uuid,
                start_at,
                end_at,
                status,
                notes,
                inserted_at,
                updated_at,
            } => {
                self.exists = true;
                self.merchant_id = merchant_id;
                self.order_uuid = order_uuid;
                self.schedule_uuid = Some(schedule_uuid);
                self.assigned_user_uuid = Some(assigned_user_uuid);
                self.start_at = Some(start_at);
                self.end_at = Some(end_at);
                self.status = Some(status);
                self.notes = notes;
                self.inserted_at = Some(inserted_at);
                self.updated_at = Some(updated_at);
            }
            ScheduleEvent::ScheduleAssignmentUpdated {
                assigned_user_uuid,
                start_at,
                end_at,
                notes,
                updated_at,
                ..
            } => {
                self.assigned_user_uuid = Some(assigned_user_uuid);
                self.start_at = Some(start_at);
                self.end_at = Some(end_at);
                self.notes = notes;
                self.updated_at = Some(updated_at);
            }
            ScheduleEvent::ScheduleStatusChanged {
                status, updated_at, ..
            } => {
                self.status = Some(status);
                self.updated_at = Some(updated_at);
            }
            ScheduleEvent::ScheduleDeleted { .. } => {
                self.exists = false;
                self.schedule_uuid = None;
                self.assigned_user_uuid = None;
                self.start_at = None;
                self.end_at = None;
                self.status = None;
                self.notes = None;
                self.inserted_at = None;
                self.updated_at = None;
            }
        }
    }
}
