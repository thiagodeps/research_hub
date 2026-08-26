# Implementation Plan: [FEATURE]

**Branch**: `[###-feature-name]` | **Date**: [DATE] | **Spec**: [link]
**Input**: Feature specification from `/specs/[###-feature-name]/spec.md`

**Note**: This template is filled in by the `/speckit-plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

[Extract from feature spec: primary requirement + technical approach from research]

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: [e.g., Python 3.11, Swift 5.9, Rust 1.75 or NEEDS CLARIFICATION]  
**Primary Dependencies**: @astrojs/tailwind, React, Astro
**Storage**: N/A
**Testing**: Vitest, Playwright
**Target Platform**: GitHub Pages (Static Site)
**Project Type**: web-service
**Performance Goals**: <200ms LCP
**Constraints**: Tailwind CSS utility-first workflow
**Scale/Scope**: UI/UX Refactor

## Phase 0: Research & Context

- **Data Models**: Nenhuma alteração.
- **Interfaces**: Nenhuma alteração.
- **Technical Context**: 
  - Integração oficial do `@astrojs/tailwind` para gerenciar as classes utilitárias no projeto Astro.
  - O projeto continuará consumindo a API perfeitamente como já faz atualmente.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] TDD Adherence: Test planning is prioritized and tasks clearly follow the Red-Green-Refactor cycle.
- [x] ResearchDomain Reuse: No domain entities (Researcher, University, etc.) are being reimagined or duplicated.
- [x] Test Strategy: Backend specifies pytest (unit/integration) and frontend specifies Vitest/Playwright (components/E2E).
- [x] Architecture: Backend in Python, Frontend in Astro (deployable as static site on GitHub Pages).
- [x] Operations: Direct CRUD operations by Admin only, no approval flows. Identity/branding strictly follows Figma specs.
- [x] O frontend em Astro está mantido e sendo polido (Princípio V).
- [x] Os testes deverão continuar passando após o refatoramento da UI, exigindo atualização de asserções no Playwright e Vitest, caso dependam de estruturas antigas (Princípio I).
- [x] Nenhuma entidade do domínio do backend é afetada (Princípio II).

## Phase 1: Models & Contracts

- **Data Model**: `specs/002-ui-ux-tailwind/data-model.md`
- **Contracts**: `specs/002-ui-ux-tailwind/contracts/api.md`
- **Developer Setup**: `specs/002-ui-ux-tailwind/quickstart.md`

## Phase 2: Architecture & Component Design

**Frontend Architecture**:
- `astro.config.mjs`: Recebe a integração do `@astrojs/tailwind`.
- `tailwind.config.mjs`: Criado na raiz do frontend com possíveis definições de cores da marca (ex: paleta do IFES ou branding genérico azul escuro).
- Componentes React (`LoginForm.jsx`, `EntityTable.jsx`, `MergeModal.jsx`, `LinkModal.jsx`): Terão suas propriedades `style` removidas, adotando `className` do Tailwind (ex: `flex`, `grid`, `bg-blue-600`, `text-white`, `p-4`, `rounded`, `shadow`, etc).
- Layout Astro (`SplitScreen.astro` e `Dashboard.astro`): Receberão classes semânticas e base do body (tipografia, reset).

**Backend Architecture**:
- Intacto. Nenhuma mudança estrutural.

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/           # Phase 1 output (/speckit-plan command)
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)
<!--
  ACTION REQUIRED: Replace the placeholder tree below with the concrete layout
  for this feature. Delete unused options and expand the chosen structure with
  real paths (e.g., apps/admin, packages/something). The delivered plan must
  not include Option labels.
-->

```text
# [REMOVE IF UNUSED] Option 1: Single project (DEFAULT)
src/
├── models/
├── services/
├── cli/
└── lib/

tests/
├── contract/
├── integration/
└── unit/

# [REMOVE IF UNUSED] Option 2: Web application (when "frontend" + "backend" detected)
backend/
├── src/
│   ├── models/
│   ├── services/
│   └── api/
└── tests/

frontend/
├── src/
│   ├── components/
│   ├── pages/
│   └── services/
└── tests/

# [REMOVE IF UNUSED] Option 3: Mobile + API (when "iOS/Android" detected)
api/
└── [same as backend above]

ios/ or android/
└── [platform-specific structure: feature modules, UI flows, platform tests]
```

**Structure Decision**: [Document the selected structure and reference the real
directories captured above]

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |
