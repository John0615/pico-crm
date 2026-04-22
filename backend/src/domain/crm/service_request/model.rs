use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ServiceRequest {
    pub uuid: String,
    pub customer_uuid: String,
    pub creator_uuid: String,
    pub service_catalog_uuid: Option<String>,
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
    pub service_catalog_uuid: Option<String>,
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
            Self::New => "new",
            Self::Confirmed => "confirmed",
            Self::Converted => "converted",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "new" => Ok(Self::New),
            "confirmed" => Ok(Self::Confirmed),
            "converted" => Ok(Self::Converted),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(format!("Invalid service request status: {}", value)),
        }
    }

    pub fn can_transition(from: Self, to: Self) -> bool {
        match from {
            Self::New => matches!(to, Self::New | Self::Confirmed | Self::Cancelled),
            Self::Confirmed => matches!(to, Self::Confirmed | Self::Converted | Self::Cancelled),
            Self::Converted => matches!(to, Self::Converted),
            Self::Cancelled => matches!(to, Self::Cancelled),
        }
    }
}

impl ServiceRequestSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SalesManual => "sales_manual",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "sales_manual" => Ok(Self::SalesManual),
            _ => Err(format!("Invalid service request source: {}", value)),
        }
    }
}

impl ServiceRequest {
    pub fn new(
        customer_uuid: String,
        creator_uuid: String,
        service_catalog_uuid: Option<String>,
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
            service_catalog_uuid,
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
        if let Some(service_catalog_uuid) = self
            .service_catalog_uuid
            .as_ref()
            .filter(|value| !value.trim().is_empty())
        {
            Uuid::parse_str(service_catalog_uuid.trim())
                .map_err(|e| format!("Invalid service catalog uuid: {}", e))?;
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
}
