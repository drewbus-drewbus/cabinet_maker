import { describe, it, expect } from 'vitest';
import {
	getExplodeOffset,
	panelColor,
	computeBoundingBox,
	boundingBoxCenter,
	boundingBoxMaxExtent,
	computePanelPosition,
	EXPLODE_OFFSET
} from './scene-utils';
import type { Panel3D } from '$lib/types';

// Helper to create a panel with defaults
function makePanel(overrides: Partial<Panel3D> = {}): Panel3D {
	return {
		label: 'test',
		width: 10,
		height: 20,
		depth: 0.75,
		x: 5,
		y: 10,
		z: 0.375,
		color: '#c4a882',
		...overrides
	};
}

describe('getExplodeOffset', () => {
	it('returns zero vector when not exploded', () => {
		const offset = getExplodeOffset('left_side', false);
		expect(offset).toEqual({ x: 0, y: 0, z: 0 });
	});

	it('returns zero vector when not exploded, regardless of label', () => {
		for (const label of ['left_side', 'right_side', 'top', 'bottom', 'back', 'shelf_1']) {
			const offset = getExplodeOffset(label, false);
			expect(offset).toEqual({ x: 0, y: 0, z: 0 });
		}
	});

	describe('when exploded', () => {
		it('offsets left panels in -X', () => {
			const offset = getExplodeOffset('left_side', true);
			expect(offset.x).toBe(-EXPLODE_OFFSET);
			expect(offset.y).toBe(0);
			expect(offset.z).toBe(0);
		});

		it('offsets right panels in +X', () => {
			const offset = getExplodeOffset('right_side', true);
			expect(offset.x).toBe(EXPLODE_OFFSET);
			expect(offset.y).toBe(0);
			expect(offset.z).toBe(0);
		});

		it('offsets top in +Y', () => {
			const offset = getExplodeOffset('top', true);
			expect(offset.x).toBe(0);
			expect(offset.y).toBe(EXPLODE_OFFSET);
			expect(offset.z).toBe(0);
		});

		it('offsets bottom in -Y', () => {
			const offset = getExplodeOffset('bottom', true);
			expect(offset.x).toBe(0);
			expect(offset.y).toBe(-EXPLODE_OFFSET);
			expect(offset.z).toBe(0);
		});

		it('offsets back in -Z', () => {
			const offset = getExplodeOffset('back', true);
			expect(offset.x).toBe(0);
			expect(offset.y).toBe(0);
			expect(offset.z).toBe(-EXPLODE_OFFSET);
		});

		it('offsets shelves in +Z', () => {
			const offset = getExplodeOffset('shelf_1', true);
			expect(offset.x).toBe(0);
			expect(offset.y).toBe(0);
			expect(offset.z).toBe(EXPLODE_OFFSET);
		});

		it('offsets multiple shelf variants in +Z', () => {
			for (const label of ['shelf_1', 'shelf_2', 'center_shelf', 'Shelf_Upper']) {
				const offset = getExplodeOffset(label, true);
				expect(offset.z).toBe(EXPLODE_OFFSET);
			}
		});

		it('offsets dividers in +X (half offset)', () => {
			const offset = getExplodeOffset('divider_1', true);
			expect(offset.x).toBe(EXPLODE_OFFSET * 0.5);
		});

		it('offsets stretchers in +Z', () => {
			const offset = getExplodeOffset('front_stretcher', true);
			expect(offset.z).toBe(EXPLODE_OFFSET);
		});

		it('offsets toe kick in -Y', () => {
			const offset = getExplodeOffset('toe_kick_notch', true);
			expect(offset.y).toBe(-EXPLODE_OFFSET);
		});

		it('offsets face frame stiles in +Z (1.5x)', () => {
			const offset = getExplodeOffset('left_stile', true);
			expect(offset.z).toBe(EXPLODE_OFFSET * 1.5);
		});

		it('offsets face frame rails in +Z (1.5x)', () => {
			const offset = getExplodeOffset('top_rail', true);
			expect(offset.z).toBe(EXPLODE_OFFSET * 1.5);
		});

		it('returns zero for unknown labels', () => {
			const offset = getExplodeOffset('mystery_part', true);
			expect(offset).toEqual({ x: 0, y: 0, z: 0 });
		});

		it('is case-insensitive', () => {
			const upper = getExplodeOffset('LEFT_SIDE', true);
			const lower = getExplodeOffset('left_side', true);
			expect(upper).toEqual(lower);
		});
	});
});

