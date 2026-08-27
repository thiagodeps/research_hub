# Research Hub - Data Curation Tool 🧬

Bem-vindo ao **Research Hub**, a plataforma centralizada de curadoria e gestão de dados acadêmicos. Este sistema atua como uma ponte (ETL/Curadoria) entre o DataLake bruto (exportações em formato `.parquet` e `.json`) e os pipelines de Machine Learning e visualização da instituição.

O sistema recebe um pacote ZIP com dezenas de bases de dados acadêmicas (Pesquisadores, Alunos, Projetos, etc), carrega esses dados em um banco relacional, oferece uma **interface amigável para correção, fusão e relacionamento** dessas informações, e exporta novamente os dados tratados no formato original (Parquet e JSON sincronizados).

## 🚀 Tecnologias Utilizadas

**Backend (Motor de Dados & API)**
- **Python 3.12+**
- **FastAPI:** Criação das rotas RESTful da nossa API.
- **Pandas / PyArrow:** Motor de ingestão e geração de alto desempenho para lidar com os arquivos `.parquet`.
- **SQLAlchemy:** ORM para mapeamento e gestão do Banco de Dados (compatível com SQLite e PostgreSQL).
- **Uvicorn:** Servidor ASGI para rodar a aplicação.

**Frontend (Interface Gráfica)**
- **Astro:** Framework de alta velocidade para o ecossistema e rotas (SSG/SSR).
- **React.js:** Componentização da UI e gestão de estado complexo (Edição de entidades, Modais).
- **Tailwind CSS:** Estilização da interface de maneira ágil e moderna.

---

## 📦 Arquitetura dos Dados

A plataforma lida com **15 Domínios de Dados Canônicos**:
1. Pesquisadores (`researchers`)
2. Alunos (`students`)
3. Grupos de Pesquisa (`research_groups`)
4. Iniciativas/Projetos (`initiatives`)
5. Premiações (`awards`)
6. Produções Científicas (`research_productions`)
7. Áreas de Conhecimento (`knowledge_areas`)
8. Orientações (`advisorships`)
9. Organizações (`organizations`)
10. Atividades Profissionais (`professional_activities`)
11. Campus (`campuses`)
12. Proficiências (`proficiencies`)
13. Bolsas (`fellowships`)
14. Idiomas (`languages`)
15. Artigos (`articles`)

**Atenção:** As tabelas utilizam colunas JSON Array para cruzamento dinâmico de chaves estrangeiras, eliminando a necessidade de tabelas associativas complexas e otimizando a leitura nos pipelines de Big Data.

---

## 🛠️ Como Instalar e Rodar Localmente

O projeto é dividido em dois serviços principais que devem ser rodados simultaneamente.

### 1. Inicializando o Backend (Python)
Abra um terminal na raiz do projeto e acesse a pasta `backend`:
```bash
cd backend
```
Crie e ative um ambiente virtual (recomendado):
```bash
python -m venv venv
source venv/bin/activate  # No Linux/Mac
# venv\Scripts\activate   # No Windows
```
Instale as dependências:
```bash
pip install fastapi uvicorn sqlalchemy pandas pyarrow python-multipart
```
Inicie o servidor local na porta 8000:
```bash
# O banco SQLite (test.db) será criado automaticamente
export STORAGE_TYPE=postgres  # Opcional: Se quiser apontar para um PostgreSQL
uvicorn src.api.main:app --reload --port 8000
```
> O backend agora está rodando e escutando as rotas da API em `http://localhost:8000`.

### 2. Inicializando o Frontend (Astro/React)
Abra um **novo terminal** na raiz do projeto e acesse a pasta `frontend`:
```bash
cd frontend
```
Instale as dependências via NPM:
```bash
npm install
```
Inicie o servidor de desenvolvimento do Astro:
```bash
npm run dev
```
> Acesse a interface web em `http://localhost:4321`.

---

## ⚙️ Fluxo de Trabalho (Curadoria de Dados)

1. **Importação (`Upload`):**
   - Na página inicial do Painel (Dashboard), utilize a central de controle para enviar o arquivo `exports_canonical.zip` proveniente do DataLake original.
   - O Backend salva uma cópia intacta do `.zip`, dropa o banco de dados atual, processa todos os Parquets importados utilizando *Pandas* e preenche automaticamente as 15 tabelas mapeadas.

2. **Edição e Vínculos:**
   - Navegue pelo menu lateral para explorar as entidades.
   - Utilize as tabelas paginadas para encontrar registros rasurados.
   - Você pode editar textos e usar a ferramenta **"Vincular"** para cruzar dados (ex: associar uma "Iniciativa" a um "Pesquisador"). O sistema lida com o parse automático dos Arrays JSON por debaixo dos panos.

3. **Exportação (`Download`):**
   - Ao finalizar sua curadoria, clique em **Exportar Curadoria**.
   - O sistema reconstrói um novo arquivo `.zip` combinando o seu banco de dados higienizado com os arquivos originais intocados (grafos e metadados).
   - O exportador sincroniza e recria ambos os formatos (`.parquet` compactado e `.json` pretty-print/minificado) mantendo exatamente o layout estrutural e tipagem (Int64, bool) exigidos pelo pipeline de destino.

---

## 📜 Regras de Negócio e Convenções
- Todas as variáveis relacionais (como `initiatives` dentro de `researchers`) são mapeadas no frontend como `json_readonly` para otimização de renderização e prevenção de travamentos do navegador.
- Arquivos passados no export ZIP que **não** constam no banco de dados mapeado (ex: `.meta.json` e `.cols.json`) são copiados **literalmente** da versão original do `.zip` fornecida no momento do Upload, preservando a integridade do pacote de dados.
- Modificações de estrutura no banco exigem recadastramento no dicionário do arquivo `backend/src/database/postgres_adapter.py`.

