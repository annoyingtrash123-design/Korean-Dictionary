//! FrequencyWords ("word count" per line) -> {word: rank}.

use std::collections::{HashMap, HashSet};
use std::io::BufRead;

const ENDINGS: &str = "어 아 여 어요 아요 여요 었 았 였 었어 았어 였어 었어요 았어요 었다 았다 었는데 았는데 \
고 지 지만 지는 지요 면 으면 서 어서 아서 니 으니 네 네요 자 게 도록 는 은 을 ㄴ ㄹ \
는데 은데 을까 을게 을래 을 거 을거 는다 ㄴ다 습니다 ㅂ니다 세요 으세요 십시오 으십시오 려고 으려고 \
ㄹ 려 러 으러 며 으며 기 음 ㅁ 다 냐 니까 으니까 어도 아도 어야 아야 어라 아라 거나 다가 더니 던 ㄹ까";

pub fn load_ranks<R: BufRead>(rd: R) -> HashMap<String, u32> {
    let mut ranks = HashMap::new();
    let mut i = 0u32;
    for line in rd.split(b'\n').map_while(Result::ok) {
        let line = String::from_utf8_lossy(&line);
        let line = line.trim();
        let Some((w, c)) = line.rsplit_once(' ') else { continue };
        if c.parse::<u64>().is_err() {
            continue;
        }
        i += 1;
        ranks.entry(w.to_string()).or_insert(i);
    }
    ranks
}

/// For verb/adjective stems: best rank of a frequent form = stem + ending.
/// Handles plain stems (먹 + 어요) and 하-contractions (공부하 + 였 -> 공부했).
pub fn stem_ranks(ranks: &HashMap<String, u32>) -> HashMap<String, u32> {
    let endings: HashSet<&str> = ENDINGS.split_whitespace().collect();
    let mut out: HashMap<String, u32> = HashMap::new();
    let mut upd = |k: String, r: u32| {
        let e = out.entry(k).or_insert(u32::MAX);
        if *e > r {
            *e = r;
        }
    };
    for (w, &r) in ranks {
        let idx: Vec<usize> = w.char_indices().map(|(i, _)| i).collect();
        for &k in idx.iter().skip(1) {
            let (stem, rem) = w.split_at(k);
            if endings.contains(rem) {
                upd(stem.to_string(), r);
            }
            let mut rc = rem.chars();
            if matches!(rc.next(), Some('해' | '했')) {
                let tail = rc.as_str();
                if tail.is_empty() || endings.contains(tail) {
                    upd(format!("{stem}하"), r);
                }
            }
        }
    }
    out
}
