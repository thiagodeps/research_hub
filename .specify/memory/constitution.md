<!--
Sync Impact Report:
- Version change: 1.0.0 → 2.0.0 (MAJOR: redefinição incompatível da stack técnica)
- Modified principles:
  - II. Reuso do Domínio (ResearchDomain) → II. Paridade Funcional Antes de Melhoria
  - III. Testes Rigorosos (Backend e Frontend) → III. Testes Rigorosos (Rust e Front-end)
  - V. Frontend em Astro com Deploy Estático → V. Aplicação Desktop Local e Autocontida
- Added sections:
  - VI. Lógica de Negócio em Rust (novo princípio)
  - Escopo Diferido (dentro de Technical Stack & Architecture)
- Removed sections:
  - Obrigação de reuso da biblioteca `research_domain` (letra morta: nunca esteve em
    requirements.txt e as entidades já são reimplementadas em orm.py)
  - Exigência de deploy estático compatível com GitHub Pages
- Preserved unchanged: I. TDD Obrigatório, IV. CRUD Completo e Operações Diretas,
  exigência de aderência ao protótipo Figma
- Templates requiring updates:
  - .specify/templates/plan-template.md (✅ updated — gates do Constitution Check reescritos)
  - .specify/templates/spec-template.md (✅ updated — CON-001..CON-004 reescritos)
  - .specify/templates/tasks-template.md (✅ verificado — sem referências de stack, nenhuma mudança necessária)
  - .specify/templates/checklist-template.md (✅ verificado — agnóstico de stack)
- Runtime guidance:
  - README.md (⚠ pendente — será reescrito na SEP-024, quando o backend Python for removido)
  - AGENTS.md (⚠ pendente — apontará para o plano da SEP-015 assim que ela existir)
  - docs/estudo-migracao-rust-tauri.md (✅ é a fonte desta emenda)
- Follow-up TODOs: nenhum
-->
# Portal do Professor IFES Serra Constitution

Este documento define os princípios fundamentais e regras não-negociáveis para o desenvolvimento do Portal do Professor IFES Serra (ResearchHub), uma ferramenta de curadoria e gestão de dados acadêmicos.

A partir da versão 2.0.0, o sistema deixa de ser uma aplicação web de dois processos (API Python + front-end servido em navegador) e passa a ser uma **aplicação desktop local** construída em Rust sobre o framework Tauri 2.0, mantendo JavaScript no front-end. A justificativa técnica completa, a análise de riscos e o backlog de features estão em `docs/estudo-migracao-rust-tauri.md`, que é o documento de referência desta emenda.

## Core Principles

### I. TDD Obrigatório (Strict Test-Driven Development)

Para toda funcionalidade (comando, componente, regra de negócio), o fluxo obrigatório é: escrever o teste primeiro, ver falhar (red), implementar até passar (green) e refatorar. Nenhuma task de implementação pode ser concluída sem que o teste correspondente esteja passando. A geração de tasks deve seguir a ordem estrita de TDD.

### II. Paridade Funcional Antes de Melhoria

Durante a migração para Rust/Tauri, nenhuma funcionalidade nova pode ser introduzida antes que o ciclo *importar ZIP → editar → exportar ZIP* produza saída equivalente à do sistema Python de referência. Enquanto o backend Python existir no repositório, ele é o **oráculo de paridade**: a verificação de uma feature migrada inclui comparar sua saída com a do oráculo sobre o mesmo arquivo de entrada.

Correções de defeitos preexistentes SÃO permitidas e desejáveis durante a migração, mas cada uma DEVE ser declarada como item explícito de escopo na spec da feature e registrada como diferença esperada no script de paridade. É proibida a correção silenciosa: uma mudança de comportamento não declarada invalida a comparação com o oráculo.

### III. Testes Rigorosos (Rust e Front-end)

O núcleo em Rust DEVE ser testado com `cargo test`, cobrindo testes unitários para lógica de domínio (que roda sem dependência do Tauri) e testes de integração contra SQLite em memória. O front-end DEVE manter testes com `Vitest` para componentes e a camada de comunicação, e testes end-to-end com `WebdriverIO` + `tauri-driver` cobrindo login e o CRUD completo de entidades de referência.

**Todo alvo de release DEVE ser alvo de teste automatizado.** É proibido declarar suporte a uma plataforma cuja verificação dependa de roteiro manual: se a automação não cobre a plataforma, ela não entra como alvo de release. Esta regra existe para impedir que meio-suporte seja apresentado como suporte.

