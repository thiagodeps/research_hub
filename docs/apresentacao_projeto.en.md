---
marp: true
theme: default
class: lead
paginate: true
backgroundColor: #ffffff
---

# Research Hub 🧬
## Academic Data Curation

*[Thiago Almeida Deps Caldeira]*


---

# What is Research Hub?

**Research Hub** is a **desktop** tool aimed at academic data curation.

- Receives the `exports_canonical.zip` package from the DataLake.
- Allows correcting, merging, and linking records.
- Exports the package back, preserving unmanaged files intact.

<!-- 
Note for editing: Add an image below showing the home screen or general application flow.
![System Overview](path_to_image.png)
-->

---

# Architecture

**Single-process** application: no external server, no dependent browser, no Python, no network.

- **Core:** Rust + Tauri 2.0 (Manages window and IPC communication)
- **Database:** SQLite (`rusqlite`, embedded in a single file)
- **Data Manipulation:** `arrow-rs` / `polars` for reading and writing `.parquet`
- **Front-end:** Astro + React + Tailwind (Statically compiled and embedded in the binary)

<!-- 
Note for editing: An architecture diagram image would fit very well here.
![Architecture Diagram](path_to_image.png)
-->

---

# Evolution (Web ➡️ Desktop Migration)

Why did we stop being a web application (Python + FastAPI)?

- **Distribution Simplicity:** No more need to manage `venv`, multiple processes, and environment variables.
- **Efficiency:** Elimination of the entire HTTP layer, CORS, multipart upload.
- **User Focus:** The usage profile is a single-user local desktop application, so the architecture must reflect this (local database and offline operation).

---

# Curator Workflow

1. **Sign in:** Single local login (`admin@admin.com`).
2. **Import:** Selection or *drag-and-drop* of `exports_canonical.zip`. The previous database receives automatic backup.
3. **Curate:** Search, edit, merge duplicates, and create links.
4. **Export:** Generation of the final ZIP with restored data types and unmanaged files kept bit-by-bit.

<!-- 
Note for editing: Insert screenshots of the "Curate" flow (tables and editing).
![Curation Screen](path_to_image.png)
-->

---

# Data Domains (Entities)

The system supports 15 mapped academic entities:
- Researchers, Students, Research Groups
- Initiatives, Awards, Scientific Productions
- Knowledge Areas, Advisees, Organizations
- Professional Activities, Campus, Proficiencies
- Scholarships, Languages, and Articles

*Associative tables are not used; relationships are maintained in JSON array columns, facilitating consumption in Big Data pipelines.*

---

# Distribution and Installation

Native binaries for major operating systems, dispensing with prerequisites like Node or Python:

- **Linux:** `.deb`, `.rpm`, `.AppImage`
- **Windows:** `.exe` (MSI/NSIS)

All persistence occurs directly in the OS data directory (`~/.local/share` or `%APPDATA%`), ensuring trivial backup (just copy a file).

---

# Conclusion and Next Steps

**Research Hub** consolidates the data curation work in a focused, fast, and self-contained distribution environment.

* **Greater robustness in data ingestion.**
* **Native and offline experience.**
* **Ease of maintenance and distribution.**

## Thank you!
Questions?
