//! Single source of truth for entity metadata (SEP-016, FR-006).
//!
//! Rust has no runtime reflection, so the Python trick of walking SQLAlchemy
//! metadata has no equivalent. Declaring it here also collapses a mapping that
//! is currently duplicated in three places in the Python/JS codebase — the
//! `postgres_adapter` dict, `link_service.column_map` and `EntityForm.getRoute`
//! — which is how `groups` vs `research_groups` drifts.

use crate::error::AppError;

pub struct EntityDef {
    /// What the frontend calls it (`/api/v1/groups` today, `invoke` arg tomorrow).
    pub route: &'static str,
    /// What SQLite and the canonical parquet call it.
    pub table: &'static str,
    pub columns: &'static [&'static str],
    /// Column used by free-text search, mirroring the Python name/title/username probe.
    pub search_column: Option<&'static str>,
    /// False for internal tables that never reach the canonical archive.
    pub exported: bool,
}

/// Columns sorted by `length()` instead of by value.
///
/// These hold JSON arrays of relationships, so ordering by the serialized text
/// would be meaningless; ordering by length approximates "how many links".
/// Verbatim from `DatabasePostgresAdapter.get_all` — a global name list, not a
/// per-entity one, and kept global for parity.
pub const SORT_BY_LENGTH: &[&str] = &[
    "initiatives",
    "research_groups",
    "groups",
    "knowledge_areas",
    "students",
    "advisorships",
    "articles",
    "awards",
    "organizations",
    "proficiencies",
    "fellowships",
    "languages",
    "professional_activities",
    "research_productions",
];

const PERSON_COLUMNS: &[&str] = &[
    "id", "name", "identification_id", "birthday", "cnpq_url", "google_scholar_url",
    "resume", "citation_names", "initiatives", "research_groups", "knowledge_areas",
    "academic_education", "articles", "advisorships", "classification",
    "classification_confidence", "classification_note", "role_evidence",
    "was_student", "was_staff", "campus",
];

pub const ENTITIES: &[EntityDef] = &[
    EntityDef {
        route: "admins",
        table: "admins",
        columns: &["id", "username", "hashed_password"],
        search_column: Some("username"),
        exported: false,
    },
    EntityDef {
        route: "researchers",
        table: "researchers",
        columns: PERSON_COLUMNS,
        search_column: Some("name"),
        exported: true,
    },
    EntityDef {
        route: "students",
        table: "students",
        columns: PERSON_COLUMNS,
        search_column: Some("name"),
        exported: true,
    },
    EntityDef {
        route: "articles",
        table: "articles",
        columns: &[
            "id", "title", "doi", "year", "type", "journal_conference", "volume",
            "pages", "campus",
        ],
        search_column: Some("title"),
        exported: true,
    },
    // The one place where route and table diverge. Centralized here on purpose.
    EntityDef {
        route: "groups",
        table: "research_groups",
        columns: &[
            "id", "name", "description", "short_name", "organization_id", "campus_id",
            "cnpq_url", "site", "organization", "campus", "knowledge_areas", "members",
            "leaders",
        ],
        search_column: Some("name"),
        exported: true,
    },
    EntityDef {
        route: "initiatives",
        table: "initiatives",
        columns: &[
            "id", "name", "status", "description", "start_date", "end_date",
            "initiative_type_id", "initiative_type", "organization_id", "organization",
            "parent_id", "team", "demandante", "campus", "research_group",
            "knowledge_areas", "enrichment", "external_partner",
            "external_research_group",
        ],
        search_column: Some("name"),
        exported: true,
    },
    EntityDef {
        route: "advisorships",
        table: "advisorships",
        columns: &[
            "id", "name", "status", "description", "start_date", "end_date", "campus",
            "advisorships", "team",
        ],
        search_column: Some("name"),
        exported: true,
    },
    EntityDef {
        route: "awards",
        table: "awards",
        columns: &["id", "researcher_id", "title", "year", "campus"],
        search_column: Some("title"),
        exported: true,
    },
    EntityDef {
        route: "campuses",
        table: "campuses",
        columns: &[
            "id", "name", "description", "short_name", "organization_id", "parent_id",
            "campus",
        ],
        search_column: Some("name"),
        exported: true,
    },
    EntityDef {
        route: "organizations",
        table: "organizations",
        columns: &["id", "name", "description", "short_name", "campus"],
        search_column: Some("name"),
        exported: true,
    },
    EntityDef {
        route: "fellowships",
        table: "fellowships",
        columns: &["id", "name", "description", "value", "campus"],
        search_column: Some("name"),
        exported: true,
    },
    EntityDef {
        route: "proficiencies",
        table: "proficiencies",
        columns: &[
            "id", "researcher_id", "language_id", "comprehension", "speaking",
            "reading", "writing", "campus",
        ],
        // No name/title/username column: search is a no-op here, as in Python.
        search_column: None,
        exported: true,
    },
    EntityDef {
        route: "professional_activities",
        table: "professional_activities",
        columns: &[
            "id", "researcher_id", "organization_id", "institution", "institution_name",
            "institution_acronym", "institution_country", "period", "start_year",
            "end_year", "bond", "classification", "work_regime", "role_function",
            "activity_type", "current", "campus",
        ],
        // The Python probe only looks for name/title/username, so this column
        // was never searchable and the search box sat dead on the page.
        search_column: Some("institution_name"),
        exported: true,
    },
    EntityDef {
        route: "knowledge_areas",
        table: "knowledge_areas",
        columns: &["id", "name", "campus"],
        search_column: Some("name"),
        exported: true,
    },
    EntityDef {
        route: "languages",
        table: "languages",
        columns: &["id", "name", "campus"],
        search_column: Some("name"),
        exported: true,
    },
    EntityDef {
        route: "research_productions",
        table: "research_productions",
        columns: &[
            "id", "title", "year", "production_type_id", "publisher", "isbn", "edition",
            "book_title", "pages", "version", "platform", "link", "campus",
        ],
        search_column: Some("title"),
        exported: true,
    },
];

