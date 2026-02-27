<script lang="ts">
	import { project, isDirty, showToast } from '$lib/stores/project';
	import { pushSnapshot } from '$lib/stores/history';
	import * as api from '$lib/api';
	import type { Material, Tool } from '$lib/types';

	let selectedMaterialIdx: number | null = $state(null);
	let selectedToolIdx: number | null = $state(null);

	function defaultMaterial(): Material {
		return {
			name: 'New Material',
			thickness: 0.75,
			sheet_width: 48.0,
			sheet_length: 96.0,
			material_type: 'plywood'
		};
	}

	function defaultTool(): Tool {
		return {
			number: ($project?.tools.length ?? 0) + 1,
			tool_type: 'endmill',
			diameter: 0.25,
			flutes: 2,
			cutting_length: 1.0,
			description: 'New Tool'
		};
	}

	function addMaterial() {
		pushSnapshot();
		project.update((p) => {
			if (p) {
				p.materials.push(defaultMaterial());
				selectedMaterialIdx = p.materials.length - 1;
			}
			return p;
		});
		isDirty.set(true);
		syncProject();
	}

	function removeMaterial(idx: number) {
		pushSnapshot();
		project.update((p) => {
			if (p) p.materials.splice(idx, 1);
			return p;
		});
		if (selectedMaterialIdx === idx) selectedMaterialIdx = null;
		isDirty.set(true);
		syncProject();
	}

	function addTool() {
		pushSnapshot();
		project.update((p) => {
			if (p) {
				p.tools.push(defaultTool());
				selectedToolIdx = p.tools.length - 1;
			}
			return p;
		});
		isDirty.set(true);
		syncProject();
	}

	function removeTool(idx: number) {
		pushSnapshot();
		project.update((p) => {
			if (p) p.tools.splice(idx, 1);
			return p;
		});
		if (selectedToolIdx === idx) selectedToolIdx = null;
		isDirty.set(true);
		syncProject();
	}

	async function syncProject() {
		pushSnapshot();
		const p = $project;
		if (!p) return;
		try {
			await api.updateProject(p);
		} catch (e) {
			showToast(`Error: ${e}`, 'error');
		}
	}

	function updateMaterialField(field: string, value: unknown) {
		project.update((p) => {
			if (p && selectedMaterialIdx !== null && selectedMaterialIdx < p.materials.length) {
				(p.materials[selectedMaterialIdx] as unknown as Record<string, unknown>)[field] = value;
			}
			return p;
		});
		isDirty.set(true);
	}

	function updateToolField(field: string, value: unknown) {
		project.update((p) => {
			if (p && selectedToolIdx !== null && selectedToolIdx < p.tools.length) {
				(p.tools[selectedToolIdx] as unknown as Record<string, unknown>)[field] = value;
			}
			return p;
		});
		isDirty.set(true);
	}

	const selectedMaterial = $derived(
		$project && selectedMaterialIdx !== null && selectedMaterialIdx < ($project?.materials.length ?? 0)
			? $project.materials[selectedMaterialIdx]
			: null
	);

	const selectedTool = $derived(
		$project && selectedToolIdx !== null && selectedToolIdx < ($project?.tools.length ?? 0)
			? $project.tools[selectedToolIdx]
			: null
	);
</script>

