use crate::domain::crm::contact::{
    ContactFilters, ContactQuery as CQuery, ContactSpecification, SortOption,
};
use crate::domain::shared::pagination::Pagination;
use rust_xlsxwriter::{Format, FormatAlign, FormatBorder, Workbook};
use shared::{
    ListResult,
    contact::{Contact, ContactQuery},
};

pub struct ContactAppService<R: CQuery> {
    contact_query: R,
}

impl<R: CQuery<Result = Contact>> ContactAppService<R> {
    pub fn new(contact_query: R) -> Self {
        Self { contact_query }
    }

    pub async fn fetch_contact(&self, uuid: String) -> Result<Option<Contact>, String> {
        let result = self
            .contact_query
            .get_contact(uuid)
            .await
            .map_err(|e| e.to_string())?
            .map(|contact| contact);
        Ok(result)
    }

    pub async fn fetch_contacts(
        &self,
        params: ContactQuery,
    ) -> Result<ListResult<Contact>, String> {
        let sort_options: Vec<SortOption> = params
            .sort
            .unwrap_or_default()
            .into_iter()
            .map(|item| item.into())
            .collect();
        // 构建领域规约
        let pagination =
            Pagination::new(params.page, params.page_size).map_err(|e| e.to_string())?;
        let filters: ContactFilters = params.filters.map(|f| f.into()).unwrap_or_default();
        let spec = ContactSpecification::new(Some(filters), Some(sort_options))
            .map_err(|e| e.to_string())?;
        println!("spec: {:?}", spec);
        let (contacts, total) = self.contact_query.contacts(spec, pagination).await?;
        let contacts: Vec<Contact> = contacts.into_iter().map(|contact| contact).collect();
        Ok(ListResult {
            items: contacts,
            total,
        })
    }

