PYTHON  := python3
VENV    := backend/venv
PIP     := $(VENV)/bin/pip

.PHONY: build build-backend build-frontend run run-backend run-frontend clean

build: build-backend build-frontend

build-backend:
	test -d $(VENV) || $(PYTHON) -m venv $(VENV)
	$(PIP) install --upgrade pip
	$(PIP) install -r backend/requirements.txt

build-frontend:
	cd frontend && npm install

run:
	@trap 'kill 0' EXIT INT TERM; \
	$(MAKE) run-backend & \
	$(MAKE) run-frontend & \
	wait

run-backend:
	cd backend && STORAGE_TYPE=postgres venv/bin/uvicorn src.api.main:app --reload --port 8000

run-frontend:
	cd frontend && npm run dev

clean:
	rm -rf $(VENV) frontend/node_modules
