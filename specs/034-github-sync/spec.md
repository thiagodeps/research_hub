# Feature Specification: Sincronização de exports com repositório GitHub

**Feature Branch**: `034-github-sync`

**Created**: 2026-09-29

**Status**: Draft

**Input**: User description: "Seria possível baixar um export canônico de um repositório do GitHub, e também fazer upload do export para um repositório GitHub — mantendo o import manual de arquivo local."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Baixar export do GitHub e importar (Priority: P1)

O curador informa a URL de um arquivo de export hospedado em um repositório do
GitHub (o zip canônico do Horizon ou o JSON consolidado do SRC, em repositório
público ou privado). O aplicativo baixa o arquivo, executa as MESMAS validações
do import manual e carrega a base do projeto correspondente — com snapshot de
segurança quando havia curadoria a proteger. O fluxo e o resultado são
idênticos aos do import por arquivo: só muda a origem dos bytes.

**Why this priority**: É o valor central da feature — permitir restaurar/replicar
uma base curada sem depender de transferência manual de arquivos entre máquinas.
Funciona sozinha e não depende do upload.

**Independent Test**: Em um repositório com um export válido, informar a URL no
app e verificar que a base fica carregada com as mesmas contagens do arquivo —
sem usar o diálogo de arquivo em nenhum momento.

**Acceptance Scenarios**:

1. **Given** um repositório público com um export canônico válido, **When** o
   curador informa a URL e confirma a importação, **Then** a base do projeto é
   carregada e as contagens finais são exibidas (iguais às do import manual).
2. **Given** um repositório PRIVADO com um export válido e um token configurado,
   **When** o curador informa a URL, **Then** o download acontece com o token e
   a importação segue normalmente.
3. **Given** um repositório privado SEM token configurado, **When** o curador
   informa a URL, **Then** um erro claro explica que aquela URL exige acesso
   autenticado e aponta para a configuração do token — a base atual não é
   alterada.
4. **Given** uma URL que não aponta para um export válido (404, arquivo
   corrompido, conteúdo inesperado), **When** o curador confirma, **Then** um
   erro tipado e legível é exibido e a base atual permanece intacta (as MESMAS
   garantias de validação do import manual).
5. **Given** uma base com curadoria existente, **When** um download/import é
   concluído, **Then** um snapshot do estado anterior é gravado e informado —
   exatamente como no import manual.

---

### User Story 2 - Publicar o export do Horizon no GitHub (Priority: P2)

O curador envia o zip canônico do Horizon diretamente para um
repositório do GitHub, informando repositório, branch e caminho do arquivo. O
envio é uma **ação única**: no momento da confirmação o aplicativo gera um
export novo a partir do estado atual da base e o grava como um commit na
branch informada, exibindo a confirmação (commit/URL do arquivo) — nunca
reenvia um arquivo de export salvo anteriormente. O modo de geração é o mesmo
do export manual — só muda o destino.

A função de envio é **definida por projeto**: Horizon e SRC são projetos
separados, com repositórios separados, e cada um tem sua própria configuração
de destino e seu próprio fluxo de envio. Hoje apenas o do Horizon está habilitado;
o JSON consolidado do SRC **não** participa desta história por política de
privacidade (FR-005) — a função de envio do SRC existe no desenho, porém
desabilitada, de modo que uma futura mudança de política a habilite sem
reestruturar nada.

**Why this priority**: Complementa a US1 (backup/compartilhamento da base
curada do Horizon), mas depende do fluxo de export já existente; entrega valor
após o download estar disponível.

**Independent Test**: Configurar destino (repo/branch/caminho) e enviar o zip
canônico do Horizon para o GitHub; verificar no repositório que o arquivo
apareceu na branch com o mesmo conteúdo do export local. Tentar enviar o
consolidado do SRC e verificar a recusa explícita.

**Acceptance Scenarios**:

1. **Given** uma base do Horizon carregada e um destino válido
   (repo/branch/caminho) com token de escrita, **When** o curador confirma o
   envio, **Then** o arquivo aparece na branch informada e o app exibe a
   confirmação com o commit gerado.
2. **Given** um caminho que já contém um arquivo no destino, **When** o curador
   confirma o envio, **Then** o arquivo é atualizado (substituição) e a
   confirmação deixa isso explícito — a escolha de sobrescrever é do curador,
   nunca silenciosa quando o arquivo de destino difere do export atual.
3. **Given** um token SEM permissão de escrita no repositório, **When** o
   curador confirma o envio, **Then** um erro claro informa falta de permissão
   e nada é gravado.
4. **Given** uma tentativa de enviar o JSON consolidado do SRC para qualquer
   repositório (público OU privado), **When** o curador confirma, **Then** o
   aplicativo recusa com mensagem explícita informando que esse formato contém
   dados pessoais e o envio não é oferecido (FR-005) — o export local do SRC
   permanece disponível sem alterações.
