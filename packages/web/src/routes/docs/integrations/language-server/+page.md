---
title: Language Server
---

`harper-ls` is the [Language Server Protocol](https://microsoft.github.io/language-server-protocol/) frontend for Harper.
Out of the box, it has built-in support for parsing the comments of most programming languages, as well as any and all Markdown files.

## Installation

### Cargo

If you have [Rust installed](https://www.rust-lang.org/tools/install), you're in luck!
To install `harper-ls`, simply run:

```bash
cargo install harper-ls --locked
```

For this to work, make sure that `~/.cargo/bin` is in your system `$PATH`. If you are on a Debian-based Linux distribution, you may need to install `build-essential`.

### Arch Linux

**Stable Release**

The latest stable release is available through the `extra` repo:

```bash
sudo pacman -S harper
```

**Bleeding-Edge**

If you want the latest bleeding-edge, you can install `harper-git` from the [Arch User Repository](https://aur.archlinux.org/packages/harper-git) with your favorite AUR helper:

```bash
paru -S harper-git
# or yay -S harper-git, etc.
```

### Scoop

You can install Harper on Windows through [Scoop](https://scoop.sh/).

```bash
scoop install harper
```

### Homebrew

You may install Harper through [Homebrew](https://brew.sh).

```bash
brew install harper
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

### GitHub Releases

If none of the previous installation methods are available to you, we also provide [portable pre-built binaries on GitHub](https://github.com/Automattic/harper/releases).

## Dictionaries

`harper-ls` has three kinds of dictionaries: user, file-local, and static dictionaries.

### User Dictionary

Each user of `harper-ls` has their own dictionary, located in the following directories on each operating system:

| Operating System |                                       Location |
| :--------------- | ---------------------------------------------: |
| Linux            |                  `$XDG_CONFIG_HOME/harper-ls/` |
| MacOS            | `$HOME/Library/Application Support/harper-ls/` |
| Windows          |             `FOLDERID_LocalAppData/harper-ls/` |

This dictionary is a simple line-separated word list in plain-text.
You can add and remove words at will.
Code actions on misspelled words allow you to add elements to this list.

This was added in response to [issue #89](https://github.com/automattic/harper/issues/89).

### File-Local Dictionary

Sometimes, you'll encounter a word (or name) that is only valid within the context of a specific file.
In this case, you can use the code action that adds the word to the file-local dictionary.
Any words added to this dictionary will, like the name implies, only be included in the dictionary when performing corrections on the file at that specific path.

You can find the file-local dictionaries in the following directories on each operation system:

| Operating System |                                                                                         Location |
| :--------------- | -----------------------------------------------------------------------------------------------: |
| Linux            | `$XDG_DATA_HOME/harper-ls/file_dictionaries` or `$HOME/.local/share/harper-ls/file_dictionaries` |
| MacOS            |                                  `$HOME/Library/Application Support/harper-ls/file_dictionaries` |
| Windows          |                                              `FOLDERID_LocalAppData/harper-ls/file_dictionaries` |

The format of these files is identical to user dictionaries.

### Configuration

Configuration of `harper-ls` varies by editor.
If you use Neovim, [read this documentation](./neovim#Configuration).

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
| Plain Text        |          `plaintext`          |               |
| Python            |           `python`            |            ✅ |
| Ruby              |            `ruby`             |            ✅ |
| Rust              |            `rust`             |            ✅ |
| Shell/Bash Script |         `shellscript`         |            ✅ |
| Swift             |            `swift`            |            ✅ |
| TOML              |            `toml`             |            ✅ |
| TypeScript        |         `typescript`          |            ✅ |
| TypeScript React  |       `typescriptreact`       |            ✅ |
| Typst             |            `typst`            |               |

Want your language added?
Let us know by [commenting on this issue](https://github.com/Automattic/harper/issues/79).

### Static Dictionary

The static dictionary is built into the binary and is (as of now) immutable.
It contains almost all words you could possibly encounter.

I _do_ take pull requests or issues for adding words to the static dictionary.
[Read the documentation on the matter before you do.](../contributors/dictionary)
