// TypeScript types mirroring Rust DTOs

// --- Core geometry ---
export interface Point2D {
	x: number;
	y: number;
}

export interface Rect {
	origin: Point2D;
	width: number;
	height: number;
}

// --- Units ---
export type Unit = 'inches' | 'millimeters';

// --- Material ---
export interface Material {
	name: string;
	thickness: number;
	sheet_width?: number | null;
	sheet_length?: number | null;
	material_type: 'plywood' | 'mdf' | 'hardwood' | 'softwood' | 'melamine' | 'particleboard';
}

// --- Tool ---
export interface Tool {
	number: number;
	tool_type: string;
	diameter: number;
	flutes: number;
	cutting_length: number;
	description: string;
}

// --- Cabinet types ---
export type CabinetType =
	| 'basic_box'
	| 'base_cabinet'
	| 'wall_cabinet'
	| 'tall_cabinet'
	| 'sink_base'
	| 'drawer_bank';

export type ShelfJoinery = 'dado' | 'butt';
export type BackJoinery = 'rabbet' | 'nailed_on';
export type ConstructionMethod = 'frameless' | 'face_frame';

export interface ToeKickConfig {
	height: number;
	setback: number;
}

export interface DrawerConfig {
	count: number;
	opening_height: number;
	slide_clearance: number;
}

export interface StretcherConfig {
	front_width: number;
	has_rear: boolean;
}

export interface FaceFrameConfig {
	stile_width: number;
	rail_width: number;
	overhang: number;
	cnc_pocket_holes: boolean;
}

export interface Cabinet {
	name: string;
	cabinet_type: CabinetType;
	width: number;
	height: number;
	depth: number;
	material_thickness: number;
	back_thickness: number;
	shelf_count: number;
	shelf_joinery: ShelfJoinery;
	dado_depth_fraction: number;
	has_back: boolean;
	back_joinery: BackJoinery;
	toe_kick?: ToeKickConfig | null;
	drawers?: DrawerConfig | null;
	stretchers?: StretcherConfig | null;
	construction: ConstructionMethod;
	face_frame?: FaceFrameConfig | null;
}

export interface CabinetEntry {
	name: string;
	cabinet_type: CabinetType;
	width: number;
	height: number;
	depth: number;
	material_thickness: number;
	back_thickness: number;
	shelf_count: number;
	shelf_joinery: ShelfJoinery;
	dado_depth_fraction: number;
	has_back: boolean;
	back_joinery: BackJoinery;
	toe_kick?: ToeKickConfig | null;
	drawers?: DrawerConfig | null;
	stretchers?: StretcherConfig | null;
	construction: ConstructionMethod;
	face_frame?: FaceFrameConfig | null;
	material_ref?: string | null;
	back_material_ref?: string | null;
}

// --- Part ---
export type GrainDirection = 'length_wise' | 'width_wise';

export interface Part {
	label: string;
	rect: Rect;
	thickness: number;
	grain_direction: GrainDirection;
	operations: PartOperation[];
	quantity: number;
}

export type PartOperation =
	| { type: 'dado'; position: number; width: number; depth: number; orientation: string }
	| { type: 'rabbet'; edge: string; width: number; depth: number }
	| { type: 'drill'; x: number; y: number; diameter: number; depth: number }
	| { type: 'pocket_hole'; x: number; y: number; edge: string; cnc_operation: boolean };

// --- Project ---
export interface ProjectMeta {
	name: string;
	units: Unit;
}

export interface Project {
	project: ProjectMeta;
	material?: Material | null;
	back_material?: Material | null;
	cabinet?: Cabinet | null;
	materials: Material[];
	cabinets: CabinetEntry[];
	tools: Tool[];
}

// --- Tagged Part ---
export interface TaggedPart {
	part: Part;
	cabinet_name: string;
	material_name: string;
	material: Material;
}

// --- Nesting ---
export interface NestingConfig {
	sheet_width: number;
	sheet_length: number;
	kerf: number;
	edge_margin: number;
	allow_rotation: boolean;
}

export interface PlacedPart {
	id: string;
	rect: Rect;
	rotated: boolean;
}

export interface SheetLayout {
	sheet_index: number;
	sheet_rect: Rect;
	parts: PlacedPart[];
	waste_area: number;
	utilization: number;
}

export interface NestingResult {
	sheets: SheetLayout[];
	unplaced: string[];
	sheet_count: number;
	overall_utilization: number;
}

export interface MaterialGroupDto {
	material_name: string;
	thickness: number;
	nesting_result: NestingResult;
}

// --- Validation ---
export interface ValidationResult {
	errors: ValidationError[];
	warnings: ValidationWarning[];
}

export type ValidationError =
	| { PartExceedsTravel: { part_label: string; part_width: number; part_height: number; travel_x: number; travel_y: number } }
	| { RpmOutOfRange: { requested: number; min: number; max: number } }
	| { CutDepthExceedsTool: { part_label: string; cut_depth: number; cutting_length: number; tool_description: string } }
	| { GcodeBoundsExceeded: { axis: string; value: number; limit: number } };

export type ValidationWarning =
	| { PartNeedsPreCutting: { part_label: string; part_width: number; part_height: number; travel_x: number; travel_y: number } }
	| { MultipleToolsNoAtc: { tool_count: number } }
	| { SheetExceedsBed: { sheet_width: number; sheet_length: number; travel_x: number; travel_y: number } };

// --- Machine ---
export type Controller = 'linux_cnc' | 'grbl' | 'mach';

export interface MachineInfo {
	name: string;
	controller: Controller;
	travel_x: number;
	travel_y: number;
	travel_z: number;
	max_rpm: number;
	min_rpm: number;
	has_atc: boolean;
}

export interface PostConfig {
	line_numbers: boolean;
	decimal_places: number;
	safe_z: number;
	rapid_z: number;
	program_end: string;
	program_header?: string | null;
}

export interface MachineProfile {
	machine: MachineInfo;
	post: PostConfig;
}

// --- 3D Preview ---
export interface Panel3D {
	label: string;
	width: number;
	height: number;
	depth: number;
	x: number;
	y: number;
	z: number;
	color: string;
}

// --- Cut List ---
export interface CutlistRow {
	cabinet: string;
	label: string;
	material: string;
	width: number;
	height: number;
	thickness: number;
	quantity: number;
	grain: string;
	operations: string[];
}
