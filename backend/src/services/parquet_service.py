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
        
        # We don't map research_groups to groups here because Base.metadata.tables uses 'research_groups'
        return base

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
                
                pq_path = f"parquet/{export_base}.parquet"
                json_path = f"{export_base}.json"
                
                # RECOVER ORIGINAL TYPES AND STRUCTURE
                import json
                if os.path.exists(ParquetService.ORIGINAL_ZIP_PATH):
                    with zipfile.ZipFile(ParquetService.ORIGINAL_ZIP_PATH, "r") as oz:
                        if pq_path in oz.namelist():
                            with oz.open(pq_path) as pf:
                                df_orig = pd.read_parquet(pf)
                                
                                # Align columns and types
                                for col in df_orig.columns:
                                    if col not in df.columns:
                                        continue
                                        
                                    # If original is boolean
                                    if pd.api.types.is_bool_dtype(df_orig[col]):
                                        df[col] = df[col].map({'1': True, '0': False, 'True': True, 'False': False, 1: True, 0: False, True: True, False: False})
                                    
                                    # Numeric types
                                    elif pd.api.types.is_numeric_dtype(df_orig[col]):
                                        df[col] = pd.to_numeric(df[col], errors='coerce')
                                        df[col] = df[col].astype(df_orig[col].dtype)
                                
                                # Reorder columns exactly like the original schema
                                ordered_cols = [c for c in df_orig.columns if c in df.columns]
                                df = df[ordered_cols]
                
                # Save Parquet (keep strings as strings)
                pq_buffer = io.BytesIO()
                df.to_parquet(pq_buffer, index=False)
                overwritten_files.add(pq_path)
                new_files_data[pq_path] = pq_buffer.getvalue()
                
                # Prepare DataFrame for JSON (parse JSON strings into actual objects)
                import json
                df_json = df.copy()
                for col in df_json.columns:
                    # Try parsing strings that look like JSON arrays or objects
                    def try_parse_json(val):
                        if isinstance(val, str) and (val.startswith('[') or val.startswith('{')):
                            try:
                                return json.loads(val)
                            except:
                                pass
                        return val
                    df_json[col] = df_json[col].apply(try_parse_json)
                
                # Save JSON
                json_buffer = io.BytesIO()
                # Replace pandas NaN/NaT with None so json.dumps outputs standard 'null' instead of 'NaN'
                df_clean = df_json.astype(object).where(pd.notnull(df_json), None)
                json_str = json.dumps(df_clean.to_dict(orient="records"), ensure_ascii=False, indent=4)
                json_buffer.write(json_str.encode('utf-8'))
                
                overwritten_files.add(json_path)
                new_files_data[json_path] = json_buffer.getvalue()
                
            except Exception as e:
                import traceback
                print(f"Error exporting {table_name}: {e}")
                traceback.print_exc()

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
