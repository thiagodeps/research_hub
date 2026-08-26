# Feature Specification: Portal Admin e Gestão de Pesquisa

**Feature Branch**: `001-admin-portal`  
**Created**: 2026-08-26  
**Status**: Draft  
**Input**: User description: "Construir o Portal do Professor IFES Serra: uma aplicação web para gestão e correção de dados de pesquisa acadêmica..."

## Clarifications

### Session 2026-08-26
- Q: Como as credenciais iniciais do administrador serão provisionadas no sistema? → A: Seed no Banco de Dados (via script/migração).
- Q: Qual o comportamento ao excluir entidade com dependentes? → A: Restrição (Bloqueio) - exige remoção manual dos dependentes.
- Q: O que acontece com os registros originais após a Fusão? → A: Soft Delete (Inativar) - ganham flag de inativos preservando histórico.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Autenticação do Admin (Priority: P1)

Como Administrador, desejo fazer login no sistema utilizando a tela de design split-screen para acessar as ferramentas de gestão.

**Why this priority**: Sem autenticação, o painel de administração não pode ser acessado de forma segura.

**Independent Test**: Pode ser testado validando o formulário de login no painel direito da tela split-screen e conferindo o redirecionamento após o sucesso na autenticação.

**Acceptance Scenarios**:
1. **Given** a tela de login inicial, **When** eu insiro credenciais válidas, **Then** sou redirecionado ao painel do CRUD.
2. **Given** a tela de login, **When** eu insiro uma senha incorreta, **Then** vejo uma mensagem de erro clara.

---

### User Story 2 - Gestão de Dados (CRUD Completo) (Priority: P1)

Como Administrador, desejo listar, visualizar, criar, editar e deletar instâncias das diversas entidades de pesquisa (Researcher, University, Campus, etc.) sem passar por fluxo de aprovação.

**Why this priority**: É o valor principal do portal nesta fase (gestão dos dados acadêmicos).

**Independent Test**: Pode ser testado interagindo com a tabela de listagem e com os formulários de criação/edição das entidades no dashboard.

**Acceptance Scenarios**:
1. **Given** o painel logado, **When** clico para criar uma "University", **Then** um formulário é exibido.
2. **Given** um formulário de entidade preenchido corretamente, **When** eu salvo, **Then** o registro é persistido e aparece na listagem.
3. **Given** um registro existente, **When** clico em deletar, **Then** o registro é removido.

---

### User Story 3 - Operações Avançadas (Fusão e Herança) (Priority: P2)

Como Administrador, desejo fundir registros duplicados e criar ligações ("herança") entre registros distintos (ex: Article para Advisorship) para corrigir dados do sistema.

**Why this priority**: É a operação complexa de curadoria de dados prometida pelos cards do painel esquerdo da UI.

**Independent Test**: Pode ser testado através de mock data selecionando dois registros e operando a fusão via UI.

**Acceptance Scenarios**:
1. **Given** a visão de listagem de Researchers, **When** seleciono 2 pesquisadores para Fusão, **Then** o sistema me permite escolher os dados conflitantes a preservar em um novo registro único.
2. **Given** um artigo, **When** seleciono a ação de herança/ligação, **Then** consigo associá-lo a uma orientação (Advisorship) existente.

---

### Edge Cases
- Tentativa de exclusão com dependentes: O sistema bloqueia a ação (ex: excluir University que possui Campus) e exibe mensagem de erro alertando o Admin a excluir ou desvincular os dependentes primeiro.
- O que ocorre na operação de Fusão se a escolha de campos resultar na quebra de constraints únicas do banco de dados?
- O que acontece se a API demorar para realizar a Fusão de registros muito grandes? (UI deve ter feedback de carregamento claro).

## Requirements *(mandatory)*

### Technical & Architectural Constraints
- **CON-001**: O frontend DEVE ser desenvolvido em Astro e ser compatível com GitHub Pages.
- **CON-002**: O backend DEVE ser desenvolvido em Python e reutilizar a biblioteca `research_domain`.
- **CON-003**: TODAS as entidades (Researcher, University, etc.) DEVEM possuir CRUD completo operado diretamente por Admin.
- **CON-004**: O design visual DEVE seguir o protótipo Figma "Sistema de Login para Professores e Admins" (painel esquerdo navy com cards, painel direito claro).

### Functional Requirements

- **FR-001**: O sistema DEVE fornecer uma tela de autenticação split-screen onde o admin insere e-mail e senha.
- **FR-002**: O sistema DEVE proteger todas as rotas internas, redirecionando usuários não autenticados para o login.
- **FR-003**: O sistema DEVE fornecer operações de criar, ler, atualizar e excluir (CRUD) para as entidades: Researcher, University, Campus, ResearchGroup, KnowledgeArea, Advisorship, Fellowship, AcademicEducation, Article, ResearchProduction, EducationType e ProductionType.
- **FR-004**: O sistema DEVE permitir a Fusão de dois registros do mesmo tipo de entidade.
- **FR-005**: O sistema DEVE permitir a criação de vínculos manuais (Herança) entre registros de entidades diferentes compatíveis com a regra de domínio.
- **FR-006**: O sistema DEVE executar as modificações diretamente e sem fluxo de aprovação (Approval workflow out of scope).
- **FR-007**: O usuário Administrador inicial DEVE ser provisionado via seed/migração no banco de dados, sem interface de registro.
- **FR-008**: O sistema DEVE bloquear exclusões de registros que possuem dependentes (restrição), emitindo um erro claro exigindo desvinculação/remoção prévia.
- **FR-009**: Após uma Fusão bem-sucedida, os 2 registros originais DEVEM sofrer "Soft Delete" (inativação) preservando seu histórico, em vez de serem excluídos fisicamente.

### Key Entities

- **Researcher**: Representa um pesquisador/professor. Atributos chave: nome, e-mails, resumo/currículo.
- **University**: Instituição de ensino. Atributos chave: nome, sigla.
- **Campus**: Vinculado a University, com nome.
- **ResearchGroup**: Vinculado a Campus e University, possui nome e sigla.
- **Advisorship**: Orientação. Atributos chave: nome, aluno, data início, data fim/cancelamento.
- **Article**: Produção científica específica. Atributos chave: título, ano, tipo, autores, DOI.
- (Outros detalhamentos presentes no schema do pacote Python `research_domain`).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: O Administrador consegue completar a ação de login e acessar o painel inicial de gestão em menos de 5 segundos.
- **SC-002**: As operações de CRUD para ao menos 2 entidades críticas (ex: Researcher e University) operam de ponta-a-ponta (do frontend ao DB) com retorno de sucesso nas requisições.
- **SC-003**: O Administrador completa um teste de fusão de 2 Researchers no ambiente de homologação reduzindo os 2 IDs a um único ID consolidado sem erros inesperados.
- **SC-004**: O layout final (HTML/CSS renderizado) adere em estrutura visual à divisão de 50/50 do protótipo Figma para a tela inicial.

## Assumptions

- O pacote de domínio Python `research_domain` (`The-Band-Solution/ResearchDomain`) possui todas as definições estruturais das entidades requisitadas.
- O sistema de autenticação base provê sessões persistentes seguras (JWT ou cookie) sem precisar de integração complexa de terceiros nesta etapa.
- A restrição de GitHub Pages implica em deploy como um Static Site Generation (SSG) ou Single Page Application (SPA), onde o backend servirá a API via servidor à parte ou subdomínio/URL diferente de base (CORS configurado apropriadamente).
