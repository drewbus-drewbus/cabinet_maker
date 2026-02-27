use std::fs;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

use cm_cabinet::project::Project;
use cm_nesting::packer::NestingConfig;
use cm_pipeline::{
    GenerateConfig, ProgressEvent, ProgressReporter,
    generate_bom, generate_pipeline,
};
use cm_post::machine::MachineProfile;

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

/// CLI progress reporter that prints events to stdout.
struct CliReporter;

impl ProgressReporter for CliReporter {
    fn report(&self, event: ProgressEvent) {
        match event {
            ProgressEvent::PartsGenerated { count } => {
                println!("\nGenerated {} part entries.", count);
            }
            ProgressEvent::HardwareOpsAdded { count } => {
                println!("Hardware: {} drill operations added", count);
            }
            ProgressEvent::NestingComplete { material, sheets, utilization } => {
                println!("\n--- Material: {} ---", material);
                println!("  Sheets required: {}", sheets);
                println!("  Utilization: {:.1}%", utilization);
            }
            ProgressEvent::GcodeEmitted { material: _, sheet } => {
                println!("  G-code generated for sheet {}", sheet);
            }
            ProgressEvent::Complete => {
                println!("\nGeneration complete.");
            }
        }
    }
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

    // Print cabinet info
    let all_cabs = project.all_cabinets();
    for cab in &all_cabs {
        println!("Cabinet: {} ({:?}, {:.1}\" x {:.1}\" x {:.1}\")",
            cab.name, cab.cabinet_type, cab.width, cab.height, cab.depth,
        );
    }

    // Load machine profile
    let machine = load_machine(cli)?;
    println!("Machine: {}", machine.machine.name);

    // Configure and run the pipeline
    let config = GenerateConfig {
        skip_validation: cli.no_validate,
        enable_hardware: !cli.no_hardware,
        rpm: cli.rpm,
    };

    let reporter = CliReporter;
    let result = generate_pipeline(&project, &machine, &config, &reporter)?;

    // Print part details
    for tp in &result.tagged_parts {
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

    println!("\nTool: {} (T{}, {} dia)", result.tool.description, result.tool.number, result.tool.diameter);
    println!("RPM: {}", result.rpm as u32);

    // Create output directory and write files
    fs::create_dir_all(&cli.output_dir)?;

    let groups = Project::group_parts_by_material(&result.tagged_parts);
    let multi_material = groups.len() > 1;

    for (group, mat_output) in groups.iter().zip(result.material_groups.iter()) {
        let mat_slug = slugify(&group.material_name);

        let group_output_dir = if multi_material {
            let dir = cli.output_dir.join(&mat_slug);
            fs::create_dir_all(&dir)?;
            dir
        } else {
            cli.output_dir.clone()
        };

        // Write G-code files
        for (i, gcode) in mat_output.sheet_gcodes.iter().enumerate() {
            let filename = if mat_output.sheet_gcodes.len() > 1 {
                format!("sheet-{}.nc", i + 1)
            } else {
                "program.nc".into()
            };
            let output_path = group_output_dir.join(&filename);
            fs::write(&output_path, gcode)?;
            println!("  G-code written to: {} ({} lines)",
                output_path.display(),
                gcode.lines().count(),
            );
        }

        // Write cut list
        write_cutlist(&group_output_dir, &project, &group.parts, &mat_output.nesting_result, &group.material_name)?;

        // Optional exports
        if cli.export_csv {
            write_csv(&group_output_dir, &project, &group.parts, &group.material_name)?;
        }
        if cli.export_svg {
            write_svg(&group_output_dir, &mat_output.nesting_result, &mat_output.nesting_config)?;
        }
    }

    // Comprehensive BOM
    if cli.export_bom {
        let primary_mat = project.primary_material();
        let cost_per_sheet = primary_mat.and_then(|m| m.cost_per_unit);
        let comprehensive_bom = generate_bom(&project, &result.tagged_parts, result.total_sheets, cost_per_sheet);
        let bom_json = serde_json::to_string_pretty(&comprehensive_bom)?;
        let bom_path = cli.output_dir.join("bom.json");
        fs::write(&bom_path, &bom_json)?;
        println!("Comprehensive BOM written to: {}", bom_path.display());
    }

    println!("\nDone. {} material group(s) processed.", groups.len());

    Ok(())
}

fn run_import(
    dxf_file: &Path,
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
    dxf_file: &Path,
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
    source_file: &Path,
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
    output_dir: &Path,
    project: &Project,
    parts: &[&cm_cabinet::project::TaggedPart],
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

    cutlist.push_str("\nNesting Layout:\n");
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
    output_dir: &Path,
    project: &Project,
    parts: &[&cm_cabinet::project::TaggedPart],
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
    output_dir: &Path,
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
