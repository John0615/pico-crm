# Contributing

Thanks for your interest in contributing.

## Ways to Contribute

- Bug reports and reproduction steps
- Documentation improvements
- Code changes and tests

## Before You Start

- Search existing issues and PRs
- Keep changes focused and small

## Development Setup

```bash
cp .env.example .env.dev
npm i
cargo leptos watch --split
```

## Code Style

- Run `cargo fmt` before submitting
- Run `cargo clippy -- -D warnings`

## Tests

- `cargo test`
- Add tests for new behavior when practical

## Pull Requests

- Use clear titles and descriptions
- Include context and screenshots for UI changes
- Make sure CI passes

## Security

Do not report security issues in public issues. See `SECURITY.md`.
