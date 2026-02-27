<script lang="ts">
	import '../app.css';
	import Sidebar from '$lib/components/layout/Sidebar.svelte';
	import TopBar from '$lib/components/layout/TopBar.svelte';
	import StatusBar from '$lib/components/layout/StatusBar.svelte';
	import Toast from '$lib/components/shared/Toast.svelte';
	import { undo, redo } from '$lib/stores/history';

	let { children } = $props();

	function handleKeydown(e: KeyboardEvent) {
		const mod = e.metaKey || e.ctrlKey;
		if (!mod) return;

		if (e.key === 'z' && !e.shiftKey) {
			e.preventDefault();
			undo();
		} else if (e.key === 'y' || (e.key === 'z' && e.shiftKey)) {
			e.preventDefault();
			redo();
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="flex flex-col h-screen">
	<TopBar />
	<div class="flex flex-1 overflow-hidden">
		<Sidebar />
		<main class="flex-1 overflow-auto p-6">
			{@render children()}
		</main>
	</div>
	<StatusBar />
	<Toast />
</div>
