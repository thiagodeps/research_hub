# Feature Specification: Área de Dados do Projeto SRC com Seleção de Projeto

**Feature Branch**: `033-src-data-tab`

**Created**: 2026-09-29

**Status**: Draft

**Input**: User description: "temos o projeto src em outra pasta, ele também usará o research hub para edição de dados, porém ele usa outro sistema, um json com tudo; preciso que faça uma nova aba para podermos mexer nos dados do src, separando por agora dos do horizon, podendo criar um menu para selecionar em qual quer mexer no dados (nova pagina principal), e fazendo adaptações para o que precisa no src"

## Contexto

Hoje o Research Hub cura exclusivamente os dados do projeto **Horizon**: as 15 tabelas canônicas importadas do pacote do DataLake. O projeto **SRC** (SRC_ETL — ferramenta de ETL do Sistema de Registro e Emissão de Certificados do Ifes) mantém seus dados em outro sistema: um único arquivo JSON consolidado ("o json com tudo"), em que cada ação de extensão/ensino carrega aninhadas suas **participações** — uma entrada por pessoa (público-alvo ou equipe de execução), com o contexto da atividade de origem — além do vínculo textual entre programas guarda-chuva e ações filhas. O consolidado contém dados pessoais dos participantes e, por definição do próprio projeto SRC, é mantido **local** (nada é publicado); o painel público do SRC continua recebendo apenas contagens, derivadas a montante — o Research Hub não publica nada. *(Correção feita durante o planejamento — ver research.md R2: o README do SRC descreve o diagrama do painel; o arquivo consolidado real achata participações por pessoa.)*

O curador precisa editar os dados do SRC pelo Research Hub **sem misturar** essas duas bases. Esta feature cria essa separação e a área de edição do SRC.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Escolher o projeto de trabalho na entrada do aplicativo (Priority: P1)

Ao abrir o Research Hub e autenticar-se, o curador não cai mais direto na área do Horizon: vê uma **nova página principal** com o menu de seleção de projeto (Horizon e SRC), cada um apresentado de forma distinguível. Ao escolher um projeto, o aplicativo abre a área de trabalho daquele projeto — com as abas de dados exclusivas dele. A escolha vale durante a sessão e o curador pode trocar de projeto a qualquer momento pela mesma seleção, sem reiniciar o aplicativo e sem que dados de um projeto apareçam na área do outro.

**Why this priority**: é o pré-requisito de segurança para abrir a base do SRC no mesmo aplicativo do Horizon. Sem a separação clara, qualquer edição subsequente fica ambígua ("de qual base é este registro?").

**Independent Test**: abrir o aplicativo, autenticar, selecionar cada projeto e confirmar que cada área de trabalho expõe apenas os dados e abas do projeto escolhido.

**Acceptance Scenarios**:

1. **Given** o curador autenticado, **When** a nova página principal carrega, **Then** o menu de seleção apresenta os dois projetos (Horizon e SRC) de forma claramente distinguível, sem nenhuma base de dados carregada ainda.
2. **Given** o menu de seleção, **When** o curador escolhe Horizon, **Then** a área de trabalho exibe exatamente as abas de dados do Horizon (as 15 tabelas atuais), sem qualquer aba ou registro do SRC.
3. **Given** o menu de seleção, **When** o curador escolhe SRC, **Then** a área de trabalho exibe apenas a(s) aba(s) de dados do SRC, sem qualquer aba ou registro do Horizon.
4. **Given** dentro de um projeto, **When** o curador troca para o outro pela seleção, **Then** a troca acontece sem reiniciar o aplicativo e a área exibida passa a conter somente os dados do novo projeto.
5. **Given** o curador com uma edição em andamento em um projeto, **When** ele troca para o outro, **Then** o aplicativo o avisa sobre a mudança não salva antes de prosseguir (ou a descarta de forma explícita), nunca perdendo o trabalho silenciosamente.

---

### User Story 2 - Carregar e devolver o JSON consolidado do SRC (Priority: P2)

