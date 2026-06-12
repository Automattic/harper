---
title: Harper Desktop
---

Harper Desktop aims to (eventually) serve the needs of users on a variety of platforms.
One day, we hope for it to include support for macOS, Windows, and Linux.
Right now, it just supports macOS.
Because we hope to add new platforms, there are structures in the code that would not make sense for an app specifically designed with only one operating system in mind.

This document will serve to give a tour of how the code for the Desktop app is structured and, more importantly, why it is structured that way.

## The Processes

When running, the Harper Desktop app will start two distinct processes.
I will give a brief overview here, then go into more detail in later sections.

The first is a conventional [Tauri](https://tauri.app/) app, with an event loop we do not have control over.
When running the Harper Desktop binary with no arguments, this "main" process is what is kicked off.
If you have not worked with a Tauri app before, I suggest you take a look at their documentation for more details.
In this main process, we try to follow their conventions whenever possible.

When the "Harper Service" is started (which happens when the main process starts, unless the user has disabled it) the main process will kick off a second "highlighter" process by running the same binary with the `highlighter` argument.
This second process is what actually reads and writes text from user applications (using a platform's Accessibility API), performs linting, and renders highlights and suggestion popups over the top of their screen.
We need to separate this highlighter service process from the main process because it needs to maintain its own custom event loop, which was not possible with Tauri.
It also needs to be able to place itself it special "modes" within the operating system, which allow it open windows without taskbar icons or frames.

## The Main Process

As I said above, the main process is a conventional Tauri app.
It holds one primary responsibility, which is to manage configuration and state for the rest of the app.
Everything else it does is supplemental.
That means that it:

- Loads configuration from disk at startup.
- Saves configuration to disk whenever it changes.
- Exposes UI for adjusting the configuration to the user's preference.

The most important module in the Desktop app is likely `config`, which contains a central record of the entire application's state.
Much of the "fluff" code in the project serves to shuffle copies of this config to various parts of the app, including, but not limited to, the settings page WebView, the highlighter process, and to disk.

## The Highlighter Process

This process is much more interesting.
To reiterate: this process's job is to read text from the screen, lint it, and display highlights and suggestion popups.
There are three main components to this:

- Its communication with the main thread.
- Its communication with the operating system via the accessibility API.
- Its communication with the user via an invisible window which it renders on top of every other window on the user's desktop.

It communicates with the main process using the protocol defined in the `communication` module.
It does so to receive updates to it's internal lint configuration so that it matches the main process's state as closely as possible.
When the user changes a setting via a Tauri window, that setting is propagated from the WebView to the main process, which is then shared to the highlighter.
The latter step happens in the `communication` module.

The communication with the operating system happens inside of a `Broker`, which is an interface that exposes any and all platform-specific functions Harper needs.
Right now, there is only one implementation of `Broker`: `MacBroker`.

Finally, the communication with the user happens via a `winit` window, to which we render [`egui`](https://github.com/emilk/egui) elements.
The exact structure of the relevant modules for this is likely subject to change and may vary by platform, so I will forgo including additional information here to avoid misleading you.

I want to emphasize this point: the highlighter process does not store any canonical state. Any updates to its state must be synchronized to the main process as soon as possible.
