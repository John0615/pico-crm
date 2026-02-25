use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ServiceRequest {
    pub uuid: String,
    pub customer_uuid: String,
    pub creator_uuid: String,
    pub service_content: String,
    pub appointment_start_at: Option<DateTime<Utc>>,
    pub appointment_end_at: Option<DateTime<Utc>>,
    pub status: ServiceRequestStatus,
    pub source: ServiceRequestSource,
    pub notes: Option<String>,
    pub inserted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct UpdateServiceRequest {
    pub uuid: String,
    pub service_content: String,
    pub appointment_start_at: Option<DateTime<Utc>>,
    pub appointment_end_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceRequestStatus {
    New,
    Confirmed,
    Converted,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceRequestSource {
    SalesManual,
}

impl ServiceRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ServiceRequestStatus::New => "new",
            ServiceRequestStatus::Confirmed => "confirmed",
            ServiceRequestStatus::Converted => "converted",
            ServiceRequestStatus::Cancelled => "cancelled",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "new" => Ok(ServiceRequestStatus::New),
            "confirmed" => Ok(ServiceRequestStatus::Confirmed),
            "converted" => Ok(ServiceRequestStatus::Converted),
            "cancelled" => Ok(ServiceRequestStatus::Cancelled),
            _ => Err(format!("Invalid service request status: {}", value)),
        }
    }

    pub fn can_transition(from: ServiceRequestStatus, to: ServiceRequestStatus) -> bool {
        match from {
            ServiceRequestStatus::New => matches!(
                to,
                ServiceRequestStatus::New
                    | ServiceRequestStatus::Confirmed
                    | ServiceRequestStatus::Cancelled
            ),
            ServiceRequestStatus::Confirmed => matches!(
                to,
                ServiceRequestStatus::Confirmed
                    | ServiceRequestStatus::Converted
                    | ServiceRequestStatus::Cancelled
            ),
            ServiceRequestStatus::Converted => matches!(to, ServiceRequestStatus::Converted),
            ServiceRequestStatus::Cancelled => matches!(to, ServiceRequestStatus::Cancelled),
        }
    }
}

impl ServiceRequestSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            ServiceRequestSource::SalesManual => "sales_manual",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "sales_manual" => Ok(ServiceRequestSource::SalesManual),
            _ => Err(format!("Invalid service request source: {}", value)),
        }
    }
}

impl ServiceRequest {
    pub fn new(
        customer_uuid: String,
        creator_uuid: String,
        service_content: String,
        appointment_start_at: Option<DateTime<Utc>>,
        appointment_end_at: Option<DateTime<Utc>>,
        notes: Option<String>,
    ) -> Result<Self, String> {
        let now = Utc::now();
        let request = Self {
            uuid: Uuid::new_v4().to_string(),
            customer_uuid,
            creator_uuid,
            service_content,
            appointment_start_at,
            appointment_end_at,
            status: ServiceRequestStatus::New,
            source: ServiceRequestSource::SalesManual,
            notes,
            inserted_at: now,
            updated_at: now,
        };
        request.verify()?;
        Ok(request)
    }

    pub fn verify(&self) -> Result<(), String> {
        if self.customer_uuid.trim().is_empty() {
            return Err("Customer is required".to_string());
        }
        if self.creator_uuid.trim().is_empty() {
            return Err("Creator is required".to_string());
        }
        if self.service_content.trim().is_empty() {
            return Err("Service content is required".to_string());
        }
        if let (Some(start), Some(end)) = (self.appointment_start_at, self.appointment_end_at) {
            if end < start {
                return Err("Appointment end must be after start".to_string());
            }
        }
        Ok(())
    }

    pub fn update_status(&mut self, status: ServiceRequestStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }
}
