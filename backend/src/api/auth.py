from fastapi import APIRouter, HTTPException, status
from src.models.admin import AdminCreate, Token
from src.services.auth_service import authenticate_admin
from src.core.security import create_access_token

router = APIRouter(prefix="/api/v1/auth", tags=["auth"])

@router.post("/login", response_model=Token)
def login(form_data: AdminCreate):
    admin = authenticate_admin(form_data.email, form_data.password)
    if not admin:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Credenciais inválidas",
        )
    access_token = create_access_token(subject=admin.email)
    return {"access_token": access_token, "token_type": "bearer"}
