# The Weir Language

Most large organizations have a style guide.
A document that decides which versions of a linguistic rule to use.
That could be whether to use the Oxford comma, or if contractions are allowed.
It could declare that a certain word should be capitalized in a specific context.

Harper can cover *most* of the rules in *most* style guides, but there will always be outliers that we can't support (or simply don't know about).
That is why it is critical that Harper allow individuals and organizations to define rules and conventions for Harper to enforce.

Try the in-browser [Weir studio](/weir/studio) to experiment with rules and tests.

## Introducing Weir

The heart of Weir is an expression language that mimics the pseudocode Harper contributors tend to use when describing the Rust code they intend to write.

Imagine you work at Google. You've just rebranded the "G Suite" collection of apps and services to the new name "Google Workspace".
Before that, they were collectively named "Google Apps for Work".
Moving forward, you don't want you or your coworkers to accidentally write "G Suite" on public documentation, because doing so might confuse users.
To solve this, you use the following Weir rule:

```plaintext
expr main [(G [Suite, Suit]), (Google Apps for Work)]

let message "Use the updated brand."
let description "`G Suite` or `Google Apps for Work` is now called `Google Workspace`"
let kind "Miscellaneous"
let becomes "Google Workspace"
```

The first line describes the pattern of the problematic text.
There are two cases here:

1. The letter "G" followed by "Suite" or its misspelling "Suit"
1. The literal phrase "Google Apps for Work"

Here is a semantically equivalent example that you may find a bit easier to read:

```plaintext
 [(G Suite), (G Suit), (Google Apps for Work)]
```

The remaining lines describe:

1. The message to be shown to the user when the error in encountered.
1. A description of the rule itself, explaining why it exists.
1. What kind of rule it is. 
1. What corrections to provide to the user.

## Comments

Comments are written using a single pound sign (`#`), like so:

```plaintext
# This is a comment and has no effect on the rest of the file.
 [(G Suite), (G Suit), (Google Apps for Work)]
```

## The Various Kinds of Expressions

As previously stated, Weir's expression language is the heart of the system.
There are a few key bits of notation you should know when writing a rule.

### Words

A word is the simplest kind of expression.
It is exactly what it sounds like: if a document contains a specific word, it will match.
Note that these matches are case-insensitive.

Here's an example:

```plaintext
expr main teh

let message "Did you mean the definite article?"
let description "Fixes especially common misspellings of the word `the`"
let kind "Typo"
let becomes "the"

test "I adore teh light of the moon." "I adore the light of the moon."
test "I adore TEH LIGHT OF THE MOON." "I adore THE LIGHT OF THE MOON."
```

When Harper encounters the literal word "teh", it will correct it directly to "the".
We'll get to the test notation later in this document.

### Sequences

A sequence, notated with round braces `()`, is exactly what it sounds like.
It is a sequence of other expressions. 
In order for a portion of a document to match against a sequence, all child expressions must match, in the sequence they are declared.

It's common to see expressions that string words together in a sequence to match against specific phrases.

```plaintext
expr main (gong to)

let message "Did you mean `going to`?"
let description "Corrects `gong to` to the intended phrase `going to`."
let kind "Typo"
let becomes "going to"
```

If the above rule is enabled, when Harper encounters the words `gong` and `to`, separated by whitespace, Harper will replace them with "going to".
The top-level expression assumed to be a sequence, so the first line can be replaced with this without changing the rule's behavior:

```plaintext
expr main gong to
```

### Alternatives

Alternatives in Weir, notated with `[]`, allow Harper to search for multiple potential options at a time.
For a document to match, it only needs to fulfill one of the options in the alternative array.

This syntax should look familiar from the first example we looked at in the introduction.
We have multiple specific phrases we want to look for, and change all of them, should they exist, to the same thing.

```plaintext
expr main [(low hanging fruit), (low hanging fruits), (low-hanging fruits)]

let message "The standard form is `low-hanging fruit` with a hyphen and singular form."
let description "Corrects nonstandard variants of `low-hanging fruit`."
let kind "Usage"
let becomes "low-hanging fruit"
```

Importantly, we can refactor this into just two sequences, by moving the arrays further into the expression.

```plaintext
expr main [(low[-, ( )]hanging fruits), (low hanging fruit)]
```

### Filters

