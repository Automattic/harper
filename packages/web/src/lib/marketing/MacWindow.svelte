<script lang="ts">
export let title: string;
export let dark = false;
</script>

<div class:dark class="mini-window">
	<div class="titlebar">
		<div class="lights"><span></span><span></span><span></span></div>
		<strong>{title}</strong>
	</div>
	<div class="body"><slot /></div>
</div>

<style>
.mini-window {
	position: relative;
	overflow: hidden;
	border: 0.5px solid rgba(28, 26, 22, 0.18);
	border-radius: 0.75rem;
	background: #fff;
	box-shadow:
		0 22px 50px -18px rgba(28, 26, 22, 0.35),
		0 4px 10px rgba(28, 26, 22, 0.1),
		0 0 0 0.5px rgba(0, 0, 0, 0.04);
	font-family: inherit;
}

.mini-window.dark {
	background: #2b2d31;
	color: #dcddde;
}

.titlebar {
	display: flex;
	align-items: center;
	gap: 0.5rem;
	height: 1.65rem;
	border-bottom: 0.5px solid rgba(28, 26, 22, 0.08);
	background: linear-gradient(180deg, #fafafa, #efece7);
	padding: 0 0.65rem;
}

.dark .titlebar {
	border-color: rgba(255, 255, 255, 0.08);
	background: #1e1f22;
}

.lights {
	display: flex;
	gap: 0.35rem;
}

.lights span {
	width: 0.55rem;
	height: 0.55rem;
	border-radius: 999px;
}

.lights span:nth-child(1) {
	background: #ff5f57;
}

.lights span:nth-child(2) {
	background: #febc2e;
}

.lights span:nth-child(3) {
	background: #28c840;
}

.titlebar strong {
	flex: 1;
	margin-right: 2.1rem;
	color: rgba(28, 26, 22, 0.56);
	font-size: 0.68rem;
	text-align: center;
}

.dark .titlebar strong {
	color: rgba(255, 255, 255, 0.7);
}

.body {
	position: relative;
	min-height: 10rem;
}
</style>
