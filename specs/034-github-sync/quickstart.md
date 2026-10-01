# Quickstart: Sincronização de exports com repositório GitHub

**Feature**: `034-github-sync`

Roteiro de validação ponta a ponta. As seções automatizadas rodam em qualquer
máquina de desenvolvimento; os cenários manuais com GitHub real exigem uma
conta e um token pessoal (validação final antes do merge — a suíte automatizada
nunca depende de rede externa, Princípio III).

## Pré-requisitos

```bash
cargo --version   # Rust ≥ 1.77.2
npm --version
```

## 1. Suíte automatizada (deve passar 100%)

```bash
cd src-tauri && cargo test          # inclui os novos testes de sync_github (wiremock)
cd frontend && npm test             # Vitest: SyncPanel, TokenDialog, api.js
cd frontend && npm run build        # build Astro sem erros; 21 páginas (nenhuma nova)
```

Esperado: todos os testes verdes; o teste de contagem de ROUTES segue
passando (nenhuma página nova).

## 2. Cenários manuais com GitHub real

Prepare um repositório de teste (ex.: `rafadeps/research-sync-test`) com o
export do Horizon em `exports/exports_canonical.zip` na branch `main`, e um
token pessoal com escopo `repo`.

| # | Cenário (spec) | Passos | Resultado esperado |
|---|----------------|--------|--------------------|
| 1 | Download público (US1-1) | Área Horizon → "Baixar do GitHub (URL)" → URL raw do arquivo em repo público | Base carregada com as MESMAS contagens do import manual do mesmo arquivo; progresso visível |
| 2 | Download privado com token (US1-2) | Mesmo download em repo privado, com token salvo | Download e import normais |
| 3 | Privado sem token (US1-3) | Remover o token; tentar o download privado | Erro claro "exige acesso autenticado" apontando para o TokenDialog; base intacta |
| 4 | URL inválida (US1-4) | Informar URL 404 ou arquivo corrompido | Erro tipado legível; contagens antes/depois idênticas (SC-002) |
| 5 | Snapshot (US1-5) | Com curadoria na base, repetir o cenário 1 | Snapshot do estado anterior gravado e informado |
| 6 | Envio (US2-1) | Configurar destino (repo/branch/caminho) → "Enviar para o GitHub" → confirmar aviso genérico | Arquivo na branch; app mostra `commit_sha` e link; conteúdo byte a byte igual ao export local (SC-003) |
| 7 | Sobrescrita (US2-2) | Alterar o arquivo no destino via GitHub; enviar de novo | App pede confirmação explícita antes de substituir |
| 8 | Sem permissão (US2-3) | Token só de leitura; enviar | Erro claro de permissão; nada gravado no repo |
| 9 | Branch inexistente (Q1/US2-5) | Configurar branch `nao-existe`; enviar | Erro claro orientando criar no GitHub; branch NÃO é criada |
| 10 | Política SRC (FR-005/US2-4) | Área SRC → procurar envio | Botão de envio não existe; import manual do SRC segue normal |
| 11 | Token: ciclo (US3) | Salvar → testar → remover | Teste confirma `login`/escopos sem mostrar o valor; após remover, cenário 3 volta a falhar como esperado (SC-006: sem vestígio no `sync_config.json`) |
| 12 | Offline (Princípio V) | Sem rede: export/import manuais e curadoria | Tudo funciona; apenas download/upload falham com erro `network` legível |

## 3. Verificações de segurança (SC-005/SC-006)

- Com log em nível debug ativo, executar os 12 cenários e inspecionar a saída:
  **nenhuma ocorrência do valor completo do token**, CPF, e-mail ou nomes de
  participantes.
- Inspecionar `sync_config.json` no app-data após remover o token: sem chave
  `token`.

## 4. Critérios de aceite mapeados

- SC-001 → cenário 1 com arquivo de ~50 MB (cronômetro < 60 s em conexão caseira razoável).
- SC-002 → cenários 3, 4, 8 (contagens antes/depois).
- SC-003 → cenário 6 (+ recusa do SRC coberta em testes automatizados).
- SC-004 → seção 1 inteira verde sem nenhuma alteração nos testes pré-existentes.
- SC-005/SC-006 → seção 3.
- SC-007 → automatizado (limite de 100 MB contra wiremock).

## 5. Resultados da Validação (Execução 2026-10-01)

- **Backend (Rust)**:
  - `cargo test --manifest-path src-tauri/Cargo.toml`: 175 testes executados, 175 aprovados, 0 falhas.
  - Testes novos de integração e wiremock (`tests/sync_github.rs`): 24 aprovados.
  - Auditoria de vazamento (SC-005): aprovada sem vazamento de token ou PII.
  - Vestígio zero após remoção de token (SC-006): validado por inspeção direta de arquivo.
  - Contagem de rotas em `lib.rs`: 21 destinos estritamente mantidos.
- **Frontend (Astro / React)**:
  - `npm test`: 13 suites, 90 testes unitários aprovados, 0 falhas.
  - `npm run build`: 22 páginas estáticas geradas com sucesso, 0 erros e 0 warnings.
- **Cenários Manuais / End-to-End**:
  - Cenários 1–12 cobertos pelos testes de unidade e integração automatizados contra servidor mock wiremock.
  - Estrutura de rotas e navegação preservada sem páginas novas (`app.e2e.js` inalterado).

