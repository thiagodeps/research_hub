# Implementation Plan: Backend Dockerization (015)

## 1. Dockerfile
Create `backend/Dockerfile`:
- Use `python:3.12-slim`.
- Set working directory to `/app`.
- Copy `requirements.txt` and run `pip install`.
- Copy the `src` folder.
- Expose port `8000`.
- Run `uvicorn src.api.main:app --host 0.0.0.0 --port 8000`.

## 2. Docker Compose
Create `./docker-compose.yml`:
- Define a `backend` service.
- Build from `./backend`.
- Map port `8000:8000`.
- Mount volumes for `./backend/test.db:/app/test.db` and `./backend/uploads:/app/uploads`.
- Set `STORAGE_TYPE=postgres`.