5. **Given** uma branch de destino que NÃO existe no repositório, **When** o
   curador confirma o envio, **Then** o aplicativo falha com erro claro
   indicando que a branch não existe e como criá-la no GitHub — nenhuma branch
   é criada automaticamente e nada é gravado.

---

### User Story 3 - Gerenciar o token de acesso localmente (Priority: P3)

O curador configura, testa e remove o token de acesso pessoal usado para
repositórios privados (download) e para envio (upload). O token fica somente no
aplicativo (máquina local), é opcional para repositórios públicos e pode ser
removido a qualquer momento. Nenhum fluxo exige conta online além do próprio
GitHub.

**Why this priority**: Necessária apenas quando o repositório é privado ou
quando há envio; o download de repositórios públicos funciona sem ela, e por
isso fica atrás das duas histórias principais.

**Independent Test**: Salvar um token, verificar que o download de um repo
privado passa a funcionar; remover o token e verificar que o acesso privado
volta a falhar com a mensagem adequada e o público continua funcionando.

**Acceptance Scenarios**:

1. **Given** um token válido, **When** o curador o salva e testa, **Then** o
   app confirma que o token funciona (acesso de leitura e/ou escrita, conforme
   o caso) sem expor o valor completo na tela.
2. **Given** um token salvo, **When** o curador o remove, **Then** nada dele
   permanece no aplicativo e o acesso a repositórios privados passa a falhar
   com a mensagem de "URL exige acesso autenticado".
3. **Given** qualquer operação de rede, **When** ela executa (ou falha), **Then**
   o token NUNCA aparece em mensagens, eventos de progresso ou diagnósticos.

---

## Clarifications

### Session 2026-09-29

- Q: O que o aplicativo deve fazer quando a branch de destino informada no
  envio ainda não existe no repositório? → A: **Opção A** — falhar com erro
  claro explicando que a branch não existe e como criá-la no GitHub; o
  aplicativo nunca cria branches automaticamente.
- Q: O envio ao GitHub deve gerar um export novo na hora ou reenviar o último
  export salvo em disco? → A: **Opção A** — ação única: no momento do envio o
  aplicativo gera um export novo a partir do estado atual da base e o envia;
  nunca reenvia um arquivo salvo anteriormente.
- Q: Qual o tamanho máximo de arquivo de export que o download e o envio
  precisam suportar nesta entrega? → A: **Opção B** — até 100 MB; acima disso,
  o aplicativo informa o limite antes de qualquer tentativa.

---

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001** — O import manual por arquivo (diálogo e arrastar-e-soltar) e o
  export manual para arquivo local continuam funcionando EXATAMENTE como hoje,
  em ambos os projetos (Horizon e SRC). Nenhum passo novo é exigido neles.
- **FR-002** — O curador pode informar a URL de um export hospedado no GitHub
  para importação, nos dois projetos, cada um no seu próprio espaço (o download
  do consolidado SRC só entra na área SRC; o zip canônico só na área Horizon).
- **FR-003** — O download segue o mesmo ritual de segurança do import manual:
  validação completa do conteúdo ANTES de qualquer alteração na base, snapshot
  do estado anterior quando havia curadoria, e substituição (nunca mescla).
- **FR-004** — A função de envio existe **por projeto**: Horizon e SRC têm
  repositórios separados e, portanto, configurações de destino separadas
  (repositório, branch e caminho por projeto). Nesta entrega, o envio do
  **zip canônico do Horizon** está habilitado; a confirmação exibida identifica
  o commit gerado e o destino. O envio não é oferecido para o JSON consolidado
  do SRC (FR-005). O envio é uma ação única: na confirmação, o aplicativo gera
  um export novo a partir do estado atual da base e o envia — nunca reenvia um
  arquivo salvo anteriormente. Se a branch de destino informada não existir no
  repositório, o envio falha com erro claro orientando o curador a criá-la no
  GitHub — o aplicativo nunca cria branches automaticamente.
- **FR-005** — Política de dados pessoais para envio (DECIDIDA — Option B):
  o JSON consolidado do SRC contém dados pessoais diretos (CPF, e-mail de
  participantes) e **não é enviado** ao GitHub nesta entrega — a operação não
  é oferecida na área SRC e qualquer tentativa direta é recusada com erro
  claro, mesmo em repositório privado. O envio do **zip canônico do Horizon**
  exibe um aviso genérico de que o arquivo pode conter dados pessoais antes de
  o curador prosseguir. O download (US1) não é afetado por esta política.
  **Este bloqueio é uma política por projeto, não uma limitação estrutural**: a
  função de envio do SRC existe no desenho, com destino próprio, e uma futura
  mudança desta política deve habilitá-la sem redesenho (bastando reverter a
  regra e acrescentar o aviso adequado).
