from pydantic import BaseModel, EmailStr
import uuid

from typing import Union

class AdminUser(BaseModel):
    id: Union[int, str]
    email: EmailStr
    password_hash: str

class AdminCreate(BaseModel):
    email: EmailStr
    password: str

class Token(BaseModel):
    access_token: str
    token_type: str
