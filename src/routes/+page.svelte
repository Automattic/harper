<script lang="ts">
  import "../app.css";
  import { WorkerLinter } from "harper.js"; 
  import { binaryInlined } from "harper.js/binaryInlined"; 
  import { Editor } from "harper-editor";
  import "harper-editor/style.css"

  let linter = new WorkerLinter({ binary: binaryInlined } );

</script>

<div class="h-screen w-screen">
  <Editor linter={linter} />
</div>
