---
title: Language Server
---

`harper-ls` is the [Language Server Protocol](https://microsoft.github.io/language-server-protocol/) frontend for Harper.
Out of the box, it has built-in support for parsing the comments of most programming languages, as well as any and all Markdown files.

## Installation

### Scoop

You can install Harper on Windows through [Scoop](https://scoop.sh/#/apps?q=harper).

```bash
scoop install harper
```

### Homebrew

You can install Harper on macOS and Linux through [Homebrew](https://formulae.brew.sh/formula/harper).

```bash
brew install harper
```

### Arch Linux

#### Stable Release

The latest stable release is available through the [`extra` repo](https://archlinux.org/packages/extra/x86_64/harper):

```bash
sudo pacman -S harper
```

#### Bleeding-Edge

If you want the latest bleeding-edge, you can install `harper-git` from the [Arch User Repository](https://aur.archlinux.org/packages/harper-git) with your favorite AUR helper:

```bash
paru -S harper-git
# or yay -S harper-git, etc.
```

### Nixpkgs/NixOS

You may install Harper via
[Nixpkgs](https://search.nixos.org/packages?channel=unstable&show=harper&from=0&size=50&sort=relevance&type=packages&query=harper).
You can install the `harper` package via any of the normal methods such as
adding it to `environment.systemPackages`. You can try Harper within an
ephemeral shell using:

```bash
nix-shell -p harper
```

or if you have the `nix-command` and `flakes` experimental features enabled:

```bash
nix shell 'nixpkgs#harper'
```

### Cargo

If you have Rust installed, `harper-ls` is on [crates.io](https://crates.io/crates/harper-ls), so you can simply run:

```bash
cargo install harper-ls --locked
```

For this to work, make sure that `~/.cargo/bin` is in your system `$PATH`. If you are on a Debian-based Linux distribution, you may need to install `build-essential`.

### GitHub Releases

If none of the previous installation methods are available to you, we also provide [portable pre-built binaries on GitHub](https://github.com/Automattic/harper/releases).

## Dictionaries

`harper-ls` has three kinds of dictionaries: user, file-local, and static dictionaries. All three dictionaries are combined and used together when spell checking files.

### User Dictionary

Each user of `harper-ls` has their own dictionary, which by default, is located at the following paths on each operating system:

| Operating System |                                                                                Location |
| :--------------- | --------------------------------------------------------------------------------------: |
| Linux            | `$XDG_CONFIG_HOME/harper-ls/dictionary.txt` or `$HOME/.config/harper-ls/dictionary.txt` |
| macOS            |                            `$HOME/Library/Application Support/harper-ls/dictionary.txt` |
| Windows          |                                    `%FOLDERID_RoamingAppData%/harper-ls/dictionary.txt` |

This dictionary is a simple line-separated word list in plaintext. You can add and remove words at will. Code actions on misspelled words allow you to add elements to this list. Additionally, [its location is configurable](#Dictionaries_).

### File-Local Dictionary

Sometimes, you'll encounter a word (or name) that is only valid within the context of a specific file. In this case, you can add this file-specific word to a file-local dictionary using code actions. Any words added to this dictionary will only be included in the combined dictionary when spell checking a file at that specific path.

You can find the file-local dictionaries in the following directories by default on each operation system:

| Operating System |                                                                                         Location |
| :--------------- | -----------------------------------------------------------------------------------------------: |
| Linux            | `$XDG_DATA_HOME/harper-ls/file_dictionaries` or `$HOME/.local/share/harper-ls/file_dictionaries` |
| macOS            |                                  `$HOME/Library/Application Support/harper-ls/file_dictionaries` |
| Windows          |                                            `%FOLDERID_LocalAppData%/harper-ls/file_dictionaries` |

The format of these files is identical to user dictionaries and [their location can also be configured](#Dictionaries_).

### Static Dictionary

The static dictionary is built into the binary and is (as of now) immutable. It contains almost all words you could possibly encounter.

We _do_ take pull requests or issues for adding words to the static dictionary. [Read the documentation on the matter before you do](../contributors/dictionary).

## Code Actions

`harper-ls` has code actions that help in quickly dealing with spelling or grammar errors you encounter. The examples below assume that you have misspelled "contained" as "containes" and have selected it to apply a code action to it.

| Code Action or Command | Description                                                | Example                                     |
| ---------------------- | ---------------------------------------------------------- | ------------------------------------------- |
| Quick Fixes            | Suggests fixes for the selected error                      | `Replace with: "contained"`                 |
| `HarperIgnoreLint`     | Ignores the selected error for the duration of the session | `Ignore Harper error.`                      |
| `HarperAddToUserDict`  | Adds the selected word to the user dictionary              | `Add "containes" to the global dictionary.` |
| `HarperAddToFileDict`  | Adds the selected word to a file-local dictionary          | `Add "containes" to the file dictionary.`   |

## Ignore Comments

`harper-ls` supports skipping comment blocks that contain any of following:

- `harper:ignore`
- `harper: ignore`
- `spellcheck:ignore`
- `spellcheck: ignore`
- `spell-checker:ignore`
- `spell-checker: ignore`
- `spellchecker:ignore`
- `spellchecker: ignore`

You may notice that the last four ignore comments are the same with some of CSpell's ignore comments. That is intentional in case users wish to use Harper and CSpell together.

Here's an example of how these comments can be used:

```js
// harper:ignore this line will not be spellcheckd
function sample() {
	// harper: ignore
	// This line and any other line after it
	// will also not be spellcheckd

	// including this this one
}
```

In the above example, "spellcheckd", "this this", and other spelling or grammar errors will not be flagged.

## Configuration

`harper-ls` expects a JSON object with a `harper-ls` key that contains your configs:

```json
{
	"harper-ls": {
		// Your config goes here...
	}
}
```

### Directories 

| Config         | Type     | Default Value | Description                                                     |
| -------------- | -------- | ------------- | --------------------------------------------------------------- |
| `userDictPath` | `string` | `""`          | Set the file path where the user dictionary is located          |
| `fileDictPath` | `string` | `""`          | Set the directory where the file-local dictionaries are located |
| `ignoredLintsPath` | `string` | `""`          | Set the directory where the ignored lint lists are located |

### Linters

These configs are under the `linters` key:

```json
{
	"harper-ls": {
		"linters": {
			// Your linter configs go here...
		}
	}
}
```

The list of linters together with their descriptions can be found at our [rules page](../rules). All linters are of `boolean` type. Here's an example config with some of them and their default values:

```json
{
	"harper-ls": {
		"linters": {
			"SpellCheck": true,
			"SpelledNumbers": false,
			"AnA": true,
			"SentenceCapitalization": true,
			"UnclosedQuotes": true,
			"WrongQuotes": false,
			"LongSentences": true,
			"RepeatedWords": true,
			"Spaces": true,
			"Matcher": true,
			"CorrectNumberSuffix": true
		}
	}
}
```

### Code Actions

These configs are under the `codeActions` key:

```json
{
	"harper-ls": {
		"codeActions": {
			// Your code action configs go here...
		}
	}
}
```

| Config        | Type      | Default Value | Description                                                                                                                                                    |
| ------------- | --------- | ------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ForceStable` | `boolean` | `false`       | Make code actions appear in "stable" positions by placing code actions that should always be available, like adding misspelled words in the dictionary, first. |

### Markdown

These configs are under the `markdown` key:

```json
{
	"harper-ls": {
		"markdown": {
			// Your Markdown configs go here...
		}
	}
}
```

| Config            | Type      | Default Value | Description              |
| ----------------- | --------- | ------------- | ------------------------ |
| `IgnoreLinkTitle` | `boolean` | `false`       | Skip linting link titles |

### Other Configs

| Config               | Type                                                    | Default Value | Description                                                                                                                                                               |
| -------------------- | ------------------------------------------------------- | ------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `diagnosticSeverity` | `"error"`, `"hint"`, `"information"`, `"warning"`       | `"hint"`      | Configures how severe diagnostics appear in your editor                                                                                                                   |
| `isolateEnglish`     | `boolean`                                               | `false`       | In documents that are a mixture of English and another language, only lint English text. This feature is incredibly new and unstable. Do not expect it to work perfectly. |
| `dialect`            | `"American"`, `"British"`, `"Australian"`, `"Canadian"` | `"American"`  | Set the dialect of English Harper should expect.                                                                                                                          |
| `maxFileLength`      | `number`                                                | `120000`      | Maximum length of file to be linted (in bytes). If a file is larger/longer than this, it will not be linted.                                                              |

## Supported Languages

`harper-ls` supports a wide variety of programming and markup languages.

| Language          |          Language ID          | Comments Only |
| :---------------- | :---------------------------: | ------------: |
| C                 |              `c`              |            ✅ |
| CMake             |            `cmake`            |            ✅ |
| C++               |             `cpp`             |            ✅ |
| C#                |           `csharp`            |            ✅ |
| Dart              |            `dart`             |            ✅ |
| Email             |            `mail`             |               |
| Git Commit        |   `git-commit`/`gitcommit`    |               |
| Go                |             `go`              |            ✅ |
| Haskell           |           `haskell`           |            ✅ |
| HTML              |            `html`             |               |
| Java              |            `java`             |            ✅ |
| JavaScript        |         `javascript`          |            ✅ |
| JavaScript React  |       `javascriptreact`       |            ✅ |
| Literate Haskell  | `literate haskell`/`lhaskell` |               |
| Lua               |             `lua`             |            ✅ |
| Markdown          |          `markdown`           |               |
| Nix               |             `nix`             |            ✅ |
| PHP               |             `php`             |            ✅ |
| Plain Text        |      `plaintext`/`text`       |               |
| Python            |           `python`            |            ✅ |
| Ruby              |            `ruby`             |            ✅ |
| Rust              |            `rust`             |            ✅ |
| Scala             |           `scala`             |            ✅ |
| Shell/Bash Script |         `shellscript`         |            ✅ |
| Solidity          |          `solidity`           |            ✅ |
| Swift             |            `swift`            |            ✅ |
| TOML              |            `toml`             |            ✅ |
| TypeScript        |         `typescript`          |            ✅ |
| TypeScript React  |       `typescriptreact`       |            ✅ |
| Typst             |            `typst`            |               |
| Kotlin            |            `kotlin`           |            ✅ |
| Clojure           |            `clojure`          |            ✅ |

Want your language added?
Let us know by [commenting on this issue](https://github.com/Automattic/harper/issues/79).
