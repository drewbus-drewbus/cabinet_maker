mod bom;

use std::fs;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use cm_cabinet::project::{Project, TaggedPart};
use cm_cabinet::part::PartOperation;
use cm_cam::ops::{
    generate_profile_cut, generate_dado_toolpath, generate_rabbet_toolpath, generate_drill,
    generate_dovetail_toolpath, generate_box_joint_toolpath, generate_mortise_toolpath,
    generate_tenon_toolpath, generate_dowel_holes,
    CamConfig, RabbetEdge, DovetailEdge,
};
use cm_cam::{Toolpath, arc_fit, optimize_rapid_order, apply_corner_fillets, FilletStyle};
use cm_core::geometry::{Point2D, Rect};
use cm_core::tool::Tool;
use cm_nesting::packer::{NestingConfig, NestingPart, nest_parts};
use cm_post::gcode::GCodeEmitter;
use cm_post::machine::MachineProfile;
use cm_post::validate::{self, PartInfo};

#[derive(Parser)]
#[command(name = "cabinet-maker", version, about = "Generate CNC G-code from cabinet designs")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to the project TOML file (shorthand for `generate <file>`)
    project_file: Option<PathBuf>,

    /// Path to the machine profile TOML file (optional, defaults to built-in PCNC 1100)
    #[arg(short, long, global = true)]
    machine: Option<PathBuf>,

    /// Output directory for generated files
    #[arg(short, long, default_value = "output", global = true)]
    output_dir: PathBuf,

    /// Spindle RPM (overrides machine profile default)
    #[arg(long, global = true)]
    rpm: Option<f64>,

    /// Skip validation checks
    #[arg(long, global = true)]
    no_validate: bool,

    /// Export CSV cut list
    #[arg(long, global = true)]
    export_csv: bool,

    /// Export SVG nesting layout
    #[arg(long, global = true)]
    export_svg: bool,

    /// Export JSON BOM (bill of materials)
    #[arg(long, global = true)]
    export_bom: bool,

    /// Skip automatic hardware boring pattern generation
    #[arg(long, global = true)]
    no_hardware: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate G-code from a TOML project file
    Generate {
        /// Path to the project TOML file
        project_file: PathBuf,
    },
    /// Import a DXF file and convert to a TOML project
    Import {
        /// Path to the DXF file
        dxf_file: PathBuf,
        /// Material thickness
        #[arg(short, long, default_value = "0.75")]
        thickness: f64,
        /// Output TOML file path
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Import mode: "layer" for layer-based, "raw" for raw rectangles
        #[arg(long, default_value = "raw")]
        mode: String,
    },
    /// Import a DXF file and directly generate G-code (import + generate in one step)
    Cut {
        /// Path to the DXF file
        dxf_file: PathBuf,
        /// Material thickness
        #[arg(short, long, default_value = "0.75")]
        thickness: f64,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Generate { project_file }) => {
            run_generate(project_file, &cli)?;
        }
        Some(Commands::Import { dxf_file, thickness, output, mode }) => {
            run_import(dxf_file, *thickness, output.as_deref(), mode, &cli)?;
        }
        Some(Commands::Cut { dxf_file, thickness }) => {
            run_cut(dxf_file, *thickness, &cli)?;
        }
        None => {
            // Backward compat: if a positional project_file is given without a subcommand
            if let Some(ref project_file) = cli.project_file {
                run_generate(project_file, &cli)?;
            } else {
                eprintln!("Usage: cabinet-maker <PROJECT_FILE> or cabinet-maker generate <PROJECT_FILE>");
                eprintln!("Run 'cabinet-maker --help' for more information.");
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

fn load_machine(cli: &Cli) -> Result<MachineProfile, Box<dyn std::error::Error>> {
    if let Some(ref machine_path) = cli.machine {
        let machine_toml = fs::read_to_string(machine_path)?;
        Ok(MachineProfile::from_toml(&machine_toml)?)
    } else {
        Ok(MachineProfile::tormach_pcnc1100())
    }
}

fn run_generate(project_file: &PathBuf, cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    // Load project
    let project_toml = fs::read_to_string(project_file)?;
    let project = Project::from_toml(&project_toml)?;

    println!("Project: {}", project.project.name);
    println!("Units: {:?}", project.project.units);

    // Print cabinet info and validate
    let all_cabs = project.all_cabinets();
    for cab in &all_cabs {
        println!("Cabinet: {} ({:?}, {:.1}\" x {:.1}\" x {:.1}\")",
            cab.name, cab.cabinet_type, cab.width, cab.height, cab.depth,
        );

        let cab_issues = cm_cabinet::cabinet::validate_cabinet(cab);
        for issue in &cab_issues {
            let prefix = match issue.severity {
                cm_cabinet::cabinet::ValidationSeverity::Warning => "WARNING",
                cm_cabinet::cabinet::ValidationSeverity::Error => "ERROR",
            };
            println!("  {}: {}", prefix, issue.message);
        }
        let has_cab_error = cab_issues.iter().any(|i|
            i.severity == cm_cabinet::cabinet::ValidationSeverity::Error);
        if has_cab_error && !cli.no_validate {
            eprintln!("Cabinet validation failed — aborting.");
            std::process::exit(1);
        }
    }

    // Load or default machine profile
    let machine = load_machine(cli)?;
    println!("Machine: {}", machine.machine.name);

    // Generate all parts (handles both legacy and multi-cabinet)
    let mut tagged_parts = project.generate_all_parts();

    // Apply hardware boring patterns (shelf pins, hinge plates, slide holes)
    if !cli.no_hardware {
        let mut hw_op_count = 0;
        for entry in &all_cabs {
            let hw_ops = cm_hardware::generate_all_hardware_ops(entry);
            for hw_op in hw_ops {
                if let Some(tp) = tagged_parts.iter_mut().find(|tp|
                    tp.cabinet_name == entry.name && tp.part.label == hw_op.target_part
                ) {
                    tp.part.operations.push(hw_op.operation);
                    hw_op_count += 1;
                }
            }
        }
        if hw_op_count > 0 {
            println!("Hardware: {} drill operations added", hw_op_count);
        }
    }

    println!("\nGenerated {} part entries:", tagged_parts.len());
    for tp in &tagged_parts {
        let prefix = if all_cabs.len() > 1 {
            format!("{}/", tp.cabinet_name)
        } else {
            String::new()
        };
        println!(
            "  {}{} - {:.3}\" x {:.3}\" (qty: {}, ops: {}) [{}]",
            prefix,
            tp.part.label,
            tp.part.rect.width,
            tp.part.rect.height,
            tp.part.quantity,
            tp.part.operations.len(),
            tp.material_name,
        );
    }

    // Determine tool and RPM
    let tool = project
        .tools
        .first()
        .cloned()
        .unwrap_or_else(Tool::quarter_inch_endmill);
    let rpm = cli.rpm.unwrap_or(machine.machine.max_rpm * 0.9);

    // --- Validation ---
    if !cli.no_validate {
        let part_infos: Vec<PartInfo> = tagged_parts.iter().map(|tp| {
            let max_op_depth = tp.part.operations.iter().map(|op| match op {
                PartOperation::Dado(d) => d.depth,
                PartOperation::Rabbet(r) => r.depth,
                PartOperation::Drill(d) => d.depth,
                PartOperation::PocketHole(_) => 0.0,
                PartOperation::Dovetail(d) => d.depth,
                PartOperation::BoxJoint(b) => b.depth,
                PartOperation::Mortise(m) => m.depth,
                PartOperation::Tenon(t) => t.shoulder_depth,
                PartOperation::Dowel(d) => d.depth,
            }).fold(0.0f64, f64::max);
            PartInfo {
                label: format!("{}/{}", tp.cabinet_name, tp.part.label),
                width: tp.part.rect.width,
                height: tp.part.rect.height,
                thickness: tp.part.thickness,
                max_operation_depth: max_op_depth,
            }
        }).collect();

        // Use the first material's sheet size for validation
        let primary_mat = project.primary_material();
        let validation = validate::validate_project(
            &part_infos,
            &project.tools,
            rpm,
            &machine,
            primary_mat.and_then(|m| m.sheet_width),
            primary_mat.and_then(|m| m.sheet_length),
        );

        if validation.has_warnings() {
            println!("\nValidation warnings:");
            for warning in &validation.warnings {
                println!("  WARNING: {}", warning);
            }
        }

        if validation.has_errors() {
            eprintln!("\nValidation ERRORS (aborting):");
            for error in &validation.errors {
                eprintln!("  ERROR: {}", error);
            }
            std::process::exit(1);
        }
    }

    // --- Group by material and nest/generate per group ---
    let groups = Project::group_parts_by_material(&tagged_parts);

    let cam_config = CamConfig {
        safe_z: machine.post.safe_z,
        rapid_z: machine.post.rapid_z,
        ..Default::default()
    };

    // Create output directory
    fs::create_dir_all(&cli.output_dir)?;

    let emitter = GCodeEmitter::new(&machine, project.project.units);
    let multi_material = groups.len() > 1;

    for group in &groups {
        let mat = &group.material;
        let mat_slug = slugify(&group.material_name);

        let group_output_dir = if multi_material {
            let dir = cli.output_dir.join(&mat_slug);
            fs::create_dir_all(&dir)?;
            dir
        } else {
            cli.output_dir.clone()
        };

        println!("\n--- Material: {} ({:.3}\" thick) ---", group.material_name, mat.thickness);

        // Build nesting parts for this material group
        let nesting_config = NestingConfig {
            sheet_width: mat.sheet_width.unwrap_or(48.0),
            sheet_length: mat.sheet_length.unwrap_or(96.0),
            kerf: 0.25,
            edge_margin: 0.5,
            allow_rotation: false,
            guillotine_compatible: false,
        };

        let mut nesting_parts = Vec::new();
        for tp in &group.parts {
            for i in 0..tp.part.quantity {
                let prefix = if all_cabs.len() > 1 {
                    format!("{}/", tp.cabinet_name)
                } else {
                    String::new()
                };
                let id = if tp.part.quantity > 1 {
                    format!("{}{}_{}", prefix, tp.part.label, i + 1)
                } else {
                    format!("{}{}", prefix, tp.part.label)
                };
                nesting_parts.push(NestingPart {
                    id,
                    width: tp.part.rect.width,
                    height: tp.part.rect.height,
                    can_rotate: false,
                });
            }
        }

        let nesting_result = nest_parts(&nesting_parts, &nesting_config);

        println!("Nesting result:");
        println!("  Sheets required: {}", nesting_result.sheet_count);
        println!("  Utilization: {:.1}%", nesting_result.overall_utilization);
        if !nesting_result.unplaced.is_empty() {
            println!("  WARNING: {} parts could not be placed:", nesting_result.unplaced.len());
            for id in &nesting_result.unplaced {
                println!("    - {}", id);
            }
        }

        for sheet in &nesting_result.sheets {
            println!("  Sheet {}: {} parts, {:.1}% utilization",
                sheet.sheet_index + 1, sheet.parts.len(), sheet.utilization);
        }

        println!("\nTool: {} (T{}, {} dia)", tool.description, tool.number, tool.diameter);
        println!("RPM: {}", rpm as u32);

        // Generate G-code per sheet
        for sheet in &nesting_result.sheets {
            let mut sheet_toolpaths: Vec<Toolpath> = Vec::new();

            println!("\nGenerating toolpaths for sheet {}:", sheet.sheet_index + 1);

            for placed in &sheet.parts {
                let base_label = strip_nesting_id(&placed.id);

                // Find the matching tagged part
                let tp_match = group.parts.iter()
                    .find(|tp| tp.part.label == base_label || placed.id.ends_with(&tp.part.label))
                    .expect(&format!("Part definition not found for '{}'", placed.id));

                let positioned_rect = Rect::new(
                    placed.rect.origin,
                    placed.rect.width,
                    placed.rect.height,
                );

                println!("  {} at ({:.3}\", {:.3}\")", placed.id, placed.rect.origin.x, placed.rect.origin.y);

                for op in &tp_match.part.operations {
                    match op {
                        PartOperation::Dado(dado) => {
                            let horizontal = dado.orientation == cm_cabinet::part::DadoOrientation::Horizontal;
                            let tp = generate_dado_toolpath(
                                &positioned_rect, dado.position, dado.width, dado.depth,
                                horizontal, &tool, rpm, &cam_config,
                            );
                            println!("    - Dado at {:.3}\", width {:.3}\", depth {:.3}\"",
                                dado.position, dado.width, dado.depth);
                            sheet_toolpaths.push(tp);
                        }
                        PartOperation::Rabbet(rabbet) => {
                            let edge = match rabbet.edge {
                                cm_cabinet::part::Edge::Top => RabbetEdge::Top,
                                cm_cabinet::part::Edge::Bottom => RabbetEdge::Bottom,
                                cm_cabinet::part::Edge::Left => RabbetEdge::Left,
                                cm_cabinet::part::Edge::Right => RabbetEdge::Right,
                            };
                            let tp = generate_rabbet_toolpath(
                                &positioned_rect, edge, rabbet.width, rabbet.depth,
                                &tool, rpm, &cam_config,
                            );
                            println!("    - Rabbet on {:?} edge, width {:.3}\", depth {:.3}\"",
                                rabbet.edge, rabbet.width, rabbet.depth);
                            sheet_toolpaths.push(tp);
                        }
                        PartOperation::Drill(drill) => {
                            let tp = generate_drill(
                                Point2D::new(
                                    positioned_rect.min_x() + drill.x,
                                    positioned_rect.min_y() + drill.y,
                                ),
                                drill.depth, &tool, rpm, &cam_config,
                            );
                            println!("    - Drill at ({:.3}\", {:.3}\"), depth {:.3}\"",
                                drill.x, drill.y, drill.depth);
                            sheet_toolpaths.push(tp);
                        }
                        PartOperation::PocketHole(ph) => {
                            if ph.cnc_operation {
                                let tp = generate_drill(
                                    Point2D::new(
                                        positioned_rect.min_x() + ph.x,
                                        positioned_rect.min_y() + ph.y,
                                    ),
                                    0.625,
                                    &tool, rpm, &cam_config,
                                );
                                println!("    - Pocket hole at ({:.3}\", {:.3}\")", ph.x, ph.y);
                                sheet_toolpaths.push(tp);
                            } else {
                                println!("    - Pocket hole at ({:.3}\", {:.3}\") (off-CNC)", ph.x, ph.y);
                            }
                        }
                        PartOperation::Dovetail(dt) => {
                            let edge = match dt.edge {
                                cm_cabinet::part::Edge::Top => DovetailEdge::Top,
                                cm_cabinet::part::Edge::Bottom => DovetailEdge::Bottom,
                                cm_cabinet::part::Edge::Left => DovetailEdge::Left,
                                cm_cabinet::part::Edge::Right => DovetailEdge::Right,
                            };
                            let tp = generate_dovetail_toolpath(
                                &positioned_rect, edge, dt.tail_count,
                                dt.tail_width, dt.pin_width, dt.depth,
                                &tool, rpm, &cam_config,
                            );
                            println!("    - Dovetail on {:?} edge, {} tails, depth {:.3}\"",
                                dt.edge, dt.tail_count, dt.depth);
                            sheet_toolpaths.push(tp);
                        }
                        PartOperation::BoxJoint(bj) => {
                            let edge = match bj.edge {
                                cm_cabinet::part::Edge::Top => DovetailEdge::Top,
                                cm_cabinet::part::Edge::Bottom => DovetailEdge::Bottom,
                                cm_cabinet::part::Edge::Left => DovetailEdge::Left,
                                cm_cabinet::part::Edge::Right => DovetailEdge::Right,
                            };
                            let tp = generate_box_joint_toolpath(
                                &positioned_rect, edge, bj.finger_width,
                                bj.finger_count, bj.depth,
                                &tool, rpm, &cam_config,
                            );
                            println!("    - Box joint on {:?} edge, {} fingers, depth {:.3}\"",
                                bj.edge, bj.finger_count, bj.depth);
                            sheet_toolpaths.push(tp);
                        }
                        PartOperation::Mortise(m) => {
                            let tp = generate_mortise_toolpath(
                                &positioned_rect, m.x, m.y,
                                m.width, m.length, m.depth,
                                &tool, rpm, &cam_config,
                            );
                            println!("    - Mortise at ({:.3}\", {:.3}\"), {:.3}\" x {:.3}\" x {:.3}\"",
                                m.x, m.y, m.width, m.length, m.depth);
                            sheet_toolpaths.push(tp);
                        }
                        PartOperation::Tenon(t) => {
                            let edge = match t.edge {
                                cm_cabinet::part::Edge::Top => DovetailEdge::Top,
                                cm_cabinet::part::Edge::Bottom => DovetailEdge::Bottom,
                                cm_cabinet::part::Edge::Left => DovetailEdge::Left,
                                cm_cabinet::part::Edge::Right => DovetailEdge::Right,
                            };
                            let tp = generate_tenon_toolpath(
                                &positioned_rect, edge, t.thickness,
                                t.width, t.length, t.shoulder_depth,
                                &tool, rpm, &cam_config,
                            );
                            println!("    - Tenon on {:?} edge, {:.3}\" x {:.3}\" x {:.3}\"",
                                t.edge, t.thickness, t.width, t.length);
                            sheet_toolpaths.push(tp);
                        }
                        PartOperation::Dowel(d) => {
                            let hole_positions: Vec<(f64, f64)> = d.holes
                                .iter()
                                .map(|h| (h.x, h.y))
                                .collect();
                            let tp = generate_dowel_holes(
                                &positioned_rect, &hole_positions,
                                d.dowel_diameter, d.depth,
                                &tool, rpm, &cam_config,
                            );
                            println!("    - Dowel holes: {} holes, {:.3}\" dia, depth {:.3}\"",
                                d.holes.len(), d.dowel_diameter, d.depth);
                            sheet_toolpaths.push(tp);
                        }
                    }
                }

                // Profile cut at nested position
                let tp = generate_profile_cut(&positioned_rect, tp_match.part.thickness, &tool, rpm, &cam_config);
                println!("    - Profile cut: {:.3}\" x {:.3}\" through {:.3}\"",
                    positioned_rect.width, positioned_rect.height, tp_match.part.thickness);
                sheet_toolpaths.push(tp);
            }

            // Apply dog-bone corner fillets to dado/rabbet toolpaths
            for tp in &mut sheet_toolpaths {
                apply_corner_fillets(tp, tool.diameter / 2.0, FilletStyle::DogBone);
            }

            // Arc fit: collapse linear segments into arcs where possible
            for tp in &mut sheet_toolpaths {
                arc_fit(tp, 0.001);
            }

            // Optimize inter-part rapid travel order
            optimize_rapid_order(&mut sheet_toolpaths);

            // Post-generation toolpath validation
            if !cli.no_validate {
                let tp_validation = validate::validate_toolpaths(&sheet_toolpaths, &machine);
                if tp_validation.has_errors() {
                    eprintln!("\nToolpath validation ERRORS on sheet {} (aborting):", sheet.sheet_index + 1);
                    for error in &tp_validation.errors {
                        eprintln!("  ERROR: {}", error);
                    }
                    std::process::exit(1);
                }
            }

            // Emit G-code for this sheet
            let gcode = emitter.emit(&sheet_toolpaths);
            let filename = if nesting_result.sheet_count > 1 {
                format!("sheet-{}.nc", sheet.sheet_index + 1)
            } else {
                "program.nc".into()
            };
            let output_path = group_output_dir.join(&filename);
            fs::write(&output_path, &gcode)?;

            println!("  G-code written to: {} ({} lines, {} toolpaths)",
                output_path.display(),
                gcode.lines().count(),
                sheet_toolpaths.len(),
            );
        }

        // Write cut list for this material group
        write_cutlist(&group_output_dir, &project, &group.parts, &nesting_result, &group.material_name)?;

        // Optional exports
        if cli.export_csv {
            write_csv(&group_output_dir, &project, &group.parts, &group.material_name)?;
        }
        if cli.export_svg {
            write_svg(&group_output_dir, &nesting_result, &nesting_config)?;
        }
    }

    // Comprehensive BOM (outside the per-material loop — covers all cabinets/materials)
    if cli.export_bom {
        let primary_mat = project.primary_material();
        let cost_per_sheet = primary_mat.and_then(|m| m.cost_per_unit);
        // Sum sheets across all material groups
        let total_sheets: u32 = groups.iter()
            .map(|g| {
                let config = NestingConfig {
                    sheet_width: g.material.sheet_width.unwrap_or(48.0),
                    sheet_length: g.material.sheet_length.unwrap_or(96.0),
                    kerf: 0.25,
                    edge_margin: 0.5,
                    allow_rotation: false,
                    guillotine_compatible: false,
                };
                let mut np = Vec::new();
                for tp in &g.parts {
                    for i in 0..tp.part.quantity {
                        np.push(NestingPart {
                            id: format!("{}_{}", tp.part.label, i),
                            width: tp.part.rect.width,
                            height: tp.part.rect.height,
                            can_rotate: false,
                        });
                    }
                }
                nest_parts(&np, &config).sheet_count as u32
            })
            .sum();

        let comprehensive_bom = bom::generate_bom(&project, &tagged_parts, total_sheets, cost_per_sheet);
        let bom_json = serde_json::to_string_pretty(&comprehensive_bom)?;
        let bom_path = cli.output_dir.join("bom.json");
        fs::write(&bom_path, &bom_json)?;
        println!("Comprehensive BOM written to: {}", bom_path.display());
    }

    println!("\nDone. {} material group(s) processed.", groups.len());

    Ok(())
}

fn run_import(
    dxf_file: &PathBuf,
    thickness: f64,
    output: Option<&std::path::Path>,
    mode: &str,
    _cli: &Cli,
) -> Result<(), Box<dyn std::error::Error>> {
    let import_mode = match mode {
        "layer" => cm_import::ImportMode::LayerBased,
        "raw" => cm_import::ImportMode::Raw,
        other => {
            eprintln!("Unknown import mode '{}'. Use 'layer' or 'raw'.", other);
            std::process::exit(1);
        }
    };

    let options = cm_import::ImportOptions {
        mode: import_mode,
        thickness,
        ..Default::default()
    };

    println!("Importing DXF: {}", dxf_file.display());
    let result = cm_import::import_dxf(dxf_file, &options)?;

    println!("Imported {} parts ({} entities skipped)", result.parts.len(), result.skipped_entities);
    for warning in &result.warnings {
        println!("  WARNING: {}", warning);
    }
    for part in &result.parts {
        println!("  {} - {:.3}\" x {:.3}\" ({} ops)",
            part.label, part.rect.width, part.rect.height, part.operations.len());
    }

    // Build a minimal Project TOML
    let output_path = output
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| {
            let stem = dxf_file.file_stem().unwrap_or_default().to_string_lossy();
            PathBuf::from(format!("{}.toml", stem))
        });

    let project = build_project_from_parts(&result.parts, thickness, dxf_file);
    let toml_str = project.to_toml()?;
    fs::write(&output_path, &toml_str)?;

    println!("Project written to: {}", output_path.display());
    println!("Run: cabinet-maker generate {}", output_path.display());

    Ok(())
}

fn run_cut(
    dxf_file: &PathBuf,
    thickness: f64,
    cli: &Cli,
) -> Result<(), Box<dyn std::error::Error>> {
    let options = cm_import::ImportOptions {
        mode: cm_import::ImportMode::Raw,
        thickness,
        ..Default::default()
    };

    println!("Importing DXF: {}", dxf_file.display());
    let result = cm_import::import_dxf(dxf_file, &options)?;
    println!("Imported {} parts", result.parts.len());

    // Build a project and write to temp, then generate
    let project = build_project_from_parts(&result.parts, thickness, dxf_file);

    // Write a temp TOML file
    let stem = dxf_file.file_stem().unwrap_or_default().to_string_lossy();
    let temp_path = cli.output_dir.join(format!("{}.toml", stem));
    fs::create_dir_all(&cli.output_dir)?;
    let toml_str = project.to_toml()?;
    fs::write(&temp_path, &toml_str)?;

    // Now run generate on it
    run_generate(&temp_path, cli)?;

    Ok(())
}

fn build_project_from_parts(
    parts: &[cm_cabinet::part::Part],
    thickness: f64,
    source_file: &PathBuf,
) -> Project {
    use cm_cabinet::cabinet::{Cabinet, CabinetType, ShelfJoinery, BackJoinery};
    use cm_core::material::{Material, MaterialType};
    use cm_core::units::Unit;

    let stem = source_file.file_stem().unwrap_or_default().to_string_lossy();

    // Find bounding dimensions from imported parts
    let max_width = parts.iter().map(|p| p.rect.width).fold(0.0f64, f64::max);
    let max_height = parts.iter().map(|p| p.rect.height).fold(0.0f64, f64::max);

    Project {
        project: cm_cabinet::project::ProjectMeta {
            name: format!("Imported: {}", stem),
            units: Unit::Inches,
        },
        material: Some(Material {
            name: format!("{:.3}\" Plywood", thickness),
            thickness,
            sheet_width: Some(48.0),
            sheet_length: Some(96.0),
            cost_per_unit: None,
            material_type: MaterialType::Plywood,
            density_lb_per_ft3: None,
        }),
        back_material: None,
        cabinet: Some(Cabinet {
            name: stem.to_string(),
            cabinet_type: CabinetType::BasicBox,
            width: max_width,
            height: max_height,
            depth: 12.0,
            material_thickness: thickness,
            back_thickness: 0.25,
            shelf_count: 0,
            shelf_joinery: ShelfJoinery::Butt,
            dado_depth_fraction: 0.5,
            has_back: false,
            back_joinery: BackJoinery::NailedOn,
            toe_kick: None,
            drawers: None,
            stretchers: None,
            construction: cm_cabinet::cabinet::ConstructionMethod::Frameless,
            face_frame: None,
            corner_type: None,
            plumbing_cutout: None,
        }),
        materials: vec![],
        cabinets: vec![],
        tools: vec![cm_core::tool::Tool::quarter_inch_endmill()],
    }
}

/// Strip the cabinet prefix and/or quantity suffix from a nesting part ID.
/// E.g., "sink_base/left_side_1" → "left_side", "shelf_2" → "shelf"
fn strip_nesting_id(id: &str) -> String {
    // Strip cabinet prefix if present (e.g., "sink_base/left_side" → "left_side")
    let label = if let Some(slash) = id.find('/') {
        &id[slash + 1..]
    } else {
        id
    };

    // Strip quantity suffix (e.g., "shelf_2" → "shelf")
    if let Some(last_underscore) = label.rfind('_') {
        let suffix = &label[last_underscore + 1..];
        if suffix.parse::<u32>().is_ok() {
            return label[..last_underscore].to_string();
        }
    }
    label.to_string()
}

/// Create a filesystem-safe slug from a material name.
fn slugify(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .replace("__", "_")
        .trim_matches('_')
        .to_string()
}

fn write_cutlist(
    output_dir: &PathBuf,
    project: &Project,
    parts: &[&TaggedPart],
    nesting_result: &cm_nesting::packer::NestingResult,
    material_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let cutlist_path = output_dir.join("cutlist.txt");
    let mut cutlist = String::new();
    cutlist.push_str(&format!("Cut List: {}\n", project.project.name));
    cutlist.push_str(&format!("Material: {}\n", material_name));
    cutlist.push_str(&format!("Sheets required: {}\n\n", nesting_result.sheet_count));
    cutlist.push_str(&format!("{:<25} {:>10} {:>10} {:>5} {:>5}\n", "Part", "Width", "Height", "Qty", "Ops"));
    cutlist.push_str(&format!("{:-<60}\n", ""));
    for tp in parts {
        let label = if project.all_cabinets().len() > 1 {
            format!("{}/{}", tp.cabinet_name, tp.part.label)
        } else {
            tp.part.label.clone()
        };
        cutlist.push_str(&format!(
            "{:<25} {:>9.3}\" {:>9.3}\" {:>5} {:>5}\n",
            label, tp.part.rect.width, tp.part.rect.height, tp.part.quantity, tp.part.operations.len(),
        ));
    }

    cutlist.push_str(&format!("\nNesting Layout:\n"));
    for sheet in &nesting_result.sheets {
        cutlist.push_str(&format!("  Sheet {} ({:.1}% utilization):\n", sheet.sheet_index + 1, sheet.utilization));
        for placed in &sheet.parts {
            cutlist.push_str(&format!(
                "    {:<25} at ({:>7.3}\", {:>7.3}\")\n",
                placed.id, placed.rect.origin.x, placed.rect.origin.y,
            ));
        }
    }

    fs::write(&cutlist_path, &cutlist)?;
    println!("Cut list written to: {}", cutlist_path.display());
    Ok(())
}

/// Export CSV cut list (spreadsheet-friendly).
fn write_csv(
    output_dir: &PathBuf,
    project: &Project,
    parts: &[&TaggedPart],
    material_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let csv_path = output_dir.join("cutlist.csv");
    let multi = project.all_cabinets().len() > 1;
    let mut csv = String::new();

    // Header
    if multi {
        csv.push_str("Cabinet,Part,Width,Height,Thickness,Quantity,Operations,Material\n");
    } else {
        csv.push_str("Part,Width,Height,Thickness,Quantity,Operations,Material\n");
    }

    for tp in parts {
        if multi {
            csv.push_str(&format!(
                "{},{},{:.4},{:.4},{:.4},{},{},{}\n",
                csv_escape(&tp.cabinet_name),
                csv_escape(&tp.part.label),
                tp.part.rect.width,
                tp.part.rect.height,
                tp.part.thickness,
                tp.part.quantity,
                tp.part.operations.len(),
                csv_escape(material_name),
            ));
        } else {
            csv.push_str(&format!(
                "{},{:.4},{:.4},{:.4},{},{},{}\n",
                csv_escape(&tp.part.label),
                tp.part.rect.width,
                tp.part.rect.height,
                tp.part.thickness,
                tp.part.quantity,
                tp.part.operations.len(),
                csv_escape(material_name),
            ));
        }
    }

    fs::write(&csv_path, &csv)?;
    println!("CSV cut list written to: {}", csv_path.display());
    Ok(())
}

/// Escape a value for CSV (wrap in quotes if it contains comma or quote).
fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// Export SVG nesting layout for visual verification.
fn write_svg(
    output_dir: &PathBuf,
    nesting_result: &cm_nesting::packer::NestingResult,
    config: &NestingConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    for sheet in &nesting_result.sheets {
        let svg_path = output_dir.join(format!("sheet-{}.svg", sheet.sheet_index + 1));
        let scale = 10.0; // 10 pixels per unit (inch or mm)
        let svg_w = config.sheet_width * scale;
        let svg_h = config.sheet_length * scale;

        let mut svg = String::new();
        svg.push_str(&format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="{svg_w}" height="{svg_h}" viewBox="0 0 {sw} {sl}">
  <style>
    rect.sheet {{ fill: #f5f0e0; stroke: #333; stroke-width: 0.5; }}
    rect.part {{ fill: #d4a574; stroke: #333; stroke-width: 0.25; opacity: 0.85; }}
    text {{ font-family: monospace; font-size: 1.2; fill: #222; }}
  </style>
  <rect class="sheet" x="0" y="0" width="{sw}" height="{sl}" />
"#,
            sw = config.sheet_width,
            sl = config.sheet_length,
        ));

        for placed in &sheet.parts {
            let x = placed.rect.origin.x;
            let y = placed.rect.origin.y;
            let w = placed.rect.width;
            let h = placed.rect.height;

            svg.push_str(&format!(
                r#"  <rect class="part" x="{x:.4}" y="{y:.4}" width="{w:.4}" height="{h:.4}" />
  <text x="{tx:.4}" y="{ty:.4}" text-anchor="middle" dominant-baseline="central">{label}</text>
"#,
                tx = x + w / 2.0,
                ty = y + h / 2.0,
                label = placed.id,
            ));
        }

        svg.push_str("</svg>\n");
        fs::write(&svg_path, &svg)?;
        println!("SVG nesting layout written to: {}", svg_path.display());
    }
    Ok(())
}