- **FR-006** — O token de acesso é opcional (repositórios públicos dispensam),
  fica armazenado apenas localmente no aplicativo, pode ser removido a qualquer
  momento e nunca aparece em mensagens de erro, eventos ou diagnósticos.
- **FR-007** — Todas as falhas de rede/autenticação/permissão/tamanho produzem
  erros tipados com mensagem acionável (o que houve e o que fazer), sem deixar a
  base em estado parcial: ou a operação completa, ou nada muda.
- **FR-008** — Operações de download/upload exibem progresso (início, etapas e
  conclusão) e podem ser reconhecidas como concluídas ou falhas pelo curador;
  em caso de queda de conexão no meio, o estado local permanece o de antes.
- **FR-009** — O download de um export do Horizon nunca toca dados do SRC e
  vice-versa — a separação de projetos já estabelecida vale também para a
  sincronização.
- **FR-010** — Download e envio suportam arquivos de export de até **100 MB**.
  O aplicativo informa limites operacionais quando o destino/origem os excede
  (por exemplo, arquivo maior do que o aceito pelo destino ou acima do limite
  desta entrega), com mensagem clara antes de qualquer tentativa fadada a
  falhar.

### Assumptions

- O curador possui conta GitHub; para repositórios privados e para envio, um
  token de acesso pessoal com permissões adequadas (leitura para download
  privado; leitura e escrita para envio).
- O destino padrão de envio é um arquivo na branch informada do repositório
  (commit direto), dentro dos limites de tamanho aceitos pelo GitHub para esse
  mecanismo; não há criação de releases nem uso de Git LFS neste escopo.
- A URL informada pode ser tanto o link direto do arquivo (raw) quanto um link
  de release asset; o aplicativo reconhece e trata ambos.
- Repositórios são git padrão hospedados em github.com (não há suporte a
  GitLab/outros provedores nesta feature).
- Sobrescrever um arquivo existente no destino é permitido, mas sempre com
  confirmação explícita quando o conteúdo de destino difere do export atual.
- O token salvo vale para os dois projetos (Horizon e SRC), configurado uma
  única vez no aplicativo; os **destinos**, ao contrário, são sempre por
  projeto — o repositório do Horizon e o do SRC são independentes (projetos e
  gits separados).

### Dependencies & Constraints

- Requer conectividade com github.com a partir da máquina do curador; sem rede,
  as funcionalidades desta feature ficam indisponíveis — o restante do
  aplicativo (incluindo import/export manuais) não depende de rede.
- Os mecanismos públicos do GitHub impõem limites de tamanho e de taxa de
  requisições; a feature deve respeitá-los e comunicá-los (FR-010).
- Dados pessoais presentes nos exports (CPF, e-mail, nomes) tornam a política
  de envio (FR-005) uma decisão de privacidade, não apenas técnica.

### Key Entities

- **Configuração de sincronização por projeto** (local ao aplicativo): Horizon e
  SRC são projetos independentes, cada um com a sua própria configuração —
  destino de envio (repositório, branch, caminho) e última origem usada para
  download. Os repositórios dos dois projetos são separados por definição.
- **Token de acesso** (compartilhado entre os projetos, opcional e removível).
- **Operação de sincronização** (transiente, sempre associada a um único
  projeto): origem ou destino, formato (zip canônico do Horizon / JSON
  consolidado do SRC), estado (em progresso, concluída, falha) e resultado
  (contagens importadas ou commit gerado).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001** — Um export hospedado em repositório público é baixado e importado
  com o MESMO resultado do import manual do mesmo arquivo (contagens idênticas),
  em até 60 segundos para um arquivo de 50 MB em conexão caseira razoável.
- **SC-002** — Em 100% dos casos de falha de rede/autenticação/permissão, o
  curador recebe uma mensagem que identifica a causa e a base local permanece
  exatamente no estado anterior (verificável por contagens antes/depois).
- **SC-003** — O envio do zip canônico do Horizon para um
  repositório/branch/caminho válido conclui com o arquivo acessível na branch
  informada e o app exibindo a confirmação com o commit; o conteúdo no destino é
  idêntico byte a byte ao export local. Tentativas de envio do consolidado SRC
  são 100% recusadas com mensagem explícita (FR-005).
- **SC-004** — O import manual por arquivo continua passando em TODOS os testes
  e cenários que passavam antes desta feature (regressão zero), nos dois projetos.
- **SC-005** — Nenhuma mensagem, evento ou diagnóstico produzido pela feature
  contém o valor do token ou dados pessoais dos registros (verificável por
  inspeção de todas as mensagens emitidas nos fluxos de erro).
- **SC-006** — Remover o token elimina todo vestígio dele do armazenamento
  local do aplicativo (verificável por inspeção dos arquivos de configuração).
- **SC-007** — Um export de 100 MB completa o download/import e é aceito pelo
  envio; arquivos acima de 100 MB são 100% bloqueados com mensagem clara antes
  de qualquer transferência iniciar.
