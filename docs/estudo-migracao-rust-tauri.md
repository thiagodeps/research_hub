# Estudo de Migração: Research Hub → Rust + Tauri 2.0

**Status**: Estudo (não implementado)
**Data**: 2026-09-02
**Autor**: análise técnica do repositório `research_hub` em `main` @ `7595ad6`
**Objetivo**: subsidiar a criação das próximas specs do speckit (`/speckit.specify`) para reescrever o sistema em Rust sobre Tauri 2.0, mantendo JavaScript no front-end.

> Este documento é **insumo de especificação**, não uma especificação. A Seção 14 propõe a quebra em SEPs numeradas, cada uma pronta para virar uma feature branch do speckit com `spec.md` / `plan.md` / `tasks.md`.
>
> **A Seção 18 registra as 7 decisões de escopo já tomadas** (plataformas-alvo, assinatura, tabelas descartadas, correções aceitas). Leia antes de abrir a primeira SEP: várias delas mudam contagens e escopo ao longo do texto.

---

## 1. Sumário Executivo

O Research Hub hoje é uma aplicação web de dois processos: uma API FastAPI (Python, ~1.100 LOC) e um front-end Astro + React servido em `localhost:4321`. Na prática ela é usada como **ferramenta local de curadoria** por um único operador (perfil Admin), com todo o dado vivendo em um SQLite local (`backend/test.db`) e o ciclo de trabalho sendo *importar ZIP → editar → exportar ZIP*.

Essa forma de uso é exatamente o perfil de uma aplicação desktop. A migração para Tauri 2.0 não é apenas troca de linguagem: ela **elimina a camada HTTP inteira**, o CORS, o multipart de upload, o download via `<a href>`, o gerenciamento de dois processos pelo `Makefile` e a dependência de um `venv` Python na máquina do curador. O que sobra é um binário único.

**Veredito**: a migração é viável e o esforço é dominado por **uma única área de risco** — a fidelidade da exportação Parquet/JSON (Seção 10.1), que hoje depende de comportamentos implícitos do pandas. Todo o resto (CRUD genérico, paginação, busca, ordenação, merge, link, auth) é código mecânico.

**Restrição de banco de dados (definida)**: o sistema usará **um banco simples, SQLite em arquivo único**, embarcado no binário, acessado via **`rusqlite`**. Não há servidor de banco, não há Postgres, não há processo separado, não há pool assíncrono. Isso é uma decisão tomada, não uma opção em avaliação — ver AD-02.

**Esforço estimado**: 27–37 dias-desenvolvedor, assumindo familiaridade prévia com Rust. Sem essa familiaridade, adicione 30–50% para a curva de aprendizado de ownership/lifetimes e do modelo assíncrono.

**O que muda / o que permanece**:

| Camada | Hoje | Depois | Reescrita? |
|---|---|---|---|
| Componentes React (`.jsx`) | React 18 + Tailwind | **idêntico** | ❌ Não |
| Páginas e layouts Astro | Astro 4 SSG | idêntico, menos as rotas (Seção 10.6) | ⚠️ Ajuste pontual |
| `services/api.js` | `fetch` para REST | shim `fetch`→`invoke` | ⚠️ ~80 linhas |
| `DataControlCenter.jsx` | upload multipart + `<a download>` | diálogos nativos | ⚠️ Reescrever |
| API FastAPI (routers) | 10 endpoints REST | comandos `#[tauri::command]` | ✅ Sim |
| Services Python | pandas / SQLAlchemy | polars / rusqlite | ✅ Sim |
| ORM SQLAlchemy | 17 modelos declarativos | registry estático de **16** entidades + migrações SQL | ✅ Sim |
| Persistência | SQLite via SQLAlchemy | SQLite via rusqlite (embarcado) | ✅ Sim |
| Distribuição | `make run` (2 processos + venv) | `.deb` / `.rpm` / `.AppImage` / `.msi` | ✅ Sim |

---

## 2. Retrato do Sistema Atual

### 2.1 Inventário de código

```
backend/                    1.142 LOC Python (excl. venv/tests)
  src/api/                    107 LOC  4 routers, 10 endpoints
  src/services/               236 LOC  parquet, crud, link, merge, auth
  src/database/               233 LOC  2 adapters (memory, "postgres"), repositories
  src/models/orm.py           220 LOC  17 modelos, todos com colunas String
  src/core/                    40 LOC  security (bcrypt + JWT), exception handlers
  tests/                      104 LOC  pytest (contract, integration, unit)

frontend/                   1.044 LOC JS/Astro
  src/components/             597 LOC  7 componentes React
  src/pages/                  310 LOC  15 páginas de dashboard + login + root
  src/layouts/                 67 LOC  Dashboard, SplitScreen
  src/services/api.js          25 LOC  ← único ponto de acoplamento HTTP dos componentes
  tests/                              vitest (unit) + playwright (e2e)
```

### 2.2 Superfície da API

| Método | Rota | Handler | Comando Tauri proposto |
|---|---|---|---|
| POST | `/api/v1/auth/login` | `auth.login` | `login` |
| GET | `/api/v1/{entity}` | `crud.get_all` | `list_entities` |
| GET | `/api/v1/{entity}/{id}` | `crud.get_one` | `get_entity` |
| POST | `/api/v1/{entity}` | `crud.create` | `create_entity` |
| PUT | `/api/v1/{entity}/{id}` | `crud.update` | `update_entity` |
| DELETE | `/api/v1/{entity}/{id}` | `crud.delete` | `delete_entity` |
| POST | `/api/v1/merge/{entity_type}` | `special_ops.merge_entities` | `merge_entities` |
| POST | `/api/v1/link` | `special_ops.link_entities` | `link_entities` |
| POST | `/api/v1/data/import` | `routes.data.import_data` | `import_canonical_zip` |
| GET | `/api/v1/data/export` | `routes.data.export_data` | `export_canonical_zip` |

**10 endpoints → 10 comandos**, um para um (contagem conferida com `grep '@router\.'`: 5 em `crud.py`, 2 em `special_ops.py`, 2 em `routes/data.py`, 1 em `auth.py`). As rotas `import`/`export` deixam de ser HTTP e passam a operar sobre caminhos de arquivo escolhidos em diálogo nativo.

A sessão local (AD-05) acrescenta dois comandos que não têm equivalente REST hoje — `logout` e `session_status` —, totalizando **12 comandos** no app final.

### 2.3 Volume de dados (medido em `exports_canonical.zip`, 35 MB)

O ZIP contém **626 entradas**: 561 `.json`, 49 `.parquet`, 14 `.html`, 1 `.pdf` e **1 `.zip` aninhado** (`data_snapshot.zip`, 22,6 MB — sozinho, 65% do arquivo). Dessas, o sistema mapeia apenas 15 tabelas:

| Tabela | Linhas | Colunas | Tabela | Linhas | Colunas |
|---|---:|---:|---|---:|---:|
| `researchers` | 4.225 | 21 | `organizations` | 123 | 5 |
| `initiatives` | 4.122 | 19 | `proficiencies` | 209 | 8 |
| `students` | 2.625 | 21 | `advisorships` | 183 | 9 |
| `articles` | 2.027 | 9 | `awards` | 51 | 5 |
| `professional_activities` | 2.041 | 17 | `campuses` | 23 | 7 |
| `research_productions` | 951 | 13 | `fellowships` | 19 | 5 |
| `knowledge_areas` | 415 | 3 | `languages` | 8 | 3 |
| `research_groups` | 347 | 13 | | | |

Total: **~17,3 mil linhas**. Isso é pequeno — cabe folgadamente em memória. A implicação é importante: **a escolha do motor de dados não é dirigida por performance**, e sim por fidelidade de tipos e simplicidade. Não vale complicar a arquitetura por causa de volume.

O export sobrescreve exatamente **30 caminhos** (15 `parquet/*_canonical.parquet` + 15 `*_canonical.json`, todos já presentes no original). As outras **596 entradas** — 34 parquets não mapeados, grafos de colaboração, marts analíticos, `*.cols.json`, trackings, relatórios HTML, um PDF e o `data_snapshot.zip` — **não são tocadas** pelo sistema, mas precisam sobreviver ao round-trip (Seção 10.2).

### 2.4 Peculiaridade central do schema

Todas as 17 entidades em `backend/src/models/orm.py` declaram **todas as colunas como `Column(String)`**, inclusive `id` (que é `Integer`) e campos que no Parquet original são `bool` ou numéricos. Campos de relacionamento (`initiatives`, `research_groups`, `knowledge_areas`, `articles`, `advisorships`, `members`, `leaders`, `team`, ...) guardam **arrays JSON serializados como texto**.

Isso é ao mesmo tempo a maior facilidade e a maior armadilha da migração:

- **Facilidade**: o modelo de dados é essencialmente `HashMap<String, String>`. Em Rust não é preciso gerar 17 structs com 200+ campos — um registry genérico resolve (Seção 5, AD-07).
- **Armadilha**: os tipos originais só existem no ZIP importado, e o export precisa restaurá-los coluna a coluna (Seção 10.1).

---

## 3. Princípios da Migração

1. **Paridade funcional antes de melhoria.** Nenhuma feature nova entra até que o round-trip *importar → editar → exportar* produza um ZIP equivalente ao que o sistema Python produz hoje. A Seção 16 traz a matriz de paridade.
2. **A lógica de negócio vive em Rust.** O front-end continua burro: renderiza, coleta input, chama comando. Nada de SQL no JavaScript.
3. **Um ponto de acoplamento.** Todo o tráfego JS→Rust passa por `services/api.js`. Se esse contrato for preservado, os 7 componentes React não precisam ser tocados.
4. **Migração incremental verificável.** Cada SEP entrega algo demonstrável; o backend Python só é removido na última (SEP-024).
5. **TDD mantido.** A Constitution exige Red-Green-Refactor. `cargo test` substitui `pytest`; `vitest` permanece.
6. **Banco simples e embarcado.** SQLite em arquivo único dentro de `app_data_dir()`, sem servidor de banco e sem processo auxiliar. Nenhuma SEP pode introduzir Postgres, MySQL, DuckDB ou qualquer motor que exija instalação separada — se uma necessidade futura parecer pedir isso, é caso de emenda explícita a este estudo, não de decisão local dentro de uma feature.

