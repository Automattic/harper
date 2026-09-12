# `harper-cli`

## What?

`harper-cli` is a small, experimental frontend for Harper.
It can be used in any situation where you might need to check a large number of files automatically (like in continuous integration).

Right now it is quite feature barren, mainly because an external use-case has not been defined yet.
If you have any thoughts, feel free to reach out.

## Interactive linting

Run `harper-cli repl` to check plain English one line at a time without restarting
Harper. Each non-empty line is checked independently, with diagnostics in text
order, rule names, and suggestions. Blank lines are skipped. Press Ctrl-C or send
EOF (Ctrl-D on Unix terminals) to exit.

Use `harper-cli repl --dialect uk` to select a dialect, or
`harper-cli repl --only AnA,SpellCheck` to restrict the rules. As with `lint`,
`--no-color` and the `NO_COLOR` environment variable disable colored output.

Input can also be piped in: each line is checked as it arrives, including a final
line without a newline. Prompts appear on standard error only when standard input
is a terminal; lint results go to standard output. This mode uses the curated
dictionary. For files, custom dictionaries, or structured output, use `lint`.

## Possible Future Features

- On-disk caching
- Custom dictionaries (maybe use the same ones as `harper-ls`?)
- Machine-readable output
