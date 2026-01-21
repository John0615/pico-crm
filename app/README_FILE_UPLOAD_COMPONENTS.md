# 文件上传组件使用指南

本项目提供了一套完整的文件上传组件，基于正确的设计思路实现：**通过 FormField 的 field_type 配置文件上传，上传完成后将文件路径存储在表单字段的 value 中**。

## 设计思路

### 正确的文件上传流程：

1. **配置 FormField**：设置 `field_type` 为 `FieldType::File`
2. **用户选择文件**：通过文件选择组件选择文件
3. **触发上传回调**：`on_upload` 回调函数处理文件上传到后端
4. **存储文件路径**：上传成功后，将返回的文件路径存储在 FormField 的 `value` 中
5. **表单提交**：提交表单时，`value` 包含文件路径，供后端处理

### 与之前错误实现的区别：

- ❌ **错误**：在 FormField 中添加 `files` 字段存储文件对象
- ✅ **正确**：通过 `field_type: FieldType::File` 配置，`value` 存储文件路径

## 组件概览

### 1. FieldType::File - 文件字段类型

```rust
FieldType::File {
    accept: Option<String>,           // 接受的文件类型
    multiple: bool,                   // 是否支持多文件
    max_size: Option<u64>,           // 最大文件大小
    on_upload: Option<Callback<FileInfo>>, // 上传回调
}
```

### 2. FileFieldInput - 文件字段输入组件

自动集成到 DaisyForm 中，当 FormField 的 field_type 为 File 时使用。

**功能特性：**
- 文件选择界面
- 上传进度显示
- 上传状态反馈（上传中、成功、失败）
- 已上传文件列表
- 文件删除功能
- 自动将文件路径存储到 FormField.value

## 使用示例

### 基础用法

```rust
use crate::components::ui::form::{FormField, FieldType, DaisyForm};
use crate::components::ui::file_input::FileInfo;

// 定义上传回调
let handle_upload = move |file_info: FileInfo| {
    leptos::logging::log!("上传文件: {}", file_info.name);
    // 调用实际的上传 API
    // let file_path = upload_to_server(file_info).await;
};

// 创建文件字段
let form_fields = vec![
    FormField {
        name: "avatar".to_string(),
        label: "头像".to_string(),
        field_type: FieldType::File {
            accept: Some("image/*".to_string()),
            multiple: false,
            max_size: Some(5 * 1024 * 1024), // 5MB
            on_upload: Some(Callback::new(handle_upload)),
        },
        required: false,
        value: ArcRwSignal::new(String::new()), // 存储文件路径
        placeholder: None,
        error_message: ArcRwSignal::new(None),
        validation: None,
    },
];

// 表单提交处理
let handle_submit = move |fields: Vec<FormField>| async move {
    // fields[0].value.get() 包含上传后的文件路径
    let avatar_path = fields[0].value.get();
    leptos::logging::log!("头像路径: {}", avatar_path);
    Ok(())
};

view! {
    <DaisyForm
        initial_fields=form_fields
        on_submit=handle_submit
    />
}
```

### 多文件上传

```rust
FormField {
    name: "documents".to_string(),
    label: "文档".to_string(),
    field_type: FieldType::File {
        accept: Some(".pdf,.doc,.docx".to_string()),
        multiple: true,  // 支持多文件
        max_size: Some(10 * 1024 * 1024),
        on_upload: Some(Callback::new(handle_document_upload)),
    },
    required: false,
    value: ArcRwSignal::new(String::new()), // 多文件路径用逗号分隔
    // ... 其他字段
}
```

### 与后端集成

```rust
let handle_upload = move |file_info: FileInfo| {
    leptos::task::spawn_local(async move {
        // 调用后端上传 API
        match upload_file_to_backend(file_info).await {
            Ok(response) => {
                // response.file_path 包含上传后的文件路径
                leptos::logging::log!("上传成功: {}", response.file_path);
                // 文件路径会自动存储到 FormField.value 中
            }
            Err(error) => {
                leptos::logging::error!("上传失败: {}", error);
            }
        }
    });
};
```

## 文件类型配置

### 常用 accept 值

```rust
// 图片文件
accept: Some("image/*".to_string())
accept: Some("image/jpeg,image/png,image/gif".to_string())

// 文档文件
accept: Some(".pdf,.doc,.docx,.txt".to_string())
accept: Some("application/pdf,application/msword".to_string())

// 所有文件
accept: Some("*/*".to_string())
```

### 文件大小限制

```rust
max_size: Some(5 * 1024 * 1024)    // 5MB
max_size: Some(10 * 1024 * 1024)   // 10MB
max_size: Some(100 * 1024 * 1024)  // 100MB
```

## 组件状态管理

### 上传状态

- `ready`: 准备上传
- `uploading`: 上传中
- `success`: 上传成功
- `error`: 上传失败

### 文件路径存储

- **单文件**: 直接存储文件路径字符串
- **多文件**: 用逗号分隔的路径字符串，如 `"/uploads/file1.pdf,/uploads/file2.doc"`

## 样式定制

组件使用 DaisyUI 类名，支持主题定制：

```rust
// 上传状态样式
.text-info     // 上传中
.text-success  // 上传成功  
.text-error    // 上传失败

// 文件列表样式
.bg-base-200   // 文件列表背景
.btn-ghost     // 删除按钮
```

## 最佳实践

### 1. 上传回调处理

```rust
let handle_upload = move |file_info: FileInfo| {
    leptos::task::spawn_local(async move {
        // 显示上传进度
        set_upload_status("uploading");
        
        // 调用上传 API
        match api::upload_file(file_info).await {
            Ok(response) => {
                set_upload_status("success");
                // 文件路径自动存储到表单字段
            }
            Err(error) => {
                set_upload_status("error");
                show_error_message(error.to_string());
            }
        }
    });
};
```

### 2. 表单验证

```rust
FormField {
    // ... 其他配置
    validation: Some(ValidationRule::Custom(CustomValidator::new(|value: &str| {
        if value.is_empty() {
            Err("请上传文件".to_string())
        } else {
            Ok(())
        }
    }))),
}
```

### 3. 错误处理

```rust
// 在 on_upload 回调中处理错误
let handle_upload = move |file_info: FileInfo| {
    if file_info.size > MAX_FILE_SIZE {
        show_error("文件大小超过限制");
        return;
    }
    
    // 继续上传处理...
};
```

## 技术实现

### 核心组件

1. **FileFieldInput**: 文件字段输入组件
2. **SimpleFileInput**: 简化的文件选择组件
3. **FileInput**: 完整功能的文件上传组件
4. **AvatarUpload**: 专用头像上传组件

### 数据流

```
用户选择文件 → FileFieldInput → on_upload 回调 → 后端 API → 返回文件路径 → 存储到 FormField.value → 表单提交
```

### 线程安全

- 使用 `Vec<u8>` 存储文件数据而不是 `web_sys::File`
- 所有信号使用 `ArcRwSignal` 确保线程安全
- 兼容 Leptos 响应式系统

这种设计确保了文件上传功能的正确性和可维护性，符合表单系统的设计原则。