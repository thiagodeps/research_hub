from src.database.session import SessionLocal, engine
from src.models.orm import Base, Admin
from src.core.security import get_password_hash

# Create tables
Base.metadata.create_all(bind=engine)

db = SessionLocal()
admin = db.query(Admin).filter_by(username="admin@admin.com").first()
if not admin:
    admin = Admin(
        username="admin@admin.com", 
        hashed_password=get_password_hash("admin123")
    )
    db.add(admin)
    db.commit()
    print("Admin created successfully.")
else:
    print("Admin already exists.")
