.mode list
.separator " | "
SELECT '== meta'; SELECT key, substr(value,1,300) FROM meta;
SELECT '== by pos'; SELECT pos, count(*) FROM entries GROUP BY 1 ORDER BY 2 DESC LIMIT 25;
SELECT '== spot'; SELECT headword, homonym, hanja, pos, rank, substr(gloss,1,70) FROM entries WHERE hw_norm IN ('사랑','학교','눈','먹다','갈무리','넘사벽') ORDER BY hw_norm, rank LIMIT 40;
SELECT '== random'; SELECT headword, hanja, pos, substr(gloss,1,70) FROM entries WHERE id % 20000 = 5 LIMIT 25;
SELECT '== data 사랑'; SELECT substr(data,1,1200) FROM entries WHERE hw_norm='사랑' ORDER BY rank LIMIT 1;
SELECT '== examples count'; SELECT count(*) FROM entries WHERE data LIKE '%"examples":[{%';
