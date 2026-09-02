# Feature Specification: Shell Desktop Tauri 2.0

**Feature Branch**: `015-tauri-shell-bootstrap`
**Created**: 2026-09-02
**Status**: Draft
**Input**: User description: "Criar o shell da aplicação desktop Tauri 2.0 que carrega o build estático do frontend Astro existente, sem nenhuma lógica de negócio. Deve abrir uma janela, exibir a tela de login e permitir navegar por todas as 15 páginas do dashboard sem erro de rota, em Linux e Windows. Inclui o spike que decide a estratégia de roteamento do Astro sob o protocolo tauri://. Inclui também a correção da sidebar em Dashboard.astro."

**Referência**: `docs/estudo-migracao-rust-tauri.md` — Seções 4 (arquitetura alvo), 8.2/8.3/8.7 (esboços de configuração), 10.6 (risco de roteamento), 18 (decisões Q1, Q2, Q7).

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Abrir a ferramenta como aplicativo (Priority: P1)

O curador abre o ResearchHub como qualquer outro programa instalado na máquina — pelo menu de aplicativos ou por um atalho — e a janela aparece com a tela de login. Não é preciso abrir terminal, ativar ambiente virtual, subir dois servidores nem digitar um endereço no navegador.

**Why this priority**: É a mudança de natureza que justifica o projeto inteiro. Sem ela, nada mais da migração tem sentido. Também é o pré-requisito técnico de todas as features seguintes: sem shell, não há onde registrar comandos.

**Independent Test**: Executar o binário compilado em uma máquina onde o backend Python **não** esteja rodando e nenhum servidor esteja escutando nas portas 8000 ou 4321. A janela deve abrir com a interface completa.

**Acceptance Scenarios**:

1. **Given** nenhum processo do projeto em execução, **When** o binário do ResearchHub é iniciado, **Then** uma janela nativa abre exibindo a tela de login com o layout split-screen do Figma.
2. **Given** a aplicação aberta, **When** o usuário inspeciona as portas de rede da máquina, **Then** nenhuma porta foi aberta pela aplicação.
3. **Given** a máquina sem conexão de rede, **When** a aplicação é iniciada, **Then** ela abre e navega normalmente.
4. **Given** a janela aberta, **When** o usuário a redimensiona para a largura mínima, **Then** o layout permanece utilizável e sem rolagem horizontal.

---

### User Story 2 - Navegar por todo o painel sem erro de rota (Priority: P1)

Dentro do aplicativo, o curador clica em cada item do menu lateral e chega à página correspondente. Nenhum clique leva a tela em branco, erro de arquivo não encontrado ou página do webview.

**Why this priority**: É o risco técnico que este trabalho existe para eliminar. O Astro gera rotas como diretórios com `index.html`, e o protocolo de asset do Tauri resolve caminhos de forma diferente de um servidor HTTP. Se a estratégia de roteamento não for decidida e validada aqui, o problema reaparece em toda feature seguinte, com custo multiplicado.

**Independent Test**: Percorrer os 16 destinos de navegação (dashboard + 15 entidades) e o redirecionamento da raiz, verificando que cada um renderiza sua página.

**Acceptance Scenarios**:

1. **Given** o aplicativo aberto no painel, **When** o usuário clica em cada um dos 15 itens de entidade do menu lateral, **Then** cada clique renderiza a página correspondente, com o título da entidade visível.
2. **Given** o usuário em qualquer página de entidade, **When** clica em "Dashboard", **Then** a Central de Curadoria de Dados é exibida.
3. **Given** o aplicativo recém-aberto sem sessão, **When** a rota raiz é carregada, **Then** o usuário é levado à tela de login.
4. **Given** um link profundo de edição (`?openId=N`) construído pela interface, **When** ele é seguido, **Then** a página de destino carrega sem erro de rota. *(O comportamento de abrir o formulário depende de dados e é escopo da SEP-017; aqui verifica-se apenas que a rota resolve.)*

---

### User Story 3 - Enxergar todos os itens do menu (Priority: P2)

