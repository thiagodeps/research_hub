# Feature Specification: Curadoria e Edição de Parquets (Import/Export)

**Feature Branch**: `005-parquet-curation-tool`
**Created**: 2026-08-26
**Status**: Draft
**User Input**: "precisamos fazer um salvador e tbm n importa diretamente , e sim ao escolher arquivo , pq a ideia do programa e para editar esses dados , pois caso tenha algo errado neles corrigir para facilitar o trabalho de outro projeto"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Importação de Arquivos pelo Usuário (Priority: P1)
Como Administrador de Dados, quero poder escolher e enviar um arquivo (`.parquet` ou um arquivo compactado `.zip` contendo os dados) por meio da interface gráfica, para que esses dados substituam a base atual e eu possa começar a corrigir anomalias nas tabelas.

**Acceptance Scenarios**:
1. **Given** a página inicial do Dashboard, **When** eu clicar em "Importar Dados" e enviar um arquivo válido, **Then** o sistema deve processar o arquivo, limpar o banco de dados atual, e preencher as tabelas com o conteúdo do arquivo enviado.
2. **Given** o processo de importação concluído, **When** eu navegar para as abas de "Grupos de Pesquisa" ou "Artigos", **Then** devo ver exatamente os dados que constavam no arquivo original que fiz upload.

### User Story 2 - Exportação de Arquivos Corrigidos (Priority: P1)
Como Administrador de Dados, quero exportar o estado atual do banco de dados de volta para o formato Parquet (`.parquet`) clicando em um botão, para que eu possa entregar os arquivos limpos e corrigidos para a pipeline de Inteligência Artificial do outro projeto.

**Acceptance Scenarios**:
1. **Given** que fiz edições e correções textuais em um "Pesquisador", **When** eu clicar em "Exportar Banco de Dados", **Then** devo receber um download de um arquivo compactado `.zip` (ou os parquets puros) contendo todos os dados, refletindo perfeitamente a minha correção.

## Requirements *(mandatory)*

### Technical Constraints
- **CON-001**: O upload e processamento do Parquet deve ser feito utilizando a biblioteca `pandas` ou `duckdb` instalada no backend, convertendo o dataframe para dicionários e chamando `save` no banco, ou via carga em massa no SQLAlchemy.
- **CON-002**: A UI deve possuir botões visíveis de "Importar" (com input de arquivo) e "Exportar" na tela principal `index.astro` do Dashboard.
- **CON-003**: O backend do FastAPI deve ter um endpoint `POST /api/v1/import` aceitando um `UploadFile` (multipart/form-data).
- **CON-004**: O backend do FastAPI deve ter um endpoint `GET /api/v1/export` que gera o binário `.parquet` na memória (ou temporário) e o envia como um `FileResponse`.

### Functional Requirements
- **FR-001**: O sistema deve ser capaz de importar um ZIP contendo múltiplos `.parquet` e rotear os dados para suas respectivas tabelas (ex: se for `articles_*.parquet`, salvar em `articles`).
- **FR-002**: A exportação deve gerar um `.zip` espelhando a estrutura original, convertendo as tabelas do SQL para `.parquet` novamente.

## Success Criteria *(mandatory)*

- **SC-001**: O fluxo completo (Round-trip) funciona sem perdas de coluna: Upload de um `.parquet` -> Edição na UI -> Download do novo `.parquet` gerado e a alteração está nele.
- **SC-002**: Nenhum acesso direto ao terminal será necessário pelo usuário final para manipular esses dados; tudo deve estar na Interface Web.
