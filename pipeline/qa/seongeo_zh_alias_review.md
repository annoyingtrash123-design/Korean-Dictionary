# 성어 Chinese aliases: review sheet

`pipeline/data/seongeo_zh_alias.tsv` maps a 성어's Korean hanja to the traditional headword of **the same idiom** in CC-CEDICT when
the Chinese wording differs (after the automatic character-variant step, which already equates 爲/為, 靑/青, 擧/舉, 衆/眾, 傍/旁 ...).
CC-CEDICT is fetched only in CI, so none of the target headwords could be looked up while these were written: a row whose target is
not in CEDICT simply matches nothing (the build log prints how many rows matched through variants and through aliases).

**All rows are `draft`: they do not ship until a reviewer changes the review column to `approved` (or `rejected: reason`).**

Reviewer: for each row decide only one thing: is the Chinese idiom the same idiom as the Korean 성어 (same story or image, same
meaning), not merely a similar one? Reject anything where you would not write "same idiom" in a dictionary.

| 성어 | Korean hanja | CEDICT headword | Why it is the same idiom | Confidence |
|---|---|---|---|---|
| 새옹지마 | 塞翁之馬 | 塞翁失馬 | the old man who lost his horse; 之 vs 失 | high |
| 파죽지세 | 破竹之勢 | 勢如破竹 | splitting bamboo; word order | high |
| 오합지졸 | 烏合之卒 | 烏合之眾 | rabble like gathered crows; 卒 vs 眾 | high |
| 삼고초려 | 三顧草廬 | 三顧茅廬 | Liu Bei's three visits; 草廬 = 茅廬 | high |
| 우후죽순 | 雨後竹筍 | 雨後春筍 | shoots after rain; 竹 vs 春 | high |
| 호사다마 | 好事多魔 | 好事多磨 | 魔 is the Korean spelling of 磨 | high |
| 상전벽해 | 桑田碧海 | 滄海桑田 | sea turns to mulberry fields | high |
| 동고동락 | 同苦同樂 | 同甘共苦 | share joys and hardships | high |
| 배은망덕 | 背恩忘德 | 忘恩負義 | ungrateful to a benefactor | high |
| 우유부단 | 優柔不斷 | 優柔寡斷 | indecisive; 不 vs 寡 | high |
| 금의환향 | 錦衣還鄕 | 衣錦還鄉 | return home in glory | high |
| 결초보은 | 結草報恩 | 結草銜環 | two repayment stories; Korean keeps one | medium |
| 괄목상대 | 刮目相對 | 刮目相看 | see with new eyes; 對 vs 看 | medium-high |
| 고군분투 | 孤軍奮鬪 | 孤軍奮戰 | fight on alone; CEDICT may also have 孤軍奮鬥 (found by the variant step first) | medium-high |
| 천태만상 | 千態萬象 | 千態萬狀 | myriad forms; 象 vs 狀 | medium-high |
| 안하무인 | 眼下無人 | 目中無人 | looks down on everyone | medium |
| 금수강산 | 錦繡江山 | 錦繡河山 | a splendid land; 江 vs 河 | medium |
| 천고마비 | 天高馬肥 | 秋高馬肥 | autumn: high sky, fat horses | medium |
| 임기응변 | 臨機應變 | 隨機應變 | adapt to circumstances; CEDICT may list 臨機應變 itself, which matches exactly first | medium-high |

19 rows. Not proposed on purpose (similar, not the same idiom): 針小棒大 / 小題大做, 自家撞着 / 自相矛盾, 苦肉之策 / 苦肉計,
風前燈火 / 風中之燭, 刻骨難忘 / 刻骨銘心, 竹馬故友 / 青梅竹馬.
