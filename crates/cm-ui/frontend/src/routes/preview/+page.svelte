<script lang="ts">
	import { project, showToast } from '$lib/stores/project';
	import { selectedCabinetIndex } from '$lib/stores/ui';
	import * as api from '$lib/api';
	import CabinetScene from '$lib/components/three/CabinetScene.svelte';
	import type { Panel3D } from '$lib/types';

	let panels: Panel3D[] = $state([]);
	let exploded: boolean = $state(false);
	let wireframe: boolean = $state(false);
	let selectedPanel: string | null = $state(null);

	async function loadAssembly() {
		const idx = $selectedCabinetIndex ?? 0;
		try {
			panels = await api.get3dAssembly(idx);
		} catch (e) {
			showToast(`Error: ${e}`, 'error');
			panels = [];
		}
	}

	$effect(() => {
		if ($project) {
			loadAssembly();
		}
	});

	const cabinets = $derived(() => {
		if (!$project) return [];
		const cabs: { name: string; index: number }[] = [];
		if ($project.cabinet) cabs.push({ name: $project.cabinet.name, index: 0 });
		$project.cabinets.forEach((c, i) => cabs.push({ name: c.name, index: i }));
		return cabs;
	});

	function handleSelectPanel(label: string | null) {
		selectedPanel = label;
	}
</script>

{#if $project}
	<div class="flex flex-col h-full">
		<div class="flex items-center justify-between mb-4">
			<h1 class="text-2xl font-bold">3D Preview</h1>
			<div class="flex items-center gap-3">
				{#if cabinets().length > 1}
					<select
						class="px-3 py-1 text-xs bg-surface border border-border rounded text-text-primary"
						onchange={(e) => { selectedCabinetIndex.set(parseInt((e.target as HTMLSelectElement).value)); loadAssembly(); }}
					>
						{#each cabinets() as cab}
							<option value={cab.index}>{cab.name}</option>
						{/each}
					</select>
				{/if}
				<label class="flex items-center gap-1 text-xs text-text-secondary cursor-pointer">
					<input type="checkbox" bind:checked={exploded} class="accent-accent" />
					Exploded
				</label>
				<label class="flex items-center gap-1 text-xs text-text-secondary cursor-pointer">
					<input type="checkbox" bind:checked={wireframe} class="accent-accent" />
					Wireframe
				</label>
			</div>
		</div>

		{#if panels.length > 0}
			<div class="flex gap-6 flex-1 min-h-0">
				<!-- 3D View -->
				<div class="flex-1 bg-bg-secondary rounded-lg border border-border overflow-hidden">
					<CabinetScene
						{panels}
						{exploded}
						{wireframe}
						{selectedPanel}
						onSelectPanel={handleSelectPanel}
					/>
				</div>

				<!-- Panel list -->
				<div class="w-48 flex-shrink-0 overflow-auto">
					<h3 class="text-sm font-semibold mb-2">Panels</h3>
					<div class="space-y-1">
						{#each panels as panel}
							<button
								class="w-full text-left px-2 py-1 text-xs rounded transition-colors"
								class:bg-accent={selectedPanel === panel.label}
								class:text-white={selectedPanel === panel.label}
								class:hover:bg-surface-hover={selectedPanel !== panel.label}
								onclick={() => selectedPanel = selectedPanel === panel.label ? null : panel.label}
							>
								<div class="font-medium">{panel.label}</div>
								<div class="text-text-secondary">{panel.width.toFixed(2)} x {panel.height.toFixed(2)} x {panel.depth.toFixed(2)}</div>
							</button>
						{/each}
					</div>
				</div>
			</div>
		{:else}
			<p class="text-text-secondary">No cabinet selected or no parts generated.</p>
		{/if}
	</div>
{:else}
	<div class="text-text-secondary">
		<p>No project loaded. <a href="/" class="text-accent hover:underline">Go home</a> to create or open one.</p>
	</div>
{/if}
