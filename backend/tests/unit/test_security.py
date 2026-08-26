import os
from src.core.security import create_access_token, verify_password, get_password_hash

def test_password_hashing():
    password = "supersecretpassword"
    hashed = get_password_hash(password)
    assert hashed != password
    assert verify_password(password, hashed) is True
    assert verify_password("wrongpassword", hashed) is False

def test_create_access_token():
    os.environ["SECRET_KEY"] = "testsecretkey"
    data = {"sub": "admin@example.com"}
    token = create_access_token(data)
    assert isinstance(token, str)
    assert len(token) > 20
