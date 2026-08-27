from src.database.session import SessionLocal
from src.models.orm import (
    Admin, University, Researcher, Article, ResearchGroup, Initiative, Advisorship, Award,
    Student, Campus, Organization, Fellowship, Proficiency, ProfessionalActivity, KnowledgeArea, Language, ResearchProduction
)

class DatabasePostgresAdapter:
    def __init__(self):
        self.models = {
            "admins": Admin,
            "universities": University,
            "researchers": Researcher,
            "articles": Article,
            "groups": ResearchGroup,
            "initiatives": Initiative,
            "advisorships": Advisorship,
            "awards": Award,
            "students": Student,
            "campuses": Campus,
            "organizations": Organization,
            "fellowships": Fellowship,
            "proficiencies": Proficiency,
            "professional_activities": ProfessionalActivity,
            "knowledge_areas": KnowledgeArea,
            "languages": Language,
            "research_productions": ResearchProduction
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

    def get_all(self, table: str, limit: int = 50, offset: int = 0, search: str = None, sort: str = None, order: str = "asc"):
        model = self._get_model(table)
        session = SessionLocal()
        try:
            query = session.query(model)
            
            if search:
                searchable_col = None
                for col_name in ["name", "title", "username"]:
                    if hasattr(model, col_name):
                        searchable_col = getattr(model, col_name)
                        break
                
                if searchable_col is not None:
                    query = query.filter(searchable_col.ilike(f"%{search}%"))
            
            if sort and hasattr(model, sort):
                from sqlalchemy import func
                sort_col = getattr(model, sort)
                
                # Check if this column is likely a JSON relationship array by its name
                json_columns = ['initiatives', 'research_groups', 'groups', 'knowledge_areas', 'students', 
                                'advisorships', 'articles', 'awards', 'organizations', 'proficiencies', 
                                'fellowships', 'languages', 'professional_activities', 'research_productions']
                
                if sort in json_columns:
                    sort_attr = func.length(sort_col)
                else:
                    sort_attr = sort_col
                
                if order == "desc":
                    query = query.order_by(sort_attr.desc().nulls_last())
                else:
                    query = query.order_by(sort_attr.asc().nulls_last())
            
            total = query.count()
            items = query.offset(offset).limit(limit).all()
            return [self._to_dict(item) for item in items], total
        finally:
            session.close()

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
