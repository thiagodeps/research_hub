from fastapi import APIRouter, HTTPException
from pydantic import BaseModel
from typing import List
from src.services.merge_service import MergeService
from src.services.link_service import LinkService

router = APIRouter(prefix="/api/v1")

class MergeRequest(BaseModel):
    source_ids: List[int]
    resolved_data: dict

class LinkRequest(BaseModel):
    parent_type: str
    parent_id: int
    child_type: str
    child_id: int

@router.post("/merge/{entity_type}")
def merge_entities(entity_type: str, request: MergeRequest):
    service = MergeService()
    result = service.merge(entity_type, request.source_ids, request.resolved_data)
    return result

@router.post("/link")
def link_entities(request: LinkRequest):
    service = LinkService()
    success = service.link_entities(
        request.parent_type, request.parent_id, 
        request.child_type, request.child_id
    )
    if not success:
        raise HTTPException(status_code=400, detail="Failed to link entities")
    return {"status": "success"}
