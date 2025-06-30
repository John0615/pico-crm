# pico-cRM - 轻量级客户关系管理系统

**基于 Rust 全栈技术构建的高性能 CRM 系统**
🚀 零成本抽象 · ⚡ 极致性能 · � 模块化设计

## 技术栈

### 核心框架
- **前端**: Leptos (SSR + WASM)
- **后端**: Axum + Tokio
- **数据库**: PostgreSQL
- **样式**: TailwindCSS

### 关键库
- 状态管理: `leptos-signals`
- ORM: `SeaORM`
- 表单验证: `validator`

## 功能特性

### 客户管理
- 客户信息 CRUD
- 标签分类系统
- 交互历史追踪

### 销售自动化
- 销售管道可视化
- 智能提醒系统
- 业绩分析看板

### 系统功能
- JWT 身份验证
- RBAC 权限控制
- 数据导出 (CSV/Excel)

### 后端调用关系
- app server function
- 调用 backend::application::services::service_a
- 调用 backend::domain::services::domain_service_a
- 调用 backend::infranstructure::repositories::repository_a
- 使用 backend::infranstructure::mappers::mapper_a
- 返回 shared::dtos::dto_a

### 关键原则
## 领域纯洁性
- domain/models/ 只包含业务逻辑和领域行为
- 完全不涉及任何序列化/持久化相关逻辑
## 转换责任
- 数据库实体->DTO的转换属于技术设施层的职责
- 领域模型->DTO的转换可以放在应用层
## 依赖方向
- 共享DTO <- 应用层 <- 领域层 <-基础设施层

## 快速开始

### 开发环境
```bash
# 克隆项目
git clone https://github.com/yourname/pico-crm.git

# 启动开发服务器
cargo leptos watch
