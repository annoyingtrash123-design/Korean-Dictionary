//! SQLite schema (the binding "Database contract" of docs/SCOPE.md, plus a few additions).

pub const COMMON: &str = r#"
CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT);
CREATE TABLE entries (
  id        INTEGER PRIMARY KEY,
  headword  TEXT NOT NULL,
  hw_norm   TEXT NOT NULL,
  homonym   INTEGER,
  hanja     TEXT,
  pos       TEXT,
  pron      TEXT,
  source    TEXT NOT NULL,
  lang      TEXT NOT NULL,
  level     INTEGER,
  rank      INTEGER NOT NULL,
  kind      TEXT NOT NULL,
  gloss     TEXT,
  data      TEXT NOT NULL,
  ext_id    TEXT,
  quality   INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE forms (form TEXT NOT NULL, entry_id INTEGER NOT NULL);
CREATE TABLE hanja_words (ch TEXT NOT NULL, entry_id INTEGER NOT NULL, rank INTEGER NOT NULL DEFAULT 0);
"#;

/// Run before `COMMON_INDEXES`: `hanja_words.rank` = the entry's rank (copied so that "words with
/// this character, most important first" is answered from the covering index alone).
pub const FILL_HANJA_RANK: &str =
    "UPDATE hanja_words SET rank = (SELECT rank FROM entries WHERE entries.id = hanja_words.entry_id);";

pub const COMMON_INDEXES: &str = r#"
CREATE INDEX entries_hw_rank ON entries(hw_norm, rank);
CREATE INDEX entries_hanja ON entries(hanja) WHERE hanja IS NOT NULL;
CREATE INDEX entries_rank ON entries(rank);
CREATE INDEX forms_form ON forms(form);
CREATE INDEX hanja_words_ch ON hanja_words(ch, rank, entry_id);
"#;

pub const CORE_ONLY: &str = r#"
CREATE VIRTUAL TABLE entries_fts USING fts5(head, en, content='', tokenize='porter unicode61');
CREATE TABLE hanja_chars (
  ch TEXT PRIMARY KEY, readings TEXT,
  meaning_en TEXT, strokes INTEGER, radical TEXT, word_count INTEGER,
  radical_num INTEGER
);
CREATE TABLE sentences (id INTEGER PRIMARY KEY, ko TEXT NOT NULL, en TEXT, source TEXT);
CREATE VIRTUAL TABLE sentences_fts USING fts5(ko, content='sentences', content_rowid='id', tokenize='trigram');
CREATE TABLE gloss_terms (
  term TEXT NOT NULL, tier INTEGER NOT NULL, score INTEGER NOT NULL, entry_id INTEGER NOT NULL,
  PRIMARY KEY (term, score, entry_id)
) WITHOUT ROWID;
CREATE TABLE wotd (n INTEGER PRIMARY KEY, entry_id INTEGER NOT NULL);
CREATE TABLE grammar (
  id INTEGER PRIMARY KEY, entry_id INTEGER, pattern TEXT NOT NULL,
  category TEXT NOT NULL,
  level INTEGER, summary_en TEXT, sort INTEGER
);
CREATE INDEX grammar_cat ON grammar(category, sort);
CREATE INDEX grammar_entry ON grammar(entry_id);
"#;
