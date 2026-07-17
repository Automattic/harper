# Harper's Testing Strategy

Harper is a complex project. We deliver many artifacts to many users, and the automated processes we use to do so can seem intimidating.
While the hope is that contributors become familiar with our processes over time, it is only strictly necessary to understand their fundamental goal: __to assist in the production of extremely high-quality software__.

What does "quality" mean?

In one sense, quality is in the eye of the beholder. It is up to you to determine what high quality software looks like. In another sense, there are very definable characteristics of high quality software. Quality is _more_ than these characteristics, but we are not wrong to strive to produce software that supports these characteristics.

We will dive into the characteristics of high quality software, then we will discuss how we get there.

## Characteristics of High Quality Software

### High Quality Software Is Fast

Nobody in the history of the universe has said, "I wish I had to wait even longer for this software to load".

High quality software is fast. High quality grammar checking is also fast.
The sooner we can check if a user's document contains errors, the sooner they can fix them and continue writing.
Any substantial slowdowns in Harper's system are considered bugs, and are treated as such.

This is an example of a way Quality cannot be guaranteed by tests.

### High Quality Software Is Intuitive

Nobody likes software that is difficult to learn.
On the other hand, the human brain _loves_ to learn and experiment.
What does that mean for us?

High quality software behaves the way we expect it to on the surface.
If more complex behavior is needed from a user, it should be taught through non-destructive play.
Users should not need a deep knowledge of any field to be productive with Harper.

### High Quality Software Gets out of the Way

The purpose of most software, especially Harper, is to get something done.
Software should not distract users from their goal.
Harper's purpose is to assist our users in expressing their thoughts and emotions through writing.
Harper should not distract users from their writing.

## How We Get There

To achieve high quality software, it is critical that we maintain a tight feedback loop and iterate as many times as possible.
There are many components to how we achieve that, but they all contribute to the same sequence:

1. Identify what faults exist in the software. These can be "bugs", but they can also take form of performance problems, unintuitive user interfaces, or distracting behaviors.
2. Identify how to fix those faults and perform those necessary actions.
3. Deploy those fixes to real users, and we go back to step one.

We use prerelease checks, manual testing, and user feedback to identify faults
We fix them with careful judgment and engineering.
We deploy whenever possible so that improvements reach users quickly.

### Linting

Static analysis is the fastest form of feedback available for Harper code.
A detailed breakdown of the exact tools we use is outside of the scope of this document.

At a high level, most contributors do not need to access anything beyond what is surfaced through the `just check-js` and `just check-rust` commands, which are self-explanatory.
These checks are also run as a part of every PR.

### Prerelease Checks + Testing

Harper runs a manifold of checks before each release, before a PR is reviewed, and after a PR is merged.
These checks perform assertions on the behavior of `harper-core` (the main grammar checking engine) as well as the various integrations we officially support.

Our checks can be run locally (using the various [`just`](https://github.com/casey/just) commands provided in the repository) or using GitHub Actions.

#### Unit Checks

The plurality of our checks are on our core grammar engine, which resides in the `harper-core` package.
Core checks often take the form of an expected input and output, with `harper-core` sitting in between.

```plaintext
Grammatically Incorrect Input Text       
                 │                       
                 ▼                       
            harper-core                  
                 │                       
                 ▼                       
Grammatically Correct Output Text        
```

Unit tests exist in other integration packages, and usually test algorithmic logic.

#### Web Integration Checks 

Web integration checks, employed primarily for the Chrome Extension, exist to ensure that Harper plays correctly with various online web text editors.
That includes, but is not limited to:

- Reading text from a text editor.
- Non-destructively writing text to a text editor.
  "Non-destructive" in this context means that undo/redo history is preserved and irrelevant portions of a document are preserved.
- Non-destructively highlighting text.
  "Non-destructively" in this context means that relevant highlight DOM elements are NOT incorporated into the state of the editor, and do not impact the existing functionality of the text editor.

Harper's web integration checks are written using Playwright and work by performing each of these actions on an __actual__ instance of the web text editors we support.
As such, web integration checks are very expensive to run and they are challenging to write in a way that does not create a flaky test, simply due to the sheer number of systems they touch.

Therefore, it is critical that each web integration check has a clear, defined purpose. 
Any web integration check that does not create clear substantial value should be removed.

### Learning from Production

Most bugs follow a Pareto distributor of impact.
Put another way, ~ 20% of bugs are responsible for 80% of negative user experience.
If we can eliminate those few bugs that are most destructive to the user experience, Harper contributors can have a much larger impact on the final user experience.
The question is: __how can we possibly know which 20% we should focus on?__

The answer is to get continuous user feedback.

To get at what that means, let's use a specific example.
Harper provides support for specific types of grammatical errors through individual modules we call "linters".
The exact structure of these linters matters very little.
All one needs to know is that there are individual non-overlapping linters.
For example, there's one that detects improper use of Oxford Commas, one that detects misspelled words, and one that differentiates between "there", "their", and "they're".
Each one of these linters has a linter ID, which is a short (< 10 character) string that ties a suggestion to the piece of code that produced it.

When a user reviews a suggestion (via the Chrome Extension or otherwise), they are given the option to report it if they believe it was made in error.
If they do so, a POST request is made to `https://writewithharper.com` and it is tallied up.
On the backend, the project maintainers can then analyze these reports and identify which linters are responsible for the most incorrect suggestions.

Using this information, maintainers can better allocate their efforts and may, on occasion, publish "challenge" lint IDs.
These are known lint IDs that are known to be especially problematic to users, and thus need special attention.
Taking on challenge issues may accelerate a contributor's path to becoming a [committer](https://writewithharper.com/docs/contributors/committer).

While the example outlined above is about improving the quality of Harper's core grammar engine, we employ similar techniques for supported domains in the Chrome Extension and in other places.

### Manual Testing

Whenever feasible, Harper contributors manually test the product, as well as any changes they make.
We do this because it is the highest signal form of quality assurance.
It covers deterministic logic in addition to UX.

To make this easier, we highly recommend that contributors dogfood any and all of Harper's integrations wherever reasonable.

## Glossary

### Check

Often also referred to as "a test".
A check is an atomic piece of code that determines whether a program expresses a certain characteristic.
Checks are run as part of a prerelease process.

### Testing

Testing is a human activity and it does not happen automatically.
It happens when a person sits down and uses the software to determine whether it behaves as expected.
Testing is the most reliable way to determine the quality of software, but it is also the most expensive.

Create checks whenever possible to get high confidence in Harper's behavior, but perform manual testing every so often to be absolutely sure your code behaves as you expect.

## Additional Resources

- [Quality Requires Visual Design](http://elijahpotter.dev/articles/quality-requires-visual-design)
- [Quality Is the Most Important Metric](https://elijahpotter.dev/articles/quality-is-the-most-important-metric)
- [Dealing with Flaky Tests](https://elijahpotter.dev/articles/dealing-with-flaky-tests)
- [Integration Testing Thousands of Websites with Playwright](https://elijahpotter.dev/articles/integration-testing-thousands-of-sites-with-playwright)
- [3 Traits of Good Test Suites](https://elijahpotter.dev/articles/3-traits-of-good-test-suites)
- [LLM Assisted Fuzzing](https://elijahpotter.dev/articles/LLM-assisted-fuzzing)
