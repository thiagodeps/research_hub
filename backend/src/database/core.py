import os

class DatabaseMemoryAdapter:
    def __init__(self):
        self._data = {}

    def get(self, table: str, record_id):
        return self._data.get(table, {}).get(int(record_id) if str(record_id).isdigit() else record_id)

    def save(self, table: str, record: dict):
        if table not in self._data:
            self._data[table] = {}
        
        if "id" in record and str(record["id"]).isdigit():
            record["id"] = int(record["id"])
            
        if "id" not in record or not record["id"]:
            new_id = len(self._data[table]) + 1
            while new_id in self._data[table]:
                new_id += 1
            record["id"] = new_id
            
        self._data[table][record["id"]] = record
        return record

_db_instance = None

def get_db():
    global _db_instance
    if _db_instance is None:
        storage_type = os.environ.get("STORAGE_TYPE", "memory")
        if storage_type == "memory":
            _db_instance = DatabaseMemoryAdapter()
            # SEED DEFAULT ADMIN
            from src.core.security import get_password_hash
            _db_instance.save("admins", {
                "id": 1,
                "email": "admin@admin.com",
                "password_hash": get_password_hash("admin123")
            })
        else:
            raise NotImplementedError("Postgres adapter not implemented yet")
    return _db_instance
