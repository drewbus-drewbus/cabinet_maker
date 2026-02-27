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

const BASE_URL = import.meta.env.VITE_API_URL ?? '/api';

let sessionId: string | null = null;

async function ensureSession(): Promise<string> {
	if (sessionId) return sessionId;
	const resp = await fetch(`${BASE_URL}/sessions`, { method: 'POST' });
	if (!resp.ok) throw new Error('Failed to create session');
	const data = await resp.json();
	sessionId = data.id;
	return sessionId!;
}

function sessionUrl(path: string): string {
	return `${BASE_URL}/sessions/${sessionId}${path}`;
}

async function apiGet<T>(path: string): Promise<T> {
	await ensureSession();
	const resp = await fetch(sessionUrl(path));
	if (!resp.ok) {
		const err = await resp.json().catch(() => ({ error: resp.statusText }));
		throw new Error(err.error || resp.statusText);
	}
	return resp.json();
}

async function apiPost<T>(path: string, body?: unknown): Promise<T> {
	await ensureSession();
	const resp = await fetch(sessionUrl(path), {
		method: 'POST',
		headers: body !== undefined ? { 'Content-Type': 'application/json' } : {},
		body: body !== undefined ? JSON.stringify(body) : undefined
	});
	if (!resp.ok) {
		const err = await resp.json().catch(() => ({ error: resp.statusText }));
		throw new Error(err.error || resp.statusText);
	}
	const text = await resp.text();
	return text ? JSON.parse(text) : (undefined as T);
}

async function apiPut<T>(path: string, body?: unknown): Promise<T> {
	await ensureSession();
	const resp = await fetch(sessionUrl(path), {
		method: 'PUT',
		headers: body !== undefined ? { 'Content-Type': 'application/json' } : {},
		body: body !== undefined ? JSON.stringify(body) : undefined
	});
	if (!resp.ok) {
		const err = await resp.json().catch(() => ({ error: resp.statusText }));
		throw new Error(err.error || resp.statusText);
	}
	const text = await resp.text();
	return text ? JSON.parse(text) : (undefined as T);
}

async function apiDelete<T>(path: string): Promise<T> {
	await ensureSession();
	const resp = await fetch(sessionUrl(path), { method: 'DELETE' });
	if (!resp.ok) {
		const err = await resp.json().catch(() => ({ error: resp.statusText }));
		throw new Error(err.error || resp.statusText);
	}
	return resp.json();
}

async function apiGetText(path: string): Promise<string> {
	await ensureSession();
	const resp = await fetch(sessionUrl(path));
	if (!resp.ok) {
		const err = await resp.json().catch(() => ({ error: resp.statusText }));
		throw new Error(err.error || resp.statusText);
	}
	return resp.text();
}

// --- Project commands ---

export async function newProject(): Promise<Project> {
	return apiPost('/project/new');
}

export async function getProject(): Promise<Project | null> {
	return apiGet('/project');
}

export async function updateProject(project: Project): Promise<void> {
	return apiPut('/project', project);
}

export async function openProjectFromToml(tomlContent: string, filename?: string): Promise<Project> {
	return apiPost('/project/open', { toml_content: tomlContent, filename });
}

export async function saveProject(): Promise<string> {
	await ensureSession();
	const resp = await fetch(sessionUrl('/project/save'), { method: 'POST' });
	if (!resp.ok) {
		const err = await resp.json().catch(() => ({ error: resp.statusText }));
		throw new Error(err.error || resp.statusText);
	}
	return resp.text();
}

export async function loadTemplate(name: string): Promise<Project> {
	return apiPost(`/project/template/${name}`);
}

export async function listTemplates(): Promise<string[]> {
	const resp = await fetch(`${BASE_URL}/templates`);
	if (!resp.ok) throw new Error('Failed to list templates');
	return resp.json();
}

// --- Cabinet commands ---

export async function generateParts(): Promise<TaggedPart[]> {
	return apiPost('/parts');
}

export async function addCabinet(entry: CabinetEntry): Promise<number> {
	return apiPost('/cabinets', entry);
}

export async function updateCabinet(index: number, entry: CabinetEntry): Promise<void> {
	return apiPut(`/cabinets/${index}`, entry);
}

export async function removeCabinet(index: number): Promise<void> {
	return apiDelete(`/cabinets/${index}`);
}

export async function previewCabinetParts(cabinet: Cabinet): Promise<Part[]> {
	return apiPost('/cabinets/preview', cabinet);
}

