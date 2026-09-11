# `posslq`

**POSSLQ: Parts-of-Speech Sharing Living Quarters**

A combinatorial analysis tool for Harper that checks fine-grained POS property adjacencies rather than just POS-to-POS combinations. Previous experiments with simple POS bigram filtering found that almost all combinations occur in natural language, making that approach ineffective for finding "obvious mistakes."

POSSLQ addresses this by examining the specific properties of parts of speech (like `is_proper`, `is_linking`, `person`) rather than just their categories. It scans `harper-core`'s POS data structures and generates an enum with bitfield payloads representing each POS's boolean properties, creating a "shadow lane" alongside the tokenized stream for various analyses.
