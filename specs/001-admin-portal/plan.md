# Implementation Plan: Portal Admin e Gestão de Pesquisa

**Branch**: `001-admin-portal` | **Date**: 2026-08-26 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `specs/001-admin-portal/spec.md`

## Summary

Construção do Portal do Professor IFES Serra: uma aplicação web para gestão de dados de pesquisa acadêmica focada no perfil Admin. Utiliza arquitetura monorepo com backend Python/FastAPI (consumindo o domínio `research_domain`) e frontend estático em Astro com design split-screen para login. A aplicação permite CRUD completo de todas as entidades do domínio, bem como operações avançadas de Fusão e Herança.

## Technical Context

**Language/Version**: Python 3.11+, TypeScript/JavaScript (Astro)  
**Primary Dependencies**: FastAPI, `research_domain`, Astro, JWT (Authentication)  
**Storage**: Em memória (dev) / PostgreSQL (prod)  
**Testing**: pytest (backend), Vitest/Playwright (frontend)  
**Target Platform**: GitHub Pages (Frontend Estático) e Render/Railway (Backend - a ser definido no deploy)  
**Project Type**: Web Application (REST API + Static Frontend)  
**Performance Goals**: Login e listagem em < 5s. Interface responsiva e com carregamento rápido.  
**Constraints**: Sem fluxo de aprovação. Soft delete para registros originais de Fusão. Bloqueio ao invés de cascade delete. Apenas um papel de usuário (Admin).  
**Scale/Scope**: Gestão interna por administradores da instituição.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] TDD Adherence: Test planning is prioritized and tasks clearly follow the Red-Green-Refactor cycle.
- [x] ResearchDomain Reuse: No domain entities (Researcher, University, etc.) are being reimagined or duplicated.
- [x] Test Strategy: Backend specifies pytest (unit/integration) and frontend specifies Vitest/Playwright (components/E2E).
- [x] Architecture: Backend in Python, Frontend in Astro (deployable as static site on GitHub Pages).
- [x] Operations: Direct CRUD operations by Admin only, no approval flows. Identity/branding strictly follows Figma specs.

## Project Structure

### Documentation (this feature)

```text
specs/001-admin-portal/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output
```

### Source Code (repository root)

```text
backend/
├── src/
│   ├── api/          # FastAPI routers & endpoints
│   ├── core/         # Config, security (JWT), exceptions
│   ├── services/     # App services (bridges to research_domain)
│   └── database/     # Memory/Postgres adapters
└── tests/
    ├── integration/
    └── unit/

frontend/
├── src/
│   ├── components/   # Reusable UI (Table, Form, Cards)
│   ├── layouts/      # Split-screen layout
│   ├── pages/        # Astro pages (Login, Dashboard, Entity CRUDs)
│   └── services/     # API fetch clients
└── tests/
    ├── e2e/          # Playwright tests
    └── unit/         # Vitest tests
```

**Structure Decision**: A abordagem monorepo (`backend/` e `frontend/` na raiz) foi selecionada conforme instrução do usuário. Separa claramente a API FastAPI do frontend Astro, permitindo o deploy independente (Frontend no GitHub Pages, Backend no Render/Railway).

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

Nenhuma violação encontrada. A arquitetura obedece perfeitamente à Constituição.
