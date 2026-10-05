-- SEP-033: domínio SRC no mesmo arquivo SQLite (Constituição V).
-- Nenhuma tabela do Horizon é alterada; o prefixo src_ mantém os domínios
-- separados no nível do esquema (FR-004). Constraints conforme data-model.md.

CREATE TABLE src_meta (
    campus TEXT,
    imported_at TEXT NOT NULL DEFAULT ''
);

INSERT INTO src_meta (campus, imported_at) VALUES (NULL, '');

CREATE TABLE src_acoes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    acao_id TEXT NOT NULL UNIQUE,
    raw_json TEXT NOT NULL,
    processo TEXT,
    titulo TEXT,
    natureza TEXT,
    tipo TEXT,
    coordenador TEXT,
    acao_vinculante TEXT,
    campus TEXT,
    total_participacoes INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE src_participacoes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    acao_row_id INTEGER NOT NULL REFERENCES src_acoes(id) ON DELETE CASCADE,
    ord INTEGER NOT NULL,
    tipo TEXT NOT NULL,
    atividade_num TEXT,
    atividade_id TEXT,
    atividade TEXT,
    nome TEXT,
    raw_json TEXT NOT NULL
);

CREATE INDEX idx_src_participacoes_acao ON src_participacoes(acao_row_id, ord);
CREATE INDEX idx_src_acoes_titulo ON src_acoes(titulo);
