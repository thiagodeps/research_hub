# Research Hub 🧬

Ferramenta **desktop de alta performance** de curadoria, higienização e vinculação de dados acadêmicos e extensionistas.

O Research Hub foi concebido como um aplicativo de processo único e arquitetura **offline-first**: sem servidor em segundo plano, sem portas de rede abertas, sem dependência de runtime Python e com banco de dados SQLite embarcado em arquivo único.

---

## 📸 Demonstração e Capturas de Tela

<div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(400px, 1fr)); gap: 12px; margin-bottom: 2rem;">
  <img width="100%" alt="Listagem e Gestão de Entidades" src="https://github.com/user-attachments/assets/cf25ce56-8ac7-44e6-b2c6-119faac088d9" style="border-radius: 8px; border: 1px solid #e2e8f0;" />
  <img width="100%" alt="Painel Geral de Curadoria" src="https://github.com/user-attachments/assets/d3e824c3-5f48-4d73-baa4-1430ba497f1a" style="border-radius: 8px; border: 1px solid #e2e8f0;" />
  <img width="100%" alt="Visualização e Edição de Registros" src="https://github.com/user-attachments/assets/cbc63fc3-f67c-4925-9f62-d1c3b825134c" style="border-radius: 8px; border: 1px solid #e2e8f0;" />
  <img width="100%" alt="Vínculos e Relacionamentos entre Entidades" src="https://github.com/user-attachments/assets/3d0b22a5-f7f3-4a21-9140-8d9e58d856f2" style="border-radius: 8px; border: 1px solid #e2e8f0;" />
  <img width="100%" alt="Importação e Exportação com Backup Automático" src="https://github.com/user-attachments/assets/df9658de-af86-43e6-9584-1ea0c01741c9" style="border-radius: 8px; border: 1px solid #e2e8f0;" />
  <img width="100%" alt="Resolução de Conflitos e Fusão de Registros" src="https://github.com/user-attachments/assets/9a16f816-cf73-4cd1-8c13-8914197b665e" style="border-radius: 8px; border: 1px solid #e2e8f0;" />
</div>

---

## 🛠️ Tecnologias e Arquitetura

