from src.database.repositories import BaseRepository

class LinkService:
    def link_entities(self, parent_type: str, parent_id: int, child_type: str, child_id: int):
        parent_repo = BaseRepository(parent_type)
        child_repo = BaseRepository(child_type)
        
        parent = parent_repo.get_by_id(parent_id)
        child = child_repo.get_by_id(child_id)
        
        if not parent or not child:
            return False
            
        # Implementation of linking logic
        child["parent_id"] = parent_id
        child_repo.update(child_id, child)
        return True