---

## 4. Arquitetura Alvo

```
┌─────────────────────────────────────────────────────────────────┐
│  Janela Tauri (WebView: WebKitGTK / WKWebView / WebView2)        │
│                                                                 │
│  Astro 4 (build estático) + React 18 + Tailwind                 │
│  ├── pages/dashboard/*.astro    ← inalterado                    │
│  ├── components/*.jsx           ← inalterado                    │
│  └── services/api.js            ← SHIM: apiFetch() → invoke()   │
└───────────────────────────┬─────────────────────────────────────┘
                            │  IPC (invoke / emit), serde_json
┌───────────────────────────┴─────────────────────────────────────┐
│  src-tauri/ — processo Rust                                     │
│                                                                 │
│  commands/     #[tauri::command] — fronteira fina, sem lógica   │
│    ├── crud.rs        list / get / create / update / delete     │
│    ├── data.rs        import_canonical_zip / export_...         │
│    ├── special.rs     merge_entities / link_entities            │
│    └── auth.rs        login / logout / session_status           │
│                                                                 │
│  domain/       lógica pura, testável sem Tauri                  │
│    ├── registry.rs    EntityDef: route ↔ table ↔ colunas        │
│    ├── merge.rs       fusão transacional                        │
│    ├── link.rs        manipulação dos arrays JSON               │
│    └── canonical.rs   fidelidade de tipos no round-trip         │
│                                                                 │
│  infra/                                                         │
│    ├── db.rs          rusqlite::Connection + migrações          │
│    ├── parquet.rs     polars: leitura/escrita + coerção         │
│    ├── archive.rs     zip: leitura/escrita preservando entradas │
│    └── session.rs     State<Mutex<Option<Session>>>             │
└───────────────────────────┬─────────────────────────────────────┘
                            │
              ┌─────────────┴──────────────┐
              │                            │
      app_data_dir()/hub.db      app_data_dir()/original.zip
        (SQLite, dados curados)     (ZIP intacto, para o export)
```

Ganhos estruturais em relação ao desenho atual:

- **Um processo, não dois.** Some o `Makefile` de orquestração, o `trap 'kill 0'`, as portas 8000/4321 e o CORS.
- **Some a variável `STORAGE_TYPE`.** Hoje o README precisa avisar em negrito que sem `STORAGE_TYPE=postgres` a API cai silenciosamente no adaptador em memória e o dashboard fica vazio. Esse modo de falha desaparece — existe uma única implementação de persistência.
- **Some o adaptador em memória inteiro** (`DatabaseMemoryAdapter`, 68 LOC) e o `DatabasePostgresAdapter` (122 LOC), substituídos por um único módulo.
- **Caminhos corretos por SO.** Hoje o SQLite nasce em `./test.db` e o ZIP em `./uploads/`, relativos ao diretório de execução. Passam a viver em `app_data_dir()` (`~/.local/share/br.edu.ifes.researchhub/` no Linux, `%APPDATA%` no Windows, `~/Library/Application Support/` no macOS).

---

## 5. Decisões Arquiteturais

### AD-01 — Transporte: IPC nativo vs. servidor HTTP embutido

| Opção | Prós | Contras |
|---|---|---|
| **A. `invoke` nativo** ✅ | Sem porta TCP, sem CORS, sem serialização HTTP; tipos e erros fortemente tipados na fronteira; é o caminho idiomático do Tauri | Exige adaptar `api.js` |
| B. `axum` embutido no processo Tauri | `api.js` intocado; permite manter o front-end web em paralelo | Reintroduz porta local (conflito, firewall, superfície de rede), CORS e multipart — carrega todo o peso que a migração deveria eliminar |
| C. Backend Python como *sidecar* | Migração quase nula no curto prazo | Empacota o CPython dentro do bundle; anula os ganhos de distribuição; adia o problema |

**Decisão: A.** O argumento decisivo é que o custo de B (manter a camada HTTP para sempre) é maior que o custo de A (~80 linhas em um único arquivo). Todos os componentes React chamam `apiFetch()` de `services/api.js` — reescrever essa função como despachante para `invoke` mantém `EntityPage.jsx`, `LoginForm.jsx`, `EntityTable.jsx`, `EntityForm.jsx`, `MergeModal.jsx` e `LinkModal.jsx` **literalmente sem alteração**. A única exceção é `DataControlCenter.jsx`, que hoje usa `fetch` cru e um `<a href download>`, e que precisa ser reescrito de qualquer forma para usar diálogos nativos.

### AD-02 — Persistência: SQLite (decidido)

> **Decisão fechada.** O banco será **SQLite, arquivo único, embarcado no binário**, acessado via **`rusqlite`** (binding síncrono direto). As alternativas abaixo ficam registradas apenas como histórico da análise; não estão em aberto.

**Por que SQLite é a escolha certa aqui, e não apenas a mais simples:**

- **O volume permite.** São ~17,3 mil linhas no total (Seção 2.3). Isso é pequeno o suficiente para caber em memória inteiro — a escolha do motor não é dirigida por performance, então a variável que sobra é simplicidade operacional, e aí SQLite ganha por larga margem.
- **É o que já roda hoje.** O sistema atual já usa SQLite (`backend/test.db`); apesar do nome `DatabasePostgresAdapter` e da variável `STORAGE_TYPE=postgres`, nunca houve um Postgres de verdade no caminho. Manter SQLite significa **semântica idêntica** de tipos, `LIKE`, `ORDER BY` e coerção — o que reduz o risco de divergência de paridade (Seção 16).
- **Casa com o modelo desktop.** Um curador, uma máquina, um arquivo. Não faz sentido pedir que o usuário instale e administre um servidor de banco para editar planilhas acadêmicas.
- **Backup e suporte triviais.** O estado inteiro da curadoria é um arquivo copiável. Diagnosticar um problema de usuário é pedir esse arquivo.
- **Sem dependência externa no bundle.** `libsqlite3` é compilada estaticamente (feature `bundled`), então o instalador não ganha pré-requisitos.

**Configuração concreta:**

| Item | Definição |
|---|---|
| Motor | SQLite 3, embarcado (`bundled`, sem libsqlite do sistema) |
| Driver | `rusqlite` com features `bundled`, `serde_json`, `backup`, `functions` |
| Arquivo | `app_data_dir()/hub.db` (`~/.local/share/br.edu.ifes.researchhub/` no Linux, `%APPDATA%` no Windows, `~/Library/Application Support/` no macOS) |
| Schema | **16 tabelas** (as 17 de hoje menos `universities`, descartada em Q5), todas as colunas `TEXT` exceto `id INTEGER PRIMARY KEY` |
| Migrações | `rusqlite_migration` sobre `src-tauri/migrations/*.sql`, versionadas no repositório e ancoradas em `PRAGMA user_version` |
| Journal | WAL, com `busy_timeout` configurado |
| Concorrência | `Mutex<Connection>` em `tauri::State` para o CRUD interativo; o ETL (import/export) abre a própria `Connection` na thread de trabalho, para não segurar o mutex por segundos. Com WAL, as duas convivem sem bloqueio mútuo |
| Threading | Comandos declarados `#[tauri::command(async)]` com corpo síncrono — o Tauri os executa fora da main thread sem exigir `async fn` |

**Por que `rusqlite` e não `sqlx`** — a pergunta que sobrou depois de fixar o SQLite:

O `sqlx` é uma excelente biblioteca **para servidores**. Os três pilares que a justificam são async, connection pooling e queries verificadas em tempo de compilação. Ao deixar de ser web, o sistema perde os três:

| Vantagem do `sqlx` | Vale aqui? |
|---|---|
| Async nativo | **Não.** O Tauri já resolve isso: `#[tauri::command(async)]` executa um corpo *síncrono* fora da main thread. Não é preciso `async fn`, nem `spawn_blocking` espalhado. E as queries deste sistema levam microssegundos sobre 17 mil linhas — não há E/S concorrente a multiplexar |
| Connection pool | **Não.** Um curador, uma janela, um arquivo. Pool existe para amortizar handshake de rede e servir centenas de sessões simultâneas; abrir uma conexão SQLite local é uma chamada de função |
| `query!` — SQL checado em compilação | **Não.** Este é o argumento decisivo: o sistema é um **CRUD genérico dirigido por registry** (AD-07). Nome de tabela, lista de colunas, cláusula de busca e `ORDER BY` são montados em tempo de execução (Seção 8.5). O macro `query!` só valida SQL literal — com SQL dinâmico ele não pode ser usado, então justamente o recurso que mais distingue o `sqlx` fica inacessível neste código |

E o `rusqlite` traz ganhos concretos que o `sqlx` não oferece:

- **Compilação muito mais leve.** O `sqlx` arrasta `sqlx-core`, `sqlx-macros` e a stack async inteira. Com o Polars já pesando no build (risco 10.7), cortar essa árvore é economia real de minutos por build limpo, e o binário final encolhe.
- **`Connection::backup` (feature `backup`).** A importação **apaga a base inteira** (`drop_all` + `create_all` no código atual). Um snapshot automático de `hub.db` antes de cada import é uma rede de segurança óbvia para uma ferramenta de curadoria — e com o `rusqlite` é uma chamada de método. Incorporado ao escopo da SEP-018.
- **`create_scalar_function` / collations customizadas (feature `functions`).** É o caminho limpo para resolver a busca com acentos (risco 10.5) sem coluna desnormalizada: registra-se uma função de normalização em Rust e usa-se direto no `WHERE`. O `sqlx` não expõe esse ponto de extensão do SQLite.
- **Acesso direto a `PRAGMA`, transações e API C.** Menos camadas entre o código e o SQLite, o que casa com o princípio 6 ("banco simples").

Custo assumido: perde-se o `sqlx::migrate!`, substituído pelo crate `rusqlite_migration` (ou por um passo manual sobre `PRAGMA user_version`, ~30 linhas). Troca barata.

