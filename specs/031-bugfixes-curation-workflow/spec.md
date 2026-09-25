# Feature Specification: Correção de Bugs do Fluxo de Curadoria

**Feature Branch**: `031-bugfixes-curation-workflow`

**Created**: 2026-09-23

**Status**: Draft

**Input**: User description: "Corrigir bugs do fluxo de curadoria: dados obsoletos no formulário de edição, paginação fora do alcance após deleção, seleção obsoleta de ids deletados para fusão, merge ignorando entidades sem coluna name, corrida na busca, username de sessão não normalizado, verificação de sessão após diálogo de arquivo, e importação de zip sem tabelas canônicas apagando a base"

## Clarifications

### Session 2026-09-23

- Q: Ao trocar de página ou executar uma busca com registros marcados para fusão, o que deve
  acontecer com a seleção atual? → A: Limpar sempre: deleção, busca e troca de página zeram a
  seleção; a fusão é oferecida apenas dentro da página visível (Opção A).
- Q: Ao fundir dois registros, o que deve acontecer com os demais campos — os que o registro
  principal já tem e os que só existiam nos duplicados? → A: Unir vínculos: além do campo
  resolvido, as colunas de relacionamento dos duplicados são somadas às do principal, sem
  repetir vínculos já existentes; os demais campos dos duplicados continuam descartados
  (Opção B).
- Q: Um pacote de importação que contenha apenas algumas das tabelas canônicas gerenciadas deve
  ser aceito ou recusado? → A: Aceito com pelo menos uma tabela canônica gerenciada; a recusa se
  aplica somente quando nenhuma for encontrada (Opção A).

## User Scenarios & Testing *(mandatory)*

<!--
  IMPORTANT: User stories should be PRIORITIZED as user journeys ordered by importance.
  Each user story/journey must be INDEPENDENTLY TESTABLE - meaning if you implement just ONE of them,
  you should still have a viable MVP (Minimum Viable Product) that delivers value.

  Assign priorities (P1, P2, P3, etc.) to each story, where P1 is the most critical.
  Think of each story as a standalone slice of functionality that can be:
  - Developed independently
  - Tested independently
  - Deployed independently
  - Demonstrated to users independently
-->

### User Story 1 - Editar um registro grava no registro certo (Priority: P1)

O Admin abre o formulário de um registro (botão "Visualizar") para corrigi-lo. Enquanto o
formulário está aberto, ele clica em "Visualizar" em outro registro da listagem. O formulário
DEVE passar a exibir exatamente os valores do segundo registro, e ao salvar, os valores exibidos
são gravados somente nesse segundo registro. Hoje, o formulário continua mostrando os valores do
primeiro registro e, ao salvar, esses valores obsoletos são gravados no segundo registro — uma
corrupção de dados silenciosa que sobrescreve informações curadas à mão.

**Why this priority**: É o único dos defeitos que altera dados permanentemente sem que o Admin
perceba: horas de curadoria podem ser perdidas por uma sobrescrita silenciosa. Todos os demais
defeitos desta feature são de comportamento visível; este é de integridade de dados.

**Independent Test**: Pode ser testado abrindo o formulário do registro A, clicando em
"Visualizar" no registro B e verificando que os campos mostram B; salvar e verificar que B
recebeu os valores exibidos e A permaneceu intacto.

**Acceptance Scenarios**:

1. **Given** que o formulário do registro A está aberto, **When** o Admin clica em "Visualizar"
   no registro B, **Then** todos os campos do formulário passam a exibir os valores de B, sem
   nenhum valor residual de A.
2. **Given** que o formulário está exibindo o registro B (após ter exibido A), **When** o Admin
   salva, **Then** somente B é alterado, com os valores exibidos no formulário, e A permanece
   exatamente como estava.
3. **Given** que o formulário do registro A está aberto, **When** o Admin clica em "Novo
   Registro", **Then** o formulário abre vazio, com o identificador bloqueado para atribuição
   automática, e ao salvar cria um registro novo (não altera A).

---

### User Story 2 - Fusão funciona em todas as entidades (Priority: P1)

