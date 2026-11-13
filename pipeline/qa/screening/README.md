# Example-English screening (AI translations of krdict examples)

All 47,353 AI translations in `pipeline/data/example_en.tsv` are being re-checked for meaning
errors by outside models (Gemini, Grok, ChatGPT), then each flagged row is judged and fixed here.

- `prompt_template.txt`: the instructions at the top of every batch.
- `batches_2000.idmap.tsv`: batch row id (B01-0001 …) → sentence key, headword and meaning. B01–B18 are
  고급 words, most common first; B19–B23 are 초급/중급. Sentences already screened in the trial
  batches are not included.
- `make_batches.py OUT_DIR [B04 …]`: rebuilds the one-paste batch files from the current English.
- `results/`: model replies (`Bxx_<model>.txt`) and the decisions applied (`Bxx_decided.json`).

Progress is the list of `results/Bxx_decided.json` files.