**Alternativas consideradas e descartadas** *(registro histórico — não reabrir sem emenda a este estudo)*:

| Opção | Motivo do descarte |
|---|---|
| `sqlx` com feature `sqlite` | Continua sendo SQLite, então seria uma escolha defensável. Descartado pelos motivos detalhados logo acima: os três recursos que justificam o `sqlx` (async, pool, queries checadas em compilação) não se aplicam a uma aplicação local monousuária com SQL dinâmico |
| Postgres | Exigiria servidor, instalação e administração pelo usuário final. Contradiz o objetivo de "um binário único" |
| DuckDB | Lê e escreve Parquet nativamente, o que é tentador para o import/export. Mas o workload real é CRUD linha a linha com paginação, busca e ordenação, onde um motor colunar não ajuda; e adiciona uma dependência pesada por um ganho que o Polars já entrega nas bordas |
| Só Polars em memória, sem banco | Sem durabilidade entre sessões; mutações pontuais em `DataFrame` são desconfortáveis; perde `WHERE`/`LIMIT`/`OFFSET` de graça |

Polars entra **apenas nas bordas de ETL** — ler o Parquet na importação e escrevê-lo na exportação (AD-03). Entre uma coisa e outra, a fonte da verdade é o SQLite.

> Se em algum momento o SQL dinâmico for substituído por queries literais por entidade, o `sqlx` volta a ser discutível. Enquanto o registry genérico for a arquitetura, `rusqlite` é a escolha certa.

### AD-03 — Motor Parquet

**Decisão: `polars`** com as features `lazy`, `parquet`, `dtype-full`. É o substituto direto do pandas, e a operação crítica do sistema — ler o schema do Parquet original e coagir as colunas de volta (Seção 10.1) — é expressa em Polars de forma **mais limpa e mais explícita** do que o código pandas atual, que depende de `pd.api.types.is_bool_dtype` e de um `.map()` com dicionário de conversões ad-hoc.

Alternativa: `arrow-rs` + crate `parquet` diretamente. Dá controle total sobre o schema, mas obriga a escrever manualmente toda a manipulação tabular. Só vale se aparecer alguma incompatibilidade de escrita no Polars.

### AD-04 — Front-end: manter Astro?

**Decisão: manter Astro**, em modo estático (`output: 'static'`, que já é o default). Motivos: (a) o usuário pediu explicitamente para manter JavaScript no front; (b) as 15 páginas de dashboard já existem e são triviais; (c) o build gera HTML/JS/CSS puro, exatamente o que o `frontendDist` do Tauri consome; (d) evita reescrever `Dashboard.astro`, `SplitScreen.astro` e a configuração do Tailwind.

O único ponto de atenção é o roteamento por sistema de arquivos sob o protocolo `tauri://` — tratado como risco na Seção 10.6 e como spike na SEP-015.

Alternativa descartada: trocar por Vite + React Router. Ganha um SPA com roteamento client-side (mais natural em desktop), mas exige reescrever 15 páginas e 2 layouts para obter o mesmo resultado visual. Não se paga agora; pode ser reavaliado depois da paridade.

### AD-05 — Autenticação

Constatação relevante da leitura do código: **a autenticação atual é decorativa**. `src/core/security.py` cria o JWT, mas `grep` por `decode`/`Depends(...)` mostra que **nenhum endpoint valida o token**. O `Authorization: Bearer` enviado pelos componentes React é ignorado pelo servidor; qualquer cliente pode chamar `/api/v1/researchers` sem credencial. O login serve hoje só como porta visual do front-end.

Numa aplicação desktop local, JWT não faz sentido algum — não há canal a proteger.

**Decisão**: manter o *fluxo* de login (a tela existe, o Figma exige, a Constitution menciona o perfil Admin único) e trocar o mecanismo por **sessão em memória no processo Rust**: `State<Mutex<Option<Session>>>`, com verificação de senha via `bcrypt` contra o hash na tabela `admins`. Os comandos passam a checar a sessão de verdade — o que torna a autenticação *mais* real do que a de hoje, não menos. `localStorage.setItem('token', ...)` continua funcionando no front-end como flag de UI, sem valor criptográfico.

Se a paridade estrita for exigida, o crate `jsonwebtoken` reproduz o comportamento atual em ~20 linhas. Registrar como decisão explícita na SEP-021.

### AD-06 — Import / Export nativos

Hoje: `<input type="file">` → `FormData` → `multipart/form-data` → `UploadFile.read()` → 35 MB em memória do processo Python.
Depois: `tauri-plugin-dialog` → caminho absoluto → Rust lê o arquivo direto do disco (streaming, sem cópia pela IPC).

Hoje: `<a href="http://localhost:8000/.../export" download>` → `StreamingResponse` que faz `iter([buffer.getvalue()])` (ou seja, materializa o ZIP inteiro em memória antes de "stream-ar").
Depois: diálogo de salvar → Rust escreve direto no caminho escolhido.

Ganho adicional: o import atual não tem feedback de progresso — o botão só diz "Processando...". Com `app.emit("import://progress", ...)` e `listen()` no front, dá para mostrar tabela a tabela. Barato e visível.

Suportar também drag-and-drop do ZIP na janela via evento `tauri://drag-drop` (v2; era `tauri://file-drop` na v1), com `"dragDropEnabled": true` na config da janela.

### AD-07 — Registry de entidades

Rust não tem reflexão em runtime, então o truque do `postgres_adapter.py` (dicionário `{"groups": ResearchGroup, ...}` + `obj.__table__.columns`) precisa de equivalente explícito. Como todas as colunas são texto, não é preciso gerar structs: basta uma tabela estática de metadados.

Detalhe crítico que o registry precisa capturar: **o namespace de rota e o namespace de tabela divergem**. O front-end chama `/groups`, mas a tabela é `research_groups`, e o arquivo canônico é `research_groups_canonical.parquet`. Hoje esse mapeamento está espalhado por três lugares (o dict do `postgres_adapter`, o `column_map` do `link_service`, o `getRoute()` do `EntityForm.jsx`). Centralizar isso é uma melhoria de manutenibilidade que sai de graça na reescrita.

```rust
pub struct EntityDef {
    pub route:           &'static str,   // "groups"          — o que o front-end usa
    pub table:           &'static str,   // "research_groups" — SQLite e Parquet
    pub columns:         &'static [&'static str],
    pub search_column:   Option<&'static str>,  // name | title | username
    pub json_columns:    &'static [&'static str], // ordenados por length()
    pub exported:        bool,           // false apenas para admins
}
```

### AD-08 — Estado e concorrência

- `Mutex<rusqlite::Connection>` guardado em `tauri::State<AppState>`, aberto no `setup()`.
- Comandos declarados `#[tauri::command(async)]` com **corpo síncrono**. O Tauri executa comandos sem a palavra-chave `async` na main thread; com o modificador `(async)`, um corpo síncrono passa a rodar em thread separada do runtime. É isso que torna o `rusqlite` confortável aqui — não há `async fn` nem `spawn_blocking` em todo comando.
- Retorno `Result<T, AppError>` com `AppError: serde::Serialize` — o front recebe `{ message, kind }` e o shim converte em `throw new Error(message)`, preservando o contrato do `apiFetch` atual (que faz `throw new Error(data.detail)`).
- Import e export rodam em `tauri::async_runtime::spawn_blocking` com **conexão própria** (Polars é síncrono e CPU-bound, e a operação dura segundos) — assim o mutex do CRUD nunca fica retido durante o ETL.
- SQLite em modo WAL, `busy_timeout` configurado. Com WAL, a leitura interativa e a escrita do ETL não se bloqueiam.

---

## 6. Mapa de Equivalência Python → Rust

| Origem | Destino | Crate/mecanismo | Observação |
|---|---|---|---|
| `src/api/main.py` | `src-tauri/src/lib.rs` | `tauri::Builder` | CORS e middleware somem |
| `src/api/crud.py` | `commands/crud.rs` | `#[tauri::command]` | 5 comandos genéricos |
| `src/api/auth.py` | `commands/auth.rs` | — | ver AD-05 |
| `src/api/special_ops.py` | `commands/special.rs` | — | + correção do bug 10.3 |
| `src/api/routes/data.py` | `commands/data.rs` | `tauri-plugin-dialog` | sem multipart |
| `src/services/parquet_service.py` | `domain/canonical.rs` + `infra/parquet.rs` + `infra/archive.rs` | `polars`, `zip` | **núcleo de risco** |
| `src/services/crud_service.py` | absorvido em `commands/crud.rs` | — | camada era pass-through pura |
| `src/services/merge_service.py` | `domain/merge.rs` | transação `rusqlite` | hoje sem transação |
| `src/services/link_service.py` | `domain/link.rs` | `serde_json` | hoje quebrado, ver 10.3 |
| `src/services/auth_service.py` | `domain/auth.rs` | `bcrypt` | unifica os 2 caminhos divergentes |
| `src/database/core.py` | **removido** | — | adaptador em memória some |
| `src/database/postgres_adapter.py` | `infra/db.rs` | `rusqlite` | |
| `src/database/repositories.py` | `infra/db.rs` | `rusqlite` | |
| `src/database/session.py` | `infra/db.rs` (conexão no `setup`) | `Mutex<rusqlite::Connection>` | |
| `src/models/orm.py` | `domain/registry.rs` + `migrations/001_init.sql` | const estáticas | 220 LOC → ~120 |
| `src/core/security.py` | `domain/auth.rs` | `bcrypt` | |
| `src/core/exceptions.py` | `error.rs` | `thiserror` + `serde` | |
| `create_admin.py`, `scripts/seed.py` | `infra/db.rs::seed_admin()` | — | roda no `setup()`, idempotente |
| `Makefile` (run/build) | `cargo tauri dev` / `cargo tauri build` | — | |

---

## 7. Estrutura de Diretórios Proposta

