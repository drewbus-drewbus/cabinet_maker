/**
 * Pure utility functions for the 3D cabinet scene.
 * Extracted from CabinetScene.svelte for testability.
 */
import type { Panel3D } from '$lib/types';

/** Explosion offset distance in model units (inches). */
export const EXPLODE_OFFSET = 3;

export interface Vec3 {
	x: number;
	y: number;
	z: number;
}

/**
 * Compute the explosion displacement vector for a panel based on its label.
 * Returns {0,0,0} when not exploded.
 */
export function getExplodeOffset(label: string, exploded: boolean): Vec3 {
	if (!exploded) return { x: 0, y: 0, z: 0 };

	const lower = label.toLowerCase();
	// Face frame parts must be checked before left/right/top/bottom since
	// labels like "left_stile" and "top_rail" contain those substrings.
	if (lower.includes('stile') || lower.includes('rail'))
		return { x: 0, y: 0, z: EXPLODE_OFFSET * 1.5 };
	if (lower.includes('left')) return { x: -EXPLODE_OFFSET, y: 0, z: 0 };
	if (lower.includes('right')) return { x: EXPLODE_OFFSET, y: 0, z: 0 };
	if (lower === 'top') return { x: 0, y: EXPLODE_OFFSET, z: 0 };
	if (lower === 'bottom') return { x: 0, y: -EXPLODE_OFFSET, z: 0 };
	if (lower === 'back') return { x: 0, y: 0, z: -EXPLODE_OFFSET };
	if (lower.includes('shelf')) return { x: 0, y: 0, z: EXPLODE_OFFSET };
	if (lower.includes('divider')) return { x: EXPLODE_OFFSET * 0.5, y: 0, z: 0 };
	if (lower.includes('stretcher')) return { x: 0, y: 0, z: EXPLODE_OFFSET };
	if (lower.includes('toe')) return { x: 0, y: -EXPLODE_OFFSET, z: 0 };
	return { x: 0, y: 0, z: 0 };
}

/**
 * Parse a hex color string (e.g. "#c4a882") into a numeric RGB value.
 * Falls back to a default warm wood color.
 */
export function panelColor(color: string): number {
	if (color.startsWith('#') && color.length >= 7) {
		return parseInt(color.slice(1), 16);
	}
	return 0xc4a882; // default wood color
}

export interface BoundingBox {
	minX: number;
	maxX: number;
	minY: number;
	maxY: number;
	minZ: number;
	maxZ: number;
}

/**
 * Compute the axis-aligned bounding box of a set of panels.
 * Returns null if the array is empty.
 */
export function computeBoundingBox(panels: Panel3D[]): BoundingBox | null {
	if (panels.length === 0) return null;

	let minX = Infinity,
		maxX = -Infinity;
	let minY = Infinity,
		maxY = -Infinity;
	let minZ = Infinity,
		maxZ = -Infinity;

	for (const p of panels) {
		minX = Math.min(minX, p.x - p.width / 2);
		maxX = Math.max(maxX, p.x + p.width / 2);
		minY = Math.min(minY, p.y - p.height / 2);
		maxY = Math.max(maxY, p.y + p.height / 2);
		minZ = Math.min(minZ, p.z - p.depth / 2);
		maxZ = Math.max(maxZ, p.z + p.depth / 2);
	}

	return { minX, maxX, minY, maxY, minZ, maxZ };
}

/**
 * Compute the center point of a bounding box.
 */
export function boundingBoxCenter(bb: BoundingBox): Vec3 {
	return {
		x: (bb.minX + bb.maxX) / 2,
		y: (bb.minY + bb.maxY) / 2,
		z: (bb.minZ + bb.maxZ) / 2
	};
}

/**
 * Compute the maximum extent of a bounding box (for camera distance calculation).
 */
export function boundingBoxMaxExtent(bb: BoundingBox): number {
	return Math.max(bb.maxX - bb.minX, bb.maxY - bb.minY, bb.maxZ - bb.minZ);
}

/**
 * Compute the final world position of a panel mesh, centered around origin
 * with optional explosion offset.
 */
export function computePanelPosition(panel: Panel3D, center: Vec3, exploded: boolean): Vec3 {
	const offset = getExplodeOffset(panel.label, exploded);
	return {
		x: panel.x - center.x + offset.x,
		y: panel.y - center.y + offset.y,
		z: panel.z - center.z + offset.z
	};
}
