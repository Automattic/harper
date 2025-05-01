<!--
source: https://en.wikipedia.org/w/index.php?title=Part-of-speech_tagging&oldid=1275774341
license: CC BY-SA 4.0
-->

# Part-of-speech tagging

In corpus linguistics, part-of-speech tagging (POS tagging or PoS tagging or
POST), also called grammatical tagging is the process of marking up a word in a
text (corpus) as corresponding to a particular part of speech, based on both its
definition and its context. A simplified form of this is commonly taught to
school-age children, in the identification of words as nouns, verbs, adjectives,
adverbs, etc.

Once performed by hand, POS tagging is now done in the context of computational
linguistics, using algorithms which associate discrete terms, as well as hidden
parts of speech, by a set of descriptive tags. POS-tagging algorithms fall into
two distinctive groups: rule-based and stochastic. E. Brill's tagger, one of the
first and most widely used English POS-taggers, employs rule-based algorithms.

## Principle

Part-of-speech tagging is harder than just having a list of words and their
parts of speech, because some words can represent more than one part of speech
at different times, and because some parts of speech are complex. This is not
rare—in natural languages (as opposed to many artificial languages), a large
percentage of word-forms are ambiguous. For example, even "dogs", which is
usually thought of as just a plural noun, can also be a verb:

> The sailor dogs the hatch.

Correct grammatical tagging will reflect that "dogs" is here used as a verb, not
as the more common plural noun. Grammatical context is one way to determine
this; semantic analysis can also be used to infer that "sailor" and "hatch"
implicate "dogs" as 1) in the nautical context and 2) an action applied to the
object "hatch" (in this context, "dogs" is a nautical term meaning "fastens (a
watertight door) securely").

### Tag sets

Schools commonly teach that there are 9 parts of speech in English: noun, verb,
article, adjective, preposition, pronoun, adverb, conjunction, and interjection.
However, there are clearly many more categories and sub-categories. For nouns,
the plural, possessive, and singular forms can be distinguished. In many
languages words are also marked for their "case" (role as subject, object,
etc.), grammatical gender, and so on; while verbs are marked for tense, aspect,
and other things. In some tagging systems, different inflections of the same
root word will get different parts of speech, resulting in a large number of
tags. For example, NN for singular common nouns, NNS for plural common nouns, NP
for singular proper nouns (see the POS tags used in the Brown Corpus). Other
tagging systems use a smaller number of tags and ignore fine differences or
model them as features somewhat independent from part-of-speech.

In part-of-speech tagging by computer, it is typical to distinguish from 50 to
150 separate parts of speech for English. Work on stochastic methods for tagging
Koine Greek (DeRose 1990) has used over 1,000 parts of speech and found that
about as many words were ambiguous in that language as in English. A
morphosyntactic descriptor in the case of morphologically rich languages is
commonly expressed using very short mnemonics, such as Ncmsan for Category=Noun,
Type = common, Gender = masculine, Number = singular, Case = accusative, Animate
= no.

The most popular "tag set" for POS tagging for American English is probably the
Penn tag set, developed in the Penn Treebank project. It is largely similar to
the earlier Brown Corpus and LOB Corpus tag sets, though much smaller. In
Europe, tag sets from the Eagles Guidelines see wide use and include versions
for multiple languages.

POS tagging work has been done in a variety of languages, and the set of POS
tags used varies greatly with language. Tags usually are designed to include
overt morphological distinctions, although this leads to inconsistencies such as
case-marking for pronouns but not nouns in English, and much larger
cross-language differences. The tag sets for heavily inflected languages such as
Greek and Latin can be very large; tagging words in agglutinative languages such
as Inuit languages may be virtually impossible. At the other extreme, Petrov et
al. have proposed a "universal" tag set, with 12 categories (for example, no
subtypes of nouns, verbs, punctuation, and so on). Whether a very small set of
very broad tags or a much larger set of more precise ones is preferable, depends
on the purpose at hand. Automatic tagging is easier on smaller tag-sets.

## History

### The Brown Corpus

Research on part-of-speech tagging has been closely tied to corpus linguistics.
The first major corpus of English for computer analysis was the Brown Corpus
developed at Brown University by Henry Kučera and W. Nelson Francis, in the
mid-1960s. It consists of about 1,000,000 words of running English prose text,
made up of 500 samples from randomly chosen publications. Each sample is 2,000
or more words (ending at the first sentence-end after 2,000 words, so that the
corpus contains only complete sentences).

The Brown Corpus was painstakingly "tagged" with part-of-speech markers over
many years. A first approximation was done with a program by Greene and Rubin,
which consisted of a huge handmade list of what categories could co-occur at
all. For example, article then noun can occur, but article then verb (arguably)
cannot. The program got about 70% correct. Its results were repeatedly reviewed
and corrected by hand, and later users sent in errata so that by the late 70s
the tagging was nearly perfect (allowing for some cases on which even human
speakers might not agree).

This corpus has been used for innumerable studies of word-frequency and of
part-of-speech and inspired the development of similar "tagged" corpora in many
other languages. Statistics derived by analyzing it formed the basis for most
later part-of-speech tagging systems, such as CLAWS and VOLSUNGA. However, by
this time (2005) it has been superseded by larger corpora such as the 100
million word British National Corpus, even though larger corpora are rarely so
thoroughly curated.

