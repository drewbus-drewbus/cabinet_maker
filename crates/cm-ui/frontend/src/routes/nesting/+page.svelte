<script lang="ts">
	import { project, nestingResults, showToast, isLoading } from '$lib/stores/project';
	import { selectedMaterialIndex, selectedSheetIndex } from '$lib/stores/ui';
	import * as api from '$lib/api';

	let svgContent: string = $state('');

	async function handleNest() {
		if (!$project) return;
		try {
			isLoading.set(true);
			const results = await api.nestAll();
			nestingResults.set(results);
			selectedMaterialIndex.set(0);
			selectedSheetIndex.set(0);
			if (results.length > 0) {
				await loadSvg();
			}
			showToast(`Nested onto ${results.reduce((sum, g) => sum + g.nesting_result.sheet_count, 0)} sheet(s)`, 'success');
		} catch (e) {
			showToast(`Error: ${e}`, 'error');
		} finally {
			isLoading.set(false);
		}
	}

	async function loadSvg() {
		try {
			svgContent = await api.getNestingSvg($selectedMaterialIndex, $selectedSheetIndex);
		} catch (e) {
			svgContent = '';
			showToast(`Error loading SVG: ${e}`, 'error');
		}
	}

	$effect(() => {
		if ($nestingResults.length > 0) {
			loadSvg();
		}
	});

	const currentGroup = $derived($nestingResults[$selectedMaterialIndex]);
	const currentResult = $derived(currentGroup?.nesting_result);
</script>

{#if $project}
	<div>
		<div class="flex items-center justify-between mb-4">
			<h1 class="text-2xl font-bold">Nesting</h1>
			<button
				class="px-4 py-2 text-sm bg-accent hover:bg-accent/80 text-white rounded"
				onclick={handleNest}
			>
				Nest All Parts
			</button>
		</div>

		{#if $nestingResults.length > 0}
			<!-- Material tabs -->
			<div class="flex gap-2 mb-4">
				{#each $nestingResults as group, i}
					<button
						class="px-3 py-1 text-xs rounded"
						class:bg-accent={$selectedMaterialIndex === i}
						class:text-white={$selectedMaterialIndex === i}
						class:bg-surface={$selectedMaterialIndex !== i}
						class:hover:bg-surface-hover={$selectedMaterialIndex !== i}
						onclick={() => { selectedMaterialIndex.set(i); selectedSheetIndex.set(0); }}
					>
						{group.material_name} ({group.thickness}")
					</button>
				{/each}
			</div>

			{#if currentResult}
				<!-- Sheet selector -->
				<div class="flex items-center gap-4 mb-4">
					<div class="flex gap-1">
						{#each currentResult.sheets as _, i}
							<button
								class="px-2 py-1 text-xs rounded"
								class:bg-surface-hover={$selectedSheetIndex === i}
								class:bg-surface={$selectedSheetIndex !== i}
								onclick={() => selectedSheetIndex.set(i)}
							>
								Sheet {i + 1}
							</button>
						{/each}
					</div>

					<div class="text-xs text-text-secondary">
						{currentResult.sheet_count} sheet(s) | {currentResult.overall_utilization.toFixed(1)}% utilization
						{#if currentResult.unplaced.length > 0}
							| <span class="text-error">{currentResult.unplaced.length} unplaced</span>
						{/if}
					</div>
				</div>

				<!-- Stats for current sheet -->
				{#if currentResult.sheets[$selectedSheetIndex]}
					{@const sheet = currentResult.sheets[$selectedSheetIndex]}
					<div class="flex gap-4 mb-4 text-xs text-text-secondary">
						<span>Parts: {sheet.parts.length}</span>
						<span>Utilization: {sheet.utilization.toFixed(1)}%</span>
						<span>Waste: {sheet.waste_area.toFixed(1)} sq in</span>
					</div>
				{/if}

				<!-- SVG preview -->
				<div class="bg-white rounded-lg p-4 overflow-auto max-h-[500px]">
					{@html svgContent}
				</div>
			{/if}
		{:else}
			<p class="text-text-secondary">Click "Nest All Parts" to arrange parts on sheets.</p>
		{/if}
	</div>
{:else}
	<div class="text-text-secondary">
		<p>No project loaded. <a href="/" class="text-accent hover:underline">Go home</a> to create or open one.</p>
	</div>
{/if}
