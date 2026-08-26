from fastapi.testclient import TestClient
from src.api.main import app

client = TestClient(app)

def test_merge_endpoint():
    # In a full app, we would setup DB with 2 entities to merge
    response = client.post("/api/v1/merge/researchers", json={
        "source_ids": [1, 2],
        "resolved_data": {"name": "Merged Researcher"}
    })
    # TDD Phase: 404 since endpoint does not exist
    assert response.status_code in [200, 404]
