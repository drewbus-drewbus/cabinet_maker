import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { project } from './project';
import {
	pushSnapshot,
	undo,
	redo,
	clearHistory,
	canUndo,
	canRedo
} from './history';
import type { Project } from '../types';

// Mock the Tauri invoke API so api.updateProject doesn't hit a real backend
vi.mock('@tauri-apps/api/core', () => ({
	invoke: vi.fn().mockResolvedValue(undefined)
}));

function makeProject(name: string): Project {
	return {
		project: { name, units: 'inches' },
		material: null,
		back_material: null,
		cabinet: null,
		materials: [],
		cabinets: [],
		tools: []
	};
}

beforeEach(() => {
	clearHistory();
	project.set(null);
});

describe('history store', () => {
	describe('canUndo / canRedo', () => {
		it('starts with both false', () => {
			expect(get(canUndo)).toBe(false);
			expect(get(canRedo)).toBe(false);
		});

		it('canUndo becomes true after pushSnapshot', () => {
			project.set(makeProject('v1'));
			pushSnapshot();
			expect(get(canUndo)).toBe(true);
			expect(get(canRedo)).toBe(false);
		});

		it('canRedo becomes true after undo', async () => {
			project.set(makeProject('v1'));
			pushSnapshot();
			project.set(makeProject('v2'));

			await undo();
			expect(get(canRedo)).toBe(true);
		});
	});

	describe('pushSnapshot', () => {
		it('does nothing when project is null', () => {
			project.set(null);
			pushSnapshot();
			expect(get(canUndo)).toBe(false);
		});

		it('captures the current project state', () => {
			project.set(makeProject('snapshot_test'));
			pushSnapshot();
			expect(get(canUndo)).toBe(true);
		});

		it('captures multiple snapshots', () => {
			project.set(makeProject('v1'));
			pushSnapshot();
			project.set(makeProject('v2'));
			pushSnapshot();
			project.set(makeProject('v3'));
			pushSnapshot();
			// Three snapshots pushed
			expect(get(canUndo)).toBe(true);
		});

		it('clears redo stack on new push', async () => {
			project.set(makeProject('v1'));
			pushSnapshot();
			project.set(makeProject('v2'));

			await undo();
			expect(get(canRedo)).toBe(true);

			// New mutation should clear redo
			pushSnapshot();
			expect(get(canRedo)).toBe(false);
		});

		it('enforces max 50 snapshots', () => {
			for (let i = 0; i < 60; i++) {
				project.set(makeProject(`v${i}`));
				pushSnapshot();
			}
			// After 60 pushes, should still only have 50 in the stack
			// We can verify by undoing 50 times — the 51st should not work
			expect(get(canUndo)).toBe(true);
		});
	});

	describe('undo', () => {
		it('does nothing when stack is empty', async () => {
			project.set(makeProject('current'));
			await undo();
			expect(get(project)?.project.name).toBe('current');
		});

		it('restores the previous snapshot', async () => {
			project.set(makeProject('v1'));
			pushSnapshot();
			project.set(makeProject('v2'));

			await undo();
			expect(get(project)?.project.name).toBe('v1');
		});

		it('pushes current state to redo stack', async () => {
			project.set(makeProject('v1'));
			pushSnapshot();
			project.set(makeProject('v2'));

			await undo();
			expect(get(canRedo)).toBe(true);
		});

		it('supports multiple undos in sequence', async () => {
			project.set(makeProject('v1'));
			pushSnapshot();
			project.set(makeProject('v2'));
			pushSnapshot();
			project.set(makeProject('v3'));

			await undo();
			expect(get(project)?.project.name).toBe('v2');

			await undo();
			expect(get(project)?.project.name).toBe('v1');
		});

		it('canUndo becomes false after exhausting stack', async () => {
			project.set(makeProject('v1'));
			pushSnapshot();
			project.set(makeProject('v2'));

			await undo();
			expect(get(canUndo)).toBe(false);
		});

		it('syncs restored state to backend', async () => {
			const { invoke } = await import('@tauri-apps/api/core');
			(invoke as ReturnType<typeof vi.fn>).mockClear();

			project.set(makeProject('v1'));
			pushSnapshot();
			project.set(makeProject('v2'));

			await undo();

			expect(invoke).toHaveBeenCalledWith('update_project', {
				project: expect.objectContaining({
					project: { name: 'v1', units: 'inches' }
				})
			});
		});
	});

	describe('redo', () => {
		it('does nothing when redo stack is empty', async () => {
			project.set(makeProject('current'));
			await redo();
			expect(get(project)?.project.name).toBe('current');
		});

		it('restores the undone state', async () => {
			project.set(makeProject('v1'));
			pushSnapshot();
			project.set(makeProject('v2'));

			await undo();
			expect(get(project)?.project.name).toBe('v1');

			await redo();
			expect(get(project)?.project.name).toBe('v2');
		});

		it('pushes current state back to undo stack', async () => {
			project.set(makeProject('v1'));
			pushSnapshot();
			project.set(makeProject('v2'));

			await undo();
			await redo();

			// Should be able to undo again
			expect(get(canUndo)).toBe(true);
		});

		it('supports multiple redos in sequence', async () => {
			project.set(makeProject('v1'));
			pushSnapshot();
			project.set(makeProject('v2'));
			pushSnapshot();
			project.set(makeProject('v3'));

			await undo();
			await undo();
			expect(get(project)?.project.name).toBe('v1');

			await redo();
			expect(get(project)?.project.name).toBe('v2');

			await redo();
			expect(get(project)?.project.name).toBe('v3');
		});

		it('canRedo becomes false after exhausting stack', async () => {
			project.set(makeProject('v1'));
			pushSnapshot();
			project.set(makeProject('v2'));

			await undo();
			await redo();
			expect(get(canRedo)).toBe(false);
		});

		it('syncs restored state to backend', async () => {
			const { invoke } = await import('@tauri-apps/api/core');

			project.set(makeProject('v1'));
			pushSnapshot();
			project.set(makeProject('v2'));

			await undo();
			(invoke as ReturnType<typeof vi.fn>).mockClear();

			await redo();

			expect(invoke).toHaveBeenCalledWith('update_project', {
				project: expect.objectContaining({
					project: { name: 'v2', units: 'inches' }
				})
			});
		});
	});

	describe('clearHistory', () => {
		it('clears both undo and redo stacks', async () => {
			project.set(makeProject('v1'));
			pushSnapshot();
			project.set(makeProject('v2'));
			pushSnapshot();
			project.set(makeProject('v3'));

			await undo();
			expect(get(canUndo)).toBe(true);
			expect(get(canRedo)).toBe(true);

			clearHistory();
			expect(get(canUndo)).toBe(false);
			expect(get(canRedo)).toBe(false);
		});

		it('does not affect the current project', () => {
			project.set(makeProject('keep_me'));
			pushSnapshot();
			clearHistory();
			expect(get(project)?.project.name).toBe('keep_me');
		});
	});

	describe('undo/redo with complex project data', () => {
		it('preserves cabinet array through undo/redo cycle', async () => {
			const p1 = makeProject('with_cabinets');
			p1.cabinets = [
				{
					name: 'Upper',
					cabinet_type: 'wall_cabinet',
					width: 30,
					height: 30,
					depth: 12,
					material_thickness: 0.75,
					back_thickness: 0.25,
					shelf_count: 2,
					shelf_joinery: 'dado',
					dado_depth_fraction: 0.5,
					has_back: true,
					back_joinery: 'rabbet',
					construction: 'frameless'
				}
			];

			project.set(p1);
			pushSnapshot();

			// Mutate — add another cabinet
			const p2 = JSON.parse(JSON.stringify(get(project))) as Project;
			p2.cabinets.push({
				name: 'Lower',
				cabinet_type: 'base_cabinet',
				width: 36,
				height: 34.5,
				depth: 24,
				material_thickness: 0.75,
				back_thickness: 0.25,
				shelf_count: 1,
				shelf_joinery: 'dado',
				dado_depth_fraction: 0.5,
				has_back: true,
				back_joinery: 'rabbet',
				construction: 'frameless',
				toe_kick: { height: 4, setback: 3 }
			});
			project.set(p2);

			await undo();
			const restored = get(project)!;
			expect(restored.cabinets).toHaveLength(1);
			expect(restored.cabinets[0].name).toBe('Upper');
			expect(restored.cabinets[0].cabinet_type).toBe('wall_cabinet');

			await redo();
			const redone = get(project)!;
			expect(redone.cabinets).toHaveLength(2);
			expect(redone.cabinets[1].name).toBe('Lower');
			expect(redone.cabinets[1].toe_kick?.height).toBe(4);
		});

		it('preserves materials and tools through undo', async () => {
			const p = makeProject('with_materials');
			p.materials = [
				{
					name: '3/4" Plywood',
					thickness: 0.75,
					sheet_width: 48,
					sheet_length: 96,
					material_type: 'plywood'
				}
			];
			p.tools = [
				{
					number: 1,
					tool_type: 'endmill',
					diameter: 0.25,
					flutes: 2,
					cutting_length: 1.0,
					description: '1/4" Endmill'
				}
			];

			project.set(p);
			pushSnapshot();

			// Remove materials
			const p2 = JSON.parse(JSON.stringify(get(project))) as Project;
			p2.materials = [];
			p2.tools = [];
			project.set(p2);

			await undo();
			const restored = get(project)!;
			expect(restored.materials).toHaveLength(1);
			expect(restored.materials[0].name).toBe('3/4" Plywood');
			expect(restored.tools).toHaveLength(1);
			expect(restored.tools[0].description).toBe('1/4" Endmill');
		});
	});

	describe('max snapshot limit enforcement', () => {
		it('drops oldest snapshot when exceeding 50', async () => {
			// Push 55 snapshots with incrementing names
			for (let i = 0; i < 55; i++) {
				project.set(makeProject(`v${i}`));
				pushSnapshot();
			}
			project.set(makeProject('final'));

			// Undo 50 times (max stack size)
			for (let i = 0; i < 50; i++) {
				await undo();
			}
			// The oldest 5 (v0-v4) should have been dropped
			// After 50 undos we should be at v5
			expect(get(project)?.project.name).toBe('v5');

			// Stack should be exhausted
			expect(get(canUndo)).toBe(false);
		});
	});

	describe('edge cases', () => {
		it('handles rapid undo-redo-undo cycles', async () => {
			project.set(makeProject('v1'));
			pushSnapshot();
			project.set(makeProject('v2'));
			pushSnapshot();
			project.set(makeProject('v3'));

			await undo();
			await redo();
			await undo();
			await undo();
			expect(get(project)?.project.name).toBe('v1');

			await redo();
			await redo();
			expect(get(project)?.project.name).toBe('v3');
		});

		it('undo when project is null does nothing', async () => {
			pushSnapshot(); // no-op since project is null
			project.set(null);
			await undo(); // should not throw
			expect(get(project)).toBeNull();
		});

		it('handles backend sync error gracefully', async () => {
			const { invoke } = await import('@tauri-apps/api/core');
			(invoke as ReturnType<typeof vi.fn>).mockRejectedValueOnce(new Error('Backend down'));

			project.set(makeProject('v1'));
			pushSnapshot();
			project.set(makeProject('v2'));

			// Should not throw even if sync fails
			await undo();
			expect(get(project)?.project.name).toBe('v1');
		});

		it('snapshot captures deep clone, not reference', async () => {
			const p = makeProject('mutable');
			p.cabinets = [
				{
					name: 'Test',
					cabinet_type: 'basic_box',
					width: 36,
					height: 30,
					depth: 12,
					material_thickness: 0.75,
					back_thickness: 0.25,
					shelf_count: 2,
					shelf_joinery: 'dado',
					dado_depth_fraction: 0.5,
					has_back: true,
					back_joinery: 'rabbet',
					construction: 'frameless'
				}
			];
			project.set(p);
			pushSnapshot();

			// Mutate the live project
			p.cabinets[0].width = 999;
			project.set(p);

			// Undo should restore the width from the snapshot, not the mutated reference
			await undo();
			expect(get(project)?.cabinets[0].width).toBe(36);
		});
	});
});