```
research_hub/
├── frontend/                    ← permanece, com ajustes mínimos
│   ├── astro.config.mjs         ← + output/trailingSlash (Seção 10.6)
│   ├── src/
│   │   ├── components/          ← 6 de 7 intocados
│   │   ├── pages/  layouts/     ← intocados (salvo hrefs da nav)
│   │   └── services/api.js      ← SHIM invoke
│   └── tests/
│       ├── unit/                ← vitest, permanece
│       └── e2e/                 ← Playwright → WebdriverIO (Seção 11)
│
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── capabilities/default.json
│   ├── icons/
│   ├── migrations/001_init.sql
│   └── src/
│       ├── main.rs  lib.rs  error.rs  state.rs
│       ├── commands/  auth.rs crud.rs data.rs special.rs mod.rs
│       ├── domain/    registry.rs auth.rs merge.rs link.rs canonical.rs mod.rs
│       └── infra/     db.rs parquet.rs archive.rs mod.rs
│
├── backend/                     ← congelado; removido na SEP-024
├── specs/                       ← SEPs 015+ nascem aqui
├── docs/estudo-migracao-rust-tauri.md   ← este arquivo
└── exports_canonical.zip        ← fixture de referência dos testes
```

---

## 8. Esboços de Configuração e Código

> Versões de crates são **indicativas**. Fixe as vigentes no momento de rodar `cargo add`.

### 8.1 `src-tauri/Cargo.toml`

```toml
[package]
name         = "research-hub"
version      = "0.1.0"
edition      = "2021"
rust-version = "1.77.2"          # MSRV do Tauri 2.0

[build-dependencies]
tauri-build = "2"

[dependencies]
tauri               = { version = "2", features = [] }
tauri-plugin-dialog = "2"   # usado só do lado Rust (Q1) — sem permissão no front
tauri-plugin-log    = "2"
# tauri-plugin-fs NAO entra: todo I/O de arquivo acontece em Rust com std::fs

serde      = { version = "1", features = ["derive"] }
serde_json = "1"

# Banco local, embarcado, síncrono. Sem servidor, sem pool, sem async.
rusqlite = { version = "0.32", features = [
  "bundled",      # compila o SQLite junto: zero dependência de sistema
  "serde_json",   # colunas JSON <-> serde_json::Value
  "backup",       # snapshot de hub.db antes de cada import (SEP-018)
  "functions",    # função de normalização p/ busca com acentos (risco 10.5)
] }
rusqlite_migration = "1"

polars = { version = "0.4x", features = [
  "lazy", "parquet", "dtype-full", "strings", "is_in"
] }

zip       = "2"
bcrypt    = "0.15"
thiserror = "2"
tracing   = "0.1"
```

### 8.2 `src-tauri/tauri.conf.json`

> ⚠️ Os comentários abaixo são **didáticos**. `tauri.conf.json` é JSON estrito e não aceita `//` — copiar este bloco como está causa erro de parse. Para manter comentários, use `tauri.conf.json5` (ou `Tauri.toml`); caso contrário, remova-os.

```jsonc
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "ResearchHub",
  "version": "0.1.0",
  "identifier": "br.edu.ifes.researchhub",
  "build": {
    "beforeDevCommand":   "npm --prefix ../frontend run dev",
    "devUrl":             "http://localhost:4321",
    "beforeBuildCommand": "npm --prefix ../frontend run build",
    "frontendDist":       "../frontend/dist"
  },
  "app": {
    "windows": [{
      "title": "ResearchHub — Curadoria de Dados",
      "width": 1440, "height": 900, "minWidth": 1024,
      "dragDropEnabled": true
    }],
    "security": {
      // Sem chamadas remotas: CSP pode ser estrita.
      // 'unsafe-inline' em style-src é exigido pelos estilos inline do Astro/Tailwind.
      "csp": "default-src 'self'; img-src 'self' asset: data:; style-src 'self' 'unsafe-inline'"
    }
  },
  "bundle": {
    "active": true,
    "targets": ["deb", "rpm", "appimage", "nsis", "dmg"],
    "icon": ["icons/32x32.png", "icons/128x128.png", "icons/icon.icns", "icons/icon.ico"]
  }
}
```

### 8.3 `src-tauri/capabilities/default.json`

Na v2 as permissões são explícitas por janela — nada de `allowlist` global como na v1.

> **Decidido (Q1): o Rust abre o diálogo.** O comando `import_canonical_zip` chama `app.dialog().file()` internamente e lê o arquivo com `std::fs`; o front-end só faz `invoke('import_canonical_zip')`. Consequências: `tauri-plugin-fs` sai do `Cargo.toml`, nenhuma permissão `dialog:*` ou `fs:*` é declarada, e o conjunto fica reduzido a `core:default`. Menos superfície e a lógica toda em um lugar.

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "windows": ["main"],
  "permissions": [
    "core:default"
  ]
}
```

### 8.4 Registry (`domain/registry.rs`)

```rust
pub const ENTITIES: &[EntityDef] = &[
    EntityDef {
        route: "researchers", table: "researchers",
        columns: &["id","name","identification_id","birthday","cnpq_url",
                   "google_scholar_url","resume","citation_names","initiatives",
                   "research_groups","knowledge_areas","academic_education",
                   "articles","advisorships","classification",
                   "classification_confidence","classification_note",
                   "role_evidence","was_student","was_staff","campus"],
        search_column: Some("name"),
        json_columns: &["initiatives","research_groups","knowledge_areas",
                        "academic_education","articles","advisorships"],
        exported: true,
    },
    // 'groups' na rota, 'research_groups' na tabela — divergência centralizada aqui
    EntityDef {
        route: "groups", table: "research_groups",
        columns: &["id","name","description","short_name","organization_id",
                   "campus_id","cnpq_url","site","organization","campus",
                   "knowledge_areas","members","leaders"],
        search_column: Some("name"),
        json_columns: &["knowledge_areas","members","leaders"],
        exported: true,
    },
    // ... demais 13 entidades exportáveis + admins (exported: false) = 16 no total
    // 'universities' foi descartada (Q5): sem página, sem parquet, sem uso
];

pub fn by_route(route: &str) -> Result<&'static EntityDef, AppError> {
    ENTITIES.iter().find(|e| e.route == route)
        .ok_or_else(|| AppError::UnknownEntity(route.into()))
}
```

O `find` linear sobre 16 elementos é irrelevante em custo e **fecha a superfície**: hoje `/api/v1/{entity}` aceita qualquer string e só falha lá no fundo com um `ValueError` que vira 500.

### 8.5 Comando de listagem (`commands/crud.rs`)

Reproduz a semântica de `DatabasePostgresAdapter.get_all`, incluindo a ordenação por `length()` nas colunas JSON e o `NULLS LAST`.

```rust
#[derive(serde::Serialize)]
pub struct Page {
    pub items: Vec<serde_json::Value>,
    pub total: i64,
}

// (async) => corpo síncrono executado fora da main thread pelo runtime do Tauri
#[tauri::command(async)]
pub fn list_entities(
    state:  tauri::State<'_, AppState>,
    entity: String,
    limit:  Option<i64>,
    offset: Option<i64>,
    search: Option<String>,
    sort:   Option<String>,
    order:  Option<String>,
) -> Result<Page, AppError> {
    state.require_session()?;
    let conn   = state.db.lock()?;
    let def    = registry::by_route(&entity)?;
    let limit  = limit.unwrap_or(100).clamp(1, 1000);
    let offset = offset.unwrap_or(0).max(0);

    let mut where_sql = String::new();
    if let (Some(term), Some(col)) = (search.as_ref(), def.search_column) {
        if !term.is_empty() {
            // LIKE do SQLite é case-insensitive para ASCII; ver risco 10.5
            where_sql = format!(" WHERE {col} LIKE ?1");
        }
    }

    // Ordenação: coluna validada contra o registry (nunca interpolar input cru)
    let order_sql = match sort.as_deref().filter(|s| def.columns.contains(s)) {
        Some(col) => {
            let expr = if def.json_columns.contains(&col) {
                format!("length({col})")          // paridade com func.length()
            } else {
                col.to_string()
            };
            let dir = if order.as_deref() == Some("desc") { "DESC" } else { "ASC" };
            format!(" ORDER BY ({expr} IS NULL), {expr} {dir}")  // NULLS LAST
        }
        None => String::new(),
    };
    // ... contagem + página, mapeando linhas para serde_json::Value
}
```

### 8.6 Shim do front-end (`frontend/src/services/api.js`)

Este é o arquivo que faz os componentes React sobreviverem sem alteração. Ele preserva o contrato observável do `apiFetch` atual: recebe `endpoint` + `options`, devolve o JSON, e lança `Error` com a mensagem de erro.

```js
import { invoke } from '@tauri-apps/api/core';

// [regex, método, handler(match, body)]
const ROUTES = [
  [/^\/auth\/login$/, 'POST', (_m, b) =>
      invoke('login', { email: b.email, password: b.password })],

  [/^\/merge\/([^/?]+)$/, 'POST', (m, b) =>
      invoke('merge_entities', {
        entity: m[1], sourceIds: b.source_ids, resolvedData: b.resolved_data })],

  [/^\/link$/, 'POST', (_m, b) =>
      invoke('link_entities', {
        parentType: b.parent_type, parentId: b.parent_id,
        childType:  b.child_type,  childId:  b.child_id })],

  [/^\/([^/?]+)\/(\d+)$/, 'GET',    (m) => invoke('get_entity',    { entity: m[1], id: +m[2] })],
  [/^\/([^/?]+)\/(\d+)$/, 'PUT',    (m, b) => invoke('update_entity', { entity: m[1], id: +m[2], payload: b })],
  [/^\/([^/?]+)\/(\d+)$/, 'DELETE', (m) => invoke('delete_entity', { entity: m[1], id: +m[2] })],
  [/^\/([^/?]+)$/,        'POST',   (m, b) => invoke('create_entity', { entity: m[1], payload: b })],
  [/^\/([^/?]+)$/,        'GET',    (m, _b, q) => invoke('list_entities', {
        entity: m[1],
        limit:  q.get('limit')  ? +q.get('limit')  : null,
        offset: q.get('offset') ? +q.get('offset') : null,
        search: q.get('search'), sort: q.get('sort'), order: q.get('order') })],
];

