import sys
import os
import asyncio
import io
import zipfile
import pandas as pd
from fastapi import UploadFile

# Add backend to path
sys.path.insert(0, os.path.abspath('backend'))
from src.services.parquet_service import ParquetService

async def run():
    print("Simulating upload...")
    with open('../exports_canonical.zip', 'rb') as f:
        file_obj = UploadFile(filename='../exports_canonical.zip', file=f)
        await ParquetService.import_zip(file_obj)
    print("Import complete.")

    print("Exporting...")
    zip_buffer = ParquetService.export_zip()
    print("Export complete.")

    with zipfile.ZipFile(zip_buffer, 'r') as z:
        for fname in ['researchers_canonical.json', 'parquet/researchers_canonical.parquet']:
            if fname in z.namelist():
                print(f"Exported {fname} size: {z.getinfo(fname).file_size}")
            else:
                print(f"Missing {fname} in exported zip!")

asyncio.run(run())
