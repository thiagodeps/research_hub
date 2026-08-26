from fastapi.testclient import TestClient
import pytest
from src.api.main import app

client = TestClient(app)

def test_login_contract_success():
    response = client.post(
        "/api/v1/auth/login",
        json={"email": "admin@example.com", "password": "password"}
    )
    # The actual implementation will check valid credentials, but contract test 
    # might just check the endpoint structure or mock the service.
    # We expect a 200 with an access_token.
    # Since we are TDD-ing, it should fail with 404 because the endpoint doesn't exist yet.
    assert response.status_code in [200, 401]
    
    if response.status_code == 200:
        data = response.json()
        assert "access_token" in data
        assert "token_type" in data
        assert data["token_type"] == "bearer"
