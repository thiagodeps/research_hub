# Feature Specification: Table Page Navigation

**Feature Branch**: `028-table-page-nav`  
**Created**: 2026-09-10  
**Status**: Draft  
**Input**: User description: "ao navegar nas abas da tableas , so existe como navegar em abas clicando em proximo ou anterior , gostaria que pudesse navegar pelo numero da aba da tabela"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Navigate to a specific page number (Priority: P1)

As a user viewing a paginated table, I want to be able to jump to a specific page number directly, instead of repeatedly clicking "next" or "previous", so that I can quickly reach the data I am looking for.

**Why this priority**: Navigating large datasets is tedious if the user can only go one page at a time. This is the core functionality requested.

**Independent Test**: Can be fully tested by opening any paginated table with multiple pages, entering a valid page number in the pagination control, and verifying the table updates to display that exact page.

**Acceptance Scenarios**:

1. **Given** a table with 10 pages and currently on page 1, **When** the user clicks on the page number input/selector and enters "5", **Then** the table should update to display the data for page 5, and the current page indicator should show "5".
2. **Given** a table with 10 pages, **When** the user tries to navigate to page "15" (out of bounds), **Then** the system should either prevent entering the invalid number, or gracefully fallback to the last valid page (page 10).
3. **Given** a table with 10 pages, **When** the user tries to navigate to page "0" or a negative number, **Then** the system should gracefully fallback to the first valid page (page 1).

---

### Edge Cases

- What happens when the user types non-numeric characters? (Should be prevented or ignored).
- What happens when there is only 1 page of data? (The page navigation control should probably be disabled or hidden).
- How does the system handle extremely large page numbers?

## Requirements *(mandatory)*

### Technical & Architectural Constraints

- **CON-001**: A aplicação DEVE ser um app desktop de processo único em Tauri 2.0, com núcleo em Rust e front-end em Astro/React/Tailwind compilado estaticamente e embarcado no binário, seguindo o protótipo Figma. Funciona integralmente sem rede.
- **CON-002**: A persistência DEVE usar SQLite em arquivo único, embarcado (`rusqlite`, feature `bundled`), no diretório de dados da aplicação. É proibido qualquer motor que exija servidor ou instalação separada.
- **CON-003**: Toda regra de negócio DEVE residir no processo Rust. É proibido SQL, acesso a arquivos ou duplicação de regra no JavaScript; a comunicação se dá por IPC do Tauri, concentrada em um único módulo do front-end.
- **CON-004**: Todas as entidades do domínio exigem CRUD completo operado unicamente por perfil de Admin (sem fluxo de aprovação).
- **CON-005**: Paridade funcional precede melhoria. Correções de comportamento preexistente DEVEM ser declaradas como escopo explícito desta spec, nunca aplicadas em silêncio.
- **CON-006**: Alvos de release são Linux e Windows. macOS, assinatura de código, auto-update e bancos não-SQLite estão fora de escopo por decisão de arquitetura.

### Functional Requirements

- **FR-001**: System MUST allow users to input or select a specific page number to navigate to in any paginated table.
- **FR-002**: System MUST visually indicate the current page number and the total number of pages.
- **FR-003**: System MUST prevent navigation to page numbers less than 1 or greater than the total number of pages.
- **FR-004**: System MUST ignore or prevent input of non-numeric characters in the page number field.

### Key Entities *(include if feature involves data)*

- **Pagination Control**: The UI component that handles the pagination state (current page, total pages) and triggers navigation events.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can successfully jump to any valid page number in a paginated table in a single action.
- **SC-002**: Invalid page numbers (out of bounds, non-numeric) are handled without crashing the application or showing empty data states when data exists.

## Assumptions

- The frontend uses a reusable pagination component for tables, so implementing this feature will apply to all tables in the application.
- The backend API for fetching table data already supports arbitrary page offsets/numbers, so no backend changes are required to support jumping to a specific page.
