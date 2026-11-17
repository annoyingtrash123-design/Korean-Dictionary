# 한뿌리 (Hanppuri) — Korean Dictionary

An offline, Pleco-inspired Korean–English dictionary, an Android app. After a one-time download of the
dictionary data it works with no connection. There is no website.

## Install

Download `hanppuri.apk` from the [`android-latest` release](https://github.com/annoyingtrash123-design/Korean-Dictionary/releases/tag/android-latest),
install it, open the app and tap **Download**. New versions: Settings → Check for updates.

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

Every push to `main` builds the data in GitHub Actions and publishes it to GitHub Pages, which only
hosts the packs the app downloads (`.github/workflows/deploy.yml`). `.github/workflows/android.yml`
builds the signed APK.
