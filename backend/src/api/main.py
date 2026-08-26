from fastapi import FastAPI
from src.core.exceptions import setup_exception_handlers
from src.api.auth import router as auth_router
from src.api.crud import router as crud_router
from src.api.special_ops import router as special_ops_router

app = FastAPI(title="Portal Admin e Gestão de Pesquisa")
setup_exception_handlers(app)

app.include_router(auth_router)
app.include_router(special_ops_router)
app.include_router(crud_router)