{#if $project}
	<div class="grid grid-cols-2 gap-8">
		<!-- Materials -->
		<div>
			<div class="flex items-center justify-between mb-3">
				<h2 class="text-lg font-semibold">Materials</h2>
				<button class="px-2 py-1 text-xs bg-accent hover:bg-accent/80 text-white rounded" onclick={addMaterial}>+ Add</button>
			</div>

			<div class="space-y-1 mb-4">
				{#each $project.materials as mat, i}
					<div
						class="w-full flex items-center justify-between px-3 py-2 text-sm rounded text-left cursor-pointer"
						class:bg-surface={selectedMaterialIdx === i}
						class:hover:bg-surface-hover={selectedMaterialIdx !== i}
						role="button"
						tabindex="0"
						onclick={() => { selectedMaterialIdx = i; selectedToolIdx = null; }}
						onkeydown={(e) => { if (e.key === 'Enter') { selectedMaterialIdx = i; selectedToolIdx = null; } }}
					>
						<div>
							<div class="font-medium">{mat.name}</div>
							<div class="text-xs text-text-secondary">{mat.thickness}" {mat.material_type}</div>
						</div>
						<button class="text-text-secondary hover:text-error text-xs px-1" onclick={(e) => { e.stopPropagation(); removeMaterial(i); }}>x</button>
					</div>
				{/each}
			</div>

			{#if selectedMaterial}
				<div class="p-4 bg-surface rounded border border-border space-y-3">
					<div>
						<label class="block text-xs text-text-secondary mb-1">Name</label>
						<input type="text" value={selectedMaterial.name} oninput={(e) => updateMaterialField('name', (e.target as HTMLInputElement).value)} onblur={syncProject} class="w-full px-3 py-2 bg-bg border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent" />
					</div>
					<div class="grid grid-cols-2 gap-3">
						<div>
							<label class="block text-xs text-text-secondary mb-1">Thickness</label>
							<input type="number" step="0.125" value={selectedMaterial.thickness} oninput={(e) => updateMaterialField('thickness', parseFloat((e.target as HTMLInputElement).value))} onblur={syncProject} class="w-full px-3 py-2 bg-bg border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent" />
						</div>
						<div>
							<label class="block text-xs text-text-secondary mb-1">Type</label>
							<select value={selectedMaterial.material_type} onchange={(e) => { updateMaterialField('material_type', (e.target as HTMLSelectElement).value); syncProject(); }} class="w-full px-3 py-2 bg-bg border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent">
								<option value="plywood">Plywood</option>
								<option value="hardwood">Hardwood</option>
								<option value="mdf">MDF</option>
								<option value="melamine">Melamine</option>
							</select>
						</div>
						<div>
							<label class="block text-xs text-text-secondary mb-1">Sheet Width</label>
							<input type="number" step="1" value={selectedMaterial.sheet_width} oninput={(e) => updateMaterialField('sheet_width', parseFloat((e.target as HTMLInputElement).value))} onblur={syncProject} class="w-full px-3 py-2 bg-bg border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent" />
						</div>
						<div>
							<label class="block text-xs text-text-secondary mb-1">Sheet Length</label>
							<input type="number" step="1" value={selectedMaterial.sheet_length} oninput={(e) => updateMaterialField('sheet_length', parseFloat((e.target as HTMLInputElement).value))} onblur={syncProject} class="w-full px-3 py-2 bg-bg border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent" />
						</div>
					</div>
				</div>
			{/if}
		</div>

		<!-- Tools -->
		<div>
			<div class="flex items-center justify-between mb-3">
				<h2 class="text-lg font-semibold">Tools</h2>
				<button class="px-2 py-1 text-xs bg-accent hover:bg-accent/80 text-white rounded" onclick={addTool}>+ Add</button>
			</div>

			<div class="space-y-1 mb-4">
				{#each $project.tools as tool, i}
					<div
						class="w-full flex items-center justify-between px-3 py-2 text-sm rounded text-left cursor-pointer"
						class:bg-surface={selectedToolIdx === i}
						class:hover:bg-surface-hover={selectedToolIdx !== i}
						role="button"
						tabindex="0"
						onclick={() => { selectedToolIdx = i; selectedMaterialIdx = null; }}
						onkeydown={(e) => { if (e.key === 'Enter') { selectedToolIdx = i; selectedMaterialIdx = null; } }}
					>
						<div>
							<div class="font-medium">T{tool.number}: {tool.description}</div>
							<div class="text-xs text-text-secondary">{tool.diameter}" {tool.tool_type}</div>
						</div>
						<button class="text-text-secondary hover:text-error text-xs px-1" onclick={(e) => { e.stopPropagation(); removeTool(i); }}>x</button>
					</div>
				{/each}
			</div>

			{#if selectedTool}
				<div class="p-4 bg-surface rounded border border-border space-y-3">
					<div>
						<label class="block text-xs text-text-secondary mb-1">Description</label>
						<input type="text" value={selectedTool.description} oninput={(e) => updateToolField('description', (e.target as HTMLInputElement).value)} onblur={syncProject} class="w-full px-3 py-2 bg-bg border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent" />
					</div>
					<div class="grid grid-cols-2 gap-3">
						<div>
							<label class="block text-xs text-text-secondary mb-1">Number</label>
							<input type="number" min="1" value={selectedTool.number} oninput={(e) => updateToolField('number', parseInt((e.target as HTMLInputElement).value))} onblur={syncProject} class="w-full px-3 py-2 bg-bg border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent" />
						</div>
						<div>
							<label class="block text-xs text-text-secondary mb-1">Diameter</label>
							<input type="number" step="0.0625" value={selectedTool.diameter} oninput={(e) => updateToolField('diameter', parseFloat((e.target as HTMLInputElement).value))} onblur={syncProject} class="w-full px-3 py-2 bg-bg border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent" />
						</div>
						<div>
							<label class="block text-xs text-text-secondary mb-1">Flutes</label>
							<input type="number" min="1" max="8" value={selectedTool.flutes} oninput={(e) => updateToolField('flutes', parseInt((e.target as HTMLInputElement).value))} onblur={syncProject} class="w-full px-3 py-2 bg-bg border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent" />
						</div>
						<div>
							<label class="block text-xs text-text-secondary mb-1">Cutting Length</label>
							<input type="number" step="0.125" value={selectedTool.cutting_length} oninput={(e) => updateToolField('cutting_length', parseFloat((e.target as HTMLInputElement).value))} onblur={syncProject} class="w-full px-3 py-2 bg-bg border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent" />
						</div>
					</div>
				</div>
			{/if}
		</div>
	</div>
{:else}
	<div class="text-text-secondary">
		<p>No project loaded. <a href="/" class="text-accent hover:underline">Go home</a> to create or open one.</p>
	</div>
{/if}
