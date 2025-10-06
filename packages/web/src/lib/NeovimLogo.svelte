<script lang="ts">
export let width = '100%';
export let height = '100%';
</script>

<svg {width} {height} viewBox="0 0 602 734" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <linearGradient id="g1" x1="50%" y1="0%" x2="50%" y2="100%">
      <stop offset="0%"  stop-color="#16B0ED" stop-opacity="0.80"/>
      <stop offset="100%" stop-color="#0F59B2" stop-opacity="0.84"/>
    </linearGradient>
    <linearGradient id="g2" x1="50%" y1="0%" x2="50%" y2="100%">
      <stop offset="0%" stop-color="#7DB643"/>
      <stop offset="100%" stop-color="#367533"/>
    </linearGradient>
    <linearGradient id="g3" x1="50%" y1="0%" x2="50%" y2="100%">
      <stop offset="0%" stop-color="#88C649" stop-opacity="0.8"/>
      <stop offset="100%" stop-color="#439240" stop-opacity="0.84"/>
    </linearGradient>
  </defs>
  <g fill="none" stroke="none" fill-rule="evenodd">
    <!-- left green section -->
    <path d="M0 155.57 155 0v733.43L0 578.24V155.57z" fill="url(#g1)"/>
    <!-- right green section -->
    <path d="M598 155.57 443 0v733.43l155-155.19V155.57z" fill="url(#g2)" transform="translate(598 366.71) scale(-1 1) translate(0px -366.71)"/>
    <!-- central cross -->
    <path d="M155 0l403 615.19-112.78 112.81L42 114.17z" fill="url(#g3)"/>
    <!-- subtle shadow -->
    <path d="M155 283.83l-.21 24.17L31 124.71l11.46-11.71z" fill="#000" opacity="0.13"/>
  </g>
</svg>