Filters in Weir allow you to locate an expression, then narrow down the search to a shorter segment of that same expression.
This can be done as many times as you like.
The example below uses the filter syntax (the `<>`) to first select the broader phrase, then another to select the whitespace in between (that's the `( )` part).

```plaintext
expr main <([right, middle, left] $click), ( )>
let message "Hyphenate this mouse command"
let description "Hyphenates right-click style mouse commands."
let kind "Punctuation"
let becomes "-"

test "Right click the icon." "Right-click the icon."
test "Please right click on the link." "Please right-click on the link."
test "They right clicked the submit button." "They right-clicked the submit button."
test "Right clicking the item highlights it." "Right-clicking the item highlights it."
test "Right clicks are tracked in the log." "Right-clicks are tracked in the log."
test "He RIGHT CLICKED the file." "He RIGHT-CLICKED the file."
test "Left click the checkbox." "Left-click the checkbox."
test "Middle click to open in a new tab." "Middle-click to open in a new tab."
```

This is extremely useful for identifying exceptions to a rule, since you can include the exceptions in the first layer of the filter, then slowly become more specific.

See the testing section of this document for a more detailed description of the `test` syntax.

### UPOS

Weir allows you to require that certain words assume a specific role in the sentence by writing the [Universal Part-Of-Speech tag](https://universaldependencies.org/u/pos/index.html), literally, in the position you wish to require it..
For example, if you wanted to locate phrases that start with a determiner and are succeeded by a noun, you can write:

```plaintext
expr main DET NOUN
```

This will match against "the word", "a banana", "an apple", among others.

### Progressive Verbs

You can require that a token be a progressive verb with the keyword PROG.
For example:

```plaintext
expr main PROG
```

### Exceptions

You can add exceptions to expressions by throwing an exclamation mark in front of any other expression.
This is useful for excluding certain contexts from a match.
For example, if we wanted to locate all determiners when they are not followed by a noun, we can do this:

```
expr main DET !NOUN
```

Similarly, we can do the same with arrays or sequences of words:

```
expr main DET ![break, braking]
```

### Wildcards

If you need to allow a token between two others, or otherwise want to include an unknown and unrestricted token in the expression, you can use a wildcard.
Although they look like the single-character wildcard from regex (Weir borrows the `*` syntax), Weir wildcards match over entire tokens, including words, punctuation, and whitespace.

Example:

```plaintext
# To match any token that is preceded by a noun and succeeded by a noun.
expr main NOUN * NOUN
```

### Punctuation

Any punctuation that does not otherwise hold semantic meaning in Weir (like wildcards, etc.) is an expression that matches itself.
That means you can match a period with `.` or a question mark with `?`.

```
# Looks for all doubled hyphens.
expr main --

let message "The second mark is redundant."
let description "Looks for redundant doubling of hyphens."
let kind "Punctuation"
let becomes "-"
```

## Replacement Strategies

You can dictate how Harper will suggest a replacement using the `strategy` tag.
This allows rule authors to describe which strategy Harper will use when applying the replacements.
Right now, the only two options are `Exact` or `MatchCase`, which apply either the exact text, or the exact text but matching the capitalization of the text it replaces.
In the below example, we use `Exact` because we want to correct the capitalization of a proper noun, and it doesn't matter what the original text looked like.

```plaintext
expr main [(G [Suite, Suit]), (Google Apps for Work)]
let message "Use the updated brand."
let description "`G Suite` or `Google Apps for Work` is now called `Google Workspace`"
let kind "Miscellaneous"
let becomes "Google Workspace"
let strategy "Exact"
```

## Adding Tests

The Weir language supports the inclusion of tests directly in the file.
You can imagine these to be a lot like assertions.
It is a way of saying, "I expect this rule to transform this text into this other text."

It is pretty much always a good idea to include tests, just to make sure your rule does what you expect.

The syntax is pretty simple:

```plaintext
# I expect A to become B
test "A" "B"
```

You can also assert that the rule _will not_ change anything.

```plaintext
# I don't expect the rule to change anything
test "A" "A"
```

In the future, expect new types of tests to become available.

If you have `harper-cli` available, you can run the tests in a given Weir file by running `harper-cli test <path to the Weir file>`.

## Weirpacks

Weirpacks are zip archives of Weir rules meant for distribution. They use the `.weirpack` extension for clarity, but the contents are standard ZIP.

### Layout

- The archive root contains one or more `.weir` files.
- Each rule name is the filename stem (for example, `TheirToThere.weir` becomes `TheirToThere`).
- A `manifest.json` file is required at the root.

### Manifest

The manifest is JSON. It must include the required fields below and may include any extra metadata you want.

Required fields:
- `author`
- `version`
- `description`
- `license`

Example:

```json
{
  "author": "Ada Lovelace",
  "version": "1.0.0",
  "description": "Rules for Victorian technical writing.",
  "license": "MIT",
  "keywords": ["victorian", "technical", "style"],
  "website": "https://example.com"
}
```

### Loading

In `harper-cli`, use `--weirpack` to load one or more packs at lint time:

```bash
harper-cli lint --weirpack path/to/rules.weirpack README.md
```

## See Also:

- [Building the Weir Language](https://elijahpotter.dev/articles/building-the-weir-language)
- [Updates on the Weir Language](https://elijahpotter.dev/articles/updates-on-the-weir-language)

## Additional Examples

```plaintext
expr main (like as if)

let message "Avoid redundancy. Use either `like` or `as if`."
let description "Corrects redundant `like as if` to `like` or `as if`."
let becomes ["like", "as if"]

test "And looks like as if linux-personality hasn't got any changes for 8 years." "And looks as if linux-personality hasn't got any changes for 8 years."
```

```plaintext
expr main (w/o)

let message "Use `without` instead of `w/o`"
let description "Expands the abbreviation `w/o` to the full word `without` for clarity."
let kind "Style"
let becomes "without"
```
