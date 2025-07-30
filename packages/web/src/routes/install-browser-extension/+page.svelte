<script lang="ts">
import { Textarea } from 'flowbite-svelte';

let demoText =
	'Ths is an text box you can type in.\n\nany other site on the web will work the the same!';
</script>

<div class="fixed left-0 top-0 w-screen h-screen bg-white dark:bg-black z-1000">
  <div class="max-w-6xl mx-auto shadow-md border-gray-300 dark:border-x h-full overflow-y-auto">
    <section class="px-6 sm:px-10 py-16 sm:py-24">
      <h1 class="text-center font-bold text-primary text-4xl sm:text-6xl leading-tight mb-6">
        Thanks for Installing Harper!
      </h1>

      <p class="text-center text-lg sm:text-xl mb-4 max-w-3xl mx-auto">
        Writing is hard. Writing well is harder. Now that Harper is available on most websites you visit, it should be a little easier.
      </p>

      <h2 class="text-center text-3xl mb-4 font-bold max-w-3xl mx-auto">
        Here's How It Works:
      </h2>

      <div class="grid grid-cols-3 gap-4 mb-2">
        <div class="p-4 border rounded-xl shadow-md">
          <p class="font-semibold py-0">1. Launch Any Site</p>
          <p>Harper plugs right into the page of any site you visit.</p>
        </div>
        <div class="p-4 border rounded-xl shadow-md">
          <p class="font-semibold py-0">2. Type Naturally</p>
          <p>When Harper notices an error in your work, it'll underline it.</p>
        </div>
        <div class="p-4 border rounded-xl shadow-md">
          <p class="font-semibold py-0">3. Review & Accept</p>
          <p>Tap the underline to fix the problem.</p>
        </div>
      </div>


      <h2 class="text-center text-3xl mb-4 font-bold max-w-3xl mx-auto">
        Give It a Whirl!
      </h2>

      <h3 class="text-center text-xl mb-4 max-w-xl mx-auto">
        It may take a moment for the engine to start...
      </h3>

      <div class="md:col-span-2 relative">
        <Textarea
          rows={10}
          bind:value={demoText}
          class="w-full rounded-lg border border-base-300 focus:ring-4 focus:ring-primary/30 transition text-xl!"
        />
      </div>


      <h2 class="text-center text-3xl mb-4 font-bold max-w-3xl mx-auto">
        Nothing is Happening?
      </h2>


    <div class="flex flex-row">
      <p class="text-left text-lg sm:text-xl mb-4 max-w-3xl mx-auto">
        Harper will only enable itself automatically on sites we've tested before. 
        <br/>
        <br/>
        If you work somewhere that isn't on our list of supported sites, you can enable the Chrome extension anyway by opening the Harper extension popup and clicking the power button.
        <br/>
        <br/>
        Alternatively, <a href="/request-browser-support">let us know</a> which sites you want us to support and we'll add it as soon as we can.
      </p>

      <img
        src="/images/chrome_extension_popup.png"
        alt="The Chrome extension’s popup page"
        class="max-w-full h-auto object-contain"
      />
    <div>

    </section>
  </div>
</div>
