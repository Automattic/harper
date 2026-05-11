<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";

  const SOURCE_URL = "https://github.com/Automattic/harper";
  const ISSUE_URL = "https://github.com/Automattic/harper/issues/new/choose";
</script>

<section class="about">
        <div class="about-mark">H</div>
        <h1>Harper for Mac</h1>
        <p class="muted">Version 1.4.2 (build 2048)</p>
        <p>
          An open-source grammar checker that runs entirely on your device. No accounts, no
          telemetry, no cloud.
        </p>
        <div class="actions-row center">
          <button class="button" type="button" disabled title="Not wired yet">Release notes</button>
          <button class="button" type="button" on:click={() => void openUrl(SOURCE_URL)}>Source on GitHub</button>
          <button class="button" type="button" on:click={() => void openUrl(ISSUE_URL)}>Report an issue</button>
        </div>
        <div class="about-footer">
          Harper is free software released under the Apache 2.0 license.
          <br />
          Copyright 2026 The Harper Contributors.
        </div>
      </section>
