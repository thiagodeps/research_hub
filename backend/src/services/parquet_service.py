import os
import zipfile
import io
import pandas as pd
from fastapi import UploadFile
from src.database.session import engine, SessionLocal
from src.models.orm import Base

class ParquetService:
    @staticmethod
    def _get_table_name_from_filename(filename: str):
        # Example: 'articles_canonical.parquet' -> 'articles'
        # 'research_groups_canonical.parquet' -> 'research_groups'
        base = os.path.basename(filename).replace(".parquet", "")
        if base.endswith("_canonical"):
            base = base.replace("_canonical", "")
        
        # map to our table names
        table_map = {
            "research_groups": "groups"
        }
        return table_map.get(base, base)

    @staticmethod
    async def import_zip(file: UploadFile):
        content = await file.read()
        
        # Recreate tables to wipe old data before import
        Base.metadata.drop_all(bind=engine)
        Base.metadata.create_all(bind=engine)
        
        # Only process if valid zip
        with zipfile.ZipFile(io.BytesIO(content)) as z:
            parquet_files = [f for f in z.namelist() if f.endswith('.parquet')]
            
            # For each file, try to map to our DB tables
            for pfile in parquet_files:
                table_name = ParquetService._get_table_name_from_filename(pfile)
                # Check if this table actually exists in our ORM
                if table_name in Base.metadata.tables:
                    with z.open(pfile) as pf:
                        df = pd.read_parquet(pf)
                        # We only want columns that exist in our ORM for this table
                        orm_columns = [c.name for c in Base.metadata.tables[table_name].columns]
                        # Filter dataframe to only include columns that exist in our DB
                        df_filtered = df[[col for col in df.columns if col in orm_columns]]
                        
                        # Use pandas to_sql to insert efficiently
                        df_filtered.to_sql(table_name, engine, if_exists='append', index=False)
                        
        return {"status": "success", "message": "Dados importados com sucesso."}

    @staticmethod
    def export_zip() -> io.BytesIO:
        zip_buffer = io.BytesIO()
        
        with zipfile.ZipFile(zip_buffer, "w", zipfile.ZIP_DEFLATED) as z:
            # For every table in our database, export to parquet
            for table_name in Base.metadata.tables.keys():
                df = pd.read_sql_table(table_name, engine)
                # We save to parquet in memory
                pq_buffer = io.BytesIO()
                df.to_parquet(pq_buffer, index=False)
                # Write to zip
                # Standardize name back to _canonical for the external pipeline
                export_name = f"{table_name}_canonical.parquet"
                if table_name == "groups":
                    export_name = "research_groups_canonical.parquet"
                
                z.writestr(f"parquet/{export_name}", pq_buffer.getvalue())
                
        zip_buffer.seek(0)
        return zip_buffer
