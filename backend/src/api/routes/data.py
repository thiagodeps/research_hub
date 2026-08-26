from fastapi import APIRouter, UploadFile, File
from fastapi.responses import StreamingResponse
from src.services.parquet_service import ParquetService

router = APIRouter()

@router.post("/import")
async def import_data(file: UploadFile = File(...)):
    if not file.filename.endswith('.zip'):
        return {"error": "Apenas arquivos .zip contendo .parquet são aceitos."}
    return await ParquetService.import_zip(file)

@router.get("/export")
async def export_data():
    zip_buffer = ParquetService.export_zip()
    return StreamingResponse(
        iter([zip_buffer.getvalue()]),
        media_type="application/zip",
        headers={
            "Content-Disposition": "attachment; filename=portal_export_canonical.zip"
        }
    )
