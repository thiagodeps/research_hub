from pydantic import BaseModel, EmailStr
import uuid

class AdminUser(BaseModel):
    id: str
    email: EmailStr
    password_hash: str

class AdminCreate(BaseModel):
    email: EmailStr
    password: str

class Token(BaseModel):
    access_token: str
    token_type: str
