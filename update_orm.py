import re

content = open('backend/schema_dump.txt').read()
tables = {}
current_table = None

for line in content.split('\n'):
    line = line.strip()
    if not line: continue
    m = re.match(r'--- (.*)_canonical.parquet ---', line)
    if m:
        current_table = m.group(1)
        tables[current_table] = []
        continue
    
    if current_table and 'dtype:' not in line and line != '--':
        parts = line.split()
        if len(parts) >= 2:
            col_name = parts[0]
            tables[current_table].append(col_name)

# Map our class names to table names
class_map = {
    "Admin": "admins",
    "University": "universities",
    "Researcher": "researchers",
    "Article": "articles",
    "ResearchGroup": "research_groups",
    "Initiative": "initiatives",
    "Advisorship": "advisorships",
    "Award": "awards",
    "Student": "students",
    "Campus": "campuses",
    "Organization": "organizations",
    "Fellowship": "fellowships",
    "Proficiency": "proficiencies",
    "ProfessionalActivity": "professional_activities",
    "KnowledgeArea": "knowledge_areas",
    "Language": "languages",
    "ResearchProduction": "research_productions"
}

# The header of orm.py
orm_content = """from sqlalchemy import Column, Integer, String, Boolean
from sqlalchemy.ext.declarative import declarative_base

Base = declarative_base()

class Admin(Base):
    __tablename__ = "admins"
    id = Column(Integer, primary_key=True, index=True)
    username = Column(String, unique=True, index=True)
    hashed_password = Column(String)

class University(Base):
    __tablename__ = "universities"
    id = Column(Integer, primary_key=True, index=True)
    name = Column(String, index=True)
    location = Column(String)

"""

for cls_name, table_name in class_map.items():
    if table_name in ["admins", "universities"]: continue
    
    orm_content += f"class {cls_name}(Base):\n"
    orm_content += f'    __tablename__ = "{table_name}"\n'
    
    if table_name in tables:
        for col in tables[table_name]:
            if col == 'id':
                orm_content += f"    id = Column(Integer, primary_key=True, index=True)\n"
            else:
                orm_content += f"    {col} = Column(String)\n"
    else:
        orm_content += f"    id = Column(Integer, primary_key=True, index=True)\n"
        orm_content += f"    name = Column(String)\n"
    
    orm_content += "\n"

with open('backend/src/models/orm.py', 'w') as f:
    f.write(orm_content)

print("orm.py updated successfully!")
