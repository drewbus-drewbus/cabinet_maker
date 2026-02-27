<script lang="ts">
	import { project, isDirty, projectPath, showToast, isLoading } from '$lib/stores/project';
	import { undo, redo, canUndo, canRedo, clearHistory } from '$lib/stores/history';
	import * as api from '$lib/api';

	let fileInput: HTMLInputElement;

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

	function handleOpenClick() {
		fileInput.click();
	}

	async function handleFileSelected(event: Event) {
		const input = event.target as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;

		try {
			isLoading.set(true);
			const tomlContent = await file.text();
			const p = await api.openProjectFromToml(tomlContent, file.name);
			project.set(p);
			clearHistory();
			projectPath.set(file.name);
			isDirty.set(false);
			showToast('Project opened', 'success');
		} catch (e) {
			showToast(`Error: ${e}`, 'error');
		} finally {
			isLoading.set(false);
			input.value = '';
		}
	}

	async function handleSave() {
		try {
			isLoading.set(true);
			const toml = await api.saveProject();
			const blob = new Blob([toml], { type: 'application/toml' });
			const filename = $projectPath || 'project.toml';
			downloadBlob(blob, filename);
			isDirty.set(false);
			showToast('Project saved', 'success');
		} catch (e) {
			showToast(`Error: ${e}`, 'error');
		} finally {
			isLoading.set(false);
		}
	}

	function downloadBlob(blob: Blob, filename: string) {
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = filename;
		document.body.appendChild(a);
		a.click();
		document.body.removeChild(a);
		URL.revokeObjectURL(url);
	}
</script>

<input bind:this={fileInput} type="file" accept=".toml" class="hidden" onchange={handleFileSelected} />

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
			onclick={handleOpenClick}
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
