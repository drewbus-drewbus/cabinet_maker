<script lang="ts">
	import { project, showToast, isLoading } from '$lib/stores/project';
	import { isDirty, projectPath } from '$lib/stores/project';
	import { clearHistory } from '$lib/stores/history';
	import * as api from '$lib/api';
	import { goto } from '$app/navigation';

	let templates: string[] = $state([]);

	async function loadTemplates() {
		try {
			templates = await api.listTemplates();
		} catch {
			templates = [];
		}
	}

	async function handleLoadTemplate(name: string) {
		try {
			isLoading.set(true);
			const p = await api.loadTemplate(name);
			project.set(p);
			clearHistory();
			projectPath.set(null);
			isDirty.set(false);
			showToast(`Template "${name}" loaded`, 'success');
			goto('/project');
		} catch (e) {
			showToast(`Error loading template: ${e}`, 'error');
		} finally {
			isLoading.set(false);
		}
	}

	async function handleNewProject() {
		try {
			isLoading.set(true);
			const p = await api.newProject();
			project.set(p);
			clearHistory();
			projectPath.set(null);
			isDirty.set(false);
			showToast('New project created', 'success');
			goto('/project');
		} catch (e) {
			showToast(`Error: ${e}`, 'error');
		} finally {
			isLoading.set(false);
		}
	}

	$effect(() => {
		loadTemplates();
	});

	function formatTemplateName(name: string): string {
		return name.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase());
	}
</script>

<div class="max-w-2xl mx-auto">
	<h1 class="text-3xl font-bold mb-2">Cabinet Maker</h1>
	<p class="text-text-secondary mb-8">
		Parametric cabinet design to CNC G-code pipeline
	</p>

	<div class="grid grid-cols-2 gap-4 mb-8">
		<button
			class="p-6 bg-surface hover:bg-surface-hover rounded-lg text-left transition-colors border border-border"
			onclick={handleNewProject}
		>
			<h3 class="text-lg font-semibold mb-1">New Project</h3>
			<p class="text-text-secondary text-sm">Start with a blank multi-cabinet project</p>
		</button>

		<a
			href="/project"
			class="p-6 bg-surface rounded-lg text-left border border-border"
			class:opacity-50={!$project}
			class:hover:bg-surface-hover={!!$project}
			class:pointer-events-none={!$project}
		>
			<h3 class="text-lg font-semibold mb-1">Continue</h3>
			<p class="text-text-secondary text-sm">
				{#if $project}
					Resume "{$project.project.name}"
				{:else}
					No project loaded
				{/if}
			</p>
		</a>
	</div>

	{#if templates.length > 0}
		<h2 class="text-xl font-semibold mb-4">Templates</h2>
		<div class="grid grid-cols-3 gap-3">
			{#each templates as template}
				<button
					class="p-4 bg-surface hover:bg-surface-hover rounded-lg text-left transition-colors border border-border"
					onclick={() => handleLoadTemplate(template)}
				>
					<h3 class="text-sm font-semibold">{formatTemplateName(template)}</h3>
					<p class="text-text-secondary text-xs mt-1">{template}.toml</p>
				</button>
			{/each}
		</div>
	{/if}
</div>
