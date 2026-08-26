# Quickstart: Portal Admin e Gestão de Pesquisa

Este guia descreve como iniciar o ambiente de desenvolvimento monorepo contendo o Backend FastAPI e o Frontend Astro.

## Pré-requisitos
- Python 3.11+
- Node.js 18+ e npm (ou pnpm/yarn)
- Variáveis de ambiente configuradas (`.env`) em ambos os sub-diretórios.

## Passo 1: Inicializando o Backend
O backend provê a API REST usando FastAPI e integra-se à biblioteca `research_domain`.

```bash
cd backend
python -m venv venv
source venv/bin/activate
pip install -r requirements.txt
```

No `.env` do backend, configure o ambiente para banco em memória (dev):
```env
STORAGE_TYPE=memory
ADMIN_EMAIL=admin@admin.com
ADMIN_PASSWORD=senha_segura
SECRET_KEY=super_secret_jwt_key
```

Execute o seed e levante o servidor:
```bash
python scripts/seed.py
uvicorn src.api.main:app --reload --port 8000
```
A API estará disponível em `http://localhost:8000` e o Swagger em `http://localhost:8000/docs`.

## Passo 2: Inicializando o Frontend
O frontend Astro provê o painel Administrativo (SSG + requisições cliente para o dashboard).

```bash
cd frontend
npm install
```

No `.env` do frontend, aponte para a API local:
```env
PUBLIC_API_URL=http://localhost:8000/api/v1
```

Inicie o servidor de desenvolvimento:
```bash
npm run dev
```
O portal estará disponível em `http://localhost:4321`.
O Admin fará o login usando as credenciais seed (admin@admin.com).
