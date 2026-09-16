---
marp: true
theme: default
class: lead
paginate: true
backgroundColor: #ffffff
---

# Research Hub 🧬
## Curadoria de Dados Acadêmicos

*[Nome do Apresentador / Equipe]*
*[Data]*

---

# O que é o Research Hub?

O **Research Hub** é uma ferramenta **desktop** voltada para a curadoria de dados acadêmicos.

- Recebe o pacote `exports_canonical.zip` do DataLake.
- Permite corrigir, fundir e vincular registros.
- Exporta o pacote de volta, preservando intactos os arquivos não gerenciados.

<!-- 
Nota para edição: Adicione uma imagem abaixo mostrando a tela inicial ou o fluxo geral da aplicação.
![Visão Geral do Sistema](caminho_para_imagem.png)
-->

---

# A Arquitetura

Aplicativo de **processo único**: sem servidor externo, sem navegador dependente, sem Python, sem rede.

- **Núcleo:** Rust + Tauri 2.0 (Gerencia a janela e comunicação via IPC)
- **Banco de Dados:** SQLite (`rusqlite`, embarcado em arquivo único)
- **Manipulação de Dados:** `arrow-rs` / `polars` para leitura e escrita de `.parquet`
- **Front-end:** Astro + React + Tailwind (Compilado estaticamente e embutido no binário)

<!-- 
Nota para edição: Uma imagem com o diagrama de arquitetura cairia muito bem aqui.
![Diagrama de Arquitetura](caminho_para_imagem.png)
-->

---

# A Evolução (Migração Web ➡️ Desktop)

Por que deixamos de ser uma aplicação web (Python + FastAPI)?

- **Simplicidade de Distribuição:** Fim da necessidade de gerenciar `venv`, múltiplos processos e variáveis de ambiente.
- **Eficiência:** Eliminação da camada HTTP inteira, do CORS, do upload multipart.
- **Foco no Usuário:** O perfil de uso é de um aplicativo de desktop local monousuário, logo, a arquitetura deve refletir isso (banco local e operação sem rede).

---

# Fluxo de Trabalho do Curador

1. **Entrar:** Login único local (`admin@admin.com`).
2. **Importar:** Seleção ou *drag-and-drop* do `exports_canonical.zip`. A base anterior recebe backup automático.
3. **Curar:** Busca, edição, fusão de duplicatas e criação de vínculos.
4. **Exportar:** Geração do ZIP final com tipos de dados restaurados e arquivos não gerenciados mantidos bit a bit.

<!-- 
Nota para edição: Inserir capturas de tela do fluxo de "Curar" (tabelas e edição).
![Tela de Curadoria](caminho_para_imagem.png)
-->

---

# Os Domínios de Dados (Entidades)

O sistema suporta 15 entidades acadêmicas mapeadas:
- Pesquisadores, Alunos, Grupos de Pesquisa
- Iniciativas, Premiações, Produções Científicas
- Áreas de Conhecimento, Orientações, Organizações
- Atividades Profissionais, Campus, Proficiências
- Bolsas, Idiomas e Artigos

*Tabelas associativas não são utilizadas; relacionamentos são mantidos em colunas de arrays JSON, facilitando o consumo em pipelines de Big Data.*

---

# Distribuição e Instalação

Binários nativos para os principais sistemas operacionais, dispensando pré-requisitos como Node ou Python:

- **Linux:** `.deb`, `.rpm`, `.AppImage`
- **Windows:** `.exe` (MSI/NSIS)

Toda persistência ocorre diretamente no diretório de dados do SO (`~/.local/share` ou `%APPDATA%`), garantindo backup trivial (basta copiar um arquivo).

---

# Conclusão e Próximos Passos

O **Research Hub** consolida o trabalho de curadoria de dados em um ambiente focado, rápido e de distribuição contida.

* **Maior robustez na ingestão de dados.**
* **Experiência nativa e offline.**
* **Facilidade de manutenção e distribuição.**

## Obrigado!
Dúvidas?
