from sqlalchemy.orm import declarative_base
from sqlalchemy import Column, Integer, String

Base = declarative_base()

class Admin(Base):
    __tablename__ = "admins"
    id = Column(Integer, primary_key=True, index=True)
    email = Column(String, unique=True, index=True, nullable=False)
    password_hash = Column(String, nullable=False)

class University(Base):
    __tablename__ = "universities"
    id = Column(Integer, primary_key=True, index=True)
    name = Column(String, index=True, nullable=False)
    abbreviation = Column(String, nullable=False)

class Researcher(Base):
    __tablename__ = "researchers"
    id = Column(Integer, primary_key=True, index=True)
    name = Column(String, index=True)
    identification_id = Column(Integer)
    cnpq_url = Column(String)
    classification = Column(String)
    initiatives = Column(String)
    research_groups = Column(String)
    articles = Column(String)
    advisorships = Column(String)
    # kept for backwards compatibility with UI if needed
    email = Column(String, nullable=True)
    lattes_id = Column(String, nullable=True)
    orcid = Column(String, nullable=True)

class Article(Base):
    __tablename__ = "articles"
    id = Column(Integer, primary_key=True, index=True)
    title = Column(String)
    doi = Column(String)
    year = Column(Integer)
    type = Column(String)
    journal_conference = Column(String)

class ResearchGroup(Base):
    __tablename__ = "research_groups"
    id = Column(Integer, primary_key=True, index=True)
    name = Column(String)
    description = Column(String)
    short_name = Column(String)
    cnpq_url = Column(String)

class Initiative(Base):
    __tablename__ = "initiatives"
    id = Column(Integer, primary_key=True, index=True)
    name = Column(String)
    status = Column(String)
    description = Column(String)
    start_date = Column(String)

class Advisorship(Base):
    __tablename__ = "advisorships"
    id = Column(Integer, primary_key=True, index=True)
    name = Column(String)
    status = Column(String)
    description = Column(String)
    start_date = Column(String)

class Award(Base):
    __tablename__ = "awards"
    id = Column(Integer, primary_key=True, index=True)
    title = Column(String)
    year = Column(Integer)

class Student(Base):
    __tablename__ = "students"
    id = Column(Integer, primary_key=True, index=True)
    name = Column(String)

class Campus(Base):
    __tablename__ = "campuses"
    id = Column(Integer, primary_key=True, index=True)
    name = Column(String)

class Organization(Base):
    __tablename__ = "organizations"
    id = Column(Integer, primary_key=True, index=True)
    name = Column(String)

class Fellowship(Base):
    __tablename__ = "fellowships"
    id = Column(Integer, primary_key=True, index=True)
    name = Column(String)

class Proficiency(Base):
    __tablename__ = "proficiencies"
    id = Column(Integer, primary_key=True, index=True)
    name = Column(String)

class ProfessionalActivity(Base):
    __tablename__ = "professional_activities"
    id = Column(Integer, primary_key=True, index=True)
    name = Column(String)

class KnowledgeArea(Base):
    __tablename__ = "knowledge_areas"
    id = Column(Integer, primary_key=True, index=True)
    name = Column(String)

class Language(Base):
    __tablename__ = "languages"
    id = Column(Integer, primary_key=True, index=True)
    name = Column(String)

class ResearchProduction(Base):
    __tablename__ = "research_productions"
    id = Column(Integer, primary_key=True, index=True)
    title = Column(String)