/// Resolve a frontend route. Unknown routes fail here, typed, instead of
/// reaching SQL — closing the surface that today accepts any string (FR-007).
pub fn by_route(route: &str) -> Result<&'static EntityDef, AppError> {
    ENTITIES
        .iter()
        .find(|e| e.route == route)
        .ok_or_else(|| AppError::UnknownEntity(route.to_string()))
}

pub fn exported() -> impl Iterator<Item = &'static EntityDef> {
    ENTITIES.iter().filter(|e| e.exported)
}

impl EntityDef {
    pub fn has_column(&self, name: &str) -> bool {
        self.columns.contains(&name)
    }

    /// SQL expression to order by, given a validated column name.
    pub fn sort_expr(column: &str) -> String {
        if SORT_BY_LENGTH.contains(&column) {
            format!("length({column})")
        } else {
            column.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Guards against the class of bug that made `proficiencies` and
    /// `professional_activities` show an always-empty "name" column: the Astro
    /// pages were generated from a template assuming every entity has `name`,
    /// and nothing connected those declarations to the real schema.
    #[test]
    fn dashboard_pages_only_declare_columns_that_exist() {
        let pages = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap()
            .join("frontend/src/pages/dashboard");

        let mut problems = Vec::new();
        for file in std::fs::read_dir(&pages).expect("pasta de páginas").flatten() {
            let text = std::fs::read_to_string(file.path()).unwrap_or_default();

            let Some(route) = between(&text, "entity=\"", "\"") else { continue };
            let Some(def) = ENTITIES.iter().find(|e| e.route == route) else {
                problems.push(format!("{route}: rota não existe no registry"));
                continue;
            };

            let Some(list) = between(&text, "columns={[", "]}") else { continue };
            for column in list.split('\'').skip(1).step_by(2) {
                if !def.has_column(column) {
                    problems.push(format!("{route}: declara '{column}', que não existe"));
                }
            }
        }
        assert!(problems.is_empty(), "páginas fora de sincronia:\n  {}", problems.join("\n  "));
    }

    fn between<'a>(haystack: &'a str, start: &str, end: &str) -> Option<&'a str> {
        let i = haystack.find(start)? + start.len();
        let rest = &haystack[i..];
        Some(&rest[..rest.find(end)?])
    }
}
