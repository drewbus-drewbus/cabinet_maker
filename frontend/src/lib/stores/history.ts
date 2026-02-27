import { writable, derived, get } from 'svelte/store';
import { project, showToast } from './project';
import * as api from '../api';
import type { Project } from '../types';

const MAX_SNAPSHOTS = 50;

/** Stack of JSON-stringified project snapshots for undo. */
const undoStack = writable<string[]>([]);

/** Stack of JSON-stringified project snapshots for redo. */
const redoStack = writable<string[]>([]);

/** Whether there are snapshots to undo. */
export const canUndo = derived(undoStack, ($stack) => $stack.length > 0);

/** Whether there are snapshots to redo. */
export const canRedo = derived(redoStack, ($stack) => $stack.length > 0);

/**
 * Take a snapshot of the current project state before a mutation.
 * Call this at the start of every "commit point" (blur, add, remove).
 */
export function pushSnapshot(): void {
	const current = get(project);
	if (!current) return;

	const snapshot = JSON.stringify(current);
	undoStack.update((stack) => {
		const next = [...stack, snapshot];
		if (next.length > MAX_SNAPSHOTS) next.shift();
		return next;
	});
	// Any new mutation invalidates the redo stack
	redoStack.set([]);
}

/**
 * Undo: restore the most recent snapshot and push current state to redo stack.
 */
export async function undo(): Promise<void> {
	const stack = get(undoStack);
	if (stack.length === 0) return;

	const current = get(project);
	if (current) {
		redoStack.update((rs) => [...rs, JSON.stringify(current)]);
	}

	const snapshot = stack[stack.length - 1];
	undoStack.update((s) => s.slice(0, -1));

	const restored: Project = JSON.parse(snapshot);
	project.set(restored);

	try {
		await api.updateProject(restored);
	} catch (e) {
		showToast(`Undo sync error: ${e}`, 'error');
	}
}

/**
 * Redo: restore the most recent redo snapshot and push current state to undo stack.
 */
export async function redo(): Promise<void> {
	const stack = get(redoStack);
	if (stack.length === 0) return;

	const current = get(project);
	if (current) {
		undoStack.update((us) => [...us, JSON.stringify(current)]);
	}

	const snapshot = stack[stack.length - 1];
	redoStack.update((s) => s.slice(0, -1));

	const restored: Project = JSON.parse(snapshot);
	project.set(restored);

	try {
		await api.updateProject(restored);
	} catch (e) {
		showToast(`Redo sync error: ${e}`, 'error');
	}
}

/**
 * Clear all history. Call on new project, open, or template load.
 */
export function clearHistory(): void {
	undoStack.set([]);
	redoStack.set([]);
}
