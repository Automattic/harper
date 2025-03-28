---
title: Updating the Curated Dictionary
---

The curated dictionary is the English dictionary Harper uses as reference internally when analyzing or modifying English text.
It is common, especially with technical language, to come across words that are not in this dictionary.
If this happens to you, please open a PR to get them in.

PR [#343](https://github.com/Automattic/harper/pull/343) is a practical example of the ideas described here.

There are two files you need to worry about.
[`harper-core/dictionary.dict`](https://github.com/Automattic/harper/blob/master/harper-core/dictionary.dict) and [`harper-core/affixes.json`](https://github.com/Automattic/harper/blob/master/harper-core/affixes.json).
The first is a list of words, tagged with modifiers defined in the second.

For example, all words, such as "move", tagged with `L`, will be expanded to two dictionary entries, "move" and "movement".
In `affixes.json`, this expansion rule looks like this:

```js title=affixes.json
{
	"L": {
        // Denotes that the area of interest is at the _end_ of the base word.
		"kind": "suffix",
        // Declare that it is OK to use the result of the expansion with other expansions.
		"cross_product": true,
        // The actual replacement rules that result in an expansion.
		"replacements": [
			{
                // If present, remove this text from the area of interest before expansion.
				"remove": "",
				"add": "ment",
                // A simplified regex-like pattern that describes what the area of interest must look like in order for this particular replacement to be applied.
				"condition": "."
			}
		],
        // The word metadata that should be applied to the expanded word.
		"adds_metadata": {},
        // The word metadata that should be applied to the base word.
		"gifts_metadata": {}
	}
}
```

Those familiar with `hunspell` might notice some similarities with their dictionary format.
The main differences are the [metadata fields.](https://docs.rs/harper-core/latest/harper_core/struct.WordMetadata.html)

Most words in `dictionary.dict` have simple rules applied to them that result in no expansion, but apply metadata through the `gifts_metadata`.
Those particular rules are done automatically.
When adding words to the dictionary, just worry about expansions, not metadata.

## Adding Nouns

You don't need to know any of the nitty-gritty details to add nouns to the dictionary.
Use the tool we have in the repo:

```bash
just addnoun <YOUR NOUN HERE>
```

If this command doesn't look familiar, [read our setup documentation for contributors](./environment).
