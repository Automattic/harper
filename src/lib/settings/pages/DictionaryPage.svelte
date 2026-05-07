<script lang="ts">
  import { onMount } from "svelte";
  import { Client } from "$lib/client";

  let dictionary: string[] = [];
  let newDictionaryWord = "";
  let dictionarySearch = "";
  let isDictionaryLoading = true;
  let isDictionarySaving = false;
  let dictionaryError = "";

  $: filteredWords = dictionary.filter((word: string) =>
    word.toLowerCase().includes(dictionarySearch.trim().toLowerCase()),
  );

  onMount(() => {
    void loadDictionary();

    const refreshDictionary = () => {
      if (!isDictionarySaving) {
        void loadDictionary();
      }
    };

    window.addEventListener("focus", refreshDictionary);

    return () => {
      window.removeEventListener("focus", refreshDictionary);
    };
  });

  async function loadDictionary() {
    isDictionaryLoading = true;
    dictionaryError = "";

    try {
      dictionary = await Client.getDictionary();
    } catch (error) {
      dictionaryError = `Unable to load dictionary: ${error}`;
    } finally {
      isDictionaryLoading = false;
    }
  }

  async function saveDictionary(nextDictionary: string[]) {
    const previousDictionary = dictionary;

    dictionary = nextDictionary;
    isDictionarySaving = true;
    dictionaryError = "";

    try {
      await Client.setDictionary(nextDictionary);
    } catch (error) {
      dictionary = previousDictionary;
      dictionaryError = `Unable to save dictionary: ${error}`;
    } finally {
      isDictionarySaving = false;
    }
  }

  function sortDictionary(words: string[]) {
    return [...words].sort((a, b) => a.localeCompare(b));
  }

  async function addDictionaryWord(inputWord: string) {
    const word = inputWord.trim();

    if (!word || dictionary.includes(word)) {
      return;
    }

    const previousDictionary = dictionary;
    dictionary = sortDictionary([...dictionary, word]);
    isDictionarySaving = true;
    dictionaryError = "";

    try {
      await Client.addToDictionary(word);
      dictionary = await Client.getDictionary();
    } catch (error) {
      dictionary = previousDictionary;
      dictionaryError = `Unable to add dictionary word: ${error}`;
    } finally {
      isDictionarySaving = false;
    }
  }

  async function removeDictionaryWord(word: string) {
    await saveDictionary(dictionary.filter((item) => item !== word));
  }

  async function clearDictionary() {
    await saveDictionary([]);
  }

  async function submitDictionaryWord() {
    const word = newDictionaryWord;
    newDictionaryWord = "";
    await addDictionaryWord(word);
  }
</script>

<section>
        <div class="stanza">
          <div class="eyebrow">User Dictionary</div>
          <p class="section-copy">
            Words and names Harper should never flag. This list syncs with Harper's local app config.
          </p>

          {#if isDictionaryLoading}
            <p class="result-summary">Loading dictionary...</p>
          {:else if dictionaryError}
            <p class="result-summary">{dictionaryError}</p>
          {:else if isDictionarySaving}
            <p class="result-summary">Saving dictionary...</p>
          {/if}

          <div class="add-row">
            <input
              class="text-field"
              type="text"
              placeholder="Add a word..."
              disabled={isDictionaryLoading || isDictionarySaving}
              bind:value={newDictionaryWord}
              on:keydown={(event) => event.key === "Enter" && submitDictionaryWord()}
            />
            <button
              class="button primary"
              type="button"
              disabled={isDictionaryLoading || isDictionarySaving}
              on:click={submitDictionaryWord}
            >Add</button>
          </div>

          <div class="list-card">
            <div class="search-strip">
              <span class="settings-icon icon-search" aria-hidden="true"></span>
              <input
                type="text"
                placeholder={`Search ${dictionary.length} words`}
                bind:value={dictionarySearch}
              />
              <span>{filteredWords.length} of {dictionary.length}</span>
            </div>

            <div class="dictionary-list">
              {#if filteredWords.length === 0}
                <div class="empty">No matching words.</div>
              {:else}
                {#each filteredWords as word}
                  <div class="list-row">
                    <code>{word}</code>
                    <button
                      class="icon-button danger"
                      type="button"
                      disabled={isDictionaryLoading || isDictionarySaving}
                      aria-label={`Remove ${word}`}
                      on:click={() => removeDictionaryWord(word)}
                    >
                      <span class="settings-icon icon-x" aria-hidden="true"></span>
                    </button>
                  </div>
                {/each}
              {/if}
            </div>
          </div>

          <div class="actions-row">
            <button class="button" type="button" disabled title="Not wired yet">Import from file...</button>
            <button class="button" type="button" disabled title="Not wired yet">Export dictionary</button>
            <span class="spacer"></span>
            <button
              class="button danger"
              type="button"
              disabled={isDictionaryLoading || isDictionarySaving}
              on:click={clearDictionary}
            >
              Clear all
            </button>
          </div>
        </div>
      </section>
