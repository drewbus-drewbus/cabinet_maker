import { writable } from 'svelte/store';

/** Which cabinet is selected in the cabinet list. */
export const selectedCabinetIndex = writable<number | null>(null);

/** Which tab/page is active in the main content area. */
export const activeTab = writable<string>('home');

/** Whether the sidebar is collapsed. */
export const sidebarCollapsed = writable(false);

/** Selected material index for nesting view. */
export const selectedMaterialIndex = writable(0);

/** Selected sheet index for nesting view. */
export const selectedSheetIndex = writable(0);
