import { writable, derived } from 'svelte/store';
import type {
	Project,
	TaggedPart,
	MaterialGroupDto,
	ValidationResult,
	CutlistRow
} from '../types';

/** The current project (source of truth is Rust, this is a reactive copy). */
export const project = writable<Project | null>(null);

/** Cached generated parts from the backend. */
export const cachedParts = writable<TaggedPart[]>([]);

/** Nesting results from the backend. */
export const nestingResults = writable<MaterialGroupDto[]>([]);

/** Validation result from the backend. */
export const validationResult = writable<ValidationResult | null>(null);

/** Cut list rows from the backend. */
export const cutlistRows = writable<CutlistRow[]>([]);

/** Whether the project has unsaved changes. */
export const isDirty = writable(false);

/** File path of the currently open project. */
export const projectPath = writable<string | null>(null);

/** Whether a backend operation is in progress. */
export const isLoading = writable(false);

/** Toast notification messages. */
export const toasts = writable<{ id: number; message: string; type: 'info' | 'success' | 'error' | 'warning' }[]>([]);

let toastId = 0;

export function showToast(message: string, type: 'info' | 'success' | 'error' | 'warning' = 'info') {
	const id = ++toastId;
	toasts.update((t) => [...t, { id, message, type }]);
	setTimeout(() => {
		toasts.update((t) => t.filter((toast) => toast.id !== id));
	}, 4000);
}

/** Derived: number of cabinets in the project. */
export const cabinetCount = derived(project, ($project) => {
	if (!$project) return 0;
	let count = $project.cabinets.length;
	if ($project.cabinet) count += 1;
	return count;
});

/** Derived: total part count. */
export const totalPartCount = derived(cachedParts, ($parts) => $parts.length);

/** Derived: whether the project has validation errors. */
export const hasValidationErrors = derived(
	validationResult,
	($result) => $result !== null && $result.errors.length > 0
);
