import os

class DatabaseMemoryAdapter:
    def __init__(self):
        self._data = {}

    def get(self, table: str, record_id: int):
        return self._data.get(table, {}).get(record_id)

    def save(self, table: str, record: dict):
        if table not in self._data:
            self._data[table] = {}
        if "id" not in record:
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
        else:
            raise NotImplementedError("Postgres adapter not implemented yet")
    return _db_instance
