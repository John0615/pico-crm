use crate::domain::models::pagination::Pagination;
use crate::domain::repositories::contact::ContactRepository;
use crate::domain::specifications::contact_spec::{
    ContactFilters, ContactSpecification, SortOption,
};
use rust_xlsxwriter::{Format, FormatAlign, FormatBorder, Workbook};
use shared::{
    ListResult,
    contact::{Contact, ContactQuery, UpdateContact},
};

pub struct ContactAppService<R: ContactRepository> {
    contact_repo: R,
}

impl<R: ContactRepository> ContactAppService<R> {
    pub fn new(contact_repo: R) -> Self {
        Self { contact_repo }
    }

    pub async fn fetch_contact(&self, uuid: String) -> Result<Option<Contact>, String> {
        let result = self
            .contact_repo
            .get_contact(uuid)
            .await
            .map_err(|e| e.to_string())?
            .map(|contact| contact.into());
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
        let (contacts, total) = self.contact_repo.contacts(spec, pagination).await?;
        let contacts: Vec<Contact> = contacts.into_iter().map(|contact| contact.into()).collect();
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
        let contacts = self.contact_repo.all_contacts(spec).await?;
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
            .set_column_width(1, 20)
            .map_err(|e| e.to_string())?; // 公司列
        worksheet
            .set_column_width(2, 20)
            .map_err(|e| e.to_string())?; // 职位列
        worksheet
            .set_column_width(3, 15)
            .map_err(|e| e.to_string())?; // 电话列
        worksheet
            .set_column_width(4, 25)
            .map_err(|e| e.to_string())?; // 邮箱列
        worksheet
            .set_column_width(5, 10)
            .map_err(|e| e.to_string())?; // 状态列
        worksheet
            .set_column_width(6, 20)
            .map_err(|e| e.to_string())?; // 最后联系
        worksheet
            .set_column_width(7, 15)
            .map_err(|e| e.to_string())?; // 客户价值

        // 写入表头
        worksheet
            .write_string_with_format(0, 0, "姓名", &header_format)
            .map_err(|e| e.to_string())?;
        worksheet
            .write_string_with_format(0, 1, "公司", &header_format)
            .map_err(|e| e.to_string())?;
        worksheet
            .write_string_with_format(0, 2, "职位", &header_format)
            .map_err(|e| e.to_string())?;
        worksheet
            .write_string_with_format(0, 3, "电话", &header_format)
            .map_err(|e| e.to_string())?;
        worksheet
            .write_string_with_format(0, 4, "邮箱", &header_format)
            .map_err(|e| e.to_string())?;
        worksheet
            .write_string_with_format(0, 5, "状态", &header_format)
            .map_err(|e| e.to_string())?;
        worksheet
            .write_string_with_format(0, 6, "最后联系", &header_format)
            .map_err(|e| e.to_string())?;
        worksheet
            .write_string_with_format(0, 7, "客户价值", &header_format)
            .map_err(|e| e.to_string())?;

        // 写入联系人数据
        for (row, contact) in contacts.iter().enumerate() {
            let row = (row + 1) as u32; // 转换为u32类型
            let status = match &contact.status {
                1 => "已签约",
                2 => "待跟进",
                3 => "已流失",
                _ => "未知",
            };
            let value_level = match &contact.value_level {
                1 => "活跃客户",
                2 => "潜在客户",
                3 => "不活跃客户",
                _ => "未知",
            };

            worksheet
                .write_string_with_format(row, 0, &contact.user_name, &content_format)
                .map_err(|e| e.to_string())?;

            worksheet
                .write_string_with_format(row, 1, &contact.company, &content_format)
                .map_err(|e| e.to_string())?;

            worksheet
                .write_string_with_format(row, 2, &contact.position, &content_format)
                .map_err(|e| e.to_string())?;

            worksheet
                .write_string_with_format(row, 3, &contact.phone_number, &content_format)
                .map_err(|e| e.to_string())?;

            worksheet
                .write_string_with_format(row, 4, &contact.email, &content_format)
                .map_err(|e| e.to_string())?;

            worksheet
                .write_string_with_format(row, 5, status, &content_format)
                .map_err(|e| e.to_string())?;

            worksheet
                .write_string_with_format(row, 6, &contact.last_contact, &content_format)
                .map_err(|e| e.to_string())?;

            worksheet
                .write_string_with_format(row, 7, value_level, &content_format)
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
        let _new_contact = self.contact_repo.create_contact(contact).await?;
        Ok(())
    }

    pub async fn update_contact(&self, contact: UpdateContact) -> Result<(), String> {
        let contact = contact.into();
        let _new_contact = self.contact_repo.update_contact(contact).await?;
        Ok(())
    }

    pub async fn delete_contact(&self, uuid: String) -> Result<(), String> {
        let _deleted_contact = self.contact_repo.delete_contact(uuid).await?;
        Ok(())
    }
}
