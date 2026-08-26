# Research & Decisions

## Context
Refinamento da UI/UX utilizando Tailwind CSS para a aplicação Astro + React.

## Decisions

### 1. Integração do Tailwind CSS
- **Decision:** Utilizar a integração oficial `@astrojs/tailwind` para configurar o Tailwind globalmente no Astro.
- **Rationale:** Permite usar classes utilitárias tanto em arquivos `.astro` (layouts) quanto em componentes `.jsx` (React) de maneira transparente e otimizada.
- **Alternatives considered:** CSS customizado manual (descartado por falta de padronização e pedido explícito do usuário por Tailwind).

### 2. Estilização dos Componentes React
- **Decision:** Remover todos os estilos inline (`style={{...}}`) existentes em `LoginForm.jsx`, `EntityTable.jsx`, `MergeModal.jsx` e `LinkModal.jsx` e substituí-los por classes utilitárias do Tailwind.
- **Rationale:** Mantém o CSS concentrado no design system do Tailwind, garantindo facilidade de manutenção e responsividade.

### 3. Design da Tela de Login (Split-Screen)
- **Decision:** Implementar layout com `grid` ou `flex` onde telas grandes (`md:`) possuem duas colunas (split) e telas menores (mobile) empilham (stacking) o conteúdo de branding sobre o formulário.
- **Rationale:** Atende diretamente o FR-002 e a User Story 1.
