//! Reports the tagger's predicted UPOS for the lint-critical words in the
//! sentence contexts the linters actually check. Drives lexicon-removal
//! iteration: any line whose `got` != `want` is a residual the model must learn.
use harper_brill::brill_tagger;

/// (label, sentence tokens, focus word, wanted UPOS).
const CASES: &[(&str, &[&str], &str, &str)] = &[
    ("too-stiff", &["I", "'m", "too", "stiff", "."], "stiff", "ADJ"),
    ("instead-of", &["Instead", "of", "being", "here", ",", "we", "left", "."], "of", "ADP"),
    ("students-books", &["The", "students", "books", "are", "here", "."], "students", "NOUN"),
    ("intricacy", &["I", "admired", "its", "intricacy", "."], "intricacy", "NOUN"),
    ("tempts", &["An", "elusive", "thieve", "tempts", "teachers", "."], "tempts", "VERB"),
    ("elusive", &["An", "elusive", "fox", "ran", "."], "elusive", "ADJ"),
];

fn main() {
    let t = brill_tagger();
    let (mut ok, mut total) = (0, 0);
    for (label, toks, focus, want) in CASES {
        let s: Vec<String> = toks.iter().map(|x| x.to_string()).collect();
        let tags = t.tag_sentence(&s);
        let i = toks.iter().position(|w| w == focus).unwrap();
        let got = tags[i].map(|u| format!("{u:?}")).unwrap_or_else(|| "None".into());
        let mark = if got == *want {
            ok += 1;
            "ok "
        } else {
            "MISS"
        };
        total += 1;
        println!("  [{mark}] {label:14} {focus:12} got={got:6} want={want}");
    }
    println!("  {ok}/{total} lint-critical words correct");
}
