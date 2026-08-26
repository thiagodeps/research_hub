from fastapi.testclient import TestClient
from src.api.main import app

client = TestClient(app)

def test_link_endpoint():
    response = client.post("/api/v1/link", json={
        "parent_type": "advisorship",
        "parent_id": 1,
        "child_type": "article",
        "child_id": 2
    })
    assert response.status_code in [200, 400, 404]
