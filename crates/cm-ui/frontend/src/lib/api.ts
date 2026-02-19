import { invoke } from '@tauri-apps/api/core';
import type {
	Project,
	CabinetEntry,
	Cabinet,
	Part,
	TaggedPart,
	MaterialGroupDto,
	NestingConfig,
	ValidationResult,
	MachineProfile,
	Panel3D,
	CutlistRow
} from './types';

// --- Project commands ---

export async function newProject(): Promise<Project> {
	return invoke('new_project');
}

export async function getProject(): Promise<Project | null> {
	return invoke('get_project');
}

export async function updateProject(project: Project): Promise<void> {
	return invoke('update_project', { project });
}

export async function openProject(path: string): Promise<Project> {
	return invoke('open_project', { path });
}

export async function saveProject(path?: string): Promise<string> {
	return invoke('save_project', { path: path ?? null });
}

export async function loadTemplate(name: string): Promise<Project> {
	return invoke('load_template', { name });
}

export async function listTemplates(): Promise<string[]> {
	return invoke('list_templates');
}

export async function getProjectPath(): Promise<string | null> {
	return invoke('get_project_path');
}

// --- Cabinet commands ---

export async function generateParts(): Promise<TaggedPart[]> {
	return invoke('generate_parts');
}

export async function addCabinet(entry: CabinetEntry): Promise<number> {
	return invoke('add_cabinet', { entry });
}

export async function updateCabinet(index: number, entry: CabinetEntry): Promise<void> {
	return invoke('update_cabinet', { index, entry });
}

export async function removeCabinet(index: number): Promise<void> {
	return invoke('remove_cabinet', { index });
}

export async function previewCabinetParts(cabinet: Cabinet): Promise<Part[]> {
	return invoke('preview_cabinet_parts', { cabinet });
}

export async function get3dAssembly(cabinetIndex: number): Promise<Panel3D[]> {
	return invoke('get_3d_assembly', { cabinetIndex });
}

// --- Nesting commands ---

export async function nestAll(config?: NestingConfig): Promise<MaterialGroupDto[]> {
	return invoke('nest_all', { config: config ?? null });
}

export async function getNestingSvg(
	materialIndex: number,
	sheetIndex: number,
	config?: NestingConfig
): Promise<string> {
	return invoke('get_nesting_svg', {
		materialIndex,
		sheetIndex,
		config: config ?? null
	});
}

// --- G-code commands ---

export async function validateProjectCmd(): Promise<ValidationResult> {
	return invoke('validate_project_cmd');
}

export async function generateGcode(outputDir: string): Promise<string[]> {
	return invoke('generate_gcode', { outputDir });
}

export async function previewGcode(materialIndex: number, sheetIndex: number): Promise<string> {
	return invoke('preview_gcode', { materialIndex, sheetIndex });
}

// --- Machine commands ---

export async function getMachine(): Promise<MachineProfile> {
	return invoke('get_machine');
}

export async function setMachine(profile: MachineProfile): Promise<void> {
	return invoke('set_machine', { profile });
}

export async function loadMachineProfile(path: string): Promise<MachineProfile> {
	return invoke('load_machine_profile', { path });
}

// --- Export commands ---

export async function getCutlist(): Promise<CutlistRow[]> {
	return invoke('get_cutlist');
}

export async function exportCsv(path: string): Promise<void> {
	return invoke('export_csv', { path });
}

export async function exportBomJson(path: string): Promise<void> {
	return invoke('export_bom_json', { path });
}
