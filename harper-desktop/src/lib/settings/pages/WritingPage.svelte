<script lang="ts">
let strictness = 'standard';
let liveCheck = true;
let respectCode = true;
</script>

<section>
        <div class="stanza">
          <div class="eyebrow">Grammar checking</div>
          <p class="section-copy">
            Tune how assertive Harper is. Lower strictness surfaces fewer suggestions.
          </p>
          <div class="rows">
            <div class="inline-row">
              <label for="strictness">Strictness:</label>
              <select
                id="strictness"
                class="select"
                disabled
                title="Not wired yet"
                bind:value={strictness}
              >
                <option value="relaxed">Relaxed</option>
                <option value="standard">Standard</option>
                <option value="strict">Strict</option>
              </select>
            </div>

            <div class="row">
              <div>
                <strong>Check while I type</strong>
                <p>Suggestions appear as you write.</p>
              </div>
              <button
                class:checked={liveCheck}
                class="checkbox"
                type="button"
                role="checkbox"
                disabled
                title="Not wired yet"
                aria-checked={liveCheck}
              >
                {#if liveCheck}<span class="settings-icon icon-check" aria-hidden="true"></span>{/if}
              </button>
            </div>

            <div class="row top">
              <div>
                <strong>Respect code blocks and quotes</strong>
                <p>Harper skips fenced code, inline code, and block quotes when checking Markdown.</p>
              </div>
              <button
                class:checked={respectCode}
                class="checkbox"
                type="button"
                role="checkbox"
                disabled
                title="Not wired yet"
                aria-checked={respectCode}
              >
                {#if respectCode}<span class="settings-icon icon-check" aria-hidden="true"></span>{/if}
              </button>
            </div>
          </div>
        </div>
      </section>
