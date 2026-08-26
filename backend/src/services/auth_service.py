from src.database.core import get_db
from src.core.security import verify_password
from src.models.admin import AdminUser

def authenticate_admin(email: str, password: str) -> AdminUser | None:
    db = get_db()
    # In a real app we'd query by email. Our memory adapter stores by ID, so we iterate.
    # In postgres it would be `SELECT * FROM admins WHERE email = :email`
    if hasattr(db, "_data"):
        users = db._data.get("admins", {})
        for u in users.values():
            if u["email"] == email and verify_password(password, u["password_hash"]):
                return AdminUser(**u)
    return None
