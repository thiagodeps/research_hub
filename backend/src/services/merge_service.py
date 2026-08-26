from src.database.repositories import BaseRepository

class MergeService:
    def merge(self, entity_type: str, source_ids: list, resolved_data: dict):
        repo = BaseRepository(entity_type)
        
        # 1. Update the first source_id with the resolved data (in place merge)
        # OR create a new entity if we prefer. Let's update the first one.
        primary_id = source_ids[0]
        resolved_data["id"] = primary_id
        
        new_entity = repo.save(resolved_data)
        
        # 2. Hard delete the other original entities since our parquet schema doesn't have is_active
        for src_id in source_ids[1:]:
            repo.delete(src_id)
                
        return new_entity
