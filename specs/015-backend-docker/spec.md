# Feature Specification: Backend Dockerization (015)

## 1. Description
The user wants to run the backend API using Docker to facilitate local execution, isolation, and future cloud deployment. This feature containerizes the FastAPI backend.

## 2. Requirements
- Create a `Dockerfile` in the `backend` folder configured to run the FastAPI app using Uvicorn.
- Create a `docker-compose.yml` file in the root of the project to allow the user to easily spin up the backend via `docker-compose up`.
- Configure volume mapping in the docker-compose file so that `test.db` and uploaded ZIP files persist locally instead of being wiped when the container stops.

## 3. Acceptance Criteria
- [ ] A `Dockerfile` exists in the `backend` directory.
- [ ] A `docker-compose.yml` exists at the root.
- [ ] The backend runs correctly via Docker.
