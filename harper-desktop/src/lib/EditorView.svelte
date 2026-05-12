<script lang="ts">
import { WorkerLinter } from 'harper.js';
import { binaryInlined } from 'harper.js/binaryInlined';
import { Editor } from 'harper-editor';
import 'harper-editor/style.css';

const linter = new WorkerLinter({ binary: binaryInlined });
</script>

<div class="h-screen w-screen">
  <Editor {linter} />
</div>
