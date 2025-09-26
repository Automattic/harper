# `lint-framework`

The `lint-framework` serves one specific purpose.
It contains all the logic needed to read and write text to a text editor on a web page to perform linting actions, as well as all logic needed to render underlines and UI for reviewing those actions.
It exists separate from the Chrome/Firefox extensions because there are places where we wish to perform linting actions outside of the Chrome extension (for example, in the demo on the Harper website).
