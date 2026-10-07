fn main() {
    for a in std::env::args().skip(1) {
        println!("== {a}  hints={:?}", kdict_core::deconjugate::grammar_hints(&a));
        for (i, c) in kdict_core::deconjugate::deconjugate(&a).iter().take(8).enumerate() {
            println!("  {i} {} :: {}", c.lemma, c.rule);
        }
    }
}