describe('panelColor', () => {
	it('parses standard hex color strings', () => {
		expect(panelColor('#c4a882')).toBe(0xc4a882);
		expect(panelColor('#b89b72')).toBe(0xb89b72);
		expect(panelColor('#8b7355')).toBe(0x8b7355);
		expect(panelColor('#a88b62')).toBe(0xa88b62);
	});

	it('parses black and white', () => {
		expect(panelColor('#000000')).toBe(0x000000);
		expect(panelColor('#ffffff')).toBe(0xffffff);
	});

	it('parses the selection highlight color', () => {
		expect(panelColor('#e94560')).toBe(0xe94560);
	});

	it('returns default wood color for non-hex strings', () => {
		expect(panelColor('not-a-color')).toBe(0xc4a882);
		expect(panelColor('')).toBe(0xc4a882);
		expect(panelColor('red')).toBe(0xc4a882);
	});

	it('returns default for short hex (no 6-digit body)', () => {
		expect(panelColor('#abc')).toBe(0xc4a882);
	});
});

describe('computeBoundingBox', () => {
	it('returns null for empty panels array', () => {
		expect(computeBoundingBox([])).toBeNull();
	});

	it('computes correct bounds for a single panel', () => {
		const panel = makePanel({ width: 10, height: 20, depth: 2, x: 5, y: 10, z: 1 });
		const bb = computeBoundingBox([panel])!;

		// Panel centered at (5,10,1) with dims (10,20,2)
		expect(bb.minX).toBe(0); // 5 - 10/2
		expect(bb.maxX).toBe(10); // 5 + 10/2
		expect(bb.minY).toBe(0); // 10 - 20/2
		expect(bb.maxY).toBe(20); // 10 + 20/2
		expect(bb.minZ).toBe(0); // 1 - 2/2
		expect(bb.maxZ).toBe(2); // 1 + 2/2
	});

	it('computes bounds spanning multiple panels', () => {
		const panels = [
			makePanel({ x: 0.375, y: 15, z: 6, width: 0.75, height: 30, depth: 12 }), // left
			makePanel({ x: 35.625, y: 15, z: 6, width: 0.75, height: 30, depth: 12 }) // right
		];
		const bb = computeBoundingBox(panels)!;

		expect(bb.minX).toBe(0); // 0.375 - 0.375
		expect(bb.maxX).toBe(36); // 35.625 + 0.375
		expect(bb.minY).toBe(0); // 15 - 15
		expect(bb.maxY).toBe(30); // 15 + 15
	});

	it('handles a realistic bookshelf assembly', () => {
		const panels: Panel3D[] = [
			{ label: 'left_side', width: 0.75, height: 30, depth: 12, x: 0.375, y: 15, z: 6, color: '#c4a882' },
			{ label: 'right_side', width: 0.75, height: 30, depth: 12, x: 35.625, y: 15, z: 6, color: '#c4a882' },
			{ label: 'top', width: 34.5, height: 0.75, depth: 12, x: 18, y: 29.625, z: 6, color: '#b89b72' },
			{ label: 'bottom', width: 34.5, height: 0.75, depth: 12, x: 18, y: 0.375, z: 6, color: '#b89b72' },
			{ label: 'back', width: 34.5, height: 28.5, depth: 0.25, x: 18, y: 15, z: 0.125, color: '#8b7355' }
		];

		const bb = computeBoundingBox(panels)!;
		expect(bb.minX).toBe(0);
		expect(bb.maxX).toBe(36);
		expect(bb.minY).toBe(0);
		expect(bb.maxY).toBe(30);
		expect(bb.minZ).toBe(0);
		expect(bb.maxZ).toBe(12);
	});
});

