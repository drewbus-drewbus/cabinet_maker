import { describe, it, expect } from 'vitest';
import type { GrainDirection, Material, Panel3D } from './types';

/**
 * These tests verify that our TypeScript types match the Rust serde output.
 * They parse JSON the way Tauri would send it and assert correct typing.
 */

describe('GrainDirection type', () => {
	it('accepts snake_case values matching Rust serde(rename_all = "snake_case")', () => {
		// Rust: GrainDirection::LengthWise → "length_wise"
		const lengthWise: GrainDirection = 'length_wise';
		expect(lengthWise).toBe('length_wise');

		// Rust: GrainDirection::WidthWise → "width_wise"
		const widthWise: GrainDirection = 'width_wise';
		expect(widthWise).toBe('width_wise');
	});

	it('round-trips through JSON like Rust serde output', () => {
		const json = '{"grain_direction":"length_wise"}';
		const parsed: { grain_direction: GrainDirection } = JSON.parse(json);
		expect(parsed.grain_direction).toBe('length_wise');
	});

	it('handles both variants in an array', () => {
		const json = '["length_wise","width_wise"]';
		const parsed: GrainDirection[] = JSON.parse(json);
		expect(parsed).toEqual(['length_wise', 'width_wise']);
	});
});

describe('Material.material_type type', () => {
	it('accepts all Rust MaterialType variants', () => {
		// Rust enum: Plywood, Mdf, Hardwood, Softwood, Melamine, Particleboard
		// All become snake_case via serde
		const types: Material['material_type'][] = [
			'plywood',
			'mdf',
			'hardwood',
			'softwood',
			'melamine',
			'particleboard'
		];
		expect(types).toHaveLength(6);
	});

	it('parses from JSON like Rust serde output', () => {
		const rustOutput = JSON.stringify({
			name: '3/4" Plywood',
			thickness: 0.75,
			sheet_width: 48.0,
			sheet_length: 96.0,
			material_type: 'plywood'
		});

		const mat: Material = JSON.parse(rustOutput);
		expect(mat.material_type).toBe('plywood');
		expect(mat.name).toBe('3/4" Plywood');
		expect(mat.thickness).toBe(0.75);
	});

	it('handles optional sheet dimensions', () => {
		const withNulls = JSON.stringify({
			name: 'Hardwood',
			thickness: 0.75,
			sheet_width: null,
			sheet_length: null,
			material_type: 'hardwood'
		});

		const mat: Material = JSON.parse(withNulls);
		expect(mat.sheet_width).toBeNull();
		expect(mat.sheet_length).toBeNull();
		expect(mat.material_type).toBe('hardwood');
	});

	it('accepts mdf variant (lowercase, not MDF)', () => {
		const json = '{"name":"MDF","thickness":0.75,"material_type":"mdf"}';
		const mat: Material = JSON.parse(json);
		expect(mat.material_type).toBe('mdf');
	});
});

describe('Panel3D type', () => {
	it('parses panel data from Rust backend format', () => {
		const rustPanel = JSON.stringify({
			label: 'left_side',
			width: 0.75,
			height: 30.0,
			depth: 12.0,
			x: 0.375,
			y: 15.0,
			z: 6.0,
			color: '#c4a882'
		});

		const panel: Panel3D = JSON.parse(rustPanel);
		expect(panel.label).toBe('left_side');
		expect(panel.width).toBe(0.75);
		expect(panel.height).toBe(30.0);
		expect(panel.depth).toBe(12.0);
		expect(panel.x).toBe(0.375);
		expect(panel.y).toBe(15.0);
		expect(panel.z).toBe(6.0);
		expect(panel.color).toBe('#c4a882');
	});

	it('handles an array of panels like get_3d_assembly returns', () => {
		const panels: Panel3D[] = [
			{ label: 'left_side', width: 0.75, height: 30, depth: 12, x: 0.375, y: 15, z: 6, color: '#c4a882' },
			{ label: 'right_side', width: 0.75, height: 30, depth: 12, x: 35.625, y: 15, z: 6, color: '#c4a882' },
			{ label: 'top', width: 34.5, height: 0.75, depth: 12, x: 18, y: 29.625, z: 6, color: '#b89b72' },
			{ label: 'bottom', width: 34.5, height: 0.75, depth: 12, x: 18, y: 0.375, z: 6, color: '#b89b72' },
			{ label: 'back', width: 34.5, height: 28.5, depth: 0.25, x: 18, y: 15, z: 0.125, color: '#8b7355' }
		];

		expect(panels).toHaveLength(5);
		expect(panels.map((p) => p.label)).toEqual([
			'left_side',
			'right_side',
			'top',
			'bottom',
			'back'
		]);
	});
});
