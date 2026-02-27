<script lang="ts">
	import { project, cachedParts, isLoading, validationResult } from '$lib/stores/project';

	const partCount = $derived($cachedParts.length);
	const errorCount = $derived($validationResult?.errors?.length ?? 0);
	const warningCount = $derived($validationResult?.warnings?.length ?? 0);
</script>

<footer class="flex items-center justify-between px-4 py-1 bg-bg-secondary border-t border-border text-xs text-text-secondary">
	<div class="flex items-center gap-4">
		{#if $project}
			<span>Units: {$project.project.units}</span>
			<span>Materials: {$project.materials.length + ($project.material ? 1 : 0)}</span>
			<span>Parts: {partCount}</span>
		{:else}
			<span>No project loaded</span>
		{/if}
	</div>

	<div class="flex items-center gap-4">
		{#if errorCount > 0}
			<span class="text-error">{errorCount} error{errorCount !== 1 ? 's' : ''}</span>
		{/if}
		{#if warningCount > 0}
			<span class="text-warning">{warningCount} warning{warningCount !== 1 ? 's' : ''}</span>
		{/if}
		{#if $isLoading}
			<span class="animate-pulse">Working...</span>
		{/if}
	</div>
</footer>
