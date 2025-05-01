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

## 快速开始

### 开发环境
```bash
# 克隆项目
git clone https://github.com/yourname/pico-crm.git

# 启动开发服务器
cargo leptos watch
