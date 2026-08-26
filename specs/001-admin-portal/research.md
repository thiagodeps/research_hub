# Research & Technical Decisions: Portal Admin e Gestão de Pesquisa

## Decision 1: Stack Backend
- **Decision**: Python com FastAPI e PostgreSQL (prod) / Memória (dev).
- **Rationale**: Requisito explícito para permitir integração natural com a biblioteca `research_domain` desenvolvida em Python. FastAPI oferece performance, validação automática com Pydantic, e documentação interativa ideal para consumir no frontend. O uso de memória em dev agiliza o TDD inicial.
- **Alternatives considered**: Django (descartado por sobrecarregar o modelo de domínio já existente), Flask (descartado em favor da tipagem e validação do FastAPI).

## Decision 2: Stack Frontend
- **Decision**: Astro consumindo API via fetch, hospedado no GitHub Pages.
- **Rationale**: Requisito explícito de arquitetura. Astro permite excelente performance (SSG) e flexibilidade para uso de componentes framework-agnostic. GitHub Pages oferece hospedagem estática confiável e gratuita.
- **Alternatives considered**: React/Next.js (descartado devido à preferência por Astro para o output estático), Vue/Nuxt.

## Decision 3: Autenticação
- **Decision**: JWT (JSON Web Tokens) na API com um único perfil (Admin).
- **Rationale**: Requisito explícito. Permite que o frontend Astro gerencie o token localmente e acesse endpoints protegidos. Simplifica a fase atual que requer apenas um papel (role).

## Decision 4: Padrões de Interface UI
- **Decision**: Componentes reutilizáveis de Tabela e Formulário.
- **Rationale**: Como existem 12 entidades de domínio diferentes que exigem operações CRUD padronizadas, criar um `EntityGrid` dinâmico e formulários baseados em metadata/schema poupará trabalho redundante e facilitará a manutenção do painel Admin.
