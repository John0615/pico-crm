use super::model::{ServiceRequest, UpdateServiceRequest};

pub trait ServiceRequestRepository: Send + Sync {
    fn create_service_request(
        &self,
        request: ServiceRequest,
    ) -> impl std::future::Future<Output = Result<String, String>> + Send;

    fn update_service_request(
        &self,
        request: UpdateServiceRequest,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send;

    fn update_service_request_status(
        &self,
        uuid: String,
        status: String,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send;
}
