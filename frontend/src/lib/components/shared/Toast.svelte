<script lang="ts">
	import { toasts } from '$lib/stores/project';
</script>

{#if $toasts.length > 0}
	<div class="fixed bottom-12 right-4 z-50 flex flex-col gap-2">
		{#each $toasts as toast (toast.id)}
			<div
				class="px-4 py-2 rounded shadow-lg text-sm text-white max-w-80 animate-fade-in"
				class:bg-blue-600={toast.type === 'info'}
				class:bg-green-600={toast.type === 'success'}
				class:bg-red-600={toast.type === 'error'}
				class:bg-yellow-600={toast.type === 'warning'}
			>
				{toast.message}
			</div>
		{/each}
	</div>
{/if}
