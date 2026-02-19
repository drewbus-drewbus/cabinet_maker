<script lang="ts">
	import { project, isDirty, projectPath, showToast, isLoading } from '$lib/stores/project';
	import { undo, redo, canUndo, canRedo, clearHistory } from '$lib/stores/history';
	import * as api from '$lib/api';
	import { open, save } from '@tauri-apps/plugin-dialog';

	async function handleNew() {
		try {
			isLoading.set(true);
			const p = await api.newProject();
			project.set(p);
			clearHistory();
			projectPath.set(null);
			isDirty.set(false);
			showToast('New project created', 'success');
		} catch (e) {
			showToast(`Error: ${e}`, 'error');
		} finally {
			isLoading.set(false);
		}
	}

	async function handleOpen() {
		try {
			const path = await open({
				title: 'Open Project',
				filters: [{ name: 'TOML', extensions: ['toml'] }]
			});
			if (!path) return;

			isLoading.set(true);
			const p = await api.openProject(path);
			project.set(p);
			clearHistory();
			projectPath.set(path);
			isDirty.set(false);
			showToast('Project opened', 'success');
		} catch (e) {
			showToast(`Error: ${e}`, 'error');
		} finally {
			isLoading.set(false);
		}
	}

	async function handleSave() {
		try {
			isLoading.set(true);
			const savedPath = await api.saveProject();
			projectPath.set(savedPath);
			isDirty.set(false);
			showToast('Project saved', 'success');
		} catch {
			// No path set — do Save As
			await handleSaveAs();
		} finally {
			isLoading.set(false);
		}
	}

	async function handleSaveAs() {
		try {
			const path = await save({
				title: 'Save Project As',
				filters: [{ name: 'TOML', extensions: ['toml'] }]
			});
			if (!path) return;

			isLoading.set(true);
			const savedPath = await api.saveProject(path);
			projectPath.set(savedPath);
			isDirty.set(false);
			showToast('Project saved', 'success');
		} catch (e) {
			showToast(`Error: ${e}`, 'error');
		} finally {
			isLoading.set(false);
		}
	}
</script>

<header class="flex items-center justify-between px-4 py-2 bg-bg-secondary border-b border-border">
	<div class="flex items-center gap-2">
		<button
			class="px-3 py-1 text-xs bg-surface hover:bg-surface-hover text-text-primary rounded transition-colors"
			onclick={handleNew}
		>
			New
		</button>
		<button
			class="px-3 py-1 text-xs bg-surface hover:bg-surface-hover text-text-primary rounded transition-colors"
			onclick={handleOpen}
		>
			Open
		</button>
		<button
			class="px-3 py-1 text-xs bg-surface hover:bg-surface-hover text-text-primary rounded transition-colors"
			onclick={handleSave}
			disabled={!$project}
		>
			Save
		</button>
		<button
			class="px-3 py-1 text-xs bg-surface hover:bg-surface-hover text-text-primary rounded transition-colors"
			onclick={handleSaveAs}
			disabled={!$project}
		>
			Save As
		</button>

		<div class="w-px h-4 bg-border mx-1"></div>

		<button
			class="px-3 py-1 text-xs bg-surface hover:bg-surface-hover text-text-primary rounded transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
			onclick={undo}
			disabled={!$canUndo}
			title="Undo (Ctrl+Z)"
		>
			Undo
		</button>
		<button
			class="px-3 py-1 text-xs bg-surface hover:bg-surface-hover text-text-primary rounded transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
			onclick={redo}
			disabled={!$canRedo}
			title="Redo (Ctrl+Y)"
		>
			Redo
		</button>
	</div>

	<div class="flex items-center gap-3 text-xs text-text-secondary">
		{#if $project}
			<span class="font-medium text-text-primary">{$project.project.name}</span>
		{/if}
		{#if $isDirty}
			<span class="text-warning">Modified</span>
		{/if}
		{#if $projectPath}
			<span class="truncate max-w-64" title={$projectPath}>{$projectPath}</span>
		{/if}
	</div>
</header>
