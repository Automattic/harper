# The Weir Language

Most large organizations have a style guide.
A document that decides which versions of a linguistic rule to use.
That could be whether to use the Oxford comma, or if contractions are allowed.
It could declare that a certain word should be capitalized in a specific context.

Harper can cover *most* of the rules in *most* style guides, but there will always be outliers that we can't support (or simply don't know about).
That is why it is critical that Harper allow individuals and organizations to define rules and conventions for Harper to enforce.

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
set main [(G Suite), (G Suit), (Google Apps for Work)]
```

The remaining lines describe:

1. The message to be shown to the user when the error in encountered.
1. A description of the rule itself, explaining why it exists.
1. What kind of rule it is. 
1. What corrections to provide to the user.

## Comments

Comments are written using a single hashtag (`#`) like so:

```plaintext
# This is a comment and has no effect on the rest of the file.
set main [(G Suite), (G Suit), (Google Apps for Work)]
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

### Arrays

Arrays in Weir, notated with `[]`, allow Harper to search for multiple potential options at a time.
For a document to match, it only needs to fulfill one of the options in the array.

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
