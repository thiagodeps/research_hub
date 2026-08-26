import sys
import os

# Append backend directory to path to allow absolute imports
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from src.database.session import engine, SessionLocal
from src.models.orm import Base, Admin, University, Researcher
from src.core.security import get_password_hash
from faker import Faker

def run_seed():
    print("Recreating database tables...")
    Base.metadata.drop_all(bind=engine)
    Base.metadata.create_all(bind=engine)
    
    fake = Faker('pt_BR')
    
    with SessionLocal() as session:
        print("Seeding Admin...")
        admin = Admin(email="admin@admin.com", password_hash=get_password_hash("admin123"))
        session.add(admin)
        
        print("Seeding Universities...")
        for _ in range(5):
            uni = University(
                name=fake.company(),
                abbreviation=fake.company_suffix().upper()
            )
            session.add(uni)
            
        print("Seeding Researchers...")
        for _ in range(20):
            res = Researcher(
                name=fake.name(),
                email=fake.unique.email(),
                lattes_id=fake.numerify(text='###############'),
                orcid=f"{fake.numerify(text='####')}-{fake.numerify(text='####')}-{fake.numerify(text='####')}-{fake.numerify(text='####')}"
            )
            session.add(res)
            
        session.commit()
        print("Database seeded successfully!")

if __name__ == "__main__":
    run_seed()