O curador consegue ler e distinguir todos os itens do menu lateral, do primeiro ao último.

**Why this priority**: Hoje, os itens a partir de "Grupos de Pesquisa" usam classes de tema claro sobre o painel escuro, ficando praticamente ilegíveis — o menu parece ter quatro itens quando tem dezesseis. É um defeito visível de usabilidade e uma violação da exigência de aderência ao Figma (Constitution, Princípio V). Entra nesta feature porque o mesmo arquivo já será modificado pelo ajuste de roteamento; separá-lo significaria tocar `Dashboard.astro` duas vezes.

**Independent Test**: Inspeção visual do menu lateral com verificação de contraste, mais validação da estrutura HTML.

**Acceptance Scenarios**:

1. **Given** o painel aberto, **When** o usuário observa o menu lateral, **Then** todos os 16 itens de navegação têm contraste legível sobre o fundo escuro.
2. **Given** o menu lateral, **When** o usuário passa o cursor sobre qualquer item, **Then** o estado de hover é visível e consistente entre todos os itens.
3. **Given** o HTML da página renderizada, **When** ele é validado, **Then** não há elementos `<li>` fora de um elemento de lista.

---

### Edge Cases

- **A resolução de rota do protocolo `tauri://` não corresponde à do servidor de desenvolvimento.** É a razão de ser do spike. O comportamento deve ser idêntico em `cargo tauri dev` (que serve via HTTP em `localhost:4321`) e no binário compilado (que serve via protocolo de asset). Uma navegação que funcione só no modo de desenvolvimento não satisfaz esta spec.
- **Diretório de dados da aplicação inexistente na primeira execução.** A aplicação deve criá-lo, não falhar.
- **Segunda instância da aplicação.** Comportamento a definir no plano; aceitável nesta feature que duas janelas independentes abram, desde que documentado.
- **Janela redimensionada abaixo da largura mínima.** Deve ser impedido pela configuração da janela.
- **Front-end tentando alcançar `localhost:8000`.** Enquanto o shim de comunicação não existir (SEP-017), as chamadas do front-end falharão. Isso é esperado nesta feature: a falha deve ser silenciosa no console, sem quebrar a renderização ou a navegação.

## Requirements *(mandatory)*

### Technical & Architectural Constraints

- **CON-001**: A aplicação DEVE ser um app desktop de processo único em Tauri 2.0, com núcleo em Rust e front-end em Astro/React/Tailwind compilado estaticamente e embarcado no binário, seguindo o protótipo Figma. Funciona integralmente sem rede.
- **CON-002**: A persistência DEVE usar SQLite em arquivo único, embarcado (`rusqlite`, feature `bundled`), no diretório de dados da aplicação. *(Não exercitada nesta feature; nenhuma decisão aqui pode contrariá-la.)*
- **CON-003**: Toda regra de negócio DEVE residir no processo Rust. *(Nenhuma regra de negócio é implementada nesta feature.)*
- **CON-004**: Todas as entidades do domínio exigem CRUD completo operado unicamente por perfil de Admin. *(Fora do escopo desta feature.)*
- **CON-005**: Paridade funcional precede melhoria. A correção da sidebar (FR-011) é a única mudança de comportamento desta feature e está declarada como escopo explícito.
- **CON-006**: Alvos de release são Linux e Windows. macOS, assinatura de código, auto-update e bancos não-SQLite estão fora de escopo.

### Functional Requirements

**Shell da aplicação**

- **FR-001**: O sistema DEVE abrir uma janela nativa exibindo o front-end Astro compilado, sem depender de navegador instalado ou de qualquer processo externo.
- **FR-002**: O sistema DEVE funcionar sem abrir portas de rede e sem acesso à internet.
- **FR-003**: A janela DEVE ter título, dimensões iniciais e largura mínima definidos, e aceitar arquivos arrastados sobre ela — a capacidade fica habilitada aqui, mas o tratamento do arquivo é escopo da SEP-018.
- **FR-004**: O sistema DEVE declarar o conjunto **mínimo** de permissões necessárias. Como decidido em Q1 (o Rust abre diálogos e lê arquivos), nenhuma permissão de acesso a arquivos ou a diálogos é concedida ao front-end.
- **FR-005**: O sistema DEVE aplicar uma política de segurança de conteúdo que bloqueie carregamento de recursos remotos, dado que nenhum recurso externo é utilizado.

