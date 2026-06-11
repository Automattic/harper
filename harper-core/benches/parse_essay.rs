use criterion::{Criterion, criterion_group, criterion_main};
use harper_core::languages::{Language, LanguageFamily};
use harper_core::linting::{LintGroup, Linter};
use harper_core::spell::FstDictionary;
use harper_core::{Document, EnglishDialect};
use std::hint::black_box;

static ESSAY: &str = include_str!("./essay.md");

fn parse_essay(c: &mut Criterion) {
    c.bench_function("parse_essay", |b| {
        b.iter(|| Document::new_markdown_default_curated(black_box(ESSAY)));
    });
}

fn lint_essay(c: &mut Criterion) {
    let dictionary = FstDictionary::curated(LanguageFamily::English);
    let mut lint_set =
        LintGroup::new_curated(dictionary, Language::English(EnglishDialect::American));
    let document = Document::new_markdown_default_curated(black_box(ESSAY));

    c.bench_function("lint_essay", |b| {
        b.iter(|| lint_set.lint(&document));
    });
}

fn lint_essay_uncached(c: &mut Criterion) {
    c.bench_function("lint_essay_uncached", |b| {
        b.iter(|| {
            let dictionary = FstDictionary::curated(LanguageFamily::English);
            let mut lint_set = LintGroup::new_curated(
                dictionary.clone(),
                Language::English(EnglishDialect::American),
            );
            let document = Document::new_markdown_default(black_box(ESSAY), &dictionary);
            lint_set.lint(&document)
        })
    });
}

pub fn criterion_benchmark(c: &mut Criterion) {
    parse_essay(c);
    lint_essay(c);
    lint_essay_uncached(c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