    pub async fn fetch_contacts_excel_data(&self, params: ContactQuery) -> Result<Vec<u8>, String> {
        let sort_options: Vec<SortOption> = params
            .sort
            .unwrap_or_default()
            .into_iter()
            .map(|item| item.into())
            .collect();
        // 构建领域规约
        let filters: ContactFilters = params.filters.map(|f| f.into()).unwrap_or_default();
        let spec = ContactSpecification::new(Some(filters), Some(sort_options))
            .map_err(|e| e.to_string())?;
        println!("spec: {:?}", spec);
        let contacts = self.contact_query.all_contacts(spec).await?;
        let contacts: Vec<Contact> = contacts.into_iter().map(|contact| contact).collect();

        // 创建 Excel 工作簿
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();

        // 定义格式
        let header_format = Format::new()
            .set_bold()
            .set_border(FormatBorder::Thin)
            .set_background_color("#D3D3D3")
            .set_align(FormatAlign::Center);

        let content_format = Format::new()
            .set_border(FormatBorder::Thin)
            .set_align(FormatAlign::Left);

        // 设置列宽
        worksheet
            .set_column_width(0, 20)
            .map_err(|e| e.to_string())?; // 姓名列
        worksheet
            .set_column_width(1, 15)
            .map_err(|e| e.to_string())?; // 电话列
        worksheet
            .set_column_width(2, 10)
            .map_err(|e| e.to_string())?; // 跟进状态列
        worksheet
            .set_column_width(3, 20)
            .map_err(|e| e.to_string())?; // 小区
        worksheet
            .set_column_width(4, 28)
            .map_err(|e| e.to_string())?; // 地址
        worksheet
            .set_column_width(5, 12)
            .map_err(|e| e.to_string())?; // 房屋面积
        worksheet
            .set_column_width(6, 15)
            .map_err(|e| e.to_string())?; // 标签
        worksheet
            .set_column_width(7, 20)
            .map_err(|e| e.to_string())?; // 最近服务
        worksheet
            .set_column_width(8, 30)
            .map_err(|e| e.to_string())?; // 服务需求

        // 写入表头
        worksheet
            .write_string_with_format(0, 0, "姓名", &header_format)
            .map_err(|e| e.to_string())?;
        worksheet
            .write_string_with_format(0, 1, "电话", &header_format)
            .map_err(|e| e.to_string())?;
        worksheet
            .write_string_with_format(0, 2, "跟进状态", &header_format)
            .map_err(|e| e.to_string())?;
        worksheet
            .write_string_with_format(0, 3, "小区/社区", &header_format)
            .map_err(|e| e.to_string())?;
        worksheet
            .write_string_with_format(0, 4, "详细地址", &header_format)
            .map_err(|e| e.to_string())?;
        worksheet
            .write_string_with_format(0, 5, "房屋面积(㎡)", &header_format)
            .map_err(|e| e.to_string())?;
        worksheet
            .write_string_with_format(0, 6, "标签", &header_format)
            .map_err(|e| e.to_string())?;
        worksheet
            .write_string_with_format(0, 7, "最近服务时间", &header_format)
            .map_err(|e| e.to_string())?;
        worksheet
            .write_string_with_format(0, 8, "服务需求", &header_format)
            .map_err(|e| e.to_string())?;

        // 写入联系人数据
        for (row, contact) in contacts.iter().enumerate() {
            let row = (row + 1) as u32; // 转换为u32类型
            let follow_up_status = follow_up_status_label(contact.follow_up_status.as_deref());
            let address = build_contact_address(contact);
            let tags = if contact.tags.is_empty() {
                "-".to_string()
            } else {
                contact.tags.join("、")
            };
            let last_service_at = contact.last_service_at.as_deref().unwrap_or("-");
            let community = contact.community.as_deref().unwrap_or("-");
            let house_area = contact
                .house_area_sqm
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".to_string());
            let service_need = contact.service_need.as_deref().unwrap_or("-");

            worksheet
                .write_string_with_format(row, 0, &contact.user_name, &content_format)
                .map_err(|e| e.to_string())?;

            worksheet
                .write_string_with_format(row, 1, &contact.phone_number, &content_format)
                .map_err(|e| e.to_string())?;

            worksheet
                .write_string_with_format(row, 2, follow_up_status, &content_format)
                .map_err(|e| e.to_string())?;

            worksheet
                .write_string_with_format(row, 3, community, &content_format)
                .map_err(|e| e.to_string())?;

            worksheet
                .write_string_with_format(row, 4, &address, &content_format)
                .map_err(|e| e.to_string())?;

            worksheet
                .write_string_with_format(row, 5, &house_area, &content_format)
                .map_err(|e| e.to_string())?;

            worksheet
                .write_string_with_format(row, 6, &tags, &content_format)
                .map_err(|e| e.to_string())?;

            worksheet
                .write_string_with_format(row, 7, last_service_at, &content_format)
                .map_err(|e| e.to_string())?;

            worksheet
                .write_string_with_format(row, 8, service_need, &content_format)
                .map_err(|e| e.to_string())?;
        }

        // 添加自动筛选
        worksheet
            .autofilter(0, 0, contacts.len() as u32, 8)
            .map_err(|e| e.to_string())?;

        // 保存到内存缓冲区
        let buf = workbook
            .save_to_buffer()
            .map_err(|e| format!("保存Excel失败: {}", e))?;

        Ok(buf)
    }
}

fn follow_up_status_label(value: Option<&str>) -> &'static str {
    match value.unwrap_or("pending") {
        "pending" => "待跟进",
        "contacted" => "已联系",
        "quoted" => "已报价",
        "scheduled" => "已预约",
        "completed" => "已完成",
        _ => "未知",
    }
}

