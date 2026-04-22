use crate::domain::crm::after_sales::{
    AfterSalesCaseRepository, CreateAfterSalesCase, UpdateAfterSalesRefund,
};
use shared::after_sales::{
    AfterSalesCase as SharedAfterSalesCase, CreateAfterSalesCaseRequest,
    UpdateAfterSalesRefundRequest,
};

pub struct AfterSalesCaseAppService<R: AfterSalesCaseRepository> {
    repo: R,
}

impl<R: AfterSalesCaseRepository> AfterSalesCaseAppService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create_case(
        &self,
        order_uuid: String,
        payload: CreateAfterSalesCaseRequest,
        operator_uuid: Option<String>,
    ) -> Result<SharedAfterSalesCase, String> {
        let case = CreateAfterSalesCase {
            order_uuid,
            operator_uuid,
            case_type: payload.case_type,
            description: payload.description,
        };
        case.verify()?;

        let created = self.repo.create_case(case).await?;
        Ok(created.into())
    }

    pub async fn update_refund(
        &self,
        case_uuid: String,
        payload: UpdateAfterSalesRefundRequest,
    ) -> Result<SharedAfterSalesCase, String> {
        let refund = UpdateAfterSalesRefund {
            case_uuid,
            refund_amount_cents: payload.refund_amount_cents,
            refund_reason: payload.refund_reason.and_then(|value| {
                let trimmed = value.trim().to_string();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                }
            }),
        };
        refund.verify()?;

        let updated = self.repo.update_refund(refund).await?;
        Ok(updated.into())
    }
}
