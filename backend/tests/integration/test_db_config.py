import pytest
import os
from src.database.core import get_db, DatabaseMemoryAdapter

def test_database_memory_adapter_singleton():
    """Test if we can get a singleton memory adapter when STORAGE_TYPE=memory"""
    os.environ["STORAGE_TYPE"] = "memory"
    
    db_instance1 = get_db()
    db_instance2 = get_db()
    
    assert isinstance(db_instance1, DatabaseMemoryAdapter)
    assert db_instance1 is db_instance2
    
    # Test simple insert and get to prove memory works
    db_instance1.save("users", {"id": 1, "name": "Admin"})
    result = db_instance2.get("users", 1)
    assert result == {"id": 1, "name": "Admin"}