Dentro da área SRC, o curador importa o arquivo JSON consolidado do SRC (o "json com tudo"). O aplicativo carrega todas as ações com suas participações aninhadas (público-alvo e equipe, com o contexto da atividade de origem) e o vínculo textual programa→filhas, informa quantos registros carregou e substitui a base SRC anterior (nunca mescla), criando snapshot de segurança — o mesmo ritual de confiança do fluxo Horizon. Ao concluir a curadoria, o curador exporta um arquivo JSON com **a mesma estrutura do arquivo recebido**, de modo que ele continue válido como entrada do pipeline do SRC.

**Why this priority**: sem carregar e devolver o arquivo no formato do SRC não há o que editar; é o fluxo mínimo viável da área SRC — mas depende da separação de projetos (US1).

**Independent Test**: importar um JSON consolidado de exemplo, conferir que a contagem de ações carregadas é idêntica à do arquivo, e exportar sem editar; comparar estrutura e conteúdo do exportado com o original.

**Acceptance Scenarios**:

1. **Given** um arquivo JSON consolidado válido do SRC, **When** o curador importa, **Then** todas as ações carregam com suas participações aninhadas (público-alvo e equipe, com o contexto da atividade de origem), e o total carregado é informado ao final.
2. **Given** uma base SRC já carregada com curadoria existente, **When** o curador importa um novo JSON, **Then** um snapshot da base anterior é criado e informado, e a base é substituída (não mesclada).
3. **Given** uma base SRC carregada, **When** o curador exporta, **Then** o arquivo JSON exportado reproduz a mesma estrutura do arquivo de origem: mesmos campos, mesmos aninhamentos, vínculos programa→filhas preservados e registros não editados intocados.
4. **Given** um arquivo que não é um JSON válido, ou que não possui a estrutura esperada do consolidado do SRC, **When** o curador tenta importar, **Then** a operação falha com mensagem clara e a base SRC permanece intacta.
5. **Given** um JSON consolidado contendo zero ações, **When** o curador importa, **Then** o aplicativo informa que a base ficou vazia (em vez de falhar silenciosamente ou parecer carregada).

---

### User Story 3 - Curar os dados do SRC (ações e informações aninhadas) (Priority: P3)

Dentro da área SRC, o curador enxerga as ações em tabela com busca e ordenação e colunas principais de identificação. Ele pode criar, visualizar, editar e excluir ações — o CRUD completo já exigido para toda entidade do sistema. Os dados aninhados de cada ação (atividades, com público-alvo e equipe) são editados junto à ação, e os vínculos entre programa e ações filhas permanecem consistentes.

**Why this priority**: é o valor central da integração, mas só tem utilidade depois que os projetos estão separados (US1) e a base do SRC carrega (US2).

**Independent Test**: com uma base SRC carregada, criar, editar e excluir uma ação e confirmar que cada alteração aparece na listagem e depois no arquivo exportado.

**Acceptance Scenarios**:

1. **Given** a área SRC aberta com base carregada, **When** o curador lista as ações, **Then** a tabela apresenta colunas de identificação (título, processo, natureza, tipo, coordenador) com busca e ordenação.
2. **Given** a listagem de ações, **When** o curador cria uma nova ação preenchendo os campos principais, **Then** ela aparece na listagem e persiste na base até a exportação.
3. **Given** uma ação existente, **When** o curador edita seus campos descritivos ou suas participações (público-alvo e equipe de execução, com o contexto da atividade de origem), **Then** as alterações são salvas, refletidas na listagem e incluídas na exportação.
4. **Given** a listagem de ações, **When** o curador exclui uma ação, **Then** o aplicativo pede confirmação e a ação some da listagem e da exportação.
5. **Given** uma ação filha vinculada a um programa guarda-chuva, **When** o curador a consulta, **Then** é possível identificar a qual programa ela está vinculada (e, do programa, listar suas filhas).
6. **Given** um programa pai com ações filhas, **When** o curador o exclui, **Then** o aplicativo avisa sobre as filhas vinculadas e trata os vínculos de forma explícita (impedindo ou desvinculando com aviso), nunca deixando referências órfãs silenciosas.

