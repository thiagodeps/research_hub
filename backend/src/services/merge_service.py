from src.database.repositories import BaseRepository

class MergeService:
    def merge(self, entity_type: str, source_ids: list, resolved_data: dict):
        repo = BaseRepository(entity_type)
        
        # In a real app we'd fetch both entities, migrate their relationships, etc.
        # 1. Create the new merged entity
        new_entity = repo.create(resolved_data)
        
        # 2. Soft delete the original entities
        for src_id in source_ids:
            record = repo.get_by_id(src_id)
            if record:
                record["is_active"] = False
                record["merged_into"] = new_entity["id"]
                repo.update(src_id, record)
                
        return new_entity
