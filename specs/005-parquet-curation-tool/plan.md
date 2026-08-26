# Implementation Plan: Curadoria e Edição de Parquets

## 1. Context & Approach

A visão do sistema evoluiu. Em vez de um portal que se conecta a um banco definitivo em produção que é preenchido misteriosamente pelo terminal, o sistema é agora uma **Ferramenta de Curadoria de Dados**. O usuário (curador) recebe exports de um banco de dados analítico (ou datalake), faz upload no sistema, edita, funde e linka registros, e exporta novamente os dados higienizados para retroalimentar a pipeline.

Utilizaremos o `pandas` para intermediar a conversão `Parquet <-> SQLAlchemy`. O backend no FastAPI fornecerá rotas específicas de I/O de arquivos. 

## 2. Component Architecture

### Component 1: Engine de Conversão (Backend)
- Criar `backend/src/services/parquet_service.py`.
- **Import**: Recebe bytes de um arquivo `.zip`, extrai na memória os arquivos `.parquet`, converte para DataFrames e os injeta (usando `.to_sql()` com SQLAlchemy engine ou chamadas ao `db.save`).
- **Export**: Consulta todas as tabelas mapeadas via ORM (`pd.read_sql`), salva em `io.BytesIO` no formato `.parquet`, empaqueta tudo em um arquivo `.zip` e envia o binário.

### Component 2: Rotas de Upload/Download (`backend/src/api/routes/files.py`)
- `POST /api/v1/data/import`: Recebe `File(...)`. Processa de forma assíncrona/thread, limpando o banco atual. Retorna status.
- `GET /api/v1/data/export`: Gera os dados atualizados e envia um `.zip` com `media_type="application/zip"`.

### Component 3: UI de Gestão de Arquivos (`frontend/src/pages/dashboard/index.astro`)
- Substituir a tela vazia de boas vindas por um "Control Center".
- Card de **Importação**: Botão `input type="file" accept=".zip"`, que aciona um fetch POST. Barra de loading enquanto aguarda.
- Card de **Exportação**: Botão "Baixar Dados Limpos", aciona download.
- (Opcional) Card indicando quantidade de itens por tabela.

## 3. Data Flow & Interfaces

1. **Upload**: UI envia multipart-form-data. FastAPI pega, invoca `ParquetService.import_zip(file)`. O Banco de dados reseta tabelas e carrega tudo. FastAPI responde 200 OK. UI da um reload nas páginas.
2. **Download**: UI faz um GET simples. FastAPI chama `ParquetService.export_zip()`, retorna stream com mimetype ZIP. Navegador do usuário faz o download nativo.

## 4. Dependencies
- FastAPI multipart-form-data (`python-multipart` library).
- Pandas and PyArrow (Already installed no terminal `task-762`).
