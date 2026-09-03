# Research Hub 🧬

Ferramenta **desktop** de curadoria de dados acadêmicos. Recebe o pacote
`exports_canonical.zip` do DataLake, permite corrigir, fundir e vincular os
registros, e exporta o pacote de volta — preservando intactos todos os arquivos
que ela não gerencia.

Aplicativo de processo único: sem servidor, sem navegador, sem Python, sem rede.

## Tecnologias

- **Rust + Tauri 2.0** — núcleo, janela e comunicação por IPC
- **SQLite** (`rusqlite`, embarcado) — banco em arquivo único
- **arrow-rs** — leitura e escrita dos `.parquet` canônicos
- **Astro + React + Tailwind** — interface, compilada estaticamente e embutida
  no binário

## Instalar

Baixe o instalador da [página de releases](https://github.com/RafaelDeps/research_hub/releases):
`.deb`, `.rpm` ou `.AppImage` no Linux, `.exe` no Windows.

> **Windows:** na primeira execução o SmartScreen exibe "O Windows protegeu o
> computador". Clique em *Mais informações* → *Executar assim mesmo*. O aviso
> aparece porque o binário não é assinado — decisão consciente para uma
> ferramenta de uso pessoal.

## Usar

1. **Entrar** — `admin@admin.com` / `admin123`.
2. **Importar** — no painel, escolha o `exports_canonical.zip` ou arraste-o
   para a janela. Isso **substitui a base atual**; uma cópia de segurança é
   gravada automaticamente antes, e o caminho dela aparece na tela.
3. **Curar** — navegue pelas 15 entidades no menu lateral. Busque, ordene,
   edite, funda duplicatas e crie vínculos.
4. **Exportar** — gera o pacote canônico com os tipos originais restaurados e
   os demais arquivos do ZIP preservados byte a byte.

Os dados ficam em `~/.local/share/br.edu.ifes.researchhub/hub.db` no Linux e em
`%APPDATA%\br.edu.ifes.researchhub\hub.db` no Windows. Fazer backup é copiar
esse arquivo.

## Desenvolver

Pré-requisitos: [Rust](https://rustup.rs), Node 22+ e, no Linux:

```bash
sudo apt install -y libwebkit2gtk-4.1-dev libxdo-dev libssl-dev \
  libayatana-appindicator3-dev librsvg2-dev pkg-config
```

```bash
make build    # dependências do frontend
make dev      # roda com hot reload
make test     # cargo test + vitest
make bundle   # instaladores nativos
```

## Domínios de dados

Pesquisadores, Alunos, Grupos de Pesquisa, Iniciativas, Premiações, Produções
Científicas, Áreas de Conhecimento, Orientações, Organizações, Atividades
Profissionais, Campus, Proficiências, Bolsas, Idiomas e Artigos.

Colunas de relacionamento guardam arrays JSON, evitando tabelas associativas e
mantendo a leitura direta nos pipelines de Big Data.

## Migrando da versão web

Não há migração automática do `backend/test.db` antigo. O caminho suportado é
reimportar o `exports_canonical.zip` no aplicativo — o resultado é equivalente,
já que a base sempre foi derivada dele.

## Documentação

- `.specify/memory/constitution.md` — regras de arquitetura
- `docs/estudo-migracao-rust-tauri.md` — o estudo que originou a reescrita
- `specs/` — especificações por feature