---

### Edge Cases

- O que acontece quando o JSON consolidado traz campos ausentes ou nulos em algumas ações? A ação carrega com os campos vazios (editáveis), sem falhar a importação inteira.
- O que acontece quando uma ação filha referencia um programa pai inexistente no arquivo? O registro carrega e o vínculo quebrado é sinalizado ao curador, sem abortar a carga.
- O que acontece ao trocar de projeto com uma edição/formulário aberto? O curador é avisado antes de qualquer perda.
- Como o sistema se comporta com um arquivo consolidado grande (milhares de ações)? A listagem e a busca continuam utilizáveis, com carregamento progressivo ou paginação quando necessário.
- Como o sistema trata caracteres especiais e acentuação nos textos das ações? Preservados integralmente na edição e na exportação.
- O que acontece se o curador tentar importar um JSON que é de outra origem (não é o consolidado do SRC)? A importação é recusada com mensagem clara.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: O sistema MUST apresentar, após o login, uma nova página principal com menu de seleção de projeto, oferecendo Horizon e SRC como opções distinguíveis.
- **FR-002**: O sistema MUST carregar a base de dados somente após o curador escolher um projeto na página principal.
- **FR-003**: O sistema MUST permitir trocar de projeto a qualquer momento durante a sessão, sem reiniciar o aplicativo.
- **FR-004**: O sistema MUST manter os dados dos dois projetos totalmente separados: nenhum registro, aba, busca ou estatística de um projeto pode aparecer na área de trabalho do outro.
- **FR-005**: O sistema MUST manter a área de trabalho do Horizon exatamente como é hoje (as 15 abas/tabelas atuais), sem alteração de comportamento pela presença do SRC.
- **FR-006**: O sistema MUST oferecer, na área SRC, a importação de um arquivo JSON consolidado do SRC, substituindo a base anterior (nunca mesclando) e criando snapshot de segurança informado ao curador.
- **FR-007**: O sistema MUST validar o arquivo antes de concluir a importação e, se inválido (não-JSON ou estrutura incompatível com o consolidado do SRC), abortar com mensagem clara deixando a base intacta.
- **FR-008**: O sistema MUST carregar as participações aninhadas de cada ação (público-alvo e equipe, com o contexto da atividade de origem), o vínculo textual programa→ações filhas e os metadados do arquivo, preservando os identificadores existentes.
- **FR-009**: O sistema MUST informar ao curador o resultado da importação (quantidade de ações carregadas, inclusive o caso de base vazia).
- **FR-010**: O sistema MUST oferecer, na área SRC, listagem de ações em tabela com busca e ordenação por colunas de identificação.
- **FR-011**: O sistema MUST oferecer CRUD completo (criar, visualizar, editar, excluir) de ações, com confirmação antes de excluir.
- **FR-012**: O sistema MUST permitir editar as participações de uma ação (público-alvo e equipe de execução, com o contexto da atividade de origem) a partir do editor da própria ação.
- **FR-013**: O sistema MUST tratar vínculos programa→filhas de forma explícita: ao excluir ou desvincular um programa com filhas, avisar o curador e não deixar referências órfãs silenciosas.
- **FR-014**: O sistema MUST exportar a base SRC como arquivo JSON com a mesma estrutura do arquivo de origem (mesmos campos, mesmos aninhamentos, registros não editados intocados), preservando acentuação e caracteres especiais.
- **FR-015**: O sistema MUST preservar snapshot/backup de cada projeto de forma independente (o snapshot de um projeto nunca interfere no outro).

### Key Entities *(include if feature involves data)*

