# Feature Specification: Adaptação ao Novo Export Canonical (Somente JSON + Aba Campus)

**Feature Branch**: `032-canonical-export-json`

**Created**: 2026-09-25

**Status**: Draft

**Input**: User description: "o arquivo que usa para coleta de dados export canonical sofreu mudanças, agora além de usar somente json ao invés dos parquets, a aba campus sofreu mudanças para corrigir um bug em que tinha uma lista com outra aba campus dentro dele, analise o novo zip export canonical para corrigir e adaptar o research hub"

## Contexto do Pacote Analisado

Análise do novo `exports_canonical.zip` (395 entradas, sem nenhum `.parquet`) contra o pacote anterior (626 entradas, 15 `parquet/{tabela}_canonical.parquet` + 15 `{tabela}_canonical.json`):

1. **Formato**: o pacote novo entrega somente arquivos `{tabela}_canonical.json` na raiz do ZIP — o diretório `parquet/` não existe mais. Há 26 JSONs canônicos; os 15 gerenciados pelo Research Hub continuam presentes (`researchers`, `students`, `articles`, `research_groups`, `initiatives`, `advisorships`, `awards`, `campuses`, `organizations`, `fellowships`, `proficiencies`, `professional_activities`, `knowledge_areas`, `languages`, `research_productions`). As demais entradas (grafos de colaboração, trackings, marts, `source_records`, `entity_matches`, `entity_change_logs` etc.) não são gerenciadas.
2. **Bug da aba campus corrigido upstream**: no pacote antigo, cada linha de `campuses_canonical.json` carregava um objeto `campus` aninhado duplicando a própria aba (ex.: `{"id": 6, "name": "Serra", ..., "campus": {"id": 6, "name": "Serra"}}`). No pacote novo, cada linha de campus contém apenas `id`, `name`, `description`, `short_name`, `organization_id`, `parent_id` — sem aninhamento.
3. **Consequência direta para o app**: com o import atual, que só enxerga entradas `.parquet`, o pacote novo carregaria **zero tabelas**. E o app ainda exibe o campo fantasma "Campus (Vínculos)" herdado do bug upstream.

## Clarifications

### Session 2026-09-25

- Q: Quando o pacote importado não traz o arquivo JSON de alguma das 15 tabelas gerenciadas, o que o aplicativo deve fazer? → A: Abortar todo o import com mensagem clara; a base permanece intacta.
- Q: Na listagem da aba Campuses, após remover a coluna fantasma "Campus (Vínculos)", quais colunas devem ser exibidas para o curador? → A: Apenas `id` e `name`, como hoje, sem o fantasma.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Importar o pacote canônico em JSON (Priority: P1)

O curador recebe do DataLake o novo `exports_canonical.zip` (somente JSON) e importa no Research Hub. Todas as 15 tabelas canônicas carregam com o conteúdo íntegro dos JSONs, com backup automático da base anterior e progresso por tabela — o mesmo fluxo de confiança de hoje, agora sobre o novo formato.

**Why this priority**: sem o import do novo formato o aplicativo fica inutilizável — o pacote novo carregaria zero tabelas. É o bloqueador de tudo.

**Independent Test**: importar um pacote contendo apenas `{tabela}_canonical.json` na raiz e verificar que as 15 tabelas carregam com as linhas e tipos dos JSONs.

**Acceptance Scenarios**:

1. **Given** um pacote com `{tabela}_canonical.json` na raiz para as 15 tabelas gerenciadas (e nenhuma entrada `.parquet`), **When** o curador importa, **Then** as 15 tabelas são carregadas, cada uma com a mesma quantidade de linhas do respectivo JSON, e o progresso é reportado por tabela.
2. **Given** um pacote importado, **When** a carga termina, **Then** os ids explícitos das linhas foram preservados (os relacionamentos referenciam esses ids).
3. **Given** uma base com curadoria existente, **When** o curador importa um novo pacote, **Then** um snapshot da base anterior é criado e informado, e a base é substituída (não mesclada).
4. **Given** um pacote cujos registros carregam listas/objetos aninhados (ex.: vínculos de iniciativas, grupos e áreas), **When** o curador importa e depois exporta, **Then** essas estruturas voltam como estrutura real no JSON, e não como texto escapado.
5. **Given** um arquivo que não é um ZIP válido, não contém nenhuma tabela canônica, ou não contém o arquivo canônico de alguma tabela gerenciada, **When** o curador tenta importar, **Then** a operação falha com mensagem clara e a base permanece intacta.

---

### User Story 2 - Aba Campus sem o campus aninhado (Priority: P2)

Com o bug upstream corrigido, a aba Campuses deixa de exibir o campo fantasma "Campus (Vínculos)" — um objeto duplicado da própria linha que vinha do aninhamento defeituoso. A aba passa a tratar campus pelo que ele é: id, nome, descrição, sigla, organização e campus-pai.

**Why this priority**: corrige a exibição de dados inconsistentes na tela, mas não bloqueia o uso do aplicativo com o novo pacote.

**Independent Test**: importar o pacote novo e abrir a aba Campuses: nenhuma coluna "Campus (Vínculos)" aparece, e os demais campos de campus continuam visíveis e editáveis.