### IV. CRUD Completo e Operações Diretas

Toda entidade no sistema deve possuir um CRUD completo (criar, ver, editar, deletar). O sistema opera com um único perfil (Admin), sem fluxo de aprovação. O Admin acessa e executa operações de edição, fusão e vínculo diretamente sobre as entidades.

### V. Aplicação Desktop Local e Autocontida

O sistema DEVE ser distribuído como aplicação desktop de processo único, construída com Tauri 2.0. O front-end permanece em Astro com React e Tailwind, compilado como site estático e embarcado no binário — a interface DEVE seguir o protótipo Figma "Sistema de Login para Professores e Admins", com tela de login split-screen e as diretrizes de design do branding ResearchHub.

A persistência DEVE usar **SQLite em arquivo único**, embarcado no binário e armazenado no diretório de dados da aplicação do sistema operacional. É proibido introduzir qualquer motor de banco que exija servidor, instalação separada ou administração pelo usuário final. É igualmente proibido reintroduzir uma camada HTTP entre o front-end e a lógica de negócio: a comunicação se dá por IPC do Tauri.

A aplicação DEVE funcionar integralmente sem rede.

### VI. Lógica de Negócio em Rust

Toda regra de negócio — validação, transformação de dados, transações, montagem de consultas, leitura e escrita de arquivos — DEVE residir no processo Rust. O front-end é responsável apenas por renderizar, coletar entrada do usuário e invocar comandos.

É expressamente proibido escrever SQL no JavaScript, manipular arquivos a partir do front-end, ou duplicar no JavaScript uma regra que já exista em Rust. A superfície de comunicação entre as duas camadas DEVE permanecer concentrada em um único módulo do front-end, para que a fronteira seja auditável em um só lugar.

## Technical Stack & Architecture

O projeto adota uma separação rigorosa de responsabilidades:

- **Núcleo:** Rust sobre Tauri 2.0. Persistência em SQLite via `rusqlite` (feature `bundled`); processamento de arquivos Parquet via `polars`, restrito às bordas de importação e exportação. Erros atravessam a fronteira IPC como valores serializados, nunca como pânico.
- **Front-end:** Astro com React e Tailwind, compilado estaticamente e embarcado no binário. Identidade visual baseada no branding ResearchHub. Sem fluxos de cadastro ou seleção de perfil (apenas login único para o perfil Admin).
- **Plataformas-alvo:** Linux e Windows.

### Escopo Diferido

Os itens abaixo estão **fora do escopo** da versão desktop inicial e não podem ser reintroduzidos por decisão local dentro de uma feature. Cada um exige emenda a esta Constitution ou registro explícito no documento de estudo:

- Suporte a macOS — adiado porque a ferramenta oficial de end-to-end do Tauri não cobre a plataforma, e o Princípio III proíbe alvo de release sem teste automatizado.
- Assinatura de código e notarização.
- Atualização automática (auto-update).
- Qualquer banco de dados que não seja SQLite embarcado.

## Development & Deployment Workflow

O fluxo de trabalho é guiado por testes (TDD). A ordem de execução de tarefas (`/speckit.tasks`) deve refletir e garantir este fluxo.

Cada feature DEVE entregar algo demonstrável de forma independente. O backend Python de referência permanece no repositório até que a paridade esteja verificada, e só então é removido.

A distribuição se dá por bundles nativos gerados em pipeline de integração contínua com um runner por sistema operacional alvo, uma vez que o Tauri não oferece compilação cruzada confiável.

Mudanças que quebrem os princípios de arquitetura desktop, de persistência embarcada, de localização da lógica de negócio ou de identidade visual exigem revisão e emenda desta constituição.

## Governance

A Constituição do Portal do Professor IFES Serra orienta toda a especificação, planejamento e geração de tarefas. Nenhuma PR, especificação ou plano de implementação pode violar estes princípios.

Mudanças de escopo ou regras de negócio centrais exigem emenda oficial deste documento, com atualização de versão seguindo Semantic Versioning (SemVer): MAJOR para remoção ou redefinição incompatível de princípios, MINOR para adição ou expansão material de orientação, PATCH para esclarecimentos e correções sem efeito semântico.

Toda emenda DEVE propagar suas consequências para os templates dependentes em `.specify/templates/` na mesma alteração, e registrar essa propagação no Sync Impact Report no topo deste arquivo.

**Version**: 2.0.0 | **Ratified**: 2026-08-26 | **Last Amended**: 2026-09-02
