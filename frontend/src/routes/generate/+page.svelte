<script lang="ts">
	import { project, validationResult, showToast, isLoading } from '$lib/stores/project';
	import * as api from '$lib/api';
	import type { SheetGcode } from '$lib/api';

	let gcodePreview: string = $state('');
	let generatedSheets: SheetGcode[] = $state([]);
	let previewMaterialIdx: number = $state(0);
	let previewSheetIdx: number = $state(0);

	async function handleValidate() {
		try {
			isLoading.set(true);
			const result = await api.validateProjectCmd();
			validationResult.set(result);
			if (result.errors.length === 0) {
				showToast('Validation passed', 'success');
			} else {
				showToast(`${result.errors.length} error(s) found`, 'error');
			}
		} catch (e) {
			showToast(`Error: ${e}`, 'error');
		} finally {
			isLoading.set(false);
		}
	}

	async function handleGenerate() {
		try {
			isLoading.set(true);
			generatedSheets = await api.generateGcode();
			showToast(`Generated ${generatedSheets.length} G-code file(s)`, 'success');
		} catch (e) {
			showToast(`Error: ${e}`, 'error');
		} finally {
			isLoading.set(false);
		}
	}

	function downloadSheet(sheet: SheetGcode) {
		const blob = new Blob([sheet.gcode], { type: 'text/plain' });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = sheet.filename;
		document.body.appendChild(a);
		a.click();
		document.body.removeChild(a);
		URL.revokeObjectURL(url);
	}

	function downloadAll() {
		for (const sheet of generatedSheets) {
			downloadSheet(sheet);
		}
	}

	async function handlePreview() {
		try {
			isLoading.set(true);
			gcodePreview = await api.previewGcode(previewMaterialIdx, previewSheetIdx);
		} catch (e) {
			showToast(`Error: ${e}`, 'error');
			gcodePreview = '';
		} finally {
			isLoading.set(false);
		}
	}
</script>

{#if $project}
	<div>
		<h1 class="text-2xl font-bold mb-6">Generate G-code</h1>

		<div class="grid grid-cols-2 gap-6 mb-6">
			<!-- Validation -->
			<div class="p-4 bg-surface rounded-lg border border-border">
				<h2 class="text-lg font-semibold mb-3">Validation</h2>
				<button
					class="px-4 py-2 text-sm bg-surface-hover hover:bg-accent/50 rounded mb-3"
					onclick={handleValidate}
				>
					Validate Project
				</button>

				{#if $validationResult}
					{#if $validationResult.errors.length === 0 && $validationResult.warnings.length === 0}
						<div class="text-success text-sm">All checks passed</div>
					{/if}
					{#if $validationResult.errors.length > 0}
						<div class="text-error text-sm mb-2">{$validationResult.errors.length} error(s)</div>
						{#each $validationResult.errors as err}
							<div class="text-xs text-red-300 mb-1">{JSON.stringify(err)}</div>
						{/each}
					{/if}
					{#if $validationResult.warnings.length > 0}
						<div class="text-warning text-sm mt-2 mb-2">{$validationResult.warnings.length} warning(s)</div>
						{#each $validationResult.warnings as warn}
							<div class="text-xs text-yellow-300 mb-1">{JSON.stringify(warn)}</div>
						{/each}
					{/if}
				{/if}
			</div>

			<!-- Generate -->
			<div class="p-4 bg-surface rounded-lg border border-border">
				<h2 class="text-lg font-semibold mb-3">Generate</h2>
				<button
					class="px-4 py-2 text-sm bg-accent hover:bg-accent/80 text-white rounded mb-3"
					onclick={handleGenerate}
				>
					Generate G-code
				</button>

				{#if generatedSheets.length > 0}
					<div class="text-sm text-success mb-2">Generated {generatedSheets.length} file(s):</div>
					<ul class="text-xs text-text-secondary space-y-1">
						{#each generatedSheets as sheet}
							<li class="flex items-center gap-2">
								<span class="font-mono">{sheet.filename}</span>
								<button class="text-accent hover:underline" onclick={() => downloadSheet(sheet)}>Download</button>
							</li>
						{/each}
					</ul>
					<button class="mt-2 px-3 py-1 text-xs bg-surface hover:bg-surface-hover rounded" onclick={downloadAll}>Download All</button>
				{/if}
			</div>
		</div>

		<!-- G-code Preview -->
		<div class="p-4 bg-surface rounded-lg border border-border">
			<div class="flex items-center justify-between mb-3">
				<h2 class="text-lg font-semibold">G-code Preview</h2>
				<div class="flex items-center gap-2">
					<label class="text-xs text-text-secondary">
						Material:
						<input type="number" min="0" bind:value={previewMaterialIdx} class="w-12 px-1 py-0.5 bg-bg border border-border rounded text-xs text-text-primary ml-1" />
					</label>
					<label class="text-xs text-text-secondary">
						Sheet:
						<input type="number" min="0" bind:value={previewSheetIdx} class="w-12 px-1 py-0.5 bg-bg border border-border rounded text-xs text-text-primary ml-1" />
					</label>
					<button class="px-3 py-1 text-xs bg-surface-hover hover:bg-accent/50 rounded" onclick={handlePreview}>
						Preview
					</button>
				</div>
			</div>

			{#if gcodePreview}
				<pre class="bg-bg-secondary rounded p-4 overflow-auto max-h-96 text-xs font-mono text-green-300 leading-relaxed">{gcodePreview}</pre>
			{:else}
				<p class="text-text-secondary text-sm">Click "Preview" to see generated G-code.</p>
			{/if}
		</div>
	</div>
{:else}
	<div class="text-text-secondary">
		<p>No project loaded. <a href="/" class="text-accent hover:underline">Go home</a> to create or open one.</p>
	</div>
{/if}
