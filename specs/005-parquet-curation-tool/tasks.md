# Feature Implementation Tasks: Curadoria e Edição de Parquets

## Phase 1: Engine de Arquivos (Backend)

**Purpose**: Habilitar a API a ler, converter e baixar arquivos ZIP/Parquet.

- [ ] T001 Instalar dependência de upload `python-multipart` no backend.
- [ ] T002 Criar `backend/src/services/parquet_service.py` contendo uma classe capaz de extrair um Zip via memória, iterar arquivos, limpar tabelas do DB e usar Pandas para salvar DataFrames.
- [ ] T003 Adicionar função `export_zip` no `parquet_service.py` que gere um ZIP em memória lendo de `Base.metadata.tables` usando SQLAlchemy engine via Pandas e retornando o buffer bytes.
- [ ] T004 Criar router `backend/src/api/routes/data.py` (ou atualizar o principal) adicionando endpoints de `POST /api/v1/data/import` e `GET /api/v1/data/export`.
- [ ] T005 Acoplar a nova rota `data.py` no `backend/src/api/main.py`.

---

## Phase 2: Interface "Control Center" (Frontend)

**Purpose**: Prover a UI para o usuário enviar o arquivo de curadoria.

- [ ] T006 Adicionar `react-dropzone` ou input tradicional estilizado no componente `frontend/src/pages/dashboard/index.astro`.
- [ ] T007 Criar um componente React (`frontend/src/components/DataControlCenter.jsx`) para gerenciar upload (com loading spinner e error handling).
- [ ] T008 O Componente React deve também possuir o botão `Exportar` acionando `<a href=".../export" download>`.

## Dependencies & Execution Order
- T001 e T002 vêm primeiro, estabelecendo o motor lógico.
- Frontend (Phase 2) acopla aos endpoints no backend (Phase 1).
