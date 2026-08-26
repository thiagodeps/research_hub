from fastapi import APIRouter, HTTPException, Depends
from typing import Any
from src.services.crud_service import CrudService

router = APIRouter(prefix="/api/v1")

# Fast generic dependency to get the service based on path
def get_service(entity: str) -> CrudService:
    return CrudService(entity)

@router.get("/{entity}")
def get_all(entity: str):
    service = get_service(entity)
    items = service.list_all()
    return {"items": items, "total": len(items)}

@router.get("/{entity}/{entity_id}")
def get_one(entity: str, entity_id: int):
    service = get_service(entity)
    record = service.get_one(entity_id)
    if not record:
        raise HTTPException(status_code=404, detail="Não encontrado")
    return record

@router.post("/{entity}", status_code=201)
def create(entity: str, payload: dict):
    service = get_service(entity)
    return service.create(payload)

@router.put("/{entity}/{entity_id}")
def update(entity: str, entity_id: int, payload: dict):
    service = get_service(entity)
    record = service.update(entity_id, payload)
    if not record:
        raise HTTPException(status_code=404, detail="Não encontrado")
    return record

@router.delete("/{entity}/{entity_id}", status_code=204)
def delete(entity: str, entity_id: int):
    service = get_service(entity)
    success = service.delete(entity_id)
    if not success:
        raise HTTPException(status_code=404, detail="Não encontrado")
    return None
