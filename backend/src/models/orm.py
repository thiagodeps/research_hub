from sqlalchemy import Column, Integer, String, Boolean
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

class Researcher(Base):
    __tablename__ = "researchers"
    id = Column(Integer, primary_key=True, index=True)
    name = Column(String)
    identification_id = Column(String)
    birthday = Column(String)
    cnpq_url = Column(String)
    google_scholar_url = Column(String)
    resume = Column(String)
    citation_names = Column(String)
    initiatives = Column(String)
    research_groups = Column(String)
    knowledge_areas = Column(String)
    academic_education = Column(String)
    articles = Column(String)
    advisorships = Column(String)
    classification = Column(String)
    classification_confidence = Column(String)
    classification_note = Column(String)
    role_evidence = Column(String)
    was_student = Column(String)
    was_staff = Column(String)
    campus = Column(String)

class Article(Base):
    __tablename__ = "articles"
    id = Column(Integer, primary_key=True, index=True)
    title = Column(String)
    doi = Column(String)
    year = Column(String)
    type = Column(String)
    journal_conference = Column(String)
    volume = Column(String)
    pages = Column(String)
    campus = Column(String)

class ResearchGroup(Base):
    __tablename__ = "research_groups"
    id = Column(Integer, primary_key=True, index=True)
    name = Column(String)
    description = Column(String)
    short_name = Column(String)
    organization_id = Column(String)
    campus_id = Column(String)
    cnpq_url = Column(String)
    site = Column(String)
    organization = Column(String)
    campus = Column(String)
    knowledge_areas = Column(String)
    members = Column(String)
    leaders = Column(String)

class Initiative(Base):
    __tablename__ = "initiatives"
    id = Column(Integer, primary_key=True, index=True)
    name = Column(String)
    status = Column(String)
    description = Column(String)
    start_date = Column(String)
    end_date = Column(String)
    initiative_type_id = Column(String)
    initiative_type = Column(String)
    organization_id = Column(String)
    organization = Column(String)
    parent_id = Column(String)
    team = Column(String)
    demandante = Column(String)
    campus = Column(String)
    research_group = Column(String)
    knowledge_areas = Column(String)
    enrichment = Column(String)
    external_partner = Column(String)
    external_research_group = Column(String)

class Advisorship(Base):
    __tablename__ = "advisorships"
    id = Column(Integer, primary_key=True, index=True)
    name = Column(String)
    status = Column(String)
    description = Column(String)
    start_date = Column(String)
    end_date = Column(String)
    campus = Column(String)
    advisorships = Column(String)
    team = Column(String)

class Award(Base):
    __tablename__ = "awards"
    id = Column(Integer, primary_key=True, index=True)
    researcher_id = Column(String)
    title = Column(String)
    year = Column(String)
    campus = Column(String)

class Student(Base):
    __tablename__ = "students"
    id = Column(Integer, primary_key=True, index=True)
    name = Column(String)
    identification_id = Column(String)
    birthday = Column(String)
    cnpq_url = Column(String)
    google_scholar_url = Column(String)
    resume = Column(String)
    citation_names = Column(String)
    initiatives = Column(String)
    research_groups = Column(String)
    knowledge_areas = Column(String)
    academic_education = Column(String)
    articles = Column(String)
    advisorships = Column(String)
    classification = Column(String)
    classification_confidence = Column(String)
    classification_note = Column(String)
    role_evidence = Column(String)
    was_student = Column(String)
    was_staff = Column(String)
    campus = Column(String)

class Campus(Base):
    __tablename__ = "campuses"
    id = Column(Integer, primary_key=True, index=True)
    name = Column(String)
    description = Column(String)
    short_name = Column(String)
    organization_id = Column(String)
    parent_id = Column(String)
    campus = Column(String)

class Organization(Base):
    __tablename__ = "organizations"
    id = Column(Integer, primary_key=True, index=True)
    name = Column(String)
    description = Column(String)
    short_name = Column(String)
    campus = Column(String)

class Fellowship(Base):
    __tablename__ = "fellowships"
    id = Column(Integer, primary_key=True, index=True)
    name = Column(String)
    description = Column(String)
    value = Column(String)
    campus = Column(String)

class Proficiency(Base):
    __tablename__ = "proficiencies"
    id = Column(Integer, primary_key=True, index=True)
    researcher_id = Column(String)
    language_id = Column(String)
    comprehension = Column(String)
    speaking = Column(String)
    reading = Column(String)
    writing = Column(String)
    campus = Column(String)

class ProfessionalActivity(Base):
    __tablename__ = "professional_activities"
    id = Column(Integer, primary_key=True, index=True)
    researcher_id = Column(String)
    organization_id = Column(String)
    institution = Column(String)
    institution_name = Column(String)
    institution_acronym = Column(String)
    institution_country = Column(String)
    period = Column(String)
    start_year = Column(String)
    end_year = Column(String)
    bond = Column(String)
    classification = Column(String)
    work_regime = Column(String)
    role_function = Column(String)
    activity_type = Column(String)
    current = Column(String)
    campus = Column(String)

class KnowledgeArea(Base):
    __tablename__ = "knowledge_areas"
    id = Column(Integer, primary_key=True, index=True)
    name = Column(String)
    campus = Column(String)

class Language(Base):
    __tablename__ = "languages"
    id = Column(Integer, primary_key=True, index=True)
    name = Column(String)
    campus = Column(String)

class ResearchProduction(Base):
    __tablename__ = "research_productions"
    id = Column(Integer, primary_key=True, index=True)
    title = Column(String)
    year = Column(String)
    production_type_id = Column(String)
    publisher = Column(String)
    isbn = Column(String)
    edition = Column(String)
    book_title = Column(String)
    pages = Column(String)
    version = Column(String)
    platform = Column(String)
    link = Column(String)
    campus = Column(String)

