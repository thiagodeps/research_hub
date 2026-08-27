from src.database.core import get_db
from src.core.security import verify_password
from src.models.admin import AdminUser
from src.database.session import SessionLocal
from src.models.orm import Admin

def authenticate_admin(email: str, password: str) -> AdminUser | None:
    db = get_db()
    # In a real app we'd query by email. Our memory adapter stores by ID, so we iterate.
    if hasattr(db, "_data"):
        users = db._data.get("admins", {})
        for u in users.values():
            if (u.get("email") == email or u.get("username") == email) and verify_password(password, u.get("password_hash") or u.get("hashed_password")):
                return AdminUser(**u)
        return None
    
    # Postgres/SQLite query
    session = SessionLocal()
    try:
        admin = session.query(Admin).filter_by(username=email).first()
        if admin and verify_password(password, admin.hashed_password):
            return AdminUser(id=admin.id, email=admin.username, password_hash=admin.hashed_password)
        return None
    finally:
        session.close()
