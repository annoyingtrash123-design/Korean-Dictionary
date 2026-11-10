# Korean Dictionary

An offline, Pleco-inspired Korean–English dictionary for iPhone and Android, installed from the
browser as a web app (PWA). After a one-time download it works with no connection.

**Open:** https://annoyingtrash123-design.github.io/Korean-Dictionary/

## Install on your phone

- **iPhone (iOS 16.4+), Safari:** open the link → Share → **Add to Home Screen** → open it from the
  home screen → **Download**. Keep it on the home screen: iOS keeps home-screen apps' data.
- **Android, Chrome:** open the link → ⋮ → **Install app** (or *Add to Home screen*) → **Download**.

The core download is about 45 MB (≈ 165 MB on the device). The Korean–Korean dictionaries
표준국어대사전 (~60 MB) and 우리말샘 (~100 MB) are optional and can be added or removed in Settings.

## What's inside

| Source | Content |
|---|---|
| 한국어기초사전 (krdict) | ~55k learner entries: English definitions, hanja, levels, examples, grammar |
| Wiktionary (kaikki.org) | ~33k entries with English glosses, hanja, translated examples |
| kengdic | ~82k additional Korean–English entries |
| 표준국어대사전 | ~437k entries, Korean definitions, hanja (optional pack) |
| 우리말샘 | ~650k further entries: dialect, archaic, North Korean, technical (optional pack) |
| Tatoeba, Unihan | Korean–English sentence pairs; hanja readings, meanings, strokes, radicals |

Search accepts Hangul (conjugated forms like 갔어요 → 가다), English, hanja (學) and initial
consonants (ㅎㄱ → 학교). Licences and attributions are in Settings → About, and in
[pipeline/ATTRIBUTION.md](pipeline/ATTRIBUTION.md).

## Development

- `pipeline/`: Rust CLI that downloads the sources and builds the SQLite packs
  (`cargo run --release -p kdict-pipeline -- fetch|build`).
- `core/`: Rust engine compiled to WebAssembly (search, conjugation, SQLite on OPFS).
- `app/`: Preact + TypeScript UI. See [app/README.md](app/README.md).
- `docs/SCOPE.md`: architecture and data contract.

Every push to `main` builds the data and the app in GitHub Actions and deploys to GitHub Pages
(`.github/workflows/deploy.yml`).
