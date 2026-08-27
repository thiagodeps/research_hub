# Feature Specification: Bugfixes for Campus and Research Groups (009)

## 1. Description
- **Bug 1:** The `campus` column in the data tables displays a raw JSON object string (e.g. `{"id": 1, "name": "..."}`) because the formatter only handles JSON Arrays.
- **Bug 2:** The `groups` table (Research Groups) is completely empty in the UI because the Zip Importer was incorrectly mapping the parquet file name `research_groups` to `groups`, which caused a mismatch against the ORM table name (`research_groups`) during the drop/create/insert cycle.

## 2. Requirements
- The `EntityTable` and `EntityForm` components must safely parse single JSON objects and display their `name` or `title` fields instead of raw JSON.
- The `parquet_service.py` mapping must map the parquet file to `research_groups` to match the SQLAlchemy table metadata, allowing the ZIP import pipeline to correctly populate the table.

