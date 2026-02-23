use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct MerchantDashboardQuery {
    pub preset: Option<String>,
    pub start: Option<String>,
    pub end: Option<String>,
    pub timezone: Option<String>,
    pub granularity: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct MerchantDashboardMeta {
    pub start: String,
    pub end: String,
    pub timezone: String,
    pub granularity: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct MerchantDashboardOverview {
    pub new_contacts: u64,
    pub service_requests: u64,
    pub orders: u64,
    pub completed_orders: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct MerchantDashboardTrendPoint {
    pub bucket: String,
    pub new_contacts: u64,
    pub service_requests: u64,
    pub orders: u64,
    pub completed_orders: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct MerchantDashboardTodo {
    pub key: String,
    pub label: String,
    pub count: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct MerchantDashboardResponse {
    pub meta: MerchantDashboardMeta,
    pub overview: MerchantDashboardOverview,
    pub trend: Vec<MerchantDashboardTrendPoint>,
    pub todos: Vec<MerchantDashboardTodo>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merchant_dashboard_round_trip() {
        let payload = MerchantDashboardResponse {
            meta: MerchantDashboardMeta {
                start: "2025-01-01".to_string(),
                end: "2025-01-07".to_string(),
                timezone: "Asia/Shanghai".to_string(),
                granularity: "day".to_string(),
            },
            overview: MerchantDashboardOverview {
                new_contacts: 3,
                service_requests: 5,
                orders: 4,
                completed_orders: 2,
            },
            trend: vec![MerchantDashboardTrendPoint {
                bucket: "2025-01-01".to_string(),
                new_contacts: 1,
                service_requests: 2,
                orders: 1,
                completed_orders: 0,
            }],
            todos: vec![MerchantDashboardTodo {
                key: "pending_requests".to_string(),
                label: "待确认需求".to_string(),
                count: 2,
            }],
        };

        let serialized = serde_json::to_string(&payload).expect("serialize payload");
        let decoded: MerchantDashboardResponse =
            serde_json::from_str(&serialized).expect("deserialize payload");

        assert_eq!(decoded.overview.orders, 4);
        assert_eq!(decoded.todos.first().unwrap().key, "pending_requests");
    }
}