O Admin seleciona dois ou mais registros duplicados em qualquer entidade e os funde. O sistema
DEVE perguntar o valor final do campo de identificação próprio daquela entidade (nome, ou título
quando a entidade não tem nome), DEVE aplicar esse valor ao registro principal e DEVE transferir
para o principal os vínculos dos duplicados, somando-os aos que ele já possui — sem repetir
vínculos que o principal já tenha. Hoje, o diálogo de fusão pergunta apenas um "nome", que em
entidades cujo campo se chama título (ex.: artigos, premiações, produções científicas) é ignorado
silenciosamente — os duplicados são apagados sem que campo algum seja fundido, e o texto digitado
se perde.

**Why this priority**: A fusão é a operação de limpeza mais delicada da ferramenta: ela apaga
registros. Apagar duplicatas sem aplicar a resolução informada destrói o trabalho de
desduplicação do Admin, com aparência de sucesso.

**Independent Test**: Pode ser testado fundindo dois registros de uma entidade com título (ex.:
artigos) e verificando que o registro resultante carrega o título informado no diálogo; fundindo
registros com vínculos e verificando que os vínculos dos duplicados passam ao principal sem
repetição; e, nas entidades sem campo de texto, verificando que a operação não é oferecida (ou é
recusada com mensagem clara), sem apagar registros.

**Acceptance Scenarios**:

1. **Given** dois artigos duplicados selecionados, **When** o Admin informa o título final e
   confirma a fusão, **Then** o artigo principal passa a ter esse título e o duplicado é removido.
2. **Given** dois registros de uma entidade cujo campo de identificação é "nome" (ex.: grupos),
   **When** o Admin informa o nome final e confirma, **Then** o registro principal recebe o nome
   e o duplicado é removido.
3. **Given** dois registros de uma entidade com vínculos (ex.: pesquisadores), onde o principal
   tem 2 grupos de pesquisa e o duplicado tem 1 grupo distinto, **When** a fusão é confirmada,
   **Then** o principal passa a ter os 3 grupos, sem repetição, e nenhum vínculo do duplicado é
   perdido.
4. **Given** uma entidade que não possui campo de nome ou título (ex.: proficiências), **When**
   o Admin tenta fundir registros dessa entidade, **Then** a operação não é oferecida pela
   interface; se acionada por outro caminho, é recusada pelo núcleo com mensagem clara e
   nenhum registro é alterado.

---

### User Story 3 - A listagem nunca fica numa página que não existe (Priority: P2)

