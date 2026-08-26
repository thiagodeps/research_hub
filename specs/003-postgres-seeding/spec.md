# Feature Specification: Banco de Dados PostgreSQL e Dados Falsos (Seeding)

**Feature Branch**: `003-postgres-seeding`  
**Created**: 2026-08-26  
**Status**: Draft  
**Input**: User description: "Implementar persistência de dados utilizando PostgreSQL e SQLAlchemy, substituindo o adaptador em memória. Além disso, criar um script de seeding (povoamento) com dados falsos realistas para universidades e pesquisadores para facilitar testes."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Persistência em Banco de Dados Relacional (Priority: P1)

Como Administrador, quero que os registros do sistema (como Administradores, Universidades e Pesquisadores) sejam salvos em um banco de dados real (PostgreSQL), para que as informações não sejam perdidas ao reiniciar o servidor.

**Why this priority**: A persistência real é o núcleo de um sistema web em produção. Sem ela, qualquer dado inserido pelos usuários é efêmero.

**Independent Test**: Pode ser testado de forma independente criando um registro via API, reiniciando o serviço do backend, e consultando a API novamente para atestar que o registro sobreviveu.

**Acceptance Scenarios**:

1. **Given** o sistema rodando com o adaptador PostgreSQL ativo, **When** um novo registro de Pesquisador for criado, **Then** a aplicação salva permanentemente o dado no banco de dados.
2. **Given** o banco de dados recém criado, **When** uma operação CRUD for solicitada, **Then** o sistema deve realizar queries SQL corretamente utilizando mapeamento ORM, sem falhas de sintaxe.

---

### User Story 2 - Povoamento Inicial com Dados de Teste (Priority: P2)

Como Desenvolvedor/Administrador, quero poder rodar um comando que preencha automaticamente o banco de dados com centenas de dados realistas (nomes, emails, Lattes IDs), para que eu possa testar paginação, layout, fluxo e performance sem ter que digitar um por um.

**Why this priority**: Dados para preenchimento de UI são essenciais para testes confiáveis de layout e regras de negócio complexas como fusão e vinculação de registros.

**Independent Test**: Pode ser testado rodando o script de seeder via terminal e em seguida inspecionando o banco de dados ou a interface do dashboard para ver a massa de dados gerada.

**Acceptance Scenarios**:

1. **Given** um banco de dados vazio, **When** o administrador executa o script de seed, **Then** o sistema insere registros variados para Universidades e Pesquisadores (além do usuário Admin padrão).
2. **Given** as listagens no painel administrativo, **When** as telas de "Universities" e "Researchers" forem carregadas, **Then** elas devem exibir a lista vasta gerada pelo seed para validação do design e interatividade.

## Requirements *(mandatory)*

### Technical & Architectural Constraints

- **CON-001**: O sistema DEVE utilizar PostgreSQL como banco de dados relacional oficial.
- **CON-002**: O sistema DEVE utilizar o SQLAlchemy como ORM (Object-Relational Mapper) no backend Python.
- **CON-003**: A arquitetura de `DatabaseMemoryAdapter` atual não DEVE ser deletada, mas deve haver um mecanismo de chaveamento (via Variável de Ambiente `STORAGE_TYPE=postgres`) para injetar a dependência correta.
- **CON-004**: O Script de seeding DEVE usar bibliotecas que gerem dados realistas, não apenas strings randômicas sem sentido.

### Functional Requirements

- **FR-001**: O sistema DEVE suportar migrações estruturais ou ter as tabelas geradas automaticamente no startup.
- **FR-002**: A injeção de dependência atual em `repositories.py` (`get_db()`) DEVE retornar o banco instanciado corretamente, conectando via URL.
- **FR-003**: O script de Seeding DEVE limpar tabelas e reinserir pelo menos 1 usuário Admin, 5 Universidades e 20 Pesquisadores ao ser acionado, para não duplicar toda vez.

### Key Entities

- **AdminUser**: id, email, password_hash
- **University**: id, name, abbreviation
- **Researcher**: id, name, email, lattes_id, orcid

*(Estas refletem o modelo base simplificado atualmente em uso pelo portal para CRUD)*

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: O backend conecta de forma estável a um container PostgreSQL via TCP sem estourar pool de conexões.
- **SC-002**: O sistema não apresenta perda de dados (Data Loss) ao reiniciar a API.
- **SC-003**: O script de seeding termina sua execução com sucesso em menos de 5 segundos e popula as 3 tabelas requeridas.
- **SC-004**: Os testes de integração passam usando um banco de testes limpo ou em memória.

## Assumptions

- Presume-se que o ambiente do desenvolvedor possui Docker instalado para levantar a imagem do PostgreSQL localmente ou usará SQLite como fallback de testes nativo do SQLAlchemy.
- Assumido que os modelos de domínio do pacote `research_domain` (que foram citados em features passadas) serão adotados gradativamente ou emulados nesse mapeamento de tabelas para a PoC de seeding.