export async function apiFetch(endpoint, options = {}) {
  const [path, qs = ''] = endpoint.split('?');
  const query  = new URLSearchParams(qs);
  const method = (options.method || 'GET').toUpperCase();
  const body   = options.body ? JSON.parse(options.body) : undefined;

  for (const [re, verb, handler] of ROUTES) {
    const m = path.match(re);
    if (m && verb === method) {
      try {
        return await handler(m, body, query);
      } catch (err) {
        // Mesmo contrato de erro do apiFetch atual (throw new Error(detail))
        throw new Error(err?.message ?? String(err));
      }
    }
  }
  throw new Error(`Rota não mapeada: ${method} ${path}`);
}
```

> Nota: `invoke` converte automaticamente argumentos `camelCase` do JS para `snake_case` dos parâmetros Rust — por isso `sourceIds` acima chega como `source_ids`.

### 8.7 `astro.config.mjs`

```js
import { defineConfig } from 'astro/config';
import react from '@astrojs/react';
import tailwind from '@astrojs/tailwind';

// Nenhuma alteração necessária: o spike da SEP-015 (risco 10.6) mostrou que o
// resolvedor do Tauri já lida com as rotas em formato de diretório do Astro.
// O arquivo permanece como está hoje no repositório.
export default defineConfig({
  integrations: [react(), tailwind()],
});
```

---

## 9. Estratégia de Migração

| Fase | Entrega | Verificação |
|---|---|---|
| **0. Spike** | Shell Tauri abrindo o build Astro atual; navegação entre as 15 páginas funcionando sob `tauri://` | Clicar em todos os itens da sidebar sem 404 |
| **1. Leitura** | `list_entities` / `get_entity` sobre um `hub.db` gerado pelo backend Python | Tabelas renderizam com os mesmos dados que a versão web |
| **2. Escrita** | `create` / `update` / `delete` | CRUD completo em `campuses` (23 linhas, fácil de conferir à mão) |
| **3. Ingestão** | `import_canonical_zip` | Contagem de linhas por tabela idêntica à do import Python |
| **4. Egressão** | `export_canonical_zip` | **Diff estrutural** contra o ZIP gerado pelo Python (Seção 11.3) |
| **5. Operações** | merge + link | Comportamento de fusão preservado; link passa a funcionar |
| **6. Sessão** | login real | Comandos rejeitam chamada sem sessão |
| **7. Distribuição** | bundles `.deb` / `.AppImage` / `.msi` | Instalar em máquina limpa, sem Python nem Node |
| **8. Descomissionamento** | `backend/` removido, README/Makefile atualizados | `make` some ou vira wrapper de `cargo tauri` |

Durante as fases 1–6 o backend Python **continua no repositório e funcionando**. Isso é deliberado: ele é o **oráculo de paridade**. Rodar os dois lado a lado sobre o mesmo `exports_canonical.zip` e comparar as saídas é o teste mais forte disponível, e ele desaparece se o Python for removido cedo demais.

---

## 10. Riscos e Armadilhas

### 10.1 🔴 CRÍTICO — Fidelidade de tipos no export

Este é o risco dominante do projeto. O banco guarda **tudo como texto**; o Parquet original tem `bool`, `int64`, `float64`. `parquet_service.py` (linhas 78–103) reabre o ZIP original, lê o `DataFrame` original de cada tabela e, coluna a coluna:

1. Se a original é `bool`, mapeia `{'1','0','True','False',1,0,True,False}` → `bool`;
2. Se é numérica, faz `pd.to_numeric(errors='coerce')` e depois `astype(dtype_original)`;
3. Reordena as colunas exatamente na ordem do schema original.

Reproduzir isso em Polars é factível e fica **mais legível** (lê-se o schema do Parquet original sem materializar os dados e aplica `cast` por coluna), mas os detalhes são traiçoeiros:

- `errors='coerce'` transforma lixo em `NaN` **silenciosamente**. Polars com `strict=false` faz o análogo (vira `null`), mas `NaN` e `null` **não são a mesma coisa em Parquet**. Precisa de teste dedicado.
- Colunas que existem no banco mas não no original são mantidas por pandas (o loop itera sobre `df_orig.columns`), mas descartadas no reordenamento final. Comportamento sutil, precisa ser espelhado.
- Se `original.zip` não existir (primeira execução sem import), o bloco inteiro é pulado e tudo sai como string. Esse caminho degradado também precisa existir.

**Mitigação**: SEP-019 dedicada só a isso, com teste de round-trip byte-comparável por schema (não por bytes do arquivo, que variam com compressão/metadata do writer — comparar schema Arrow + valores).

### 10.2 🔴 CRÍTICO — Preservação das 596 entradas não gerenciadas

O export copia **todas** as entradas do `original.zip` que não foram sobrescritas (`parquet_service.py`, linhas 138–143), e só então grava as novas. São **596 entradas** (contagem verificada) que o sistema nunca lê mas que precisam sair intactas.

Em Rust, o crate `zip` permite ler cada entrada e regravá-la. Cuidado com preservação de: nome exato (incl. subdiretórios `parquet/`), método de compressão, e o comportamento atual de reusar o `ZipInfo` original (`z.writestr(item, ...)` preserva timestamp e atributos). Um export que perca esses arquivos **destrói dados do DataLake do usuário** sem aviso.

Dois detalhes que só aparecem olhando o conteúdo real e que valem teste próprio:

- **`data_snapshot.zip` é um ZIP dentro do ZIP, com 22,6 MB** — sozinho, quase dois terços do arquivo. Ele precisa ser copiado byte a byte, **sem recompressão**: um round-trip que o descomprima e recomprima muda o conteúdo e pode estourar o uso de memória. Copiar entrada por entrada com o método de compressão original resolve, mas é preciso garantir explicitamente que é isso que acontece.
- Há também **14 relatórios `.html` e 1 `.pdf` de 1,7 MB** em subdiretórios (`mestrado/`, `formandos/`). São binários e texto que nada têm a ver com o schema canônico — reforçam que o copiador precisa ser agnóstico a tipo de arquivo.

### 10.3 🟡 Bugs atuais que NÃO devem ser reproduzidos

**Decidido (Q4): corrigir todos.** Cada correção é registrada como *diferença esperada* no script de paridade (Seção 11.3) — nenhuma delas altera o formato dos dados exportados, então o oráculo Python continua válido para o que mais importa. O que não pode acontecer é correção silenciosa: cada item abaixo vira uma linha explícita de escopo na SEP correspondente.

1. **`LinkService.link_entities` está quebrado.** `src/services/link_service.py:41` chama `parent_repo.save(parent)`, mas `BaseRepository` (`src/database/repositories.py`) não define `save` — só `get_all`, `get_by_id`, `create`, `update`, `delete`. Toda vez que o link **efetivamente** anexa um vínculo novo, o método estoura `AttributeError` → HTTP 500. O front-end (`EntityPage.handleLink`) exibe "Vínculo criado com sucesso!" apenas no caminho de sucesso, então o usuário vê o erro — mas a funcionalidade nunca funcionou de fato. A versão Rust deve implementá-la corretamente e cobri-la com teste.
2. **Merge sem transação.** `MergeService.merge` salva a entidade primária e depois deleta as demais em chamadas independentes. Uma falha no meio deixa dados órfãos. Em Rust: uma transação `rusqlite` (`conn.transaction()`), com commit único ao final.
3. **Export engole exceções.** O `except Exception` por tabela apenas imprime no stdout e segue (`parquet_service.py:105-109`). Uma tabela pode sumir do ZIP exportado sem que ninguém perceba. A versão Rust deve falhar alto ou devolver um relatório explícito de tabelas com erro.
4. **Autenticação divergente entre adaptadores.** `auth_service.py` procura por `email` no caminho em memória e por `username` no caminho SQL — e os schemas de `Admin` também divergem entre `orm.py` (`username`/`hashed_password`) e `scripts/seed.py` (`email`/`password_hash`, campos que **não existem** no modelo — o seed quebra se rodado). Unificar em um único schema.
5. **Nenhum endpoint valida o JWT.** Ver AD-05.
6. **`DELETE` bem-sucedido aparece como erro para o usuário.** `services/api.js:16` chama `await response.json()` **antes** de checar `response.ok` (linha 18). O endpoint de exclusão devolve `204 No Content` com corpo vazio (`crud.py:38`), e `.json()` sobre corpo vazio rejeita. Resultado: `EntityPage.handleDelete` cai no `catch`, exibe um `alert` com uma mensagem de erro de parsing, e **não chama `loadData()`** — a linha excluída continua na tela até o usuário recarregar. Ou seja, a exclusão funciona no servidor e parece ter falhado na interface. No modelo `invoke` isso desaparece de graça (um comando que devolve `()` chega como `null`), mas o comportamento precisa estar na matriz de paridade como **corrigido**, não como equivalente.
7. **Código morto**: o caso especial `if table_name == "groups"` no export nunca dispara, porque o `__tablename__` é `research_groups`. Não portar.
8. **A tabela `universities` é órfã — e foi descartada.** Está em `orm.py` e no dicionário do `postgres_adapter`, mas não tem página no dashboard, não tem parquet no ZIP canônico e é explicitamente pulada no export. Nunca recebe nem devolve dado. **Decidido (Q5): não portar.** O schema Rust nasce com 16 tabelas.

### 10.4 🟡 IDs e chaves primárias

`DatabaseMemoryAdapter.save` gera IDs com `len(dict) + 1` e um loop de incremento — frágil após deleções. O `DatabasePostgresAdapter.save` tem um caminho de "força insert com ID explícito" que, em SQLite, funciona por acidente. E o Parquet importado traz IDs próprios que precisam ser preservados (os arrays JSON de vínculo referenciam esses IDs).

Decisão a registrar na SEP-016: `id INTEGER PRIMARY KEY` no SQLite (alias de `rowid`), aceitando ID explícito no import e usando `AUTOINCREMENT` implícito na criação manual. Documentar o comportamento em caso de colisão.

### 10.5 🟡 Busca case-insensitive com acentos

O `ilike` do SQLAlchemy sobre SQLite vira `LIKE`, que é case-insensitive **apenas para ASCII**. Buscar "joao" não acha "João" hoje, e não achará em Rust também. Paridade estrita significa manter o comportamento.

