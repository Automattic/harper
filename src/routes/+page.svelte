<script lang="ts">
  import { WorkerLinter } from "harper.js"; 
  import { binaryInlined } from "harper.js/binaryInlined"; 
  import { Editor } from "harper-editor";
  import "harper-editor/style.css"

  let linter = new WorkerLinter({ binary: binaryInlined } );

</script>

<Editor linter={linter} />