fn build_contact_address(contact: &Contact) -> String {
    let mut parts = Vec::new();
    if let Some(address) = &contact.address {
        if !address.trim().is_empty() {
            parts.push(address.trim().to_string());
        }
    }
    if let Some(community) = &contact.community {
        if !community.trim().is_empty() {
            parts.push(community.trim().to_string());
        }
    }
    if let Some(building) = &contact.building {
        if !building.trim().is_empty() {
            parts.push(building.trim().to_string());
        }
    }

    if parts.is_empty() {
        "-".to_string()
    } else {
        parts.join(" / ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::crm::contact::{ContactQuery as DomainContactQuery, ContactSpecification};
    use crate::domain::shared::pagination::Pagination;
    use std::io::{Cursor, Read};
    use zip::ZipArchive;

    struct FakeContactQuery {
        items: Vec<Contact>,
    }

    impl DomainContactQuery for FakeContactQuery {
        type Result = Contact;

        fn contacts(
            &self,
            _spec: ContactSpecification,
            _pagination: Pagination,
        ) -> impl std::future::Future<Output = Result<(Vec<Self::Result>, u64), String>> + Send
        {
            let items = self.items.clone();
            async move { Ok((items.clone(), items.len() as u64)) }
        }

        fn all_contacts(
            &self,
            _spec: ContactSpecification,
        ) -> impl std::future::Future<Output = Result<Vec<Self::Result>, String>> + Send {
            let items = self.items.clone();
            async move { Ok(items) }
        }

        fn get_contact(
            &self,
            _uuid: String,
        ) -> impl std::future::Future<Output = Result<Option<Self::Result>, String>> + Send
        {
            let item = self.items.first().cloned();
            async move { Ok(item) }
        }
    }

    #[tokio::test]
    async fn export_contacts_excel_contains_extended_headers_and_values() {
        let service = ContactAppService::new(FakeContactQuery {
            items: vec![Contact {
                contact_uuid: "11111111-1111-1111-1111-111111111111".to_string(),
                user_name: "张阿姨".to_string(),
                phone_number: "13800138000".to_string(),
                address: Some("朝阳区望京街道 8 号".to_string()),
                community: Some("望京花园".to_string()),
                building: Some("2号楼801".to_string()),
                house_area_sqm: Some(98),
                service_need: Some("深度保洁，每周一次".to_string()),
                tags: vec!["VIP".to_string(), "保洁".to_string()],
                last_service_at: Some("2026-04-17 09:00:00".to_string()),
                follow_up_status: Some("scheduled".to_string()),
                after_sales_case_count: Some(1),
                complaint_case_count: Some(0),
                refund_case_count: Some(0),
                rework_count: Some(0),
                inserted_at: "2026-04-01 08:00:00".to_string(),
                updated_at: "2026-04-18 10:00:00".to_string(),
            }],
        });

        let bytes = service
            .fetch_contacts_excel_data(ContactQuery {
                page: 1,
                page_size: 20,
                sort: Some(vec![shared::contact::SortOption {
                    field: shared::contact::SortField::Name,
                    order: shared::contact::SortOrder::Desc,
                }]),
                filters: None,
            })
            .await
            .expect("excel export should succeed");

        let workbook_xml = unzip_all_xml(&bytes);
        assert!(workbook_xml.contains("跟进状态"));
        assert!(workbook_xml.contains("详细地址"));
        assert!(workbook_xml.contains("最近服务时间"));
        assert!(workbook_xml.contains("房屋面积(㎡)"));
        assert!(workbook_xml.contains("张阿姨"));
        assert!(workbook_xml.contains("朝阳区望京街道 8 号 / 望京花园 / 2号楼801"));
        assert!(workbook_xml.contains("VIP、保洁"));
        assert!(workbook_xml.contains("已预约"));
        assert!(workbook_xml.contains("98"));
        assert!(workbook_xml.contains("深度保洁，每周一次"));
    }

    #[test]
    fn follow_up_status_label_maps_extended_states() {
        assert_eq!(follow_up_status_label(Some("pending")), "待跟进");
        assert_eq!(follow_up_status_label(Some("scheduled")), "已预约");
        assert_eq!(follow_up_status_label(Some("completed")), "已完成");
    }

    #[test]
    fn build_contact_address_merges_extended_fields() {
        let address = build_contact_address(&Contact {
            address: Some("海淀区".to_string()),
            community: Some("上地东里".to_string()),
            building: Some("1号楼302".to_string()),
            ..Default::default()
        });

        assert_eq!(address, "海淀区 / 上地东里 / 1号楼302");
    }

    fn unzip_all_xml(bytes: &[u8]) -> String {
        let cursor = Cursor::new(bytes);
        let mut archive = ZipArchive::new(cursor).expect("xlsx should be a zip archive");
        let mut combined = String::new();

        for idx in 0..archive.len() {
            let mut file = archive.by_index(idx).expect("zip entry should exist");
            if !file.name().ends_with(".xml") {
                continue;
            }
            let mut content = String::new();
            file.read_to_string(&mut content)
                .expect("xml entry should be valid utf-8");
            combined.push_str(&content);
        }

        combined
    }
}