describe('boundingBoxCenter', () => {
	it('returns the center of a bounding box', () => {
		const center = boundingBoxCenter({
			minX: 0,
			maxX: 36,
			minY: 0,
			maxY: 30,
			minZ: 0,
			maxZ: 12
		});
		expect(center.x).toBe(18);
		expect(center.y).toBe(15);
		expect(center.z).toBe(6);
	});

	it('handles non-origin-centered boxes', () => {
		const center = boundingBoxCenter({
			minX: 10,
			maxX: 20,
			minY: 5,
			maxY: 15,
			minZ: -3,
			maxZ: 3
		});
		expect(center.x).toBe(15);
		expect(center.y).toBe(10);
		expect(center.z).toBe(0);
	});
});

describe('boundingBoxMaxExtent', () => {
	it('returns the largest dimension', () => {
		const extent = boundingBoxMaxExtent({
			minX: 0,
			maxX: 36,
			minY: 0,
			maxY: 30,
			minZ: 0,
			maxZ: 12
		});
		expect(extent).toBe(36); // X is largest
	});

	it('works when Y is largest', () => {
		const extent = boundingBoxMaxExtent({
			minX: 0,
			maxX: 10,
			minY: 0,
			maxY: 84, // tall cabinet
			minZ: 0,
			maxZ: 24
		});
		expect(extent).toBe(84);
	});

	it('works when Z is largest', () => {
		const extent = boundingBoxMaxExtent({
			minX: 0,
			maxX: 5,
			minY: 0,
			maxY: 5,
			minZ: 0,
			maxZ: 100
		});
		expect(extent).toBe(100);
	});
});

describe('computePanelPosition', () => {
	const center = { x: 18, y: 15, z: 6 };

	it('centers a panel at origin when not exploded', () => {
		const panel = makePanel({ label: 'left_side', x: 0.375, y: 15, z: 6 });
		const pos = computePanelPosition(panel, center, false);
		expect(pos.x).toBeCloseTo(-17.625); // 0.375 - 18
		expect(pos.y).toBeCloseTo(0); // 15 - 15
		expect(pos.z).toBeCloseTo(0); // 6 - 6
	});

	it('adds explosion offset when exploded', () => {
		const panel = makePanel({ label: 'left_side', x: 0.375, y: 15, z: 6 });
		const pos = computePanelPosition(panel, center, true);
		expect(pos.x).toBeCloseTo(-17.625 - EXPLODE_OFFSET);
		expect(pos.y).toBeCloseTo(0);
		expect(pos.z).toBeCloseTo(0);
	});

	it('right side gets positive X offset when exploded', () => {
		const panel = makePanel({ label: 'right_side', x: 35.625, y: 15, z: 6 });
		const pos = computePanelPosition(panel, center, true);
		expect(pos.x).toBeCloseTo(35.625 - 18 + EXPLODE_OFFSET);
	});

	it('back panel gets -Z offset when exploded', () => {
		const panel = makePanel({ label: 'back', x: 18, y: 15, z: 0.125 });
		const pos = computePanelPosition(panel, center, true);
		expect(pos.z).toBeCloseTo(0.125 - 6 - EXPLODE_OFFSET);
	});

	it('shelf gets +Z offset when exploded', () => {
		const panel = makePanel({ label: 'shelf_1', x: 18, y: 10, z: 6 });
		const pos = computePanelPosition(panel, center, true);
		expect(pos.z).toBeCloseTo(0 + EXPLODE_OFFSET); // 6-6 + offset
	});

	it('unknown label gets no offset when exploded', () => {
		const panel = makePanel({ label: 'mystery', x: 18, y: 15, z: 6 });
		const posExploded = computePanelPosition(panel, center, true);
		const posNormal = computePanelPosition(panel, center, false);
		expect(posExploded).toEqual(posNormal);
	});
});

describe('EXPLODE_OFFSET constant', () => {
	it('is 3 inches', () => {
		expect(EXPLODE_OFFSET).toBe(3);
	});
});
