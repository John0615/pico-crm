# Pico-CRM (Housekeeping Edition)

Lightweight, multi-tenant CRM built with Rust and Leptos. The MVP targets the core flow: customers -> orders -> schedules -> completion.

## Status

MVP in progress.

## Highlights

- Single PostgreSQL instance, schema-per-tenant isolation
- Leptos SSR + Hydration with shared Rust types
- Clear role boundaries: admin / merchant operator / worker
- Mobile-first UI with simple interaction patterns

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
podman rm -f pico-crm-pg
podman run --name pico-crm-pg -e POSTGRES_PASSWORD=postgres -e POSTGRES_DB=pico_crm_dev -p 5432:5432 -d postgres:latest

# 3) install node deps
npm i

# 4) run dev server (SSR + CSR)
cargo leptos watch --split
```

### Build

```bash
cargo leptos build --release --split
```

### Notes

- The server loads `.env.{APP_ENV}` (default: `dev`).
- Migrations are executed on server startup.

### Environment Variables

- `APP_ENV`: dev / prod
- `DATABASE_URL`: PostgreSQL connection string
- `JWT_SECRET`: JWT signing secret
- `JWT_EXPIRY_HOURS`: access token expiry (hours)
- `JWT_REFRESH_EXPIRY_DAYS`: refresh token expiry (days)
- `TENANT_SCHEMA_PREFIX`: tenant schema prefix (default `merchant_`)
- `UPLOAD_BUCKET` / `UPLOAD_REGION`: object storage config (optional)
- `SMS_API_KEY`: SMS provider key (optional)
- `ADMIN_TRIAL_DAYS_DEFAULT`: default trial length (optional)
- `ADMIN_SMS_TEMPLATE_ID`: SMS template identifier (optional)

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
