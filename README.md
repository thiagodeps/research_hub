# Research Hub 🧬

[![CI/CD Release](https://github.com/thiagodeps/research_hub/actions/workflows/release.yml/badge.svg)](https://github.com/thiagodeps/research_hub/actions/workflows/release.yml)
[![Deploy MkDocs to GitHub Pages](https://github.com/thiagodeps/research_hub/actions/workflows/gh-pages.yml/badge.svg)](https://github.com/thiagodeps/research_hub/actions/workflows/gh-pages.yml)
[![Documentação GitHub Pages](https://img.shields.io/badge/docs-GitHub%20Pages-blue)](https://thiagodeps.github.io/research_hub/)
[![Licença](https://img.shields.io/badge/licen%C3%A7a-MIT%2FApache--2.0-green)](#)

Ferramenta **desktop de alta performance** voltada para curadoria, higienização, vinculação e resolução de entidades de dados acadêmicos e extensionistas.

O Research Hub foi concebido como um aplicativo de processo único e arquitetura **offline-first**: sem servidor web em segundo plano, sem portas de rede abertas desnecessárias, sem runtime Python na máquina do usuário e com um banco de dados relacional embarcado em arquivo único.

---

## 📑 Sumário

- [Visão Geral e Filosofia](#visão-geral-e-filosofia)
- [Tecnologias Utilizadas](#tecnologias-utilizadas)
- [Projetos e Domínios Suportados](#projetos-e-domínios-suportados)
  - [Projeto Horizon](#1-projeto-horizon-curadoria-acadêmica)
  - [Projeto SRC](#2-projeto-src-extensão-e-ensino)
  - [Sincronização com o GitHub](#3-sincronização-com-github-novidade)
- [Requisitos para o Código e Execução](#requisitos-para-o-código-e-execução)
  - [Sistemas Operacionais Suportados](#sistemas-operacionais-suportados)
  - [Dependências do Sistema Operacional (Linux)](#dependências-do-sistema-operacional-linux)
  - [Requisitos para Windows](#requisitos-para-windows)
  - [Requisitos para macOS](#requisitos-para-macos)
  - [Ferramentas de Desenvolvimento e Compiladores](#ferramentas-de-desenvolvimento-e-compiladores)
  - [Requisitos para Documentação (MkDocs / GH Pages)](#requisitos-para-documentação-mkdocs--github-pages)
  - [Armazenamento Local, Permissões e Segurança](#armazenamento-local-permissões-e-segurança)
- [Como Instalar](#como-instalar)
- [Guia de Uso Rápido](#guia-de-uso-rápido)
- [Guia do Desenvolvedor](#guia-do-desenvolvedor)
  - [Comandos do Makefile](#comandos-do-makefile)
  - [Build em Contêiner Docker](#build-em-contêiner-docker)
- [Notas de Versão (Patch Notes / Changelog)](#notas-de-versão-patch-notes--changelog)
- [Documentação do Projeto](#documentação-do-projeto)
- [Capturas de Tela](#capturas-de-tela)

---

## Visão Geral e Filosofia

O Research Hub atua como estação central de trabalho para curadores de dados. Ele recebe conjuntos de dados brutos ou processados de pipelines upstream (DataLake / Horizon / SRC_ETL), permite auditar, corrigir valores textuais, fundir registros duplicados preservando integridade referencial, e exportar os conjuntos de volta mantendo fidelidade de tipos e arquivos intocados.

### Princípios Arquiteturais Fundamentais
1. **Processo Único**: Núcleo compilado em Rust com casca nativa fornecida pelo Tauri 2.0. O frontend em React/Tailwind é compilado estaticamente e embutido diretamente no executável final.
2. **Offline-First & Privacidade**: Toda a curadoria roda localmente na máquina do curador. Não há tráfego de dados para nuvem, telemetria oculta ou dependência de servidores web locais.
3. **Fidelidade Canônica**: Na importação e exportação de pacotes (seja o ZIP canônico do Horizon ou o JSON consolidado do SRC), arquivos e registros não manipulados são preservados byte a byte, com tipos de dados originais (booleanos, inteiros, floats, nulls e listas de relacionamentos) mantidos integralmente.
4. **Resiliência e Segurança**: Todas as operações de importação realizam substituição atômica com validação prévia de integridade e criação automática de snapshots de recuperação antes de qualquer alteração na base.

---

## Tecnologias Utilizadas

- **Núcleo do Aplicativo**: [Rust](https://www.rust-lang.org/) (edição 2021, rustc 1.77.2+)
- **Shell Desktop e IPC**: [Tauri 2.0](https://v2.tauri.app/)
- **Armazenamento de Dados**: [SQLite](https://www.sqlite.org/) embarcado via crate `rusqlite` (versão bundled com suporte a transações, funções e backup nativo)
- **Serialização de Dados**: `serde`, `serde_json` (com suporte a preservação de ordem de chaves)
- **Manipulação de Arquivos e Formatos**: `zip-rs`, `arrow-rs` / `parquet-rs`
- **Sincronização de Rede (GitHub)**: `reqwest` (com `rustls-tls` puro sem dependência de OpenSSL externo e streaming assíncrono para arquivos grandes de até 100 MB)
- **Criptografia e Autenticação**: `bcrypt` para hashing seguro de credenciais locais
- **Interface com o Usuário**: [Astro 4](https://astro.build/) + [React 18](https://react.dev/) + [Tailwind CSS 3](https://tailwindcss.com/)
- **Testes Automatizados**: `cargo test`, [Vitest](https://vitest.dev/) (testes unitários com JSDOM) e [WebdriverIO](https://webdriver.io/) (testes E2E)
- **Documentação Estática**: [MkDocs](https://www.mkdocs.org/) com tema [Material for MkDocs](https://squidfunk.github.io/mkdocs-material/) e `mkdocs-static-i18n` (bilingue: PT/EN)

---

## Projetos e Domínios Suportados

A partir das versões mais recentes, o Research Hub oferece um **seletor de projetos multi-domínio** na inicialização (`/projects`), permitindo alternar de forma isolada entre diferentes ecossistemas:

### 1. Projeto Horizon (Curadoria Acadêmica)
Gerencia as 15 tabelas canônicas relacionais extraídas do DataLake institucional:
- **Pesquisadores**, **Alunos**, **Grupos de Pesquisa**, **Iniciativas**, **Premiações**
- **Produções Científicas**, **Áreas de Conhecimento**, **Orientações**, **Organizações**
- **Atividades Profissionais**, **Campi**, **Proficiências**, **Bolsas**, **Idiomas**, **Artigos**

Colunas relacionais armazenam arrays JSON normalizados, viabilizando leitura direta em pipelines de Big Data sem a necessidade de tabelas associativas lentas.

### 2. Projeto SRC (Extensão e Ensino)
Gerencia as ações do Sistema de Registro e Emissão de Certificados do Ifes através de um arquivo consolidado (`src_consolidado.json`):
- **Ações**: Processo, título, natureza (Extensão/Ensino), tipo (Projeto, Curso, Evento, etc.), coordenador, área temática, fomento e vínculo textual a programas guarda-chuva.
- **Participações Aninhadas**: Editor exclusivo para gerenciar participantes com distinção entre **Público-alvo** e **Equipe de execução**, preservando dados de atividade, funções, cargas horárias e vínculos.
- **Privacidade Rigorosa**: Os dados de participantes contêm dados pessoais sensíveis (CPF, e-mail, nomes) e são mantidos estritamente locais, com bloqueio arquitetural contra envio para repositórios remotos.

### 3. Sincronização com GitHub (Novidade)
- **Download Remoto**: Importe pacotes canônicos (ZIP do Horizon ou JSON do SRC) diretamente informando URLs do GitHub (links `raw.githubusercontent.com` ou assets de releases).
- **Upload Atômico**: Envie novos pacotes exportados do Horizon diretamente para branches do GitHub via Git Data API (criação de blobs, trees e commits sem necessidade de clonar o repositório localmente).
- **Gerenciamento Local de Token**: Armazene com segurança seu Personal Access Token (PAT) do GitHub em arquivo local com permissão restrita `0600`, sem nunca expor o token em logs ou mensagens de erro.

---

## Requisitos para o Código e Execução

Para compilar, rodar ou contribuir com o código-fonte do Research Hub, o ambiente de desenvolvimento precisa atender aos seguintes requisitos:

### Sistemas Operacionais Suportados
- **Linux**: Ubuntu 22.04 LTS ou superior, Debian 12+, Fedora 39+, Arch Linux, openSUSE Tumbleweed/Leap (x86_64 e aarch64).
- **Windows**: Windows 10 (versão 1809 ou superior) ou Windows 11 (64-bit).
- **macOS**: macOS 10.15 (Catalina) ou superior, compatível com arquiteturas Intel (`x86_64`) e Apple Silicon (`aarch64`).

### Dependências do Sistema Operacional (Linux)
O Tauri 2.0 utiliza bibliotecas nativas do sistema para gerenciar janelas, renderização WebKitGTK e IPC:

#### Ubuntu / Debian / Pop!_OS / Linux Mint:
```bash
sudo apt update
sudo apt install -y \
  build-essential \
  curl \
  wget \
  pkg-config \
  libwebkit2gtk-4.1-dev \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

#### Fedora / RHEL / CentOS Stream:
```bash
sudo dnf groupinstall "Development Tools"
sudo dnf install -y \
  webkit2gtk4.1-devel \
  openssl-devel \
  libappindicator-gtk3-devel \
  librsvg2-devel \
  gcc-c++ \
  pkgconf-pkg-config \
  curl \
  wget
```

#### Arch Linux / Manjaro:
```bash
sudo pacman -Syu --needed \
  base-devel \
  curl \
  wget \
  webkit2gtk-4.1 \
  libappindicator-gtk3 \
  openssl \
  librsvg
```

> ⚠️ **Aviso sobre Snap e VS Code no Linux:**
> Se o terminal for iniciado a partir de um aplicativo instalado via Snap (como VS Code ou emuladores de terminal Snap), ele pode injetar um `LD_LIBRARY_PATH` apontando para `/snap/core20`, o que causa conflito com a glibc do sistema e dispara o erro `symbol lookup error: undefined symbol: __libc_pthread_init`.
> O `Makefile` do projeto já limpa automaticamente essas variáveis de ambiente (`RUN := env -u LD_LIBRARY_PATH ...`). Ao rodar binários manualmente fora do make, use:
> ```bash
> env -u LD_LIBRARY_PATH -u LD_PRELOAD -u SNAP ./src-tauri/target/debug/research-hub
> ```

### Requisitos para Windows
1. **Microsoft C++ Build Tools**:
   - Instale através do instalador do [Visual Studio](https://visualstudio.microsoft.com/visual-cpp-build-tools/) selecionando a carga de trabalho **"Desenvolvimento para desktop com C++"** (*Desktop development with C++*).
2. **WebView2 Runtime**:
   - Nativo no Windows 10 (atualizado) e Windows 11. Caso não esteja presente, baixe o [Evergreen Bootstrapper da Microsoft](https://developer.microsoft.com/en-us/microsoft-edge/webview2/).
3. **Rust Toolchain para Windows**:
   - Target padrão: `x86_64-pc-windows-msvc`.

### Requisitos para macOS
1. **Xcode Command Line Tools**:
   ```bash
   xcode-select --install
   ```
2. **Rust Targets** (para build universal Intel e M1/M2/M3):
   ```bash
   rustup target add x86_64-apple-darwin aarch64-apple-darwin
   ```

### Ferramentas de Desenvolvimento e Compiladores
- **Rust Toolchain**:
  - Rust **1.77.2 ou superior** (versão recomendada: stable recente).
  - Instalação via rustup: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Node.js e Gerenciador de Pacotes**:
  - Node.js **22.x LTS** (mínimo: Node.js 18+).
  - npm **10.x ou superior**.
- **Tauri CLI**:
  - Fornecido pelo projeto via dependência de build ou instalado globalmente:
  ```bash
  cargo install tauri-cli --version "^2.0.0"
  ```
- **Git**:
  - Git 2.30+ para controle de versão.

### Requisitos para Documentação (MkDocs / GitHub Pages)
Caso deseje construir localmente a documentação ou realizar o deploy para o GitHub Pages:
- **Python**: Versão 3.10 ou superior (incluindo `python3-venv`).
- **Pacotes Python**:
  ```bash
  pip install mkdocs mkdocs-material mkdocs-static-i18n
  ```

### Armazenamento Local, Permissões e Segurança
O aplicativo armazena dados em diretórios padronizados pelo sistema operacional:
- **Banco de Dados SQLite (`hub.db`)**:
  - Linux: `~/.local/share/br.edu.ifes.researchhub/hub.db`
  - Windows: `%APPDATA%\br.edu.ifes.researchhub\hub.db`
  - macOS: `~/Library/Application Support/br.edu.ifes.researchhub/hub.db`
- **Token de Acesso GitHub (`github_token`)**:
  - Gravado com máscara restrita POSIX `0600` (leitura e escrita exclusivas do usuário corrente).
- **Rede**:
  - Funciona 100% desconectado da internet. A conectividade com `api.github.com` e `raw.githubusercontent.com` é exigida exclusivamente quando o usuário aciona os recursos da aba de sincronização remota.
  - Para repositórios privados ou envio de commits, é necessário um **Personal Access Token (PAT)** clássico ou fine-grained com escopo `repo`.

---

## Como Instalar

Instaladores pré-compilados prontos para uso são distribuídos na [página oficial de Releases](https://github.com/thiagodeps/research_hub/releases):

- **Linux**:
  - `.AppImage`: Executável universal portátil (não requer instalação).
  - `.deb`: Instalador nativo para Ubuntu, Debian e derivados.
  - `.rpm`: Instalador nativo para Fedora, RHEL e openSUSE.
- **Windows**:
  - `.exe` (NSIS Installer) ou `.msi`.
- **macOS**:
  - `.dmg` ou pacote `.app` universal (Intel e Apple Silicon).

> ℹ️ **Nota para usuários Windows:**
> Na primeira execução, o Windows SmartScreen pode apresentar a mensagem *"O Windows protegeu o computador"*. Clique em **Mais informações** → **Executar assim mesmo**. Esse aviso ocorre porque os instaladores de releases abertas não possuem assinatura de certificado digital comercial EV.

---

## Guia de Uso Rápido

1. **Autenticação**:
   - Usuário padrão semeado: `admin@admin.com` / `admin123`.
   - Você também pode criar novas contas locais na tela `/register`.
2. **Seleção de Projeto**:
   - Na tela principal `/projects`, escolha trabalhar no **Horizon** (15 entidades acadêmicas) ou no **SRC** (extensão e ensino).
3. **Importação de Dados**:
   - **Horizon**: Carregue o arquivo `exports_canonical.zip` (formato atual somente JSON ou legado com Parquet) via botão de diálogo ou arrastando o arquivo para a janela.
   - **SRC**: Carregue o arquivo `src_consolidado.json`.
   - **GitHub Sync**: Informe a URL do repositório/release para download direto.
4. **Curadoria**:
   - Navegue pelas entidades, utilize a busca global e busca direta por ID, filtre e ordene colunas.
   - Edite registros inline, gerencie participações ou funda duplicatas de forma transacional.
5. **Exportação**:
   - Gere o pacote canônico de volta com fidelidade total de tipos de dados e arquivos preservados.

---

## Guia do Desenvolvedor

### Comandos do Makefile

O projeto inclui um `Makefile` automatizado na raiz para facilitar o fluxo diário:

```bash
make build         # Instala dependências do frontend (npm install)
make dev           # Executa o aplicativo em modo desktop com hot-reload (Cargo + Astro)
make test          # Executa a suíte completa de testes: cargo test + vitest
make bundle        # Compila os instaladores nativos de distribuição da plataforma atual
make docker-bundle # Compila instaladores Linux reproduzíveis dentro de contêiner Ubuntu 24.04
make clean         # Limpa artefatos de build (target, dist, node_modules)
```

### Build em Contêiner Docker

Caso você esteja no Linux e prefira gerar os pacotes `.deb`, `.rpm` e `.AppImage` sem instalar as dependências de compilação diretamente no host:

```bash
make docker-bundle
```

Os artefatos compilados serão gerados no diretório `dist/`. Para executar o AppImage gerado:

```bash
cd dist/
sudo chown $USER:$USER ResearchHub_*.AppImage
chmod +x ResearchHub_*.AppImage
./ResearchHub_*.AppImage
```

---

## Notas de Versão (Patch Notes / Changelog)

### v1.3.4 (Atual — Feature 034: Sincronização GitHub & Pacotes Aninhados)
- **Sincronização com GitHub (SEP-034)**:
  - Download e importação direta de arquivos canônicos a partir de URLs do GitHub (repositórios públicos ou privados com token).
  - Suporte a links raw e assets de release do GitHub.
  - Envio (*upload*) do pacote canônico do Horizon para o GitHub via Git Data API (criação direta de commits atômicos sem clonar o repositório).
  - Gerenciamento local seguro de Personal Access Token (PAT) com arquivo protegido em `0600`, teste de escopos e remoção com zero vestígios.
  - Política de proteção de dados pessoais: bloqueio estrito de envio de dados do SRC para repositórios remotos devido à presença de dados sensíveis (LGPD / PII).
  - Suporte a arquivos de exportação de até 100 MB com validações atômicas e snapshots automáticos pré-importação.
- **Suporte a Pacotes Canônicos Aninhados**:
  - Abertura e ingestão transparente de arquivos ZIP aninhados ou pastas com subdiretórios no pacote canônico.
- **Auditoria de Segurança**:
  - 100% dos fluxos de erro auditados: tokens e dados pessoais nunca vazam em logs, mensagens ou telemetria.

### v1.3.3 (Feature 033: Curadoria SRC & Seletor de Projetos)
- **Seletor Multi-Projeto**:
  - Nova página inicial `/projects` pós-login permitindo escolher entre o projeto **Horizon** e o projeto **SRC**.
  - Separação completa de bases de dados, tabelas, formulários e snapshots de segurança entre os dois projetos.
- **Domínio de Extensão e Ensino (SRC)**:
  - Tabela dedicada `src_acoes` e migração de banco `002_src_tables.sql`.
  - Ingestão e exportação do arquivo JSON consolidado (`src_consolidado.json`), preservando chaves e ordem original.
  - Editor visual especializado de **Participações** (Público-alvo e Equipe de execução) aninhadas à atividade.
  - Tratamento de vínculos com programas guarda-chuva e proteção contra exclusão de programas com ações dependentes.

### v1.3.0 (Feature 032: Pacote Canônico Somente JSON & Aba Campus)
- **Novo Formato Canônico Somente JSON**:
  - Adaptação completa da importação e exportação para o novo padrão do DataLake (arquivos `{tabela}_canonical.json` na raiz do ZIP, sem diretório `parquet/`).
  - Compatibilidade retroativa mantida para pacotes legados contendo Parquet.
  - Módulo de inferência e fidelidade de tipos JSON para round-trip perfeito (preserva tipos originais booleanos, inteiros, decimais, nulls e listas de relacionamentos).
- **Correção da Aba Campus**:
  - Eliminação do campo aninhado fantasma "Campus (Vínculos)" herdado do bug upstream do DataLake.

### v1.2.0 (Release v1.2.0 — Feature 030 & 031)
- **Busca por Identificador (SEP-030)**:
  - Campo de busca com suporte a busca direta por ID numérico em todas as entidades do Horizon.
- **Correções Críticas do Fluxo de Curadoria (SEP-031)**:
  - *B-01*: Correção de estado obsoleto no formulário de edição ao alternar alvos de visualização.
  - *B-02*: Reposicionamento automático da paginação para a última página válida após exclusão de registros.
  - *B-03*: Limpeza automática da seleção de fusão após deleções, buscas ou troca de página.
  - *B-04*: Suporte a campo resolvedor "título" no diálogo de fusão (artigos, produções e premiações).
  - *B-04b*: União transacional de listas de vínculos durante a fusão de registros, evitando perda de conexões relacionais.
  - *B-05*: Eliminação de condições de corrida em requisições assíncronas de busca com debounce.
  - *B-06*: Normalização canônica do nome de usuário da sessão em letras minúsculas.
  - *B-07*: Verificação prévia de autenticação antes de abrir caixas de diálogo do sistema de arquivos.
  - *B-08*: Validação atômica de pacote ZIP na importação, rejeitando pacotes vazios ou sem tabelas canônicas sem apagar os dados existentes.

### v1.1.0 – v1.1.3 (Feature 029: Cadastro de Usuários & Criptografia)
- **Cadastro de Usuários**:
  - Página `/register` para criação de novas contas de administrador.
  - Armazenamento de senhas protegidas com hashing `bcrypt` no núcleo Rust.
- **Acessibilidade e Qualidade de Código**:
  - Correção de labels de formulários com atributos `htmlFor` e `id`.
  - Suíte de testes unitários com ambiente JSDOM e cleanup de formulários.

### v1.0.9 (Feature 028: Navegação Direta de Páginas)
- **Paginação Aprimorada**:
  - Campo numérico interativo no componente `EntityPage` para salto direto a qualquer página da listagem com clamping automático de valores inválidos.

### v1.0.0 – v1.0.7 (Migração Rust + Tauri 2.0 & Decomissionamento Python)
- **Reescrita Arquitetural Completa**:
  - Decomissionamento definitivo do antigo backend em Python FastAPI e banco PostgreSQL.
  - Criação da arquitetura desktop Tauri 2.0 em Rust com banco de dados SQLite embarcado de arquivo único.
  - Criação dos fluxos canônicos de importação e exportação de dados com preservação de integridade.
  - Implementação de pipelines automatizados de compilação e release multiplataforma no GitHub Actions.

---

## Documentação do Projeto

- **Site Oficial de Documentação**: [thiagodeps.github.io/research_hub](https://thiagodeps.github.io/research_hub/)
- **Regras Arquiteturais Vinculativas**: [`.specify/memory/constitution.md`](.specify/memory/constitution.md)
- **Estudo de Migração Rust + Tauri**: [`docs/estudo-migracao-rust-tauri.md`](docs/estudo-migracao-rust-tauri.md)
- **Especificações de Funcionalidades (Specs)**: diretório [`specs/`](specs/)

---

## Capturas de Tela

<div align="center">
  <img width="800" alt="Listagem e Gestão de Entidades" src="https://github.com/user-attachments/assets/cf25ce56-8ac7-44e6-b2c6-119faac088d9" style="margin-bottom: 12px;" />
  <img width="800" alt="Painel Geral de Curadoria" src="https://github.com/user-attachments/assets/d3e824c3-5f48-4d73-baa4-1430ba497f1a" style="margin-bottom: 12px;" />
  <img width="800" alt="Visualização e Edição de Registros" src="https://github.com/user-attachments/assets/cbc63fc3-f67c-4925-9f62-d1c3b825134c" style="margin-bottom: 12px;" />
  <img width="800" alt="Vínculos e Relacionamentos entre Entidades" src="https://github.com/user-attachments/assets/3d0b22a5-f7f3-4a21-9140-8d9e58d856f2" style="margin-bottom: 12px;" />
  <img width="800" alt="Importação e Exportação com Backup Automático" src="https://github.com/user-attachments/assets/df9658de-af86-43e6-9584-1ea0c01741c9" style="margin-bottom: 12px;" />
  <img width="800" alt="Resolução de Conflitos e Fusão de Registros" src="https://github.com/user-attachments/assets/9a16f816-cf73-4cd1-8c13-8914197b665e" />
</div>
