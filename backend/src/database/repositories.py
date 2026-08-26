from src.database.core import get_db

class BaseRepository:
    def __init__(self, table_name: str):
        self.table_name = table_name

    def get_all(self):
        db = get_db()
        return list(db._data.get(self.table_name, {}).values())

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
            record.update(data)
            db.save(self.table_name, record)
        return record

    def delete(self, record_id: int):
        db = get_db()
        key = int(record_id) if str(record_id).isdigit() else record_id
        # In memory deletion
        if self.table_name in db._data and key in db._data[self.table_name]:
            del db._data[self.table_name][key]
            return True
        return False
