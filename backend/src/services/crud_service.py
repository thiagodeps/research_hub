from src.database.repositories import BaseRepository

class CrudService:
    def __init__(self, entity_name: str):
        self.repo = BaseRepository(entity_name)

    def list_all(self, limit: int = 100, offset: int = 0, search: str = None, sort: str = None, order: str = "asc"):
        return self.repo.get_all(limit, offset, search=search, sort=sort, order=order)

    def get_one(self, entity_id: int):
        return self.repo.get_by_id(entity_id)

    def create(self, data: dict):
        return self.repo.create(data)

    def update(self, entity_id: int, data: dict):
        return self.repo.update(entity_id, data)

    def delete(self, entity_id: int):
        # Here we would implement Edge Case checking for dependent entities (Cascade block)
        return self.repo.delete(entity_id)
