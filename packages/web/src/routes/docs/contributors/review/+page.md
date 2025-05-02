---
title: Reviewing Pull Requests
---

There are a lot of individual components and artifacts that make up Harper.
How a patch gets reviewed depends significantly on which component or artifact it affects.
This page seeks to document the tooling available for downloading and testing patches on a local machine.

## Patches to `harper-core`

If a patch only affects a grammar rule, it should only be touching `harper-core`.
This means you can test the change using any Harper frontend (of which there are many).

### Using GitHub Actions Artifacts

We run builds for a variety of platforms whenever a Pull Request is pushed to.
You can use these to review changes to various aspects of Harper, including `harper-ls`, `harper-debug`, and the Visual Studio Code plugin.

![How to download the Windows Visual Studio Code plugin from the GitHub Actions run.](/images/download_artifact.gif)

### Testing Using Cargo and `harper-debug`

Most of our build tooling exists for Harper's various integrations.
If you are testing `harper-core`, you can skip all the fluff and compile the patch using [Cargo](https://doc.rust-lang.org/cargo/) directly.

```bash
cargo install --git https://github.com/automattic/harper --branch <branch-name> <binary-artifact> --locked
```

For example, for [PR #445](https://github.com/Automattic/harper/pull/455), we can install the patched version of the `harper-debug` debug tool with the following command:

```bash
cargo install --git https://github.com/automattic/harper --branch somewhat-something harper-debug --locked
```

From there, you can run the tool on any file with `harper-debug lint <path>`.

### Testing Via the Docker Image

We build our web documentation in a Docker image.
This documentation includes a [demo](/), so you can also use this image to review changes to linting rules and other aspects of the core algorithm.

```bash
git clone https://github.com/automattic/harper
cd harper
git switch <branch from PR>
IMAGE_HASH=$(docker build . -q)
docker run -p 3000:3000 -it $IMAGE_HASH
```

From there, open up `http://localhost:3000` in your web browser of choice and use the text area to test the change.
