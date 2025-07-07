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

## 交互中心
graph TB
    IC[交互中心] -->|默认视图| CL[智能客户列表]
    IC -->|点击客户| CD[客户详情面板]
    CD --> COM[沟通记录]
    CD --> TASK[关联任务]
    CD --> REL[关系图谱入口]
## 智能看板
+-----------------------------------+
| 今日行动清单 (AI生成)              |
| 1. 联系王伟（报价已超48小时） [拨打] |
| 2. 给张莉发送合同 [微信]           |
+-----------------------------------+
| 客户健康度                         |
| 高风险: 2人  (李强, 周明) [查看]    |
| 待开发: 5人  [名单]                |
+-----------------------------------+
| 本周预测                           |
| 预计收入: ¥28,000 (±15%)          |
| 最可能成交: 科讯项目 (72%) [详情]   |
+-----------------------------------+



## 快速开始

### 开发环境
```bash
# 克隆项目
git clone https://github.com/yourname/pico-crm.git

# 启动开发服务器
cargo leptos watch

# 生成数据库迁移脚本(示例)
sea-orm-cli migrate generate create_table_users

# 生成实体
sea-orm-cli generate entity -u postgres://postgres:postgres@localhost:5432/pico_crm_dev -o backend/src/entity
