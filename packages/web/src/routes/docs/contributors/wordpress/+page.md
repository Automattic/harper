---
title: WordPress Plugin
---

This page will describe most of what you need to know to build and develop the WordPress plugin locally.
You do NOT need to have a WordPress installation on your machine (or hosted on a server, for that matter) to work on the Harper plugin.

Make sure you read the [introduction to contributing](./introduction) before opening a pull request.

## Notes

- The plugin does not have any kind of automated testing.
- You can look at the project's [`justfile`](https://github.com/Automattic/harper/blob/master/justfile) to see exactly what running the `just` recipes below do.

## Prerequisites

Make sure to [set up your environment](./environment).

## Running the Plugin on Your Machine

You should have already run `just setup` to prepare your environment.
All you need to do from here is run `just dev-wp`. This will:

- Download a local copy of the WordPress Playground to your machine
- Build and start watching for changes to the plugin code.
- Run WordPress, mounting the build directory to the Playground instance.

When you make changes to the plugin code, it will be rebuild and you will be able to reload the WordPress page to see your change.

:::info[Remember]
The Harper WordPress plugin only works on the Gutenberg editor.
You will need to draft or edit a post to see the option to open the sidebar.
:::

![Open the Harper Sidebar](/images/harper_wp_sidebar_button.png)
