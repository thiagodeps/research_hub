<!--
Sync Impact Report:
- Version change: 0.0.0 → 1.0.0
- Modified principles: Initial Draft based on project requirements.
- Added sections: Core Principles, Technical Stack & Architecture, Development & Deployment Workflow, Governance.
- Removed sections: None
- Templates requiring updates: 
  - .specify/templates/plan-template.md (✅ updated)
  - .specify/templates/spec-template.md (✅ updated)
  - .specify/templates/tasks-template.md (✅ updated)
- Follow-up TODOs: None
-->
# Portal do Professor IFES Serra Constitution

Este documento define os princípios fundamentais e regras não-negociáveis para o desenvolvimento do Portal do Professor IFES Serra, um sistema de correção e gestão de dados docentes.

## Core Principles

### I. TDD Obrigatório (Strict Test-Driven Development)
Para toda funcionalidade (endpoint, componente, regra de negócio), o fluxo obrigatório é: escrever o teste primeiro, ver falhar (red), implementar até passar (green) e refatorar. Nenhuma task de implementação pode ser concluída sem que o teste correspondente esteja passando. A geração de tasks deve seguir a ordem estrita de TDD.

### II. Reuso do Domínio (ResearchDomain)
O backend em Python deve expor a biblioteca `research_domain` (`The-Band-Solution/ResearchDomain`) como camada de domínio. É expressamente proibido reimplementar entidades já existentes nela (Researcher, University, Campus, ResearchGroup, KnowledgeArea, Advisorship, Fellowship, AcademicEducation, Article, ResearchProduction, EducationType, ProductionType).

### III. Testes Rigorosos (Backend e Frontend)
O backend deve ser testado com `pytest` (testes unitários para regras de negócio e de integração para os endpoints da API). O frontend deve possuir testes com `Vitest` e `Playwright` abrangendo componentes e fluxos críticos (login e CRUD completo de entidades referência).

### IV. CRUD Completo e Operações Diretas
Toda entidade no sistema deve possuir um CRUD completo (criar, ver, editar, deletar). O sistema operará com um único perfil (Admin), sem fluxo de aprovação. O Admin acessa e executa operações de edição, fusão e herança diretamente sobre as entidades.

### V. Frontend em Astro com Deploy Estático
O frontend deve ser construído utilizando Astro, projetado de forma compatível com deploy em ambientes estáticos como o GitHub Pages. A interface deve seguir estritamente o protótipo Figma "Sistema de Login para Professores e Admins", mantendo as diretrizes de design, tela de login split-screen, e painéis de conceito.

## Technical Stack & Architecture

O projeto adota uma separação rigorosa de responsabilidades:
- **Backend:** Python, integrado com a biblioteca de domínio `research_domain`. Fornecerá endpoints de API consumidos pelo frontend.
- **Frontend:** Desenvolvido em Astro, com a identidade visual baseada no branding "ResearchHub" e focado num deploy sem atritos no GitHub Pages. Sem fluxos de cadastro ou seleção de perfil nesta fase (apenas login único para o perfil Admin).

## Development & Deployment Workflow

O fluxo de trabalho de desenvolvimento é guiado por testes (TDD). A ordem de execução de tarefas (`/speckit.tasks`) deve refletir e garantir este fluxo. Testes E2E com Playwright são essenciais antes de marcar fluxos críticos como concluídos. Mudanças de arquitetura que quebrem os princípios de reuso do domínio ou da identidade visual exigem revisão e emenda desta constituição.

## Governance

A Constituição do Portal do Professor IFES Serra orienta toda a especificação, planejamento e geração de tarefas. Nenhuma PR, especificação ou plano de implementação pode violar estes princípios.
Mudanças de escopo ou regras de negócio centrais exigem emenda oficial deste documento, com atualização de versão seguindo Semantic Versioning (SemVer).

**Version**: 1.0.0 | **Ratified**: 2026-08-26 | **Last Amended**: 2026-08-26
