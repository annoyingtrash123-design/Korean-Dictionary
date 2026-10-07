//! Polite HTTP client shared by the Reader fetchers (`fetch-texts`, `fetch-news`).

use anyhow::{anyhow, Result};
use std::io::Read;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub const USER_AGENT: &str = "KoreanDictionaryReader/1.0 (github.com/annoyingtrash123-design/Korean-Dictionary)";

/// Minimal GET abstraction so the fetchers can be tested against canned responses.
pub trait Http {
    fn get(&self, url: &str) -> Result<String>;
}

/// ureq-based client: <= 1 request/second (shared by every host), retries with exponential backoff on
/// transport errors, 429 and 5xx. 4xx (other than 429) fail immediately.
pub struct UreqHttp {
    agent: ureq::Agent,
    last: Mutex<Option<Instant>>,
    min_interval: Duration,
    retries: u32,
}

impl Default for UreqHttp {
    fn default() -> Self {
        Self::new()
    }
}

impl UreqHttp {
    pub fn new() -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(30))
            .timeout_read(Duration::from_secs(90))
            .user_agent(USER_AGENT)
            .try_proxy_from_env(true)
            .build();
        UreqHttp { agent, last: Mutex::new(None), min_interval: Duration::from_millis(1100), retries: 6 }
    }

    fn throttle(&self) {
        let mut last = self.last.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(t) = *last {
            let el = t.elapsed();
            if el < self.min_interval {
                std::thread::sleep(self.min_interval - el);
            }
        }
        *last = Some(Instant::now());
    }
}

impl Http for UreqHttp {
    fn get(&self, url: &str) -> Result<String> {
        let mut last_err = anyhow!("no attempt made");
        for attempt in 0..=self.retries {
            self.throttle();
            let mut wait = Duration::from_secs(5u64 << attempt.min(4));
            match self.agent.get(url).call() {
                Ok(resp) => {
                    let mut body = String::new();
                    match resp.into_reader().take(64 << 20).read_to_string(&mut body) {
                        Ok(_) => return Ok(body),
                        Err(e) => last_err = anyhow!("reading body of {url}: {e}"),
                    }
                }
                Err(ureq::Error::Status(code, resp)) => {
                    if code == 429 || code >= 500 {
                        if let Some(s) = resp.header("Retry-After").and_then(|v| v.trim().parse::<u64>().ok()) {
                            wait = Duration::from_secs(s.clamp(1, 180));
                        }
                        last_err = anyhow!("HTTP {code} for {url}");
                    } else {
                        return Err(anyhow!("HTTP {code} for {url}"));
                    }
                }
                Err(e) => last_err = anyhow!("{e}"),
            }
            if attempt < self.retries {
                log::warn!("retrying in {}s: {last_err:#}", wait.as_secs());
                std::thread::sleep(wait);
            }
        }
        Err(last_err)
    }
}

/// Percent-encode everything except RFC 3986 unreserved characters.
pub fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// Wiki-style page URL (spaces become underscores, `/` kept).
pub fn wiki_page_url(host: &str, title: &str) -> String {
    let t = title.replace(' ', "_");
    let enc: Vec<String> = t.split('/').map(urlencode).collect();
    format!("https://{host}/wiki/{}", enc.join("/"))
}

/// Current UTC time as `YYYY-MM-DDTHH:MM:SSZ`.
pub fn now_iso() -> String {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) as i64;
    let (days, rem) = (secs.div_euclid(86400), secs.rem_euclid(86400));
    // civil-from-days (Howard Hinnant)
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", rem / 3600, rem % 3600 / 60, rem % 60)
}
