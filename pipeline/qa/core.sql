.mode list
.separator " | "
.headers off
SELECT '== meta'; SELECT key, substr(value,1,300) FROM meta;
SELECT '== entries by source/kind'; SELECT source, kind, count(*) FROM entries GROUP BY 1,2;
SELECT '== entries by pos'; SELECT source, pos, count(*) FROM entries GROUP BY 1,2 ORDER BY 1,3 DESC;
SELECT '== entries with no gloss'; SELECT source, count(*) FROM entries WHERE gloss IS NULL OR gloss='' GROUP BY 1;
SELECT '== spot checks (exact hw_norm)';
WITH w(n,q) AS (VALUES (0,'학교'),(1,'먹다'),(2,'가다'),(3,'사랑'),(4,'눈'),(5,'배'),(6,'하다'),(7,'있다'),(8,'감사하다'),(9,'안녕하세요'),(10,'컴퓨터'),(11,'대통령'),(12,'김치'),(13,'아서'),(14,'이'),(15,'좋다'),(16,'예쁘다'),(17,'어렵다'),(18,'빨리'),(19,'그런데'),(20,'수'),(21,'것'),(22,'되다'),(23,'보다'),(24,'알다'),(25,'한국'),(26,'사람'),(27,'시간'),(28,'물'),(29,'공부'))
SELECT e.headword, e.homonym, e.hanja, e.pos, e.source, e.level, e.rank, e.kind, substr(e.gloss,1,80)
FROM w JOIN entries e ON e.hw_norm=w.q ORDER BY w.n, e.rank LIMIT 400;
SELECT '== top 40 by rank'; SELECT headword, hanja, pos, source, rank, substr(gloss,1,60) FROM entries ORDER BY rank LIMIT 40;
SELECT '== FTS';
SELECT 'eat', e.headword, e.source, e.rank, substr(e.gloss,1,60) FROM entries_fts f JOIN entries e ON e.id=f.rowid WHERE entries_fts MATCH 'eat' ORDER BY bm25(entries_fts), e.rank LIMIT 12;
SELECT 'school', e.headword, e.source, e.rank, substr(e.gloss,1,60) FROM entries_fts f JOIN entries e ON e.id=f.rowid WHERE entries_fts MATCH 'school' ORDER BY bm25(entries_fts), e.rank LIMIT 12;
SELECT 'beautiful', e.headword, e.source, e.rank, substr(e.gloss,1,60) FROM entries_fts f JOIN entries e ON e.id=f.rowid WHERE entries_fts MATCH 'beautiful' ORDER BY bm25(entries_fts), e.rank LIMIT 12;
SELECT 'love', e.headword, e.source, e.rank, substr(e.gloss,1,60) FROM entries_fts f JOIN entries e ON e.id=f.rowid WHERE entries_fts MATCH 'love' ORDER BY bm25(entries_fts), e.rank LIMIT 12;
SELECT '== hanja chars'; SELECT * FROM hanja_chars WHERE ch IN ('學','國','愛','水','龍');
SELECT '== words with 學 (top 15)'; SELECT e.headword, e.hanja, e.source, e.rank FROM hanja_words h JOIN entries e ON e.id=h.entry_id WHERE h.ch='學' ORDER BY e.rank LIMIT 15;
SELECT '== sentences sample'; SELECT source, ko, en FROM sentences WHERE id % 1500 = 0 LIMIT 15;
SELECT '== sentences for 학교'; SELECT ko, en FROM sentences WHERE id IN (SELECT rowid FROM sentences_fts WHERE sentences_fts MATCH '"학교에"') LIMIT 5;
SELECT '== grammar by category'; SELECT category, count(*) FROM grammar GROUP BY 1;
SELECT '== grammar first 25 by sort'; SELECT pattern, category, level, substr(summary_en,1,60) FROM grammar ORDER BY sort LIMIT 25;
SELECT '== random kengdic'; SELECT headword, hanja, pos, substr(gloss,1,80) FROM entries WHERE source='kengdic' AND id % 4000 = 7 LIMIT 25;
SELECT '== random wikt'; SELECT headword, hanja, pos, substr(gloss,1,80) FROM entries WHERE source='wikt' AND id % 1500 = 3 LIMIT 20;
SELECT '== random krdict'; SELECT headword, hanja, pos, level, substr(gloss,1,80) FROM entries WHERE source='krdict' AND id % 2500 = 11 LIMIT 20;
SELECT '== data JSON 먹다 krdict'; SELECT substr(data,1,1500) FROM entries WHERE hw_norm='먹다' AND source='krdict' ORDER BY rank LIMIT 1;
SELECT '== data JSON 학교 wikt'; SELECT substr(data,1,1000) FROM entries WHERE hw_norm='학교' AND source='wikt' LIMIT 1;
SELECT '== forms for 먹다'; SELECT f.form FROM forms f JOIN entries e ON e.id=f.entry_id WHERE e.hw_norm='먹다' LIMIT 30;