A escolha do `rusqlite` (AD-02) deixa a correção futura barata: com a feature `functions`, registra-se uma função escalar de normalização Unicode em Rust e usa-se direto no `WHERE`, sem coluna desnormalizada nem reindexação. Dado que o corpus é de nomes brasileiros, vale a pena — mas como SEP própria, depois da paridade.

### 10.6 ✅ RESOLVIDO — Roteamento Astro sob o protocolo `tauri://`

> **Resolvido empiricamente na SEP-015 em 2026-09-02: nenhuma ação necessária.** Detalhes em `specs/015-tauri-shell-bootstrap/plan.md`, Fase 0.

**A dúvida era**: as páginas Astro geram `dashboard/groups/index.html`, enquanto `Dashboard.astro` aponta para `/dashboard/groups`. Um servidor HTTP resolve diretório → `index.html`; o protocolo de asset do Tauri poderia não resolver.

**O spike** consultou o próprio `AppHandle::asset_resolver()` — o mesmo componente que o protocolo usa — testando as 18 rotas em quatro formas de URL. Resultado: **18/18 resolvem em todas as quatro formas**, inclusive o href nu, sem barra final e sem extensão. O Tauri normaliza o caminho e acrescenta `index.html` sozinho.

**Consequência**: os hrefs do front-end ficam como estão. As três mitigações abaixo foram todas descartadas — e duas delas eram, na verdade, **incompatíveis com a própria feature**:

| Mitigação prevista | Veredito |
|---|---|
| `trailingSlash: 'always'` + barra nos hrefs | Desnecessária — e exigiria alterar `LoginForm.jsx` e `EntityForm.jsx`, violando a regra de não tocar em componentes React |
| `build.format: 'file'` + `.html` nos hrefs | Desnecessária — mesmo problema |
| Handler de protocolo customizado em Rust | Desnecessária — seria código reimplementando o que o framework já faz |

Lição de método que vale para os riscos restantes: `LoginForm.jsx` faz `window.location.href = '/dashboard'` e `EntityForm.jsx` monta `` `/dashboard/${route}?openId=${item.id}` ``. Adotar qualquer mitigação "por precaução", sem medir, teria quebrado o critério de aceite da feature. **Medir primeiro custou um build; supor teria custado a feature.**

O spike virou teste permanente (`src-tauri/src/lib.rs`), com as 18 rotas verificadas a cada `cargo test`.

Observação sobre a sidebar: ela hoje mistura `<a>` soltos com `<li>` fora de `<ul>` e classes de tema claro dentro de um painel escuro (os itens a partir de "Grupos de Pesquisa" usam `text-slate-700 hover:bg-slate-100`, invisíveis sobre `bg-slate-900`). **Decidido (Q7): corrigir junto, dentro da SEP-015**, como item explícito de escopo — o arquivo já será tocado pelo ajuste de rotas.

### 10.7 🟢 Menores

- **Tempo de compilação**: Polars é uma dependência pesada; builds limpos podem passar de 10 min. Usar `sccache` ou cache de `target/` no CI. Ficar com `rusqlite` em vez de `sqlx` (AD-02) já poupa a árvore async inteira.
- **Tamanho do binário**: Polars com `dtype-full` engorda o bundle (dezenas de MB). Se incomodar, reduzir features para só os dtypes usados.
- **Dependências de sistema no Linux**: `libwebkit2gtk-4.1-dev`, `build-essential`, `curl`, `wget`, `file`, `libxdo-dev`, `libssl-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`. Documentar no README.
- **Toolchain ausente**: a máquina de desenvolvimento atual **não tem `cargo`/`rustc` instalados** (só Node v22.23.1). Instalar via `rustup` é pré-requisito da SEP-015.
- **`exports_canonical.zip` (35 MB) está versionado no Git.** Já é um problema hoje; se virar fixture de teste, considerar Git LFS ou um subconjunto reduzido para o CI.

---

## 11. Estratégia de Testes

### 11.1 Rust — `cargo test`

- **Unitários** (`domain/`, sem Tauri): registry, coerção de tipos canônicos, merge, manipulação dos arrays JSON de link, verificação de senha.
- **Integração**: CRUD completo contra `Connection::open_in_memory()` com as migrações aplicadas — sem fixture externa e sem I/O de disco; paginação, busca, ordenação (inclusive por `length()` nas colunas JSON e `NULLS LAST`).
- **Round-trip** (o teste que mais importa): importar um ZIP fixture → exportar → reabrir e comparar schema e valores por tabela.

### 11.2 Front-end

- **Vitest** permanece. `tests/unit/api.test.js` precisa de um mock de `@tauri-apps/api/core` — na prática ele fica *mais* fácil de testar do que a versão com `fetch`, porque `invoke` é uma função única e mockável.
- **Playwright não dirige o WebView do Tauri.** O caminho oficial é **`tauri-driver` + WebdriverIO** (WebKitWebDriver no Linux, msedgedriver no Windows). Substituir `tests/e2e/login.spec.js`.
- **Cobertura E2E de 100% dos alvos.** Com Linux e Windows como únicos alvos de v1 (Q2), o `tauri-driver` cobre todas as plataformas suportadas. Não existe verificação manual no processo — tudo que é alvo de release é alvo de teste automatizado.

### 11.3 Paridade contra o oráculo Python

Enquanto `backend/` existir, manter um script comparativo:

```
1. python: importar exports_canonical.zip → exportar → out_py.zip
2. rust:   importar exports_canonical.zip → exportar → out_rs.zip
3. comparar: conjunto de entradas, schema Arrow por parquet,
             valores por tabela, JSONs normalizados
```

Comparar **bytes** dos arquivos não funciona (compressão, metadata de writer e ordem de escrita diferem entre implementações). Comparar **conteúdo estruturado** é o critério correto — e é o que define "pronto" para a SEP-019.

---

## 12. Build, Empacotamento e Distribuição

| Item | Detalhe |
|---|---|
| Dev | `cargo tauri dev` (sobe o Astro via `beforeDevCommand` e faz HMR) |
| Build | `cargo tauri build` |
| Alvos | **v1**: `.deb`, `.rpm`, `.AppImage` (Linux) e `.msi`/NSIS (Windows). **macOS fica para a v2** (Q2) |
| Cross-compile | Não há cross-compile confiável no Tauri — **um runner por SO** no CI (`tauri-apps/tauri-action` no GitHub Actions): matriz de dois, `ubuntu-latest` e `windows-latest` |
| Assinatura | **Decidido (Q3): não assinar.** Uso pessoal e local dispensa. No Windows, o SmartScreen mostra "Mais informações → Executar assim mesmo" na primeira execução; documentar isso no README. No Linux não há barreira equivalente |
| v2 — macOS | Alvos `.dmg` + `.app`, arquiteturas `aarch64-apple-darwin` e `x86_64-apple-darwin` (ou `universal-apple-darwin`), Xcode Command Line Tools, runner `macos-latest` no CI e roteiro próprio de verificação — o `tauri-driver` não suporta macOS. Nada disso entra na v1 |
| Updater | `tauri-plugin-updater` + chave de assinatura. Opcional na v1 do desktop; se entrar, exige endpoint de manifest hospedado |
| CI atual | O repositório teve workflows de GitHub Pages e Docker revertidos (`cc0161d`). O pipeline novo é de build de binários, não de deploy web — o legado não se aproveita |

---

## 13. Impacto na Constitution do speckit

⚠️ **Bloqueador de processo.** A Constitution vigente (`.specify/memory/constitution.md`, v1.0.0) é **incompatível com esta migração** em três pontos. Nenhuma spec pode passar no *Constitution Check* do `plan-template.md` enquanto isso não for emendado:

| Cláusula | Texto | Conflito |
|---|---|---|
| Princípio II — Reuso do Domínio | "O backend em Python deve expor a biblioteca `research_domain` […] É expressamente proibido reimplementar entidades já existentes nela" | O backend deixa de ser Python. Nota: a proibição já é letra morta hoje — `orm.py` reimplementa Researcher, Campus, ResearchGroup, Article etc. e `research_domain` não aparece em `requirements.txt` |
| Princípio III — Testes Rigorosos | "o backend deve ser testado com `pytest`"; "frontend com Vitest e Playwright" | `pytest` → `cargo test`; Playwright → WebdriverIO + `tauri-driver` |
| Princípio V + Technical Stack | "Frontend em Astro com **Deploy Estático** […] compatível com GitHub Pages"; "**Backend:** Python" | O alvo passa a ser binário desktop. Astro permanece, mas o destino não é mais GitHub Pages |

O que **permanece válido e deve ser preservado**: Princípio I (TDD obrigatório), Princípio IV (CRUD completo, perfil Admin único, sem fluxo de aprovação) e a exigência de identidade visual conforme o Figma.

**Ação**: rodar `/speckit.constitution` **antes** da SEP-015, emendando para v2.0.0 (bump major — mudança incompatível de stack). O Sync Impact Report deve registrar a atualização dos gates em `plan-template.md`, cujo Constitution Check hoje verifica literalmente "Backend em Python, Frontend em Astro".

---

## 14. Backlog Proposto de SEPs

Numeração continuando de `014-fix-storage-type-dashboard`. Cada linha é um `/speckit.specify` independente, com o texto da coluna *Semente* servindo de `$ARGUMENTS`.

