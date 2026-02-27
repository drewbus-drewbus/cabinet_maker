<script lang="ts">
	import { project, cutlistRows, showToast, isLoading, validationResult } from '$lib/stores/project';
	import * as api from '$lib/api';

	let sortColumn: string = $state('cabinet');
	let sortAsc: boolean = $state(true);

	async function loadCutlist() {
		if (!$project) return;
		try {
			isLoading.set(true);
			const rows = await api.getCutlist();
			cutlistRows.set(rows);
		} catch (e) {
			showToast(`Error: ${e}`, 'error');
		} finally {
			isLoading.set(false);
		}
	}

	async function loadValidation() {
		if (!$project) return;
		try {
			const result = await api.validateProjectCmd();
			validationResult.set(result);
		} catch (e) {
			showToast(`Validation error: ${e}`, 'error');
		}
	}

	async function handleExportCsv() {
		try {
			await api.exportCsv();
			showToast('CSV downloaded', 'success');
		} catch (e) {
			showToast(`Error: ${e}`, 'error');
		}
	}

	async function handleExportJson() {
		try {
			await api.exportBomJson();
			showToast('BOM JSON downloaded', 'success');
		} catch (e) {
			showToast(`Error: ${e}`, 'error');
		}
	}

	function sortBy(col: string) {
		if (sortColumn === col) {
			sortAsc = !sortAsc;
		} else {
			sortColumn = col;
			sortAsc = true;
		}
	}

	const sortedRows = $derived(
		[...$cutlistRows].sort((a, b) => {
			const av = (a as Record<string, unknown>)[sortColumn];
			const bv = (b as Record<string, unknown>)[sortColumn];
			const cmp = typeof av === 'string' ? (av as string).localeCompare(bv as string) : (av as number) - (bv as number);
			return sortAsc ? cmp : -cmp;
		})
	);

	$effect(() => {
		if ($project) {
			loadCutlist();
			loadValidation();
		}
	});
</script>

{#if $project}
	<div>
		<div class="flex items-center justify-between mb-4">
			<h1 class="text-2xl font-bold">Cut List</h1>
			<div class="flex gap-2">
				<button class="px-3 py-1 text-xs bg-surface hover:bg-surface-hover rounded" onclick={loadCutlist}>Refresh</button>
				<button class="px-3 py-1 text-xs bg-surface hover:bg-surface-hover rounded" onclick={handleExportCsv}>Export CSV</button>
				<button class="px-3 py-1 text-xs bg-surface hover:bg-surface-hover rounded" onclick={handleExportJson}>Export JSON</button>
			</div>
		</div>

		<!-- Validation -->
		{#if $validationResult}
			{#if $validationResult.errors.length > 0}
				<div class="mb-4 p-3 bg-red-900/30 border border-red-600/50 rounded">
					<h3 class="text-sm font-semibold text-error mb-1">Errors ({$validationResult.errors.length})</h3>
					{#each $validationResult.errors as err}
						<div class="text-xs text-red-300">{JSON.stringify(err)}</div>
					{/each}
				</div>
			{/if}
			{#if $validationResult.warnings.length > 0}
				<div class="mb-4 p-3 bg-yellow-900/30 border border-yellow-600/50 rounded">
					<h3 class="text-sm font-semibold text-warning mb-1">Warnings ({$validationResult.warnings.length})</h3>
					{#each $validationResult.warnings as warn}
						<div class="text-xs text-yellow-300">{JSON.stringify(warn)}</div>
					{/each}
				</div>
			{/if}
		{/if}

		{#if sortedRows.length > 0}
			<div class="overflow-auto">
				<table class="w-full text-sm">
					<thead>
						<tr class="text-text-secondary border-b border-border">
							{#each ['cabinet', 'label', 'material', 'width', 'height', 'thickness', 'quantity'] as col}
								<th class="py-2 px-2 text-left cursor-pointer hover:text-text-primary" onclick={() => sortBy(col)}>
									{col.charAt(0).toUpperCase() + col.slice(1)}
									{#if sortColumn === col}{sortAsc ? ' ^' : ' v'}{/if}
								</th>
							{/each}
							<th class="py-2 px-2 text-left">Operations</th>
						</tr>
					</thead>
					<tbody>
						{#each sortedRows as row}
							<tr class="border-b border-border/30 hover:bg-surface/50">
								<td class="py-1 px-2">{row.cabinet}</td>
								<td class="py-1 px-2 font-medium">{row.label}</td>
								<td class="py-1 px-2">{row.material}</td>
								<td class="py-1 px-2 text-right font-mono">{row.width.toFixed(3)}"</td>
								<td class="py-1 px-2 text-right font-mono">{row.height.toFixed(3)}"</td>
								<td class="py-1 px-2 text-right font-mono">{row.thickness.toFixed(3)}"</td>
								<td class="py-1 px-2 text-center">{row.quantity}</td>
								<td class="py-1 px-2 text-xs text-text-secondary">{row.operations.join(', ') || '-'}</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
			<div class="mt-3 text-xs text-text-secondary">
				{sortedRows.length} parts total
			</div>
		{:else}
			<p class="text-text-secondary">No parts generated yet. Go to Cabinets and click "Generate All Parts".</p>
		{/if}
	</div>
{:else}
	<div class="text-text-secondary">
		<p>No project loaded. <a href="/" class="text-accent hover:underline">Go home</a> to create or open one.</p>
	</div>
{/if}
