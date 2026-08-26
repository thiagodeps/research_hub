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
    name = Column(String, index=True, nullable=False)
    email = Column(String, unique=True, index=True, nullable=False)
    lattes_id = Column(String, unique=True)
    orcid = Column(String, unique=True)

class Article(Base):
    __tablename__ = "articles"
    id = Column(Integer, primary_key=True, index=True)
    title = Column(String, nullable=False)
    doi = Column(String)
    year = Column(Integer)
    type = Column(String)
    journal_conference = Column(String)

class ResearchGroup(Base):
    __tablename__ = "research_groups"
    id = Column(Integer, primary_key=True, index=True)
    name = Column(String, nullable=False)
    description = Column(String)
    short_name = Column(String)
    cnpq_url = Column(String)

class Initiative(Base):
    __tablename__ = "initiatives"
    id = Column(Integer, primary_key=True, index=True)
    name = Column(String, nullable=False)
    status = Column(String)
    description = Column(String)
    start_date = Column(String)

class Advisorship(Base):
    __tablename__ = "advisorships"
    id = Column(Integer, primary_key=True, index=True)
    name = Column(String, nullable=False)
    status = Column(String)
    description = Column(String)
    start_date = Column(String)

class Award(Base):
    __tablename__ = "awards"
    id = Column(Integer, primary_key=True, index=True)
    title = Column(String, nullable=False)
    year = Column(Integer)