**Roteamento**

- **FR-006**: Toda rota de navegação DEVE resolver corretamente tanto no modo de desenvolvimento quanto no binário compilado. A estratégia escolhida DEVE ser documentada em `plan.md` com a justificativa e as alternativas descartadas.
- **FR-007**: A rota raiz DEVE redirecionar para a tela de login quando não houver sessão, e para o painel quando houver.
- **FR-008**: Os 16 destinos do menu lateral DEVEM ser navegáveis.
- **FR-009**: Rotas com parâmetro de consulta (`?openId=N`) DEVEM resolver sem erro.

**Build e execução**

- **FR-010**: O projeto DEVE oferecer um comando único para desenvolvimento (que sobe o front-end e abre a janela com recarga automática) e um comando único para gerar o binário.

**Interface**

- **FR-011**: O menu lateral DEVE apresentar todos os itens com contraste legível sobre o fundo escuro e marcação HTML válida, corrigindo os itens que hoje usam classes de tema claro e os elementos de lista mal aninhados.
- **FR-012**: Nenhum componente React existente pode ser alterado nesta feature. As alterações no front-end limitam-se à configuração do Astro e ao arquivo de layout do painel.

### Key Entities

Nenhuma. Esta feature não introduz, lê nem grava dados de domínio. O modelo de dados é estabelecido na SEP-016.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Um usuário abre a ferramenta e chega à tela de login em um único passo (abrir o aplicativo), contra os cinco passos atuais — abrir terminal, ativar ambiente virtual, definir variável de ambiente, subir dois servidores, digitar endereço no navegador.
- **SC-002**: 16 de 16 destinos de navegação resolvem sem erro no binário compilado. Este número, e não uma amostra, é o critério.
- **SC-003**: A aplicação abre e navega integralmente com a máquina desconectada da rede e sem Python ou Node instalados.
- **SC-004**: O binário é gerado e executa nas duas plataformas-alvo, Linux e Windows.
- **SC-005**: Zero componentes React modificados — verificável por `git diff --stat` sobre `frontend/src/components/`.
- **SC-006**: A janela abre em menos de 3 segundos em hardware de desenvolvimento típico.
- **SC-007**: Todos os 16 itens do menu lateral atingem contraste mínimo AA (4.5:1) contra o fundo do painel.

## Assumptions

- O ambiente de desenvolvimento receberá a cadeia de ferramentas Rust, hoje ausente na máquina (o estudo confirma que apenas Node está instalado). A instalação é pré-requisito desta feature.
- As dependências de sistema do Tauri para Linux serão instaladas conforme a documentação oficial.
- O build atual do Astro é funcional e serve de ponto de partida sem correções prévias.
- Chamadas do front-end ao backend Python falharão durante esta feature. É aceitável e esperado: a comunicação chega na SEP-017.
- O backend Python permanece intocado no repositório, como oráculo de paridade (Constitution, Princípio II).
- A verificação end-to-end automatizada é estabelecida na SEP-022; nesta feature a verificação de navegação é executada manualmente pelo desenvolvedor durante o spike, o que é aceitável porque nada aqui é alvo de release ainda.

## Out of Scope

Explicitamente **fora** desta feature, para evitar que o shell absorva trabalho das seguintes:

- Qualquer comando de negócio, acesso a banco ou leitura de arquivo (SEP-016 em diante).
- O shim de comunicação em `services/api.js` (SEP-017).
- Diálogos nativos de importação e exportação (SEP-018 e SEP-019).
- Autenticação real — a tela de login é renderizada, mas não autentica (SEP-021).
- Empacotamento para distribuição, ícones definitivos e pipeline de integração contínua (SEP-023).
- Suporte a macOS (escopo diferido, Constitution).