- **Núcleo do Aplicativo**: [Rust](https://www.rust-lang.org/) (edição 2021, rustc 1.77.2+)
- **Shell Desktop e IPC**: [Tauri 2.0](https://v2.tauri.app/)
- **Banco de Dados**: [SQLite](https://www.sqlite.org/) embarcado via crate `rusqlite` (bundled, arquivo único local)
- **Frontend**: [Astro 4](https://astro.build/) + [React 18](https://react.dev/) + [Tailwind CSS 3](https://tailwindcss.com/) (compilado estaticamente e embutido no executável)
- **Manipulação de Formatos**: `serde_json`, `zip-rs`, `arrow-rs` / `parquet-rs`
- **Sincronização Remota**: `reqwest` com `rustls-tls` puro e integração com a Git Data API do GitHub
- **Segurança e Criptografia**: `bcrypt` para autenticação local e permissão `0600` para armazenamento de tokens

---

## 📂 Domínios de Dados Suportados

O Research Hub oferece um **seletor multi-projeto** (`/projects`) para alternar entre diferentes fluxos de curadoria com separação estrita:

### 1. Projeto Horizon (Curadoria Acadêmica)
Gerencia as 15 tabelas canônicas relacionais do DataLake:
*Pesquisadores, Alunos, Grupos de Pesquisa, Iniciativas, Premiações, Produções Científicas, Áreas de Conhecimento, Orientações, Organizações, Atividades Profissionais, Campi, Proficiências, Bolsas, Idiomas e Artigos*.

As colunas relacionais guardam arrays JSON serializados, mantendo leitura direta nos pipelines analíticos sem exigir tabelas associativas intermediárias.

### 2. Projeto SRC (Extensão e Ensino)
Gerencia o arquivo consolidado de Extensão e Ensino (`src_consolidado.json`):
- **Ações**: Processo, título, natureza, tipo, coordenador, fomento e área temática.
- **Participações Aninhadas**: Editor exclusivo para público-alvo e equipe de execução.
- **Privacidade Rigorosa**: Dados pessoais de participantes (CPF, e-mail) mantidos 100% locais, com bloqueio arquitetural contra envio remoto.

### 3. Sincronização com GitHub
- **Download Remoto**: Importe pacotes canônicos diretamente via URLs do GitHub (links raw ou releases), em repositórios públicos ou privados com token.
- **Upload Atômico**: Envio do pacote canônico do Horizon para o GitHub via Git Data API (criação direta de commits atômicos).
- **Token Local Seguro**: Armazenamento do token de acesso pessoal em arquivo com permissão restrita `0600`.

---

## 📋 Requisitos para o Código e Execução

### Sistemas Operacionais Suportados
- **Linux**: Ubuntu 22.04+, Debian 12+, Fedora 39+, Arch Linux (x86_64 e aarch64).
- **Windows**: Windows 10 (1809+) ou Windows 11 com WebView2 Runtime.
- **macOS**: macOS 10.15+ (Catalina ou superior), Universal para Intel e Apple Silicon.

### Dependências Nativas no Linux
```bash
# Ubuntu / Debian / Mint
sudo apt update && sudo apt install -y build-essential curl wget pkg-config \
  libwebkit2gtk-4.1-dev libxdo-dev libssl-dev \
  libayatana-appindicator3-dev librsvg2-dev

# Fedora / RHEL
sudo dnf groupinstall "Development Tools" && sudo dnf install -y \
  webkit2gtk4.1-devel openssl-devel libappindicator-gtk3-devel librsvg2-devel \
  gcc-c++ pkgconf-pkg-config curl wget

# Arch Linux / Manjaro
sudo pacman -Syu --needed base-devel curl wget webkit2gtk-4.1 libappindicator-gtk3 openssl librsvg
```

### Requisitos de Ferramentas
- **Rust**: 1.77.2+ com `cargo` e `rustc` (`rustup`).
- **Node.js**: 22.x LTS e `npm` 10+.
- **Tauri CLI**: `cargo-tauri` (`cargo install tauri-cli --version "^2.0.0"`).
- **Python (Documentação)**: Python 3.10+ com `mkdocs`, `mkdocs-material`, `mkdocs-static-i18n`.

### Armazenamento Local e Permissões
- Banco SQLite: `~/.local/share/br.edu.ifes.researchhub/hub.db` (Linux) ou `%APPDATA%\br.edu.ifes.researchhub\hub.db` (Windows).
- Token GitHub: `~/.local/share/br.edu.ifes.researchhub/github_token` (permissão POSIX `0600`).
- Sem dependência de rede para curadoria local regular.

---

## 🚀 Como Instalar e Rodar

Baixe o instalador compilado na [página de releases](https://github.com/thiagodeps/research_hub/releases):
- `.deb`, `.rpm` ou `.AppImage` no Linux.
- `.exe` (NSIS) ou `.msi` no Windows.
- `.dmg` ou `.app` no macOS.

### Credenciais Padrão
- **Usuário**: `admin@admin.com`
- **Senha**: `admin123`
*(Novas contas de usuário podem ser criadas localmente em `/register`)*

---

## 💻 Guia do Desenvolvedor

```bash
make build         # Instala dependências do frontend (npm install)
make dev           # Executa com hot reload (Tauri + Astro)
make test          # Suíte completa de testes: cargo test + vitest
make bundle        # Gera os instaladores nativos para o sistema atual
make docker-bundle # Gera instaladores Linux em contêiner Ubuntu 24.04 (em dist/)
```

---

## 📜 Histórico e Notas de Versão

Consulte a página completa de notas de versão e changelog:
- 👉 [Notas de Versão (Patch Notes)](patch-notes.md)