**Acceptance Scenarios**:

1. **Given** o pacote novo importado, **When** o curador abre a aba Campuses, **Then** a listagem exibe exatamente as colunas `id` e `name` e nenhuma coluna "Campus (Vínculos)" (objeto campus aninhado) aparece.
2. **Given** a aba Campuses, **When** o curador cria ou edita um campus, **Then** o formulário não oferece campo para o objeto campus aninhado, e os campos reais (nome, descrição, sigla, organização, campus-pai) funcionam normalmente.
3. **Given** um pacote antigo importado em uma base cujos registros ainda contenham o campo campus aninhado, **When** o curador visualiza a aba Campuses, **Then** o dado aninhado é tratado como conteúdo ignorado/descartado, sem quebrar a tela nem a importação.

---

### User Story 3 - Exportar no novo formato JSON (Priority: P3)

O curador exporta a base corrigida e o pacote gerado segue o novo formato combinado com o DataLake: `{tabela}_canonical.json` para as 15 tabelas, sem parquet, com as entradas não gerenciadas preservadas byte a byte e os tipos de dados do pacote original restaurados.

**Why this priority**: o export só faz sentido depois que o import do novo formato funciona; é a devolução do pacote ao pipeline.

**Independent Test**: importar o pacote novo, exportar sem editar nada e comparar: as 15 tabelas saem como `{tabela}_canonical.json`, as demais entradas do ZIP original estão intactas e os tipos (booleanos, números, null, listas) coincidem com o pacote importado.

**Acceptance Scenarios**:

1. **Given** uma base importada do pacote novo, **When** o curador exporta, **Then** o pacote contém `{tabela}_canonical.json` para as 15 tabelas e **não** contém `parquet/{tabela}_canonical.parquet`.
2. **Given** um pacote original com entradas não gerenciadas (grafos, trackings, marts, relatórios), **When** o curador exporta, **Then** cada entrada não gerenciada sai byte a byte idêntica, sem recompressão.
3. **Given** colunas com tipos conhecidos do pacote original (booleanos, números, texto, null, listas/objetos), **When** o curador exporta sem ter editado esses valores, **Then** os valores voltam com o mesmo tipo de dados do pacote importado (ex.: `true` continua booleano, `23` continua número, null continua null).
4. **Given** valores curados manualmente no aplicativo, **When** o curador exporta, **Then** esses valores são emitidos no tipo coerente com a coluna (texto para colunas textuais; estrutura para vínculos).
5. **Given** o diálogo de exportação, **When** o curador confirma ou cancela, **Then** o comportamento é o de hoje: nome padrão sugerido e cancelamento não altera nada.

---

### Edge Cases

- O que acontece quando o pacote traz um JSON canônico cujos registros têm campos que o aplicativo não gerencia (evolução futura do formato)? Os campos desconhecidos devem ser ignorados sem erro — e nunca podem reaparecer no export.
- O que acontece ao importar um pacote no formato antigo (com `parquet/` + JSON duplicados)? A importação deve continuar funcionando, dando precedência ao JSON quando ambos existirem.
- O que acontece quando um `{tabela}_canonical.json` está vazio (`[]`) ou ausente? Tabela vazia é um estado válido e deve ser carregada como vazia; ausência do arquivo de uma tabela gerenciada aborta o import inteiro com mensagem clara, deixando a base intacta.
- O que acontece quando um JSON canônico vem malformado (JSON inválido)? A importação inteira deve falhar com mensagem clara, sem deixar a base meio carregada.
- O que acontece com valores null em qualquer coluna? Devem ser preservados como null no import e no export, nunca virados texto vazio ou `"NaN"`.

## Requirements *(mandatory)*

### Functional Requirements

**Importação (formato JSON)**

- **FR-001**: O sistema DEVE importar pacotes cujas tabelas canônicas estejam representadas por arquivos `{tabela}_canonical.json` na raiz do ZIP, cobrindo as 15 tabelas gerenciadas e nunca a tabela de administradores.
- **FR-002**: O sistema DEVE carregar de cada registro JSON todos os campos gerenciados da tabela correspondente e DEVE ignorar campos não gerenciados sem interromper a importação.
- **FR-003**: O sistema DEVE converter os valores JSON para o armazenamento interno preservando seu significado: null permanece null, números permanecem distinguíveis de texto (ordenam e filtram como números), booleanos permanecem reconhecíveis e listas/objetos permanecem estrutura.
- **FR-004**: A importação DEVE substituir a base anterior, preservar ids explícitos, ignorar a tabela de administradores, reportar progresso por tabela e criar snapshot prévio quando houver dados curados (comportamentos já vigentes, reafirmados para o novo formato).
- **FR-005**: A importação DEVE ser atômica: falha de qualquer tabela (JSON malformado, entrada ilegível) aborta tudo e deixa a base no estado anterior.
- **FR-006**: O sistema DEVE aceitar pacotes no formato legado (com `parquet/{tabela}_canonical.parquet`), usando o parquet como fonte alternativa quando o JSON correspondente não existir; quando ambos existirem, o JSON tem precedência.
- **FR-014**: A importação DEVE verificar, antes de apagar qualquer dado, que o pacote contém o arquivo canônico de cada uma das 15 tabelas gerenciadas (JSON ou, no formato legado, parquet); a ausência de qualquer uma DELAS aborta o import com mensagem clara e a base permanece intacta.