- **Projeto (contexto de trabalho)**: o "espaço de dados" que o curador escolhe (Horizon ou SRC). Determina quais abas, base, snapshots e arquivos de importação/exportação ficam visíveis. É a fronteira de separação exigida nesta feature.
- **Ação (SRC)**: registro principal da base SRC — ação de extensão/ensino com processo, título, natureza (Extensão/Ensino), tipo (Curso/Evento/Projeto), coordenador, fomento, grande área, área temática, relatório aprovado, data de cadastro e vínculo textual opcional a um programa guarda-chuva (outra ação, pelo campo "Ação vinculante").
- **Participação (SRC)**: pessoa aninhada à ação — uma entrada por pessoa, marcada com `tipo` ("Público-alvo" ou "Equipe de execução") e carregando o contexto da atividade de origem (número, identificador e nome da atividade) e os campos da pessoa como aparecem no arquivo (ex.: nome, CPF, e-mail e situação no público-alvo; nome, função e vínculo na equipe). Contém dados pessoais e permanece estritamente local.
- **Metadados do consolidado (SRC)**: campus e contadores gerais do arquivo (totais de ações, ações com participações, atividades, público-alvo e equipe) — exibidos ao curador e recomputados na exportação quando derivados das linhas.
- **Tabelas canônicas do Horizon**: as 15 entidades já gerenciadas (pesquisadores, estudantes, artigos, grupos, etc.) — citadas como contexto; nada muda nelas nesta feature.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A partir do login, o curador alcança a área de trabalho do projeto desejado em no máximo 1 interação de seleção, e a troca entre projetos conclui em menos de 5 segundos sem reiniciar o aplicativo.
- **SC-002**: Ao importar um JSON consolidado de exemplo, 100% das ações do arquivo aparecem na listagem, com contagem informada idêntica à do arquivo.
- **SC-003**: Exportar uma base SRC importada **sem nenhuma edição** produz um arquivo equivalente ao original campo a campo (100% dos campos, aninhamentos e vínculos preservados).
- **SC-004**: O curador completa uma operação simples de curadoria no SRC (localizar uma ação, corrigir um campo, salvar) em menos de 2 minutos.
- **SC-005**: Em testes de separação, zero ocorrências de dados de um projeto serem exibidos, buscáveis ou contabilizados na área do outro.
- **SC-006**: A importação de um arquivo consolidado de grande volume (na ordem de milhares de ações) conclui em menos de 30 segundos, com progresso informado e sem travar a interface.

## Assumptions

- O arquivo JSON consolidado do SRC ("o json com tudo") é o único formato de troca da área SRC nesta feature: entrada e saída. O pipeline do SRC continua sendo a fonte inicial da verdade; o Research Hub não gera o consolidado, apenas o consome e o devolve editado. O formato é o produzido por `src-etl-consolidate` (documentado em research.md R2 e no contrato `contracts/src-consolidated-json.md`).
- O consolidado contém dados pessoais (CPF/e-mail dos participantes) e permanece local, como o próprio projeto SRC determina; o Research Hub é offline, de perfil único de admin e não publica nada.
- O CRUD completo (Princípio IV da Constituição) aplica-se à **ação** como entidade principal e às **participações** como coleção aninhada, editada dentro do editor da ação (não como abas separadas nesta fase).
- Identificadores internos (acao_id; atividade_id quando presente nas participações) são preservados pela edição e pela exportação (mesma política já adotada no Horizon), pois o pipeline do SRC referencia esses identificadores.
- A área SRC reutiliza os padrões de experiência já estabelecidos no aplicativo: importação com progresso e snapshot, tabelas com busca/ordenação, formulários de edição e confirmação de exclusão.
- "Separando por agora" significa bases totalmente independentes nesta fase; qualquer unificação ou cruzamento entre bases Horizon e SRC está fora do escopo.
- A visão agregada de "extensionistas" (pessoas e funções ao longo dos anos) existente no painel do SRC é derivada da equipe e **não** será uma aba editável nesta feature.
- Ações filhas continuam apontando para o programa guarda-chuva pelo campo textual "Ação vinculante"; a edição não altera a semântica desse vínculo.