O Admin navega até a última página de uma listagem e deleta o(s) único(s) registro(s) dela. O
sistema DEVE reposicionar a visualização para a última página válida, exibindo os registros
restantes. Hoje, a tela fica presa numa página além do fim: aparece "Nenhum registro encontrado"
embora o total seja maior que zero, e o contador exibe um intervalo impossível (ex.: "Mostrando
51 a 50 de 50 registros"), parecendo que os dados sumiram.

**Why this priority**: É o defeito que mais aparenta perda de dados: o Admin deleta um registro
e a página seguinte parece vazia, sugerindo que a deleção apagou mais do que devia. Causa
sobressalto e retrabalho de verificação, mas os dados estão intactos e a navegação manual
recupera a tela.

**Independent Test**: Pode ser testado criando registros suficientes para mais de uma página,
indo até a última, deletando o único registro dela e verificando que a tela passa a exibir a
penúltima página com os registros restantes e o contador coerente.

**Acceptance Scenarios**:

1. **Given** que a última página contém um único registro, **When** o Admin o deleta, **Then** a
   listagem exibe a página anterior, com os registros restantes, sem recarregar a tela inteira.
2. **Given** qualquer deleção que reduza o total de registros, **When** a listagem é atualizada,
   **Then** o contador "Mostrando X a Y de Z" descreve uma janela real de registros exibidos
   (X ≤ Y ≤ Z).
3. **Given** que a listagem tem apenas uma página e o Admin deleta o único registro, **When** a
   listagem é atualizada, **Then** a tela exibe o estado vazio padrão ("Nenhum registro
   encontrado") com total 0.

---

### User Story 4 - A seleção para fusão reflete o que está na tela (Priority: P2)

O Admin marca registros para fusão. A seleção DEVE corresponder aos registros presentes na
listagem atual: ao deletar um registro selecionado, buscar um termo que o exclua da lista ou
navegar entre páginas, a seleção é zerada, e o botão "Fundir Selecionados" só fica habilitado
com dois ou mais registros visíveis selecionados. Hoje, um registro deletado continua
"selecionado" invisivelmente, permitindo acionar uma fusão com um identificador que não existe
mais (que falha com erro) ou fundir registros que não estão na tela.

**Why this priority**: Protege o Admin de acionar uma operação destrutiva com alvos que ele não
está vendo; porém, sem o defeito dos demais fluxos (histórias 1–3), a janela de erro é menor.

**Independent Test**: Pode ser testado selecionando dois registros, deletando um deles e
verificando que a seleção é zerada (nenhum registro permanece selecionado); e trocando de
página ou buscando e verificando que a seleção acompanha a listagem exibida (sempre vazia após
a mudança).

**Acceptance Scenarios**:

1. **Given** dois registros selecionados, **When** o Admin deleta um deles, **Then** a seleção
   é zerada e o botão de fusão fica desabilitado.
2. **Given** registros selecionados, **When** o Admin navega para outra página ou executa uma
   busca, **Then** a seleção é zerada — nenhum registro que não está na tela permanece
   selecionado.
3. **Given** qualquer estado da listagem, **When** o botão "Fundir Selecionados" está habilitado,
   **Then** todos os identificadores que serão fundidos estão visíveis e marcados na tela.

---

### User Story 5 - A busca exibe o resultado da última consulta (Priority: P2)

O Admin digita um termo de busca no campo da listagem. Enquanto digita rápido, cada tecla dispara
uma consulta; a tela DEVE exibir o resultado da última consulta submetida. Hoje, respostas que
chegam fora de ordem sobrescrevem o resultado novo com um antigo: o Admin vê uma lista que não
corresponde ao que está no campo de busca.

**Why this priority**: Degrada a confiança na busca (o Admin vê resultados errados e não sabe
por quê), mas o erro se corrige digitando de novo; não corrompe dados nem bloqueia o fluxo.

**Independent Test**: Pode ser testado simulando respostas de busca em ordem invertida
(resposta do primeiro termo chegando por último) e verificando que a tela exibe o resultado do
último termo digitado.

**Acceptance Scenarios**:

1. **Given** que o Admin digita "abc" e em seguida "abcd" em sequência rápida, **When** a
   resposta de "abc" chega depois da de "abcd", **Then** a listagem exibe o resultado de
   "abcd" (a última consulta), não o de "abc".
2. **Given** qualquer consulta, **When** apenas uma resposta chega, **Then** o comportamento é
   idêntico ao de hoje.

---

### User Story 6 - Sessão coerente e verificação antes de abrir arquivos (Priority: P3)

O Admin autentica-se com um e-mail contendo maiúsculas ou espaços. O nome de usuário da sessão
DEVE ser a forma canônica usada na autenticação (minúsculas, sem espaços nas pontas), igual em
todo o sistema. Além disso, nas operações de importação e exportação, a verificação de sessão
DEVE ocorrer antes de abrir qualquer diálogo de escolha/salvamento de arquivo: um Admin sem
sessão válida recebe o aviso de autenticação imediatamente, sem antes percorrer o diálogo e a
leitura do arquivo. Hoje o nome da sessão guarda a forma digitada (ex.: "Admin@Admin.COM") e a
verificação de sessão só acontece depois do diálogo e da leitura do arquivo.

**Why this priority**: São inconsistências de apresentação e de ordem de verificação, sem risco
aos dados; corrigi-las deixa o sistema coerente, mas qualquer uma pode ser entregue por último.

**Independent Test**: Pode ser testado autenticando com um e-mail em maiúsculas e verificando o
nome de sessão exibido; e, encerrada a sessão, acionando importação/exportação e verificando que
o aviso de autenticação aparece sem que nenhum diálogo de arquivo seja aberto.

**Acceptance Scenarios**:

1. **Given** que o Admin autentica-se com "Admin@Admin.COM", **When** o sistema consulta o nome
   do usuário da sessão, **Then** o valor retornado é "admin@admin.com" (forma canônica).
2. **Given** que não há sessão autenticada, **When** o Admin aciona importação ou exportação,
   **Then** recebe o aviso de sessão necessária imediatamente, sem que nenhum diálogo de arquivo
   seja aberto.
3. **Given** uma sessão autenticada válida, **When** o Admin usa importação e exportação,
   **Then** o fluxo é idêntico ao de hoje (diálogo, execução, relatório).

---

### User Story 7 - Importação recusa pacote sem tabelas canônicas (Priority: P3)

O Admin seleciona um arquivo .zip para importação que não contém nenhuma das tabelas canônicas
gerenciadas (ex.: um zip de fotos, ou um pacote com nomes de arquivo diferentes). O sistema DEVE
recusar a importação com uma mensagem clara ("o pacote não contém tabelas canônicas gerenciadas")
e DEVE deixar a base exatamente como está — sem apagar registros e sem gerar cópia de segurança,
pois nada foi destruído. Hoje, um pacote assim é aceito: todas as tabelas gerenciadas são
esvaziadas, o relatório informa sucesso com zero registros, e a base curada some (restando apenas
a cópia de segurança automática como socorro).

**Why this priority**: É o cenário de erro mais grave desta lista em potencial destruição, mas
exige que o Admin escolha um arquivo atipicamente errado; a cópia de segurança automática atual
mitiga o dano, e a correção é pequena e independente.

**Independent Test**: Pode ser testado importando um zip válido que não contenha nenhuma tabela
canônica gerenciada e verificando que o sistema recusa com erro claro e a contagem de registros
de todas as entidades permanece idêntica antes e depois.

**Acceptance Scenarios**:

1. **Given** um pacote .zip válido sem nenhum arquivo de tabela canônica gerenciada, **When** o
   Admin o importa, **Then** a operação falha com mensagem clara e a base permanece inalterada
   (mesmas contagens de registros em todas as entidades).
2. **Given** um pacote .zip válido com pelo menos uma tabela canônica gerenciada, **When** o
   Admin o importa, **Then** o comportamento é o de hoje (substituição da base, com cópia de
   segurança quando havia dados).
3. **Given** qualquer importação recusada por este motivo, **When** o Admin verifica as cópias de
   segurança, **Then** nenhuma cópia nova foi criada pela tentativa recusada.

---

### Edge Cases

- O que acontece quando o Admin abre "Novo Registro" logo após editar um registro existente? O
  formulário abre vazio, com identificador bloqueado — nenhum valor residual do registro anterior.
- O que acontece ao deletar o único registro da única página? A listagem exibe o estado vazio
  padrão com total 0, sem recuar para uma página negativa.
- O que acontece quando a busca que limpa a seleção é desfeita (o Admin apaga o termo)? A seleção
  não é restaurada automaticamente: o Admin re-seleciona — previsibilidade acima de conveniência.
- O que acontece quando o pacote contém apenas algumas das tabelas canônicas gerenciadas? É
  aceito e carrega o que existe (comportamento de hoje); a recusa se aplica somente ao pacote
  sem nenhuma tabela gerenciada.
- O que acontece quando o duplicado possui um vínculo que o principal já tem? O vínculo não é
  duplicado: permanece uma única ocorrência no registro principal.
- O que acontece com campos comuns (não-vínculo) preenchidos apenas nos duplicados? Continuam
  descartados — a fusão não promove valores de campos comuns dos duplicados.
- O que acontece quando a coluna de vínculo do principal guarda um vínculo único (não lista) e o
  duplicado tem outro? O valor do principal prevalece; a união aplica-se às colunas de lista.
- O que acontece ao tentar fundir em uma entidade sem campo de texto pelo caminho direto do
  núcleo? A operação é recusada com mensagem clara e nenhum registro é alterado.
- O que acontece quando duas respostas de busca chegam exatamente na mesma ordem? Comportamento
  idêntico ao de hoje — a proteção contra corrida não altera o resultado em ordem normal.
- O que acontece ao exportar sem sessão? O aviso de autenticação aparece antes de qualquer
  diálogo; nenhum arquivo é criado ou escolhido.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: O formulário de criação/edição MUST exibir, em todos os campos, exatamente os
  valores do registro que está sendo editado a cada mudança de alvo (abrir registro diferente ou
  alternar para "novo registro").
- **FR-002**: O envio do formulário MUST gravar os valores exibidos somente no registro que está
  sendo editado; nenhum valor de outro registro pode ser gravado por troca de alvo.
- **FR-003**: O diálogo de fusão MUST coletar o valor do campo de identificação próprio da
  entidade — nome quando a entidade o possui, título caso contrário.
- **FR-004**: A fusão MUST aplicar o valor informado ao campo resolvido do registro principal e
  MUST somar aos vínculos do principal os vínculos dos demais registros fundidos, sem repetir
  vínculos que o principal já possua; os demais campos dos duplicados são descartados. O núcleo
  MUST recusar uma fusão em que nenhum campo resolvido possa ser aplicado, com mensagem clara e
  sem alterar registros.
- **FR-005**: A fusão em entidade que não possui campo de nome ou título MUST não ser oferecida
  pela interface.
- **FR-006**: Após uma operação que reduza o total de registros (deleção ou fusão), a listagem
  MUST exibir uma página válida: quando a página atual deixar de existir, o sistema MUST
  reposicionar para a última página válida.
- **FR-007**: O contador de exibição "Mostrando X a Y de Z" MUST descrever uma janela real de
  registros (X ≤ Y ≤ Z) em todos os estados da listagem.
- **FR-008**: A seleção de registros para fusão MUST ser limpa integralmente sempre que a
  listagem muda: deleção, busca e troca de página zeram a seleção.
- **FR-009**: O botão de fusão MUST permanecer habilitado somente com dois ou mais registros
  presentes na listagem atual marcados.
- **FR-010**: Quando várias consultas de busca forem submetidas em sequência, a listagem MUST
  exibir o resultado correspondente à última consulta submetida.
- **FR-011**: O nome de usuário da sessão MUST ser a forma canônica usada na autenticação
  (minúsculas, sem espaços nas pontas), idêntico em toda consulta de status de sessão.
- **FR-012**: A verificação de sessão autenticada MUST preceder a abertura de qualquer diálogo de
  arquivo nas operações de importação e exportação.
- **FR-013**: A importação de um pacote que não contenha nenhuma tabela canônica gerenciada MUST
  ser recusada com mensagem clara, MUST deixar todas as entidades inalteradas e MUST não gerar
  cópia de segurança. Um pacote com pelo menos uma tabela canônica gerenciada é aceito, mesmo
  que parcial — a recusa aplica-se somente ao pacote sem nenhuma tabela gerenciada.
- **FR-014**: As regras desta spec aplicadas ao núcleo do sistema (fusão, importação, sessão)
  MUST residir no processo central; a interface apenas coleta entrada e exibe resultado.
- **FR-015**: Os fluxos existentes que hoje funcionam (fusão com nome, importação de pacote
  canônico completo, exportação, busca simples) MUST permanecer com o mesmo comportamento
  observável, exceto pelos defeitos declarados abaixo e pelas diferenças esperadas registradas
  nas seções Clarifications e Assumptions.

### Defeitos preexistentes declarados (escopo explícito de correção)

Conforme o Princípio II da Constituição (paridade antes de melhoria), as seguintes correções de
defeitos são itens explícitos de escopo e devem ser registradas como diferenças esperadas em
relação ao comportamento atual:

- **B-01** (US1): o formulário mantinha o estado do registro aberto anteriormente, gravando
  valores obsoletos no registro alvo de uma edição posterior.
- **B-02** (US3): a listagem permanecia numa página além do fim após a deleção do último registro
  da última página, com contador de janela impossível.
- **B-03** (US4): a seleção de fusão mantinha identificadores de registros deletados/ausentes da
  listagem, permitindo fusão com alvos invisíveis ou inexistentes.
- **B-04** (US2): a fusão ignorava o valor informado quando o campo resolvedor da entidade não se
  chamava "nome", apagando duplicatas sem aplicar resolução alguma.
- **B-05** (US5): respostas de busca fora de ordem sobrescreviam o resultado da última consulta.
- **B-06** (US6): o nome de usuário da sessão guardava a forma digitada, não a canônica.
- **B-07** (US6): a verificação de sessão ocorria depois do diálogo de arquivo e da leitura do
  pacote, em importação e exportação.
- **B-08** (US7): a importação de pacote sem tabelas canônicas esvaziava todas as entidades e
  reportava sucesso com zero registros.

### Key Entities *(include if feature involves data)*

- **Registro (linha de entidade)**: qualquer item gerenciado pelo sistema (pesquisadores, alunos,
  artigos, grupos de pesquisa, proficiências etc.), com identificador numérico único (ID) e um
  campo de identificação textual — nome ou título conforme a entidade.
- **Listagem de entidade**: página onde o Admin visualiza, pagina, ordena, busca, seleciona, funde
  e deleta registros; é a superfície onde B-02, B-03 e B-05 se manifestam.
- **Formulário de registro**: superfície de criação e edição de um registro; alvo do B-01.
- **Sessão do administrador**: estado autenticado do aplicativo, com nome de usuário canônico;
  alvo do B-06 e B-07.
- **Pacote canônico**: arquivo compactado contendo as tabelas canônicas gerenciadas; alvo do B-08.

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria.
  These must be technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: 100% das edições automatizadas gravam no registro aberto exatamente os valores
  exibidos no formulário; zero ocorrências de gravação cruzada entre registros.
- **SC-002**: Em 100% das deleções/fusões que reduzem o total, a listagem exibe registros reais
  quando o total é maior que zero — nenhuma página vazia inválida, nenhuma janela impossível no
  contador.
- **SC-003**: Em entidades com campo de nome ou título, toda fusão concluída resulta no registro
  principal com o valor resolvido informado e com todos os vínculos dos duplicados presentes
  nele, sem repetições; zero fusões que apenas removem duplicatas sem aplicar resolução, e zero
  vínculos perdidos na fusão.
- **SC-004**: Com consultas submetidas em sequência rápida, a listagem exibe o resultado da
  última consulta em 100% dos casos.
- **SC-005**: A importação de um pacote sem tabelas canônicas mantém a contagem de registros de
  todas as entidades idêntica antes e depois da tentativa, com erro claro exibido — em 100% dos
  testes automatizados.
- **SC-006**: O nome de usuário reportado pela sessão coincide com a forma canônica de
  autenticação em 100% das consultas, independentemente da forma digitada no login.
- **SC-007**: Nenhum dos fluxos hoje funcionais (paridade do ciclo importar → editar → exportar)
  registra regressão nos testes automatizados existentes.

## Assumptions

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right assumptions based on reasonable defaults
  chosen when the feature description did not specify certain details.
-->

- A seleção para fusão é válida apenas dentro da listagem visível: trocar de página, buscar ou
  deletar zera a seleção por completo (decisão confirmada na clarificação — Opção A). Hoje a
  seleção persiste entre páginas (permitindo fundir registros de páginas diferentes); esse caso,
  raro, deixa de ser oferecido — quem precisa fundir registros em páginas diferentes pode
  trazê-los para a mesma página pela busca. Mudança de comportamento declarada como diferença
  esperada.
- O campo resolvido do diálogo de fusão segue o cadastro da entidade: "nome" quando existe,
  "título" caso contrário. O rótulo exibido acompanha o campo real.
- Entidades sem campo de nome ou título (ex.: proficiências) deixam de oferecer fusão pela
  interface; o núcleo, por defesa, também recusa a operação sem campo resolvido.
- O comportamento de fusão do sistema de referência original era o de gravar apenas o campo
  resolvido e remover os demais. A correção B-04 corrige o campo ignorado; a união de vínculos
  dos duplicados ao registro principal (decisão confirmada na clarificação — Opção B) é uma
  ampliação deliberada em relação à referência, declarada aqui como diferença esperada.
- A união de vínculos na fusão aplica-se às colunas de relacionamento do cadastro (ex.: grupos
  de pesquisa, iniciativas, áreas de conhecimento, campus); campos comuns dos duplicados
  (descrição, ano, DOI etc.) não são promovidos ao principal. Em colunas de vínculo que guardam
  um vínculo único (não lista), o valor do principal prevalece — padrão informado, sem impacto
  material nos testes de aceitação.
- Os demais defeitos (B-01, B-02, B-03, B-05) são de comportamento introduzido na migração para a
  aplicação desktop e não existem no sistema de referência.
- Fora do escopo desta feature: exibição de "Erro ao ler vínculos" quando um campo de vínculo
  contém texto livre; conversão de valor numérico 0 em vazio nos campos do formulário; travamento
  do botão de envio durante submissão duplicada em login/cadastro; inclusão da rota de cadastro na
  guarda de rotas do binário. São deficiências menores conhecidas, registradas para features
  futuras.
- Nenhum contrato da superfície de comunicação entre interface e núcleo é alterado; as correções
  de interface não exigem mudanças de formato de dados nas operações existentes, exceto o payload
  de fusão (que passa a usar o campo resolvedor correto da entidade).