**Aba Campus (correção do aninhamento)**

- **FR-007**: O sistema DEVE tratar o campus exclusivamente pelos seus campos reais (id, nome, descrição, sigla, organização, campus-pai) e DEVE remover o campo fantasma de vínculo "Campus (Vínculos)" da aba Campuses — listagem e formulário. A listagem DEVE exibir exatamente `id` e `name`, como hoje, sem o campo fantasma.
- **FR-008**: O sistema DEVE descartar, sem erro, o campo campus aninhado caso apareça em pacotes antigos, e esse campo NÃO DEVE reaparecer no export.

**Exportação (formato JSON)**

- **FR-009**: O sistema DEVE gerar `{tabela}_canonical.json` para as 15 tabelas gerenciadas na raiz do pacote exportado e NÃO DEVE gerar `parquet/{tabela}_canonical.parquet`.
- **FR-010**: O sistema DEVE preservar byte a byte, sem recompressão, toda entrada do pacote original que não seja substituída pelo export.
- **FR-011**: O sistema DEVE restaurar no export os tipos de dados registrados no import (booleanos, números, texto, null, listas/objetos) para valores não editados, usando o tipo declarado pela coluna no pacote original; valores sem tipo conhecido saem como texto.
- **FR-012**: O sistema DEVE emitir listas e objetos de vínculo como estrutura JSON real (não texto escapado), null como null (nunca "NaN") e manter a serialização legível (recuos de 4 espaços, sem escape de caracteres não-ASCII).
- **FR-013**: Uma falha de exportação em qualquer tabela DEVE abortar o export com mensagem, em vez de omitir a tabela silenciosamente do pacote entregue.

### Key Entities *(include if feature involves data)*

- **Pacote Canônico (exports_canonical.zip)**: artefato do DataLake. Novo formato: somente `{tabela}_canonical.json` na raiz (15 gerenciadas + arquivos não gerenciados). Legado: pares `parquet/{tabela}_canonical.parquet` + `{tabela}_canonical.json`. Entradas não gerenciadas (grafos, trackings, marts, `data_snapshot.zip`, relatórios) são preservadas intactas no round-trip.
- **Campus**: registro com id, nome, descrição, sigla, organização de origem e campus-pai (autorreferência). Sem campo aninhado de campus. No pacote atual: 23 registros.
- **Tabelas canônicas gerenciadas**: researchers, students, articles, research_groups, initiatives, advisorships, awards, campuses, organizations, fellowships, proficiencies, professional_activities, knowledge_areas, languages, research_productions — cada uma com colunas declaradas no registro central de entidades.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Importar o pacote novo carrega exatamente 15 tabelas, com contagem de linhas por tabela igual à do respectivo JSON (0 divergências), em até 2 minutos no hardware de referência.
- **SC-002**: Um round-trip completo (importar → exportar sem editar) sobre o pacote novo produz um pacote com: as 15 tabelas em JSON, zero entradas parquet geradas, 100% das entradas não gerenciadas byte a byte idênticas e tipos de dados preservados em todas as colunas gerenciadas.
- **SC-003**: A aba Campuses exibe somente colunas correspondentes a campos reais de campus; nenhuma menção a "Campus (Vínculos)" permanece na listagem ou no formulário.
- **SC-004**: Um pacote legado (parquet + JSON) importado na versão adaptada carrega as mesmas 15 tabelas com as mesmas contagens da versão atual — nenhuma regressão de compatibilidade.
- **SC-005**: Todo cenário de falha (ZIP inválido, JSON malformado, pacote sem tabelas canônicas, ausência do arquivo canônico de uma tabela gerenciada) deixa a base em estado idêntico ao anterior, verificado por teste automatizado.
- **SC-006**: 100% das novas regras cobertas por testes automatizados primeiro (TDD), conforme a constituição do projeto.

## Assumptions

- O DataLake distribuiu o formato novo como padrão; pacotes futuros não terão mais o diretório `parquet/`. O formato legado continua sendo aceito por compatibilidade com pacotes antigos em poder dos usuários (FR-006).
- O export passa a reproduzir o formato novo (somente JSON), pois o pipeline consumidor é o mesmo que produziu o pacote novo; gerar parquet que ninguém espera seria lixo no pacote devolvido.
- A fidelidade de tipos passa a se apoiar nos tipos dos próprios JSONs importados (registrados na importação), substituindo o mecanismo atual de ler o schema do parquet original — que deixa de existir.
- Campos novos nos JSONs (evolução do DataLake) não exigem novas versões do aplicativo: são ignorados no import e não reaparecem no export.
- O fluxo de UI (diálogo de arquivo, snapshot informado, progresso por tabela, diálogo de salvar com nome padrão) permanece inalterado; o que muda é o formato de dados processado.
