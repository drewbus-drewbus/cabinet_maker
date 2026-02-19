<script lang="ts">
	import { project, isDirty, cachedParts, showToast, isLoading } from '$lib/stores/project';
	import { selectedCabinetIndex } from '$lib/stores/ui';
	import { pushSnapshot } from '$lib/stores/history';
	import * as api from '$lib/api';
	import type { CabinetEntry, CabinetType, Part } from '$lib/types';

	let previewParts: Part[] = $state([]);

	const cabinetTypes: { value: CabinetType; label: string }[] = [
		{ value: 'basic_box', label: 'Basic Box' },
		{ value: 'base_cabinet', label: 'Base Cabinet' },
		{ value: 'wall_cabinet', label: 'Wall Cabinet' },
		{ value: 'tall_cabinet', label: 'Tall Cabinet' },
		{ value: 'sink_base', label: 'Sink Base' },
		{ value: 'drawer_bank', label: 'Drawer Bank' }
	];

	function defaultEntry(): CabinetEntry {
		return {
			name: 'New Cabinet',
			cabinet_type: 'basic_box',
			width: 36.0,
			height: 30.0,
			depth: 12.0,
			material_thickness: 0.75,
			back_thickness: 0.25,
			shelf_count: 2,
			shelf_joinery: 'dado',
			dado_depth_fraction: 0.5,
			has_back: true,
			back_joinery: 'rabbet',
			toe_kick: null,
			drawers: null,
			stretchers: null,
			construction: 'frameless',
			face_frame: null,
			material_ref: null,
			back_material_ref: null
		};
	}

	async function handleAddCabinet() {
		pushSnapshot();
		try {
			isLoading.set(true);
			const entry = defaultEntry();
			const index = await api.addCabinet(entry);
			project.update((p) => {
				if (p) p.cabinets.push(entry);
				return p;
			});
			selectedCabinetIndex.set(index);
			isDirty.set(true);
			showToast('Cabinet added', 'success');
		} catch (e) {
			showToast(`Error: ${e}`, 'error');
		} finally {
			isLoading.set(false);
		}
	}

	async function handleRemoveCabinet(index: number) {
		pushSnapshot();
		try {
			isLoading.set(true);
			await api.removeCabinet(index);
			project.update((p) => {
				if (p) p.cabinets.splice(index, 1);
				return p;
			});
			if ($selectedCabinetIndex === index) selectedCabinetIndex.set(null);
			isDirty.set(true);
			showToast('Cabinet removed', 'success');
		} catch (e) {
			showToast(`Error: ${e}`, 'error');
		} finally {
			isLoading.set(false);
		}
	}

	async function handleUpdateCabinet() {
		pushSnapshot();
		const idx = $selectedCabinetIndex;
		const p = $project;
		if (idx === null || !p || idx >= p.cabinets.length) return;

		try {
			await api.updateCabinet(idx, p.cabinets[idx]);
			isDirty.set(true);
		} catch (e) {
			showToast(`Error: ${e}`, 'error');
		}
	}

	async function handleGenerateParts() {
		try {
			isLoading.set(true);
			const parts = await api.generateParts();
			cachedParts.set(parts);
			showToast(`Generated ${parts.length} parts`, 'success');
		} catch (e) {
			showToast(`Error: ${e}`, 'error');
		} finally {
			isLoading.set(false);
		}
	}

	async function handlePreviewParts() {
		const idx = $selectedCabinetIndex;
		const p = $project;
		if (idx === null || !p || idx >= p.cabinets.length) return;

		try {
			previewParts = await api.previewCabinetParts(p.cabinets[idx]);
		} catch (e) {
			showToast(`Error: ${e}`, 'error');
		}
	}

	function updateField(field: string, value: unknown) {
		const idx = $selectedCabinetIndex;
		project.update((p) => {
			if (p && idx !== null && idx < p.cabinets.length) {
				(p.cabinets[idx] as Record<string, unknown>)[field] = value;
			}
			return p;
		});
		isDirty.set(true);
	}

	function getSelectedCabinet(): CabinetEntry | null {
		const idx = $selectedCabinetIndex;
		const p = $project;
		if (idx === null || !p || idx >= p.cabinets.length) return null;
		return p.cabinets[idx];
	}

	$effect(() => {
		if ($selectedCabinetIndex !== null) {
			handlePreviewParts();
		}
	});

	const selectedCab = $derived(getSelectedCabinet());
	const showToeKick = $derived(
		selectedCab?.cabinet_type === 'base_cabinet' ||
		selectedCab?.cabinet_type === 'tall_cabinet' ||
		selectedCab?.cabinet_type === 'drawer_bank'
	);
	const showDrawers = $derived(selectedCab?.cabinet_type === 'drawer_bank');
	const showStretchers = $derived(selectedCab?.cabinet_type === 'sink_base');