For some time, part-of-speech tagging was considered an inseparable part of
natural language processing, because there are certain cases where the correct
part of speech cannot be decided without understanding the semantics or even the
pragmatics of the context. This is extremely expensive, especially because
analyzing the higher levels is much harder when multiple part-of-speech
possibilities must be considered for each word.

### Use of hidden Markov models

In the mid-1980s, researchers in Europe began to use hidden Markov models (HMMs)
to disambiguate parts of speech, when working to tag the Lancaster-Oslo-Bergen
Corpus of British English. HMMs involve counting cases (such as from the Brown
Corpus) and making a table of the probabilities of certain sequences. For
example, once you've seen an article such as 'the', perhaps the next word is a
noun 40% of the time, an adjective 40%, and a number 20%. Knowing this, a
program can decide that "can" in "the can" is far more likely to be a noun than
a verb or a modal. The same method can, of course, be used to benefit from
knowledge about the following words.

More advanced ("higher-order") HMMs learn the probabilities not only of pairs
but triples or even larger sequences. So, for example, if you've just seen a
noun followed by a verb, the next item may be very likely a preposition,
article, or noun, but much less likely another verb.

When several ambiguous words occur together, the possibilities multiply.
However, it is easy to enumerate every combination and to assign a relative
probability to each one, by multiplying together the probabilities of each
choice in turn. The combination with the highest probability is then chosen. The
European group developed CLAWS, a tagging program that did exactly this and
achieved accuracy in the 93–95% range.

Eugene Charniak points out in Statistical techniques for natural language
parsing (1997) that merely assigning the most common tag to each known word and
the tag "proper noun" to all unknowns will approach 90% accuracy because many
words are unambiguous, and many others only rarely represent their less-common
parts of speech.

CLAWS pioneered the field of HMM-based part of speech tagging but was quite
expensive since it enumerated all possibilities. It sometimes had to resort to
backup methods when there were simply too many options (the Brown Corpus
contains a case with 17 ambiguous words in a row, and there are words such as
"still" that can represent as many as 7 distinct parts of speech.

HMMs underlie the functioning of stochastic taggers and are used in various
algorithms one of the most widely used being the bi-directional inference
algorithm.

### Dynamic programming methods

In 1987, Steven DeRose and Kenneth W. Church independently developed dynamic
programming algorithms to solve the same problem in vastly less time. Their
methods were similar to the Viterbi algorithm known for some time in other
fields. DeRose used a table of pairs, while Church used a table of triples and a
method of estimating the values for triples that were rare or nonexistent in the
Brown Corpus (an actual measurement of triple probabilities would require a much
larger corpus). Both methods achieved an accuracy of over 95%. DeRose's 1990
dissertation at Brown University included analyses of the specific error types,
probabilities, and other related data, and replicated his work for Greek, where
it proved similarly effective.

These findings were surprisingly disruptive to the field of natural language
processing. The accuracy reported was higher than the typical accuracy of very
sophisticated algorithms that integrated part of speech choice with many higher
levels of linguistic analysis: syntax, morphology, semantics, and so on. CLAWS,
DeRose's and Church's methods did fail for some of the known cases where
semantics is required, but those proved negligibly rare. This convinced many in
the field that part-of-speech tagging could usefully be separated from the other
levels of processing; this, in turn, simplified the theory and practice of
computerized language analysis and encouraged researchers to find ways to
separate other pieces as well. Markov Models became the standard method for the
part-of-speech assignment.

#### Unsupervised taggers

The methods already discussed involve working from a pre-existing corpus to
learn tag probabilities. It is, however, also possible to bootstrap using
"unsupervised" tagging. Unsupervised tagging techniques use an untagged corpus
for their training data and produce the tagset by induction. That is, they
observe patterns in word use, and derive part-of-speech categories themselves.
For example, statistics readily reveal that "the", "a", and "an" occur in
similar contexts, while "eat" occurs in very different ones. With sufficient
iteration, similarity classes of words emerge that are remarkably similar to
those human linguists would expect; and the differences themselves sometimes
suggest valuable new insights.

These two categories can be further subdivided into rule-based, stochastic, and
neural approaches.

#### Other taggers and methods

Some current major algorithms for part-of-speech tagging include the Viterbi
algorithm, Brill tagger, Constraint Grammar, and the Baum-Welch algorithm (also
known as the forward-backward algorithm). Hidden Markov model and visible Markov
model taggers can both be implemented using the Viterbi algorithm. The
rule-based Brill tagger is unusual in that it learns a set of rule patterns, and
then applies those patterns rather than optimizing a statistical quantity.

Many machine learning methods have also been applied to the problem of POS
tagging. Methods such as SVM, maximum entropy classifier, perceptron, and
nearest-neighbor have all been tried, and most can achieve accuracy above
95%.[citation needed]

A direct comparison of several methods is reported (with references) at the ACL
Wiki. This comparison uses the Penn tag set on some of the Penn Treebank data,
so the results are directly comparable. However, many significant taggers are
not included (perhaps because of the labor involved in reconfiguring them for
this particular dataset). Thus, it should not be assumed that the results
reported here are the best that can be achieved with a given approach; nor even
the best that have been achieved with a given approach.

In 2014, a paper reporting using the structure regularization method for
part-of-speech tagging, achieving 97.36% on a standard benchmark dataset.
