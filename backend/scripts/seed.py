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
        uni_prefixes = ["Universidade Federal de", "Universidade Estadual de", "Instituto Federal do", "Pontifícia Universidade Católica de", "Universidade de"]
        states = ["São Paulo", "Minas Gerais", "Rio de Janeiro", "Bahia", "Paraná", "Rio Grande do Sul", "Pernambuco", "Ceará", "Pará", "Santa Catarina", "Goiás", "Maranhão", "Amazonas", "Espírito Santo", "Paraíba", "Mato Grosso", "Rio Grande do Norte", "Alagoas", "Piauí", "Distrito Federal", "Mato Grosso do Sul", "Sergipe", "Rondônia", "Tocantins", "Acre", "Amapá", "Roraima"]
        
        for _ in range(5):
            state = fake.random_element(elements=states)
            prefix = fake.random_element(elements=uni_prefixes)
            name = f"{prefix} {state}"
            
            # Create a realistic abbreviation based on prefix and state
            abbr = "".join([word[0] for word in name.split() if len(word) > 2]).upper()
            if prefix.startswith("Instituto"):
                abbr = "IF" + state[:2].upper()
            elif prefix.startswith("Universidade Federal"):
                abbr = "UF" + state[:2].upper()
                
            uni = University(
                name=name,
                abbreviation=abbr
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
