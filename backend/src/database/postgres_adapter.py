from src.database.session import SessionLocal
from src.models.orm import Admin, University, Researcher

class DatabasePostgresAdapter:
    def __init__(self):
        self.models = {
            "admins": Admin,
            "universities": University,
            "researchers": Researcher
        }

    def _get_model(self, table: str):
        model = self.models.get(table)
        if not model:
            raise ValueError(f"Table {table} not mapped in ORM.")
        return model

    def _to_dict(self, obj):
        if obj is None:
            return None
        return {c.name: getattr(obj, c.name) for c in obj.__table__.columns}

    def get(self, table: str, record_id: int):
        model = self._get_model(table)
        with SessionLocal() as session:
            key = int(record_id) if str(record_id).isdigit() else record_id
            obj = session.query(model).filter(model.id == key).first()
            return self._to_dict(obj)

    def get_all(self, table: str):
        model = self._get_model(table)
        with SessionLocal() as session:
            objs = session.query(model).all()
            return [self._to_dict(obj) for obj in objs]

    def save(self, table: str, record: dict):
        model = self._get_model(table)
        with SessionLocal() as session:
            if "id" in record and record["id"]:
                # Try to find existing for update
                key = int(record["id"]) if str(record["id"]).isdigit() else record["id"]
                obj = session.query(model).filter(model.id == key).first()
                if obj:
                    for k, v in record.items():
                        setattr(obj, k, v)
                else:
                    # If ID is provided but doesn't exist, we force insert it (not common but for completeness)
                    obj = model(**record)
                    session.add(obj)
            else:
                # Normal insert
                # Remove empty id if it came empty from frontend
                record_data = {k: v for k, v in record.items() if k != "id"}
                obj = model(**record_data)
                session.add(obj)
                
            session.commit()
            session.refresh(obj)
            return self._to_dict(obj)

    def delete(self, table: str, record_id: int):
        model = self._get_model(table)
        with SessionLocal() as session:
            key = int(record_id) if str(record_id).isdigit() else record_id
            obj = session.query(model).filter(model.id == key).first()
            if obj:
                session.delete(obj)
                session.commit()
                return True
            return False
