# Data Model Updates

```mermaid
erDiagram
    ADMIN {
        Integer id PK
        String email
        String password_hash
    }
    UNIVERSITY {
        Integer id PK
        String name
        String abbreviation
    }
    RESEARCHER {
        Integer id PK
        String name
        String email
        String lattes_id
        String orcid
    }
```
