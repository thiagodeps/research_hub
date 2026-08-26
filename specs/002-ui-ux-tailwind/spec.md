# Feature Specification: Refinamento de UI/UX com Tailwind CSS

**Feature Branch**: `002-ui-ux-tailwind`  
**Created**: 2026-08-26  
**Status**: Draft  
**Input**: User description: "Refinar o design UI/UX da aplicação web utilizando Tailwind CSS, tornando a interface moderna, responsiva e atrativa (login com split-screen aprimorado e tabelas com design polido)."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Aprimoramento da Tela de Login (Priority: P1)

Como Administrador, quero acessar uma tela de login moderna com design "split-screen" atrativo, responsiva e alinhada com as melhores práticas de UX, para ter uma ótima primeira impressão do sistema.

**Why this priority**: A tela de login é a porta de entrada. Um design impecável ali dita o tom do sistema inteiro e aumenta a confiança do usuário.

**Independent Test**: Pode ser testado visualmente acessando a rota `/login` no frontend (com e sem interação de inputs) para validar responsividade, foco e contrastes.

**Acceptance Scenarios**:

1. **Given** a tela de login acessada via desktop, **When** visualizada, **Then** a tela divide o conteúdo perfeitamente, exibindo formulário de um lado e um banner visual/branding do outro.
2. **Given** a tela acessada via dispositivo móvel, **When** visualizada, **Then** o layout se adapta para coluna única, focando no formulário.
3. **Given** os campos de email e senha, **When** o usuário interage (focus/hover), **Then** há feedback visual instantâneo e amigável.

---

### User Story 2 - Dashboard e Tabelas de Entidades (Priority: P2)

Como Administrador, quero ver a listagem de entidades (CRUD) em tabelas polidas, bem espaçadas e com ações claras e acessíveis, para gerenciar milhares de registros sem cansaço visual.

**Why this priority**: As tabelas compõem 90% da experiência principal do admin. Se elas forem feias ou mal estruturadas, a usabilidade cai drasticamente.

**Independent Test**: Testado carregando e visualizando as listagens no `/dashboard/universities` (ou similares), validando se as margens, espaçamentos, tipografia e botões estão estilizados com Tailwind CSS.

**Acceptance Scenarios**:

1. **Given** o painel de listagem de entidades, **When** uma tabela é renderizada, **Then** ela exibe divisórias suaves, cabeçalho em destaque e bom espaçamento (padding) para cada célula.
2. **Given** a coluna de "Ações" (Editar, Deletar, etc), **When** os botões são renderizados, **Then** eles devem ser exibidos de forma estilizada, como botões ou ícones agradáveis.

---

### User Story 3 - Modais Modernos (Priority: P3)

Como Administrador, quero que os fluxos secundários (ex: Fusão, Link, Edição) ocorram em modais sobrepostos polidos (overlays), para que minha atenção não saia da listagem em background.

**Why this priority**: Modais encerram a experiência do painel administrativo. Design atraente para popups dá a percepção de aplicação rica.

**Independent Test**: Testado interagindo com os botões "Vincular" ou "Fundir" e validando que o modal aparece como overlay fluido com animações ou estilo refinado.

**Acceptance Scenarios**:

1. **Given** a tela de entidades, **When** acionada a Fusão, **Then** o sistema exibe o conteúdo de fusão em um fundo escurecido (backdrop) com card centralizado.

### Edge Cases

- Como o layout responde em tablets (768px de largura)?
- O que acontece quando os nomes das entidades na tabela são gigantescos (text overflow)?
- Mensagens de erro de Validação (ex: "ID não encontrado", "CORS Error") estão visíveis e estilizadas adequadamente?

## Requirements *(mandatory)*

### Technical & Architectural Constraints

- **CON-001**: O refinamento visual DEVE utilizar **Tailwind CSS**.
- **CON-002**: Não deve haver quebra ou mudança de rotas backend (trata-se apenas de visual frontend).
- **CON-003**: Frontend Astro deve compilar sem erros após a inclusão das dependências do Tailwind.

### Functional Requirements

- **FR-001**: O sistema DEVE utilizar as classes utilitárias do Tailwind CSS em todos os layouts e telas da aplicação.
- **FR-002**: O sistema DEVE estilizar a tela de login no padrão "split-screen" (tela dividida) em dispositivos desktop e com "stacking" em mobile.
- **FR-003**: As tabelas de entidades DEVEM possuir design polido, com "hover" nas linhas, cabeçalho de destaque e paginação visual (se aplicável).
- **FR-004**: Botões e inputs (campos de texto, selects) DEVEM possuir feedback de `:focus` e `:hover` para acessibilidade e usabilidade.

### Key Entities

Nenhum modelo de dados será alterado nesta feature. Esta feature trata apenas da camada de apresentação (View).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Lighthouse Accessibility Score deve ser superior a 90, sem falhas de contraste de cores severas.
- **SC-002**: A interface exibe responsividade em resoluções mobile (menos de 640px) e desktop (acima de 1024px) sem elementos saindo fora da tela (overflow horizontal nulo no corpo da página).
- **SC-003**: Nenhuma classe de estilo estática codificada manualmente ("inline styles") deve sobrar nas páginas da aplicação, sendo 100% controladas pelo sistema de design CSS.

## Assumptions

- O design system não requer um plugin complexo do Tailwind, apenas as configurações padrão do framework (Cores, Espaçamentos padrão).
- Não haverá criação de novas funcionalidades de backend, foco 100% visual.
- A paleta de cores padrão será algo moderno (como Indigo/Slate/Blue para painéis admins) já que o usuário pediu design moderno, responsivo e atrativo sem estipular uma marca específica.
