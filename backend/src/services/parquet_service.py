import os
import zipfile
import io
import pandas as pd
from fastapi import UploadFile
from src.database.session import engine, SessionLocal
from src.models.orm import Base

class ParquetService:
    ORIGINAL_ZIP_PATH = "uploads/original.zip"

    @staticmethod
    def _get_table_name_from_filename(filename: str):
        # Example: 'parquet/articles_canonical.parquet' -> 'articles'
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
        
        # Save original zip to preserve untouched files (like graphs, jsons)
        os.makedirs("uploads", exist_ok=True)
        with open(ParquetService.ORIGINAL_ZIP_PATH, "wb") as f:
            f.write(content)
        
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
        
        overwritten_files = set()
        new_files_data = {}
        
        # 1. Generate new parquet AND json data from DB for mapped tables
        for table_name in Base.metadata.tables.keys():
            if table_name in ["admins", "universities"]: 
                continue # Skip internal UI tables
                
            try:
                df = pd.read_sql_table(table_name, engine)
                
                export_base = f"{table_name}_canonical"
                if table_name == "groups":
                    export_base = "research_groups_canonical"
                
                # Save Parquet
                pq_buffer = io.BytesIO()
                df.to_parquet(pq_buffer, index=False)
                pq_path = f"parquet/{export_base}.parquet"
                overwritten_files.add(pq_path)
                new_files_data[pq_path] = pq_buffer.getvalue()
                
                # Save JSON
                json_buffer = io.BytesIO()
                # Use standard json orient="records"
                df.to_json(json_buffer, orient="records", force_ascii=False)
                json_path = f"{export_base}.json"
                overwritten_files.add(json_path)
                new_files_data[json_path] = json_buffer.getvalue()
                
            except Exception as e:
                print(f"Error exporting {table_name}: {e}")

        # 2. Write to the new zip file
        with zipfile.ZipFile(zip_buffer, "w", zipfile.ZIP_DEFLATED) as z:
            # First, if we have an original zip, copy over all files we didn't overwrite
            if os.path.exists(ParquetService.ORIGINAL_ZIP_PATH):
                with zipfile.ZipFile(ParquetService.ORIGINAL_ZIP_PATH, "r") as oz:
                    for item in oz.infolist():
                        if item.filename not in overwritten_files:
                            z.writestr(item, oz.read(item.filename))
            
            # Now write all our newly updated parquet files
            for filename, data in new_files_data.items():
                z.writestr(filename, data)
                
        zip_buffer.seek(0)
        return zip_buffer
