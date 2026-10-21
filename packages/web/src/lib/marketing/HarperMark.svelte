<script lang="ts">
export let size = 32;
export let title = 'Harper';

$: width = Math.round(size * (695 / 411));
</script>

<svg
	{width}
	height={size}
	viewBox="0 0 695 411"
	role="img"
	aria-label={title}
	xmlns="http://www.w3.org/2000/svg"
	class="harper-mark"
>
		<g transform="matrix(1,0,0,1,-5,-1720)">
		<g transform="matrix(0.824576,0,0,0.749254,0.365685,430.856)">
			<rect x="5.62" y="1720.57" width="842.425" height="547.24" fill="none" stroke="none" />
			<g transform="matrix(1.21274,0,0,1.33466,-2183.71,393.157)">
				<g transform="matrix(1,0,0,1,-22.3927,1.08043)">
					<path
						d="M1930.93,1121.75C1930.93,1121.75 1974.66,1080.73 2041.34,1094.1C2086.61,1103.18 2122.83,1145.4 2122.83,1145.4"
					/>
				</g>
				<g transform="matrix(1,0,0,1,-47.3485,12.3935)">
					<path
						d="M2250.3,1107.1C2250.3,1107.1 2261.8,1065.58 2311.59,1047.05C2361.62,1028.44 2422.42,1051.13 2422.42,1051.13"
					/>
				</g>
				<g transform="matrix(1.10085,0,0,1.10085,-212.096,-122.054)">
					<g transform="matrix(1,0,0,1,14.3186,-0.853887)">
						<ellipse cx="1981.62" cy="1247.49" rx="87.401" ry="87.881" />
					</g>
					<rect x="2083.34" y="1231.11" width="66.702" height="15.521" />
					<rect x="1892.23" y="1208.69" width="16.306" height="30.182" />
					<g transform="matrix(-1,0,0,1,4281.79,-0.853887)">
						<ellipse cx="1981.62" cy="1247.49" rx="87.401" ry="87.881" />
					</g>
					<g transform="matrix(-1,0,0,1,4296.11,0)">
						<rect x="2083.34" y="1231.11" width="66.702" height="15.521" />
					</g>
					<g transform="matrix(-1,0,0,1,4296.11,0)">
						<rect x="1892.23" y="1208.69" width="16.306" height="30.182" />
					</g>
				</g>
			</g>
		</g>
	</g>
</svg>

<style>
.harper-mark {
	display: block;
	fill: none;
	stroke: currentColor;
	stroke-linecap: round;
	stroke-linejoin: round;
	stroke-miterlimit: 1.5;
	stroke-width: 22.92px;
}
</style>