| # | Feature branch | Escopo | Dep. | Tam. |
|---|---|---|---|---|
| — | *(emenda)* | `/speckit.constitution` → v2.0.0: stack Rust/Tauri, `cargo test`, alvo desktop. Preserva TDD e Admin único | — | P |
| 015 | `015-tauri-shell-bootstrap` | Andaime `src-tauri/`, `tauri.conf.json`, capabilities, ícones, Astro estático embarcado. **Spike de roteamento (risco 10.6)** + **correção da sidebar (Q7)**. Zero lógica de negócio | — | M |
| 016 | `016-rust-persistence-core` | **SQLite em arquivo único** (`rusqlite` `bundled`, WAL): `Mutex<Connection>` + migração `001_init.sql` com as **16** tabelas (17 de hoje − `universities`), `EntityDef` registry, seed idempotente do admin, `app_data_dir()/hub.db` | 015 | G |
| 017 | `017-crud-commands-ipc-bridge` | 5 comandos CRUD com paginação/busca/ordenação (incl. `length()` e `NULLS LAST`), `AppError` serializável, shim `api.js`. **Meta: os 6 componentes React não mudam** | 016 | G |
| 018 | `018-native-zip-import` | `import_canonical_zip` via diálogo nativo + drag-and-drop, Polars → bulk insert transacional, eventos de progresso, cópia do `original.zip` | 016 | G |
| 019 | `019-canonical-export-fidelity` | `export_canonical_zip`: coerção de tipos pelo schema original, ordem de colunas, JSON com indent 4, preservação das 596 entradas não gerenciadas (incl. o ZIP aninhado de 22,6 MB, sem recompressão), diálogo de salvar. **Maior risco (10.1 e 10.2)** | 018 | GG |
| 020 | `020-merge-and-link-operations` | Merge transacional; link corrigido (bug 10.3.1) com teste que falha na implementação atual | 017 | M |
| 021 | `021-local-session-auth` | Sessão em `State`, bcrypt, gate real nos comandos, unificação do schema de `Admin` | 017 | M |
| 022 | `022-test-suite-migration` | Suíte `cargo test` completa, mock de `invoke` no Vitest, E2E WebdriverIO + `tauri-driver`, script de paridade contra o oráculo Python | 019, 020, 021 | G |
| 023 | `023-packaging-distribution` | Bundles para **Linux e Windows** (Q2), GitHub Actions com matriz de 2 SOs, ícones/branding, README com o aviso do SmartScreen (Q3) | 019 | M |
| 024 | `024-decommission-python-backend` | Remoção de `backend/`, reescrita de README/Makefile/AGENTS.md, guia de migração de dados para usuários existentes | 022, 023 | P |

**Caminho crítico**: 015 → 016 → 017 → 018 → 019 → 022 → 024.
**Paralelizável**: 020 e 021 rodam em paralelo a 018/019 assim que 017 fechar; 023 pode começar junto com 022.

### Sementes para `/speckit.specify`

- **015**: "Criar o shell da aplicação desktop Tauri 2.0 que carrega o build estático do frontend Astro existente, sem nenhuma lógica de negócio. Deve abrir uma janela, exibir a tela de login e permitir navegar por todas as 15 páginas do dashboard sem erro de rota, em Linux e Windows. Inclui o spike que decide a estratégia de roteamento do Astro sob o protocolo tauri://. Inclui também a correção da sidebar em Dashboard.astro, que hoje tem li fora de ul e classes de tema claro sobre painel escuro, deixando metade dos itens de menu praticamente invisíveis — o arquivo já será tocado pelo ajuste de rotas e o estado atual não cumpre a exigência de aderência ao Figma."
- **016**: "Implementar a camada de persistência em Rust usando um banco simples e embarcado: SQLite em arquivo único, via rusqlite com a feature bundled (sem libsqlite do sistema) e journal em modo WAL, exposto como Mutex<Connection> no estado do Tauri e consumido por comandos declarados #[tauri::command(async)] com corpo síncrono, armazenado em app_data_dir()/hub.db. Inclui a migração inicial versionada com 16 tabelas — as 17 do schema atual menos universities, que é órfã e foi descartada por decisão de arquitetura (sem página no dashboard, sem parquet no ZIP canônico, nunca exportada) — todas as colunas TEXT, exceto id INTEGER PRIMARY KEY, o registry estático de entidades mapeando rota do frontend para nome de tabela, colunas, coluna de busca e colunas JSON, e o seed idempotente do admin padrão. Migrações versionadas com rusqlite_migration ancoradas em PRAGMA user_version. Postgres, DuckDB, sqlx e qualquer motor ou driver que pressuponha servidor, pool ou async estão fora de escopo por decisão de arquitetura."
- **017**: "Expor os cinco comandos CRUD genéricos via IPC do Tauri, com paginação, busca e ordenação equivalentes ao endpoint /api/v1/{entity} atual, incluindo ordenação por length() em colunas JSON e NULLS LAST. Reescrever services/api.js como shim que traduz as chamadas apiFetch existentes em invoke, sem alterar nenhum componente React."
- **018**: "Substituir o upload multipart de ZIP por importação nativa: diálogo de arquivo e drag-and-drop, leitura dos parquets canônicos com Polars, inserção em lote transacional nas 15 tabelas mapeadas, eventos de progresso por tabela para o frontend, preservação de uma cópia intacta do ZIP original em app_data_dir(), e snapshot automático de hub.db (via rusqlite backup) antes de apagar a base, já que a importação destrói todos os dados curados."
- **019**: "Reimplementar a exportação canônica em Rust com fidelidade byte-equivalente em conteúdo ao export Python atual: restaurar os tipos originais (bool, inteiros, floats) lendo o schema do parquet original, preservar a ordem das colunas, gerar os JSONs correspondentes com indentação de 4 espaços e null no lugar de NaN, e copiar intactas as 596 entradas do ZIP original que não são gerenciadas pelo sistema, incluindo o data_snapshot.zip aninhado de 22,6 MB, que deve ser copiado sem recompressão."
- **020**: "Portar merge e link para Rust. Merge deve ser transacional (hoje não é). Link deve funcionar — a implementação Python atual chama um método inexistente em BaseRepository e sempre falha ao anexar um vínculo novo. Escrever o teste que falha contra o comportamento atual antes de implementar."
- **021**: "Implementar autenticação local real para a aplicação desktop: sessão em memória no processo Rust, verificação bcrypt contra a tabela admins, e gate efetivo em todos os comandos. Hoje o JWT é criado mas nunca validado por nenhum endpoint. Unificar o schema de Admin, hoje divergente entre orm.py e scripts/seed.py."
- **022**: "Migrar a suíte de testes: cargo test para unidade e integração no Rust, mock de invoke no Vitest, e substituir os testes Playwright por WebdriverIO com tauri-driver, cobrindo Linux e Windows — que são todos os alvos de release da v1. Incluir um script de paridade que roda import/export no backend Python e no Rust sobre o mesmo ZIP e compara schema e valores."
- **023**: "Configurar empacotamento e distribuição para Linux e Windows: bundles deb, rpm, AppImage e MSI/NSIS, com pipeline de GitHub Actions em matriz de dois sistemas operacionais. Inclui ícones e branding ResearchHub. Sem assinatura de código e sem auto-update: documentar no README o aviso do SmartScreen na primeira execução no Windows. macOS está fora do escopo desta feature e será tratado numa versão 2."
- **024**: "Remover o backend Python e toda a infraestrutura de dois processos. Atualizar README, Makefile e AGENTS.md para o fluxo Tauri. Documentar como usuários da versão web migram o test.db existente para o novo local em app_data_dir()."

---

## 15. Estimativa

| Frente | Dias | Confiança |
|---|---:|---|
| Setup, shell Tauri, spike de roteamento | 2–3 | Alta |
| Persistência + registry + migrações | 4–6 | Alta |
| Comandos CRUD + shim `api.js` | 4–5 | Alta |
| Import (Polars + bulk + progresso) | 3–4 | Média |
| **Export (fidelidade de tipos)** | **5–7** | **Baixa** |
| Merge + link | 2 | Alta |
| Auth de sessão | 1–2 | Alta |
| Diálogos nativos + drag-and-drop no front | 2 | Alta |
| Testes (Rust + E2E + paridade) | 4–5 | Média |
| Empacotamento + CI (Linux e Windows) | 3–4 | Média |
| **Total** | **30–40** | |

Descontando ~3 dias de sobreposição entre frentes paralelizáveis: **27–37 dias-desenvolvedor**.

**Premissas**: um desenvolvedor com Rust em nível intermediário; sem assinatura de código (Q3) nem auto-update (Q6) na v1; Linux e Windows como alvos, macOS adiado para a v2 (Q2); escopo de paridade acrescido das 8 correções da Seção 10.3 (Q4). Sem experiência prévia em Rust, some 30–50%.

**Principais fontes de estouro**, em ordem: (1) fidelidade do export, (2) o E2E com `tauri-driver`, que é notoriamente mais frágil que Playwright, (3) o CI de dois SOs, se o runner Windows der trabalho.

---

## 16. Matriz de Paridade Funcional

Checklist de aceite do sistema migrado. Nenhum item pode ser marcado por inspeção de código — todos exigem execução.

**Autenticação**
- [ ] Login com `admin@admin.com` / `admin123` abre o dashboard
- [ ] Credencial inválida exibe "Credenciais inválidas"
- [ ] Sair volta para a tela de login
- [ ] Comandos rejeitam chamada sem sessão *(melhoria sobre o atual)*

**CRUD** — para cada uma das 15 entidades:
- [ ] Listagem paginada, 50 por página, com "Mostrando X a Y de Z registros" correto
- [ ] Busca filtra pela coluna esperada (`name` / `title`) e reseta a página
- [ ] Ordenação por coluna, alternando asc/desc, com nulos por último
- [ ] Ordenação em coluna JSON ordena por quantidade de vínculos
- [ ] Criar, editar e excluir registro
- [ ] Deep link `?openId=N` abre o formulário do registro e limpa a URL

**Renderização de JSON** (comportamento da spec 013):
- [ ] Objetos **com** `id` viram deep link "ID: n - nome"
- [ ] Objetos **sem** `id` (ex.: `academic_education`) viram lista chave-valor legível
- [ ] Tabela mostra "N vínculos" nas colunas JSON

**Operações especiais**
- [ ] Fundir ≥2 registros: primário recebe os dados resolvidos, demais são removidos
- [ ] Merge é atômico *(melhoria)*
- [ ] Vincular entidade filha anexa ao array JSON da pai sem duplicar *(hoje quebrado)*

**Import**
- [ ] Aceita ZIP por diálogo nativo e por drag-and-drop
- [ ] Rejeita arquivo que não seja `.zip`
- [ ] Substitui a base inteira (drop + recreate)
- [ ] Contagem de linhas por tabela idêntica à do import Python
- [ ] Guarda o `original.zip` intacto
- [ ] Exibe progresso *(melhoria)*