export async function get3dAssembly(cabinetIndex: number): Promise<Panel3D[]> {
	return apiGet(`/cabinets/${cabinetIndex}/3d`);
}

// --- Nesting commands ---

export async function nestAll(config?: NestingConfig): Promise<MaterialGroupDto[]> {
	return apiPost('/nesting', { config: config ?? null });
}

export async function getNestingSvg(
	materialIndex: number,
	sheetIndex: number,
	_config?: NestingConfig
): Promise<string> {
	return apiGetText(`/nesting/${materialIndex}/${sheetIndex}/svg`);
}

// --- G-code commands ---

export async function validateProjectCmd(): Promise<ValidationResult> {
	return apiPost('/gcode/validate');
}

export interface SheetGcode {
	material: string;
	sheet_index: number;
	filename: string;
	gcode: string;
}

export async function generateGcode(): Promise<SheetGcode[]> {
	return apiPost('/gcode/generate', null);
}

export async function previewGcode(materialIndex: number, sheetIndex: number): Promise<string> {
	return apiGetText(`/gcode/${materialIndex}/${sheetIndex}`);
}

// --- Machine commands ---

export async function getMachine(): Promise<MachineProfile> {
	return apiGet('/machine');
}

export async function setMachine(profile: MachineProfile): Promise<void> {
	return apiPut('/machine', profile);
}

export async function uploadMachineProfile(tomlContent: string): Promise<MachineProfile> {
	return apiPost('/machine/upload', { toml_content: tomlContent });
}

// --- Export commands ---

export async function getCutlist(): Promise<CutlistRow[]> {
	return apiGet('/export/cutlist');
}

export async function exportCsv(): Promise<void> {
	await ensureSession();
	const resp = await fetch(sessionUrl('/export/csv'));
	if (!resp.ok) throw new Error('Failed to export CSV');
	const blob = await resp.blob();
	downloadBlob(blob, 'cutlist.csv');
}

export async function exportBomJson(): Promise<void> {
	await ensureSession();
	const resp = await fetch(sessionUrl('/export/bom'));
	if (!resp.ok) throw new Error('Failed to export BOM');
	const blob = await resp.blob();
	downloadBlob(blob, 'bom.json');
}

// --- Toolpath visualization commands ---

export type OperationType =
	| 'profile'
	| 'dado'
	| 'rabbet'
	| 'drill'
	| 'pocket_hole'
	| 'dovetail'
	| 'box_joint'
	| 'mortise'
	| 'tenon'
	| 'dowel';

// Serde externally-tagged enum: simple variants are strings, struct variants are { VariantName: { fields } }
export type Motion =
	| 'Rapid'
	| 'Linear'
	| { ArcCW: { i: number; j: number } }
	| { ArcCCW: { i: number; j: number } }
	| { DrillCycle: { retract_z: number; final_z: number; peck_depth: number } };

export interface ToolpathSegment {
	motion: Motion;
	endpoint: { x: number; y: number };
	z: number;
}

export interface Toolpath {
	tool_number: number;
	rpm: number;
	feed_rate: number;
	plunge_rate: number;
	segments: ToolpathSegment[];
}

export interface AnnotatedToolpath {
	toolpath: Toolpath;
	part_label: string;
	placement_id: string;
	operation_type: OperationType;
}

export interface ToolpathVisualizationDto {
	toolpaths: AnnotatedToolpath[];
	sheet_width: number;
	sheet_height: number;
	total_segments: number;
	rapid_distance: number;
	cut_distance: number;
	part_count: number;
	estimated_time_s: number;
	bounds: [number, number, number, number];
}

export async function getToolpaths(
	materialIndex: number,
	sheetIndex: number
): Promise<ToolpathVisualizationDto> {
	return apiGet(`/gcode/${materialIndex}/${sheetIndex}/toolpaths`);
}

// --- Estimate commands ---

export interface CabinetEstimate {
	name: string;
	cabinet_type: string;
	part_count: number;
	area_sqft: number;
	weight_lb: number;
	hardware_cost: number | null;
	edge_banding_lf: number;
}

export interface QuickEstimate {
	total_parts: number;
	total_area_sqft: number;
	estimated_sheets: number;
	material_cost: number | null;
	hardware_cost: number | null;
	edge_banding_cost: number | null;
	total_estimated_cost: number | null;
	total_weight_lb: number;
	per_cabinet: CabinetEstimate[];
}

export async function getEstimate(): Promise<QuickEstimate> {
	return apiGet('/estimate');
}

// --- Helpers ---

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
