<script lang="ts">
import { Link, Textarea } from 'components';

let demoText =
	'Ths is an text box you can type in.\n\nany other site on the web will work the the same!';
</script>

<div class="fixed left-0 top-0 w-screen h-screen bg-white dark:bg-black z-1000">
  <div class="max-w-6xl mx-auto shadow-md border-gray-300 dark:border-x h-full overflow-y-auto">
    <section class="px-6 sm:px-10 py-16 sm:py-24">
      <h1 class="text-center font-bold text-primary text-4xl sm:text-6xl mb-6">
        Thanks for Installing Harper!
      </h1>

      <p class="text-center text-lg sm:text-xl mb-4 max-w-3xl mx-auto">
        Writing is hard. Writing well is harder. Now that Harper is available on most websites you visit, it should be a little easier.
      </p>

      <h2 class="text-center text-3xl mb-4 font-bold max-w-3xl mx-auto">
        Here's How It Works:
      </h2>

      <div class="w-full max-w-5xl mx-auto flex flex-col gap-4 mb-6">
        <div class="w-full p-4 sm:p-6 border rounded-xl shadow-md">
          <h3 class="font-bold text-xl sm:text-2xl text-center py-0 mb-2">Launch Any Site</h3>
          <p>Harper plugs right into the page of any site you visit.</p>
        </div>
        <div class="w-full p-4 sm:p-6 border rounded-xl shadow-md">
          <h3 class="font-bold text-xl sm:text-2xl text-center py-0 mb-2">Type Naturally</h3>
          <video
            class="mt-3 w-full rounded-lg border border-gray-200"
            autoplay
            muted
            loop
            playsinline
            src="/videos/step-2.mp4"
          >
            Your browser does not support embedded videos.
          </video>
        </div>
        <div class="w-full p-4 sm:p-6 border rounded-xl shadow-md">
          <h3 class="font-bold text-xl sm:text-2xl text-center py-0 mb-2">Review & Accept</h3>
          <video
            class="mt-3 w-full rounded-lg border border-gray-200"
            autoplay
            muted
            loop
            playsinline
            src="/videos/step-3.mp4"
          >
            Your browser does not support embedded videos.
          </video>
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
          class="w-full rounded-lg border border-base-300 focus:ring-4 focus:ring-primary/30 transition text-xl! dark:text-white"
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
        If you work somewhere that isn't on our list of supported sites, you can enable the extension anyway by opening the Harper extension popup and clicking the power button.
        <br/>
        <br/>
        Alternatively, <Link href="/request-browser-support">let us know</Link> which sites you want us to support and we'll add it as soon as we can.
      </p>

      <img
        src="/images/chrome_extension_popup.png"
        alt="The Chrome extension’s popup page"
        class="max-w-full h-auto object-contain"
      />
    </div>

    </section>
  </div>
</div>
