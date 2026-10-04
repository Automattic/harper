# Passive voice regression corpora

`passive_voice_review.tsv` contains 101 verbatim excerpts from the real-world
report in [PR #4502](https://github.com/Automattic/harper/pull/4502#issuecomment-5923253696).
Fourteen additional excerpts come from the [follow-up review](https://github.com/Automattic/harper/pull/4502#issuecomment-5978988921).
The source filenames and line numbers come from the reviewer. Only the outer
editorial ellipses were removed. Combined quotations and fragments missing the
context needed for a reliable expectation were omitted. The test checks each
excerpt as both plain English and Markdown.

`passive_voice_real_world.tsv` selects 27 complete, unmodified paragraphs from
Lewis Carroll's public-domain *Alice's Adventures in Wonderland*, already in
Harper's test corpus, and every paragraph of Harper's README. The test reads
these sources directly, preserving their original wrapping and markup, and
checks exact warning spans. Paragraph indices are zero-based, separated by
blank lines. If a source changes, review the corresponding expectations.

These are targeted regression cases, not a random sample or an accuracy
benchmark. The review excerpts are incomplete secondary quotations, so the
full paragraphs provide additional context. The original quality and validation
TSVs contain AI-authored synthetic cases, alongside the issue #1500 examples;
they are not a real-world corpus.

The review's generated classifications are not the test oracle. Get-passives,
continuous passives, agentive passives, and passive technical instructions with
an explicit event remain grammatically passive even when they are appropriate
stylistically. The rule is optional, offers no rewrites, and asks whether a
change would help. Those examples retain warnings. Reduced relatives and
standalone participial labels are outside its scope. Local exceptions suppress
conventional license declarations and common state, intention, availability,
and agentless technical-configuration readings with a local technical subject.
Ordinary narrative actions such as “he was given a medal” remain eligible.
Subjectless technical excerpts retain the conservative state reading because
they cannot provide a subject to disambiguate. Suppression is local: an explicit
agent or a dated event brings the same participles back into scope, and other
diagnostics in the same document remain eligible.
