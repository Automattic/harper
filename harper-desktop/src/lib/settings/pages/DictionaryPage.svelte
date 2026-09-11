<script lang="ts">
import { Button, CloseIcon, IconButton, Input, Panel, SearchField, SearchIcon } from 'components';
import { onMount } from 'svelte';
import { Client } from '$lib/client';

let dictionary: string[] = [];
let newDictionaryWord = '';
let dictionarySearch = '';
let isDictionaryLoading = true;
let isDictionarySaving = false;
let dictionaryError = '';
let importInput: HTMLInputElement;

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

	window.addEventListener('focus', refreshDictionary);

	return () => {
		window.removeEventListener('focus', refreshDictionary);
	};
});

async function loadDictionary() {
	isDictionaryLoading = true;
	dictionaryError = '';

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
	dictionaryError = '';

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

function parseDictionaryText(text: string) {
	return text
		.split(/\r?\n/)
		.map((word) => word.trim())
		.filter(Boolean);
}

function mergeDictionaryWords(words: string[]) {
	return sortDictionary([...new Set([...dictionary, ...words])]);
}

async function addDictionaryWord(inputWord: string) {
	const word = inputWord.trim();

	if (!word || dictionary.includes(word)) {
		return;
	}

	const previousDictionary = dictionary;
	dictionary = sortDictionary([...dictionary, word]);
	isDictionarySaving = true;
	dictionaryError = '';

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

async function importDictionary(event: Event) {
	const input = event.currentTarget as HTMLInputElement;
	const file = input.files?.[0];

	if (!file) {
		return;
	}

	try {
		const importedWords = parseDictionaryText(await file.text());
		await saveDictionary(mergeDictionaryWords(importedWords));
	} catch (error) {
		dictionaryError = `Unable to import dictionary: ${error}`;
	} finally {
		input.value = '';
	}
}

function exportDictionary() {
	try {
		const blob = new Blob([`${dictionary.join('\n')}\n`], { type: 'text/plain;charset=utf-8' });
		const url = URL.createObjectURL(blob);
		const link = document.createElement('a');

		link.href = url;
		link.download = 'harper-dictionary.txt';
		link.click();
		URL.revokeObjectURL(url);
	} catch (error) {
		dictionaryError = `Unable to export dictionary: ${error}`;
	}
}

async function submitDictionaryWord() {
	const word = newDictionaryWord;
	newDictionaryWord = '';
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
            <Input
              unstyled
              class="text-field"
              type="text"
              placeholder="Add a word..."
              disabled={isDictionaryLoading || isDictionarySaving}
              bind:value={newDictionaryWord}
              on:keydown={(event) => event.detail.key === "Enter" && submitDictionaryWord()}
            />
            <Button
              unstyled
              class="button primary"
              type="button"
              disabled={isDictionaryLoading || isDictionarySaving}
              on:click={submitDictionaryWord}
            >Add</Button>
          </div>

          <Panel>
            <SearchField
              class="search-strip"
              placeholder={`Search ${dictionary.length} words`}
              bind:value={dictionarySearch}
            >
              <SearchIcon slot="leading" className="control-icon" />
              <span slot="trailing">{filteredWords.length} of {dictionary.length}</span>
            </SearchField>

            <div class="dictionary-list">
              {#if filteredWords.length === 0}
                <div class="empty">No matching words.</div>
              {:else}
                {#each filteredWords as word}
                  <div class="list-row">
                    <code>{word}</code>
                    <IconButton
                      danger
                      disabled={isDictionaryLoading || isDictionarySaving}
                      aria-label={`Remove ${word}`}
                      on:click={() => removeDictionaryWord(word)}
                    >
                      <CloseIcon className="control-icon" />
                    </IconButton>
                  </div>
                {/each}
              {/if}
            </div>
          </Panel>

          <div class="actions-row">
            <input
              bind:this={importInput}
              type="file"
              accept=".txt,.dic,text/plain"
              hidden
              on:change={importDictionary}
            />
            <Button
              unstyled
              class="button"
              type="button"
              disabled={isDictionaryLoading || isDictionarySaving}
              on:click={() => importInput.click()}
            >Import from file...</Button>
            <Button
              unstyled
              class="button"
              type="button"
              disabled={isDictionaryLoading || isDictionarySaving || dictionary.length === 0}
              on:click={exportDictionary}
            >Export dictionary</Button>
            <span class="spacer"></span>
            <Button
              unstyled
              class="button danger"
              type="button"
              disabled={isDictionaryLoading || isDictionarySaving}
              on:click={clearDictionary}
            >
              Clear all
            </Button>
          </div>
        </div>
      </section>
