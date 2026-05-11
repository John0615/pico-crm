# Pico-CRM（家政版）

> 用 Rust 构建的家政行业定制化 CRM —— 稳定、高效、可按需扩展

A vertical CRM for housekeeping services, built with Rust and Leptos SSR. Designed for small-to-medium家政 companies managing 5–100 service staff. Core flow: customer profiles → service requests → orders → scheduling → fulfillment → after-sales.

[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)

Single PostgreSQL database with shared tables scoped by `merchant_id` for tenant isolation. Event-sourced core aggregates (Order, Schedule, ServiceRequest) with CQRS read-model projections.

## Highlights

- Single PostgreSQL database with shared-table merchant isolation
- Leptos SSR + Hydration with shared Rust types
- Clear role boundaries: admin / merchant operator / worker
- Mobile-first UI with simple interaction patterns

## Architecture Direction

- The project is being simplified away from schema-per-merchant multi-tenancy.
- The target model is shared business tables with explicit `merchant_id` ownership, role-based access control, and merchant-scoped auditing.
- The migration plan is documented in `openspec/changes/shared-schema-architecture/`.

## Tech Stack

- Rust, Leptos (SSR + WASM), Axum, SeaORM
- PostgreSQL 15+
- Tailwind (style/main.scss, style/tailwind.css)

## Project Structure

- `app/` Leptos pages, components, routes, server functions
- `frontend/` WASM entry and client hydration
- `server/` Axum + Leptos SSR entry and middleware
- `backend/` domain/application/infrastructure layers
- `migration/` SeaORM migrations
- `shared/` shared DTOs and types
- `style/` styles and Tailwind input
- `public/` static assets

## Quick Start

### Requirements

- Rust (stable) + `cargo-leptos`
- Node.js + npm (for Tailwind/build tooling)
- PostgreSQL 15+

### Setup

```bash
# 1) env
cp .env.example .env.dev

# 2) start postgres (podman or docker)
sudo podman rm -f pico-crm-pg
sudo podman run --name pico-crm-pg -e POSTGRES_PASSWORD=postgres -e POSTGRES_DB=pico_crm_dev -p 5432:5432 -d postgres:latest

# 3) install node deps
npm i

# 4) run dev server (SSR + CSR)
cargo leptos watch --split
```

### Build

```bash
cargo leptos build --release --split
./scripts/optimize-wasm-release.sh
```

`cargo-leptos 0.3.x` in this repo currently runs `wasm-opt` without `-Oz`, so the extra script applies a stronger size-focused pass across all generated `.wasm` files.

### Deploy
```bash
LEPTOS_SITE_ROOT=./site ./server
```

### Notes

- The server loads `.env.{APP_ENV}` (default: `dev`).
- Migrations are executed on server startup.
- The repository still contains legacy multi-schema implementation pieces; the target architecture is documented first and will be implemented incrementally.

### Environment Variables

- `APP_ENV`: dev / prod
- `DATABASE_URL`: PostgreSQL connection string
- `JWT_SECRET`: JWT signing secret
- `JWT_EXPIRY_HOURS`: access token expiry (hours)
- `JWT_REFRESH_EXPIRY_DAYS`: refresh token expiry (days)
- `UPLOAD_BUCKET` / `UPLOAD_REGION`: object storage config (optional)
- `SMS_API_KEY`: SMS provider key (optional)
- `ADMIN_TRIAL_DAYS_DEFAULT`: default trial length (optional)
- `ADMIN_SMS_TEMPLATE_ID`: SMS template identifier (optional)

## 联系与咨询

- **Bug 反馈 & 功能建议**：[GitHub Issues](https://github.com/John0615/pico-crm/issues)
- **定制开发**：有家政行业定制需求（私有部署、功能定制、行业适配），请邮件联系：**zhaosp0615@88.com**。

## Contributing

See `CONTRIBUTING.md`.

## Security

See `SECURITY.md`.

## License

This project is released under the Apache-2.0 license. See `LICENSE`.

## DDD Notes

### Layering Principles

- `domain/` contains pure business logic and domain behavior only
- Domain models do not depend on serialization or persistence details
- Infrastructure handles entity <-> DTO conversion
- Application layer handles domain <-> DTO conversion when needed
- Dependency direction: `shared` <- `application` <- `domain` <- `infrastructure`

### Call Flow (Server Functions)

- app server function
- calls `backend::application::services::*`
- calls `backend::domain::services::*`
- calls `backend::infrastructure::repositories::*`
- uses `backend::infrastructure::mappers::*`
- returns `shared::dtos::*`