**Export**
- [ ] Diálogo de salvar com nome padrão `portal_export_canonical.zip`
- [ ] Contém `parquet/{tabela}_canonical.parquet` para as 15 tabelas
- [ ] Contém `{tabela}_canonical.json` para as 15 tabelas
- [ ] `admins` **não** é exportada (`universities` não existe mais — Q5)
- [ ] Colunas booleanas e numéricas voltam ao dtype original
- [ ] Ordem das colunas idêntica à do parquet original
- [ ] JSON com indentação 4, UTF-8 sem escape, `null` no lugar de `NaN`
- [ ] **As 596 entradas não gerenciadas do ZIP original saem intactas**
- [ ] `data_snapshot.zip` (22,6 MB, ZIP aninhado) sai íntegro e sem recompressão
- [ ] Falha de tabela é reportada, não engolida *(melhoria)*

**Plataforma**
- [ ] Instala e roda em máquina limpa sem Python, Node ou navegador
- [ ] Dados persistem entre execuções em `app_data_dir()`
- [ ] Roda em Linux e Windows (alvos de v1 — macOS fica para a v2, Q2)

---

## 17. Anexo — Crates de Referência

| Crate | Papel | Substitui |
|---|---|---|
| `tauri` 2.x | Runtime, janela, IPC | FastAPI + Uvicorn + navegador |
| `tauri-plugin-dialog` | Diálogos de abrir/salvar, chamados do Rust (Q1) | `<input type=file>`, `<a download>` |
| `tauri-plugin-log` | Logging para arquivo e stdout | `print()` |
| `rusqlite` (`bundled`) | SQLite embarcado, síncrono; backup e funções customizadas | SQLAlchemy + driver SQLite |
| `rusqlite_migration` | Migrações versionadas por `PRAGMA user_version` | `Base.metadata.create_all` |
| `polars` | DataFrames, Parquet | pandas + PyArrow |
| `zip` | Leitura/escrita de ZIP | `zipfile` |
| `serde` / `serde_json` | Serialização, IPC | Pydantic |
| `bcrypt` | Hash de senha | `passlib[bcrypt]` |
| `thiserror` | Tipos de erro | Exceções + handlers |
| `jsonwebtoken` | *(opcional)* JWT | `python-jose` |

Ferramentas de desenvolvimento: `cargo-tauri` (CLI), `tauri-driver` + `webdriverio` (E2E), `sccache` (cache de build no CI).

---

## 18. Decisões Tomadas

As sete perguntas em aberto foram respondidas pelo dono do projeto e estão propagadas por todo o documento. Esta seção é o registro: o que foi decidido, por quê, e onde isso já aparece no texto.

**Contexto que enquadra várias delas**: o Research Hub é uma **ferramenta de uso pessoal, executada localmente**. Não há distribuição pública, não há base de usuários, não há requisito de custo. Isso elimina de saída as preocupações com assinatura, updater e canal de distribuição — e é por isso que a v1 pode ser generosa em alvos de plataforma e enxuta em infraestrutura.

| # | Decisão | Onde aparece |
|---|---|---|
| Q1 | O **Rust** abre os diálogos de arquivo | §8.1, §8.3, §17 |
| Q2 | **v1 = Linux + Windows**; macOS fica para a v2 | §11.2, §12, §15, §16, SEP-023 |
| Q3 | **Sem assinatura de código**; aviso do SmartScreen documentado | §12, README (SEP-023) |
| Q4 | **Corrigir os 8 pontos** da Seção 10.3 | §10.3, §11.3 |
| Q5 | **Descartar `universities`** — 16 tabelas, não 17 | §8.4, AD-02, §10.3, §16, SEP-016 |
| Q6 | **Sem auto-update** na v1 | §12 |
| Q7 | **Corrigir a sidebar** dentro da SEP-015 | §10.6, SEP-015 |

### Q1 — O Rust abre o diálogo

O comando `import_canonical_zip` chama `app.dialog().file()` internamente e lê o arquivo com `std::fs`; o front-end apenas faz `invoke`. O `tauri-plugin-fs` sai do `Cargo.toml` e as permissões caem para `core:default` — nenhuma entrada `dialog:*` ou `fs:*` é necessária, porque essas permissões existem para gatear o acesso do *JavaScript*, e o JavaScript deixa de tocar em arquivos. Coerente com o Princípio 2 (lógica no Rust).

### Q2 — v1 é Linux + Windows; macOS fica para a v2

A decisão passou por duas etapas. Primeiro macOS entrou na v1; depois saiu, e o motivo é bom: a única forma de incluí-lo **honestamente** na v1 seria aceitar meio-suporte — release em três plataformas, testes automatizados em duas, e uma terceira coberta por roteiro manual, porque o `tauri-driver` não suporta macOS e essa limitação não tem contorno. Meio-suporte é pior que suporte adiado: gera a impressão de plataforma testada sem a garantia correspondente.

**Com Linux e Windows apenas, o processo fica íntegro:** todo alvo de release é alvo de teste automatizado, sem verificação manual em lugar nenhum (Seção 11.2).

O custo de adiar é baixo, e é importante entender por quê: **o código Rust e o front-end são os mesmos nas três plataformas.** O que macOS acrescenta é infraestrutura — um runner `macos-latest` no CI, os alvos `aarch64`/`x86_64-apple-darwin`, Xcode Command Line Tools e uma estratégia própria de E2E. Nada disso toca a lógica da aplicação. Por isso a v2 de macOS é essencialmente uma SEP de empacotamento e testes, não uma reescrita.

**O que a v1 deve evitar** para não encarecer essa v2: nada específico. Não há decisão na v1 que precise ser tomada "pensando no macOS" — `app_data_dir()` já resolve os caminhos corretamente nos três SOs (AD-02), e o Tauri abstrai o webview. É por isso que adiar é seguro.

### Q3 — Sem assinatura de código

*Explicação, já que a pergunta original não estava clara:* assinar um binário é anexar a ele um certificado que prova **quem o publicou** e que ninguém o alterou desde então. Windows e macOS usam isso para decidir se avisam o usuário ao abrir o programa.

- **Sem assinatura**: o Windows mostra o SmartScreen na primeira execução — "O Windows protegeu o computador", com a saída em *Mais informações → Executar assim mesmo*. O Linux não tem barreira equivalente: `.deb`, `.rpm` e `.AppImage` simplesmente rodam.
- **Com assinatura**: o aviso some, ao custo de um certificado Authenticode, que custa dinheiro e leva de dias a semanas para ser emitido.

**Para uso pessoal e local, não é necessário.** O único atrito é o clique a mais na primeira execução no Windows, e isso vai documentado no README (SEP-023).

Quando o macOS entrar na v2, o tema volta com um detalhe a mais: lá o bloqueio depende da flag de quarentena, aplicada apenas a arquivos **baixados**. Um app compilado na própria máquina não a tem e abre normalmente — em Apple Silicon o linker já aplica assinatura *ad-hoc*, suficiente para execução local. Um `.dmg` vindo do CI precisaria de `xattr -dr com.apple.quarantine` uma única vez. Nada disso é problema da v1.

Se um dia o app for distribuído para terceiros, aí a assinatura volta à mesa de verdade — e o prazo de emissão do certificado vira item de caminho crítico.

### Q4 — Corrigir os 8 pontos da Seção 10.3

Nenhuma das correções altera o formato dos dados exportados, então o oráculo Python (Seção 11.3) segue válido para a comparação que importa — a do ZIP. Cada correção entra como **linha explícita de escopo** na SEP correspondente e como *diferença esperada* no script de paridade. O que fica proibido é corrigir em silêncio: a matriz de paridade (Seção 16) marca esses itens como *melhoria*, não como equivalência.

### Q5 — `universities` descartada

O schema Rust nasce com **16 tabelas**. A entidade não tem página, não tem parquet, nunca é exportada e não é referenciada em lugar nenhum do front-end — é esqueleto de algo que não chegou a existir. Se um dia houver plano de uso, ela volta como migração `002_*.sql`, quando esse plano existir.

Nota de leitura: onde o documento descreve o **sistema atual** (`orm.py`, inventário de código), a contagem continua sendo 17 — é fato sobre o que existe hoje. Onde descreve o **alvo** (AD-02, registry, SEP-016), passou a 16.

### Q6 — Sem auto-update na v1

O `tauri-plugin-updater` exige par de chaves de assinatura e um endpoint HTTP servindo o manifest de versões. Para um app pessoal, recompilar e reinstalar é mais simples que manter esse canal. Reavaliar só se houver base instalada real.

### Q7 — Sidebar corrigida na SEP-015

`Dashboard.astro` já será tocado pelo ajuste de rotas (risco 10.6), e o estado atual — `<li>` fora de `<ul>`, classes de tema claro sobre painel escuro — não cumpre a exigência de aderência ao Figma que a Constitution impõe. Corrigir junto, como item declarado de escopo.

---

### Escopo diferido para a v2

Registro do que ficou de fora e por quê, para que nenhuma dessas decisões precise ser redescoberta:

| Item | Motivo | Custo estimado quando entrar |
|---|---|---|
| Suporte a macOS | Evitar meio-suporte com verificação manual (Q2) | 1 SEP de empacotamento + E2E; sem mudança de código de aplicação |
| Assinatura de código | Uso pessoal dispensa (Q3) | Só se houver distribuição a terceiros |
| Auto-update | Sem base instalada (Q6) | Chaves de assinatura + endpoint de manifest |
| Busca sem sensibilidade a acento | Paridade primeiro (risco 10.5) | Baixo — `create_scalar_function` do `rusqlite` (AD-02) |
| Tabela `universities` | Órfã hoje (Q5) | Migração `002_*.sql`, se houver plano de uso |

---

**Situação**: com as sete respondidas, não restam bloqueios de decisão. O próximo passo é a emenda da Constitution para v2.0.0 (Seção 13) — que agora deve registrar Linux e Windows como plataformas-alvo —, e em seguida a SEP-015.
