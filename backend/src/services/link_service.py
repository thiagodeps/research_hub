import json
from src.database.repositories import BaseRepository

class LinkService:
    def link_entities(self, parent_type: str, parent_id: int, child_type: str, child_id: int):
        parent_repo = BaseRepository(parent_type)
        child_repo = BaseRepository(child_type)
        
        parent = parent_repo.get_by_id(parent_id)
        child = child_repo.get_by_id(child_id)
        
        if not parent or not child:
            return False
            
        # Implementation of linking logic for canonical parquet strings
        # Map child_type (from frontend endpoint) to actual DB column names
        column_map = {
            "groups": "research_groups",
            "articles": "articles",
            "initiatives": "initiatives",
            "advisorships": "advisorships"
        }
        
        col_name = column_map.get(child_type, child_type)
        
        # If the parent has a column matching the child
        if col_name in parent:
            current_rels = parent.get(col_name)
            try:
                rels_list = json.loads(current_rels) if current_rels else []
                if not isinstance(rels_list, list):
                    rels_list = []
            except Exception:
                rels_list = []
                
            # Avoid duplicates
            if not any(r.get("id") == child_id for r in rels_list):
                rel_name = child.get("name") or child.get("title") or f"{child_type} {child_id}"
                rels_list.append({"id": child_id, "name": rel_name})
                
                parent[col_name] = json.dumps(rels_list)
                parent_repo.save(parent)
                
        return True
