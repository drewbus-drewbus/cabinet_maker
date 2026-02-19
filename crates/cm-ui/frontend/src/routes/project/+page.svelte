<script lang="ts">
	import { project, isDirty, showToast, isLoading } from '$lib/stores/project';
	import { pushSnapshot } from '$lib/stores/history';
	import * as api from '$lib/api';

	function handleNameChange(e: Event) {
		const input = e.target as HTMLInputElement;
		project.update((p) => {
			if (p) p.project.name = input.value;
			return p;
		});
		isDirty.set(true);
	}

	function handleUnitsChange(e: Event) {
		const select = e.target as HTMLSelectElement;
		project.update((p) => {
			if (p) p.project.units = select.value as 'inches' | 'millimeters';
			return p;
		});
		isDirty.set(true);
	}

	async function syncToBackend() {
		pushSnapshot();
		const p = $project;
		if (!p) return;
		try {
			isLoading.set(true);
			await api.updateProject(p);
			showToast('Project updated', 'success');
		} catch (e) {
			showToast(`Error: ${e}`, 'error');
		} finally {
			isLoading.set(false);
		}
	}
</script>

{#if $project}
	<div class="max-w-xl">
		<h1 class="text-2xl font-bold mb-6">Project Settings</h1>

		<div class="space-y-4">
			<div>
				<label class="block text-sm text-text-secondary mb-1" for="project-name">Project Name</label>
				<input
					id="project-name"
					type="text"
					value={$project.project.name}
					oninput={handleNameChange}
					onblur={syncToBackend}
					class="w-full px-3 py-2 bg-surface border border-border rounded text-text-primary text-sm focus:outline-none focus:border-accent"
				/>
			</div>

			<div>
				<label class="block text-sm text-text-secondary mb-1" for="project-units">Units</label>
				<select
					id="project-units"
					value={$project.project.units}
					onchange={(e) => { handleUnitsChange(e); syncToBackend(); }}
					class="w-full px-3 py-2 bg-surface border border-border rounded text-text-primary text-sm focus:outline-none focus:border-accent"
				>
					<option value="inches">Inches</option>
					<option value="millimeters">Millimeters</option>
				</select>
			</div>
		</div>

		<div class="mt-8">
			<h2 class="text-lg font-semibold mb-3">Summary</h2>
			<div class="grid grid-cols-2 gap-3 text-sm">
				<div class="p-3 bg-surface rounded border border-border">
					<div class="text-text-secondary">Cabinets</div>
					<div class="text-xl font-bold">{$project.cabinets.length + ($project.cabinet ? 1 : 0)}</div>
				</div>
				<div class="p-3 bg-surface rounded border border-border">
					<div class="text-text-secondary">Materials</div>
					<div class="text-xl font-bold">{$project.materials.length + ($project.material ? 1 : 0)}</div>
				</div>
				<div class="p-3 bg-surface rounded border border-border">
					<div class="text-text-secondary">Tools</div>
					<div class="text-xl font-bold">{$project.tools.length}</div>
				</div>
				<div class="p-3 bg-surface rounded border border-border">
					<div class="text-text-secondary">Format</div>
					<div class="text-xl font-bold">{$project.cabinet ? 'Legacy' : 'Multi'}</div>
				</div>
			</div>
		</div>
	</div>
{:else}
	<div class="text-text-secondary">
		<p>No project loaded. <a href="/" class="text-accent hover:underline">Go home</a> to create or open one.</p>
	</div>
{/if}
