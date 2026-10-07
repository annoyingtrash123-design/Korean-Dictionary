fn main() {
    for w in ["먹고싶었는데요", "공부하고있었어요", "학교에서는", "갔어요", "먹었어요"] {
        let t = std::time::Instant::now();
        let n = 20;
        for _ in 0..n { std::hint::black_box(kdict_core::deconjugate::deconjugate(w)); }
        println!("{w}: {:?} per call, {} cands", t.elapsed() / n, kdict_core::deconjugate::deconjugate(w).len());
    }
}
