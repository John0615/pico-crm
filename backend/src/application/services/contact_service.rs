use crate::domain::models::pagination::Pagination;
use crate::domain::repositories::contact::ContactRepository;
use crate::domain::services::contact_service::ContactService;
use crate::domain::specifications::contact_spec::{
    ContactFilters, ContactSpecification, SortOption,
};
use rust_xlsxwriter::{Format, FormatAlign, FormatBorder, Workbook};
use shared::{
    ListResult,
    contact::{Contact, ContactExport, ContactQuery},
};

pub struct ContactAppService<R: ContactRepository> {
    contact_service: ContactService<R>,
}

impl<R: ContactRepository> ContactAppService<R> {
    pub fn new(contact_service: ContactService<R>) -> Self {
        Self { contact_service }
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
        let (contacts, total) = self
            .contact_service
            .fetch_contacts(spec, pagination)
            .await?;
        let contacts: Vec<Contact> = contacts.into_iter().map(|contact| contact.into()).collect();
        Ok(ListResult {
            items: contacts,
            total,
        })
    }

    pub async fn fetch_contacts_excel_data(
        &self,
        params: ContactExport,
    ) -> Result<Vec<u8>, String> {
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
        let contacts = self.contact_service.fetch_all_contacts(spec).await?;
        let contacts: Vec<Contact> = contacts.into_iter().map(|contact| contact.into()).collect();

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
            .set_column_width(2, 25)
            .map_err(|e| e.to_string())?; // 邮箱列
        worksheet
            .set_column_width(3, 10)
            .map_err(|e| e.to_string())?; // 状态列

        // 写入表头
        worksheet
            .write_string_with_format(0, 0, "姓名", &header_format)
            .map_err(|e| e.to_string())?;
        worksheet
            .write_string_with_format(0, 1, "电话", &header_format)
            .map_err(|e| e.to_string())?;
        worksheet
            .write_string_with_format(0, 2, "邮箱", &header_format)
            .map_err(|e| e.to_string())?;
        worksheet
            .write_string_with_format(0, 3, "状态", &header_format)
            .map_err(|e| e.to_string())?;

        // 写入联系人数据
        for (row, contact) in contacts.iter().enumerate() {
            let row = (row + 1) as u32; // 转换为u32类型

            worksheet
                .write_string_with_format(row, 0, &contact.user_name, &content_format)
                .map_err(|e| e.to_string())?;

            worksheet
                .write_string_with_format(row, 1, &contact.phone_number, &content_format)
                .map_err(|e| e.to_string())?;

            worksheet
                .write_string_with_format(row, 2, &contact.email, &content_format)
                .map_err(|e| e.to_string())?;

            worksheet
                .write_string_with_format(row, 3, &contact.status.to_string(), &content_format)
                .map_err(|e| e.to_string())?;
        }

        // 添加自动筛选
        worksheet
            .autofilter(0, 0, contacts.len() as u32, 3)
            .map_err(|e| e.to_string())?;

        // 保存到内存缓冲区
        let buf = workbook
            .save_to_buffer()
            .map_err(|e| format!("保存Excel失败: {}", e))?;

        Ok(buf)
    }

    pub async fn create_contact(&self, contact: Contact) -> Result<(), String> {
        let contact = contact.into();
        let _new_contact = self.contact_service.create_contact(contact).await?;
        Ok(())
    }
}