</script>

{#if $project}
	<div class="flex gap-6 h-full">
		<!-- Cabinet List -->
		<div class="w-56 flex-shrink-0">
			<div class="flex items-center justify-between mb-3">
				<h2 class="text-lg font-semibold">Cabinets</h2>
				<button
					class="px-2 py-1 text-xs bg-accent hover:bg-accent/80 text-white rounded"
					onclick={handleAddCabinet}
				>
					+ Add
				</button>
			</div>

			<div class="space-y-1">
				{#each $project.cabinets as entry, i}
					<div
						class="w-full flex items-center justify-between px-3 py-2 text-sm rounded transition-colors text-left cursor-pointer"
						class:bg-surface={$selectedCabinetIndex === i}
						class:text-accent={$selectedCabinetIndex === i}
						class:hover:bg-surface-hover={$selectedCabinetIndex !== i}
						role="button"
						tabindex="0"
						onclick={() => selectedCabinetIndex.set(i)}
						onkeydown={(e) => { if (e.key === 'Enter') selectedCabinetIndex.set(i); }}
					>
						<div>
							<div class="font-medium">{entry.name}</div>
							<div class="text-xs text-text-secondary">{entry.cabinet_type.replace('_', ' ')}</div>
						</div>
						<button
							class="text-text-secondary hover:text-error text-xs px-1"
							onclick={(e) => { e.stopPropagation(); handleRemoveCabinet(i); }}
						>
							x
						</button>
					</div>
				{/each}
			</div>

			{#if $project.cabinets.length > 0}
				<button
					class="mt-4 w-full px-3 py-2 text-xs bg-surface hover:bg-surface-hover rounded text-center"
					onclick={handleGenerateParts}
				>
					Generate All Parts
				</button>
			{/if}
		</div>

		<!-- Cabinet Form -->
		<div class="flex-1 overflow-auto">
			{#if selectedCab}
				<h2 class="text-lg font-semibold mb-4">Edit: {selectedCab.name}</h2>

				<div class="grid grid-cols-2 gap-4 max-w-lg">
					<div class="col-span-2">
						<label class="block text-xs text-text-secondary mb-1">Name</label>
						<input
							type="text"
							value={selectedCab.name}
							oninput={(e) => updateField('name', (e.target as HTMLInputElement).value)}
							onblur={handleUpdateCabinet}
							class="w-full px-3 py-2 bg-surface border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent"
						/>
					</div>

					<div class="col-span-2">
						<label class="block text-xs text-text-secondary mb-1">Type</label>
						<select
							value={selectedCab.cabinet_type}
							onchange={(e) => { updateField('cabinet_type', (e.target as HTMLSelectElement).value); handleUpdateCabinet(); }}
							class="w-full px-3 py-2 bg-surface border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent"
						>
							{#each cabinetTypes as ct}
								<option value={ct.value}>{ct.label}</option>
							{/each}
						</select>
					</div>

					<div>
						<label class="block text-xs text-text-secondary mb-1">Width</label>
						<input
							type="number"
							step="0.25"
							value={selectedCab.width}
							oninput={(e) => updateField('width', parseFloat((e.target as HTMLInputElement).value))}
							onblur={handleUpdateCabinet}
							class="w-full px-3 py-2 bg-surface border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent"
						/>
					</div>

					<div>
						<label class="block text-xs text-text-secondary mb-1">Height</label>
						<input
							type="number"
							step="0.25"
							value={selectedCab.height}
							oninput={(e) => updateField('height', parseFloat((e.target as HTMLInputElement).value))}
							onblur={handleUpdateCabinet}
							class="w-full px-3 py-2 bg-surface border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent"
						/>
					</div>

					<div>
						<label class="block text-xs text-text-secondary mb-1">Depth</label>
						<input
							type="number"
							step="0.25"
							value={selectedCab.depth}
							oninput={(e) => updateField('depth', parseFloat((e.target as HTMLInputElement).value))}
							onblur={handleUpdateCabinet}
							class="w-full px-3 py-2 bg-surface border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent"
						/>
					</div>

					<div>
						<label class="block text-xs text-text-secondary mb-1">Shelves</label>
						<input
							type="number"
							min="0"
							max="20"
							value={selectedCab.shelf_count}
							oninput={(e) => updateField('shelf_count', parseInt((e.target as HTMLInputElement).value))}
							onblur={handleUpdateCabinet}
							class="w-full px-3 py-2 bg-surface border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent"
						/>
					</div>

					<div>
						<label class="block text-xs text-text-secondary mb-1">Material Thickness</label>
						<input
							type="number"
							step="0.125"
							value={selectedCab.material_thickness}
							oninput={(e) => updateField('material_thickness', parseFloat((e.target as HTMLInputElement).value))}
							onblur={handleUpdateCabinet}
							class="w-full px-3 py-2 bg-surface border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent"
						/>
					</div>

					<div>
						<label class="block text-xs text-text-secondary mb-1">Back Thickness</label>
						<input
							type="number"
							step="0.125"
							value={selectedCab.back_thickness}
							oninput={(e) => updateField('back_thickness', parseFloat((e.target as HTMLInputElement).value))}
							onblur={handleUpdateCabinet}
							class="w-full px-3 py-2 bg-surface border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent"
						/>
					</div>

					<div>
						<label class="flex items-center gap-2 text-xs text-text-secondary cursor-pointer">
							<input
								type="checkbox"
								checked={selectedCab.has_back}
								onchange={(e) => { updateField('has_back', (e.target as HTMLInputElement).checked); handleUpdateCabinet(); }}
								class="accent-accent"
							/>
							Has Back Panel
						</label>
					</div>

					{#if showToeKick}
						<div class="col-span-2 border-t border-border pt-3 mt-2">
							<h3 class="text-sm font-semibold mb-2">Toe Kick</h3>
							<div class="grid grid-cols-2 gap-3">
								<div>
									<label class="block text-xs text-text-secondary mb-1">Height</label>
									<input
										type="number"
										step="0.25"
										value={selectedCab.toe_kick?.height ?? 4.0}
										oninput={(e) => {
											const tk = selectedCab?.toe_kick ?? { height: 4.0, setback: 3.0 };
											tk.height = parseFloat((e.target as HTMLInputElement).value);
											updateField('toe_kick', tk);
										}}
										onblur={handleUpdateCabinet}
										class="w-full px-3 py-2 bg-surface border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent"
									/>
								</div>
								<div>
									<label class="block text-xs text-text-secondary mb-1">Setback</label>
									<input
										type="number"
										step="0.25"
										value={selectedCab.toe_kick?.setback ?? 3.0}
										oninput={(e) => {
											const tk = selectedCab?.toe_kick ?? { height: 4.0, setback: 3.0 };
											tk.setback = parseFloat((e.target as HTMLInputElement).value);
											updateField('toe_kick', tk);
										}}
										onblur={handleUpdateCabinet}
										class="w-full px-3 py-2 bg-surface border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent"
									/>
								</div>
							</div>
						</div>
					{/if}

					{#if showDrawers}
						<div class="col-span-2 border-t border-border pt-3 mt-2">
							<h3 class="text-sm font-semibold mb-2">Drawers</h3>
							<div class="grid grid-cols-2 gap-3">
								<div>
									<label class="block text-xs text-text-secondary mb-1">Count</label>
									<input
										type="number"
										min="1"
										max="10"
										value={selectedCab.drawers?.count ?? 4}
										oninput={(e) => {
											const dr = selectedCab?.drawers ?? { count: 4, opening_height: 0, slide_clearance: 0.5 };
											dr.count = parseInt((e.target as HTMLInputElement).value);
											updateField('drawers', dr);
										}}
										onblur={handleUpdateCabinet}
										class="w-full px-3 py-2 bg-surface border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent"
									/>
								</div>
								<div>
									<label class="block text-xs text-text-secondary mb-1">Slide Clearance</label>
									<input
										type="number"
										step="0.125"
										value={selectedCab.drawers?.slide_clearance ?? 0.5}
										oninput={(e) => {
											const dr = selectedCab?.drawers ?? { count: 4, opening_height: 0, slide_clearance: 0.5 };
											dr.slide_clearance = parseFloat((e.target as HTMLInputElement).value);
											updateField('drawers', dr);
										}}
										onblur={handleUpdateCabinet}
										class="w-full px-3 py-2 bg-surface border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent"
									/>
								</div>
							</div>
						</div>
					{/if}

					{#if showStretchers}
						<div class="col-span-2 border-t border-border pt-3 mt-2">
							<h3 class="text-sm font-semibold mb-2">Stretchers</h3>
							<div class="grid grid-cols-2 gap-3">
								<div>
									<label class="block text-xs text-text-secondary mb-1">Front Width</label>
									<input
										type="number"
										step="0.25"
										value={selectedCab.stretchers?.front_width ?? 4.0}
										oninput={(e) => {
											const st = selectedCab?.stretchers ?? { front_width: 4.0, has_rear: true };
											st.front_width = parseFloat((e.target as HTMLInputElement).value);
											updateField('stretchers', st);
										}}
										onblur={handleUpdateCabinet}
										class="w-full px-3 py-2 bg-surface border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent"
									/>
								</div>
								<div>
									<label class="flex items-center gap-2 text-xs text-text-secondary cursor-pointer mt-5">
										<input
											type="checkbox"
											checked={selectedCab.stretchers?.has_rear ?? true}
											onchange={(e) => {
												const st = selectedCab?.stretchers ?? { front_width: 4.0, has_rear: true };
												st.has_rear = (e.target as HTMLInputElement).checked;
												updateField('stretchers', st);
												handleUpdateCabinet();
											}}
											class="accent-accent"
										/>
										Has Rear Stretcher
									</label>
								</div>
							</div>
						</div>
					{/if}
				</div>

				<!-- Preview Parts -->
				{#if previewParts.length > 0}
					<div class="mt-6 border-t border-border pt-4">
						<h3 class="text-sm font-semibold mb-2">Parts Preview ({previewParts.length} parts)</h3>
						<div class="overflow-auto max-h-64">
							<table class="w-full text-xs">
								<thead>
									<tr class="text-text-secondary border-b border-border">
										<th class="py-1 text-left">Label</th>
										<th class="py-1 text-right">Width</th>
										<th class="py-1 text-right">Height</th>
										<th class="py-1 text-right">Qty</th>
										<th class="py-1 text-right">Ops</th>
									</tr>
								</thead>
								<tbody>
									{#each previewParts as part}
										<tr class="border-b border-border/50">
											<td class="py-1">{part.label}</td>
											<td class="py-1 text-right">{part.rect.width.toFixed(3)}"</td>
											<td class="py-1 text-right">{part.rect.height.toFixed(3)}"</td>
											<td class="py-1 text-right">{part.quantity}</td>
											<td class="py-1 text-right">{part.operations.length}</td>
										</tr>
									{/each}
								</tbody>
							</table>
						</div>
					</div>
				{/if}
			{:else}
				<div class="flex items-center justify-center h-full text-text-secondary">
					<p>Select a cabinet from the list or add a new one</p>
				</div>
			{/if}
		</div>
	</div>
{:else}
	<div class="text-text-secondary">
		<p>No project loaded. <a href="/" class="text-accent hover:underline">Go home</a> to create or open one.</p>
	</div>
{/if}
