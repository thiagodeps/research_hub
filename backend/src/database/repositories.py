from src.database.core import get_db

class BaseRepository:
    def __init__(self, table_name: str):
        self.table_name = table_name

    def get_all(self, limit: int = 100, offset: int = 0, search: str = None):
        db = get_db()
        return db.get_all(self.table_name, limit, offset, search=search)

    def get_by_id(self, record_id: int):
        db = get_db()
        return db.get(self.table_name, record_id)

    def create(self, data: dict):
        db = get_db()
        return db.save(self.table_name, data)

    def update(self, record_id: int, data: dict):
        db = get_db()
        record = db.get(self.table_name, record_id)
        if record:
            # We construct a new dict to pass to save so that we don't mutate memory directly if it's not ORM
            merged = {**record, **data, "id": record_id}
            return db.save(self.table_name, merged)
        return record

    def delete(self, record_id: int):
        db = get_db()
        return db.delete(self.table_name, record_id)
