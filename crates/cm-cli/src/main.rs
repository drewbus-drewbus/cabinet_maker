use std::fs;
use std::path::PathBuf;

use clap::Parser;

use cm_cabinet::project::Project;
use cm_cabinet::part::PartOperation;
use cm_cam::ops::{generate_profile_cut, generate_dado_toolpath, generate_rabbet_toolpath, generate_drill, CamConfig, RabbetEdge};
use cm_cam::Toolpath;
use cm_core::geometry::{Point2D, Rect};
use cm_core::tool::Tool;
use cm_nesting::packer::{NestingConfig, NestingPart, nest_parts};
use cm_post::gcode::GCodeEmitter;
use cm_post::machine::MachineProfile;

#[derive(Parser)]
#[command(name = "cabinet-maker", version, about = "Generate CNC G-code from cabinet designs")]
struct Cli {
    /// Path to the project TOML file
    project_file: PathBuf,

    /// Path to the machine profile TOML file (optional, defaults to built-in PCNC 1100)
    #[arg(short, long)]
    machine: Option<PathBuf>,

    /// Output directory for generated files
    #[arg(short, long, default_value = "output")]
    output_dir: PathBuf,

    /// Spindle RPM (overrides machine profile default)
    #[arg(long)]
    rpm: Option<f64>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Load project
    let project_toml = fs::read_to_string(&cli.project_file)?;
    let project = Project::from_toml(&project_toml)?;

    println!("Project: {}", project.project.name);
    println!("Units: {:?}", project.project.units);
    println!("Cabinet: {} ({}\" x {}\" x {}\")",
        project.cabinet.name,
        project.cabinet.width,
        project.cabinet.height,
        project.cabinet.depth,
    );

    // Load or default machine profile
    let machine = if let Some(ref machine_path) = cli.machine {
        let machine_toml = fs::read_to_string(machine_path)?;
        MachineProfile::from_toml(&machine_toml)?
    } else {
        MachineProfile::tormach_pcnc1100()
    };

    println!("Machine: {}", machine.machine.name);

    // Generate parts
    let parts = project.cabinet.generate_parts();
    println!("\nGenerated {} part entries:", parts.len());
    for part in &parts {
        println!(
            "  {} - {:.3}\" x {:.3}\" (qty: {}, ops: {})",
            part.label,
            part.rect.width,
            part.rect.height,
            part.quantity,
            part.operations.len(),
        );
    }

    // --- Nesting: arrange parts on sheets ---
    let nesting_config = NestingConfig {
        sheet_width: project.material.sheet_width.unwrap_or(48.0),
        sheet_length: project.material.sheet_length.unwrap_or(96.0),
        kerf: 0.25, // default kerf for 1/4" bit
        edge_margin: 0.5,
        allow_rotation: false, // respect grain direction
    };

    // Expand parts by quantity for nesting
    let mut nesting_parts = Vec::new();
    for part in &parts {
        for i in 0..part.quantity {
            let id = if part.quantity > 1 {
                format!("{}_{}", part.label, i + 1)
            } else {
                part.label.clone()
            };
            nesting_parts.push(NestingPart {
                id,
                width: part.rect.width,
                height: part.rect.height,
                can_rotate: false,
            });
        }
    }

    let nesting_result = nest_parts(&nesting_parts, &nesting_config);

    println!("\nNesting result:");
    println!("  Sheets required: {}", nesting_result.sheet_count);
    println!("  Overall utilization: {:.1}%", nesting_result.overall_utilization);
    if !nesting_result.unplaced.is_empty() {
        println!("  WARNING: {} parts could not be placed:", nesting_result.unplaced.len());
        for id in &nesting_result.unplaced {
            println!("    - {}", id);
        }
    }

    for sheet in &nesting_result.sheets {
        println!("  Sheet {}: {} parts, {:.1}% utilization",
            sheet.sheet_index + 1, sheet.parts.len(), sheet.utilization);
        for placed in &sheet.parts {
            println!("    {} at ({:.3}\", {:.3}\") {}",
                placed.id,
                placed.rect.origin.x,
                placed.rect.origin.y,
                if placed.rotated { "(rotated)" } else { "" },
            );
        }
    }

    // Determine tool and RPM
    let tool = project
        .tools
        .first()
        .cloned()
        .unwrap_or_else(Tool::quarter_inch_endmill);
    let rpm = cli.rpm.unwrap_or(machine.machine.max_rpm * 0.9);

    println!("\nTool: {} (T{}, {} dia)", tool.description, tool.number, tool.diameter);
    println!("RPM: {}", rpm as u32);

    let cam_config = CamConfig {
        safe_z: machine.post.safe_z,
        rapid_z: machine.post.rapid_z,
        ..Default::default()
    };

    // Create output directory
    fs::create_dir_all(&cli.output_dir)?;

    // Generate G-code per sheet
    let emitter = GCodeEmitter::new(&machine, project.project.units);

    for sheet in &nesting_result.sheets {
        let mut sheet_toolpaths: Vec<Toolpath> = Vec::new();

        println!("\nGenerating toolpaths for sheet {}:", sheet.sheet_index + 1);

        for placed in &sheet.parts {
            // Find the original part definition for this placed part.
            // Nesting adds _N suffixes for quantity > 1 (e.g., "shelf_1", "shelf_2").
            // Strip the trailing _N to find the base label.
            let base_label = if let Some(last_underscore) = placed.id.rfind('_') {
                let suffix = &placed.id[last_underscore + 1..];
                if suffix.parse::<u32>().is_ok() {
                    placed.id[..last_underscore].to_string()
                } else {
                    placed.id.clone()
                }
            } else {
                placed.id.clone()
            };

            let part = parts.iter()
                .find(|p| p.label == base_label || p.label == placed.id)
                .expect(&format!("Part definition not found for '{}'", placed.id));

            // Create a positioned rect using the nesting placement
            let positioned_rect = Rect::new(
                placed.rect.origin,
                placed.rect.width,
                placed.rect.height,
            );

            println!("  {} at ({:.3}\", {:.3}\")", placed.id, placed.rect.origin.x, placed.rect.origin.y);

            // Generate operation toolpaths (positioned on sheet)
            for op in &part.operations {
                match op {
                    PartOperation::Dado(dado) => {
                        let horizontal = dado.orientation == cm_cabinet::part::DadoOrientation::Horizontal;
                        let tp = generate_dado_toolpath(
                            &positioned_rect,
                            dado.position,
                            dado.width,
                            dado.depth,
                            horizontal,
                            &tool,
                            rpm,
                            &cam_config,
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
                            &positioned_rect,
                            edge,
                            rabbet.width,
                            rabbet.depth,
                            &tool,
                            rpm,
                            &cam_config,
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
                            drill.depth,
                            &tool,
                            rpm,
                            &cam_config,
                        );
                        println!("    - Drill at ({:.3}\", {:.3}\"), depth {:.3}\"",
                            drill.x, drill.y, drill.depth);
                        sheet_toolpaths.push(tp);
                    }
                }
            }

            // Profile cut at nested position
            let tp = generate_profile_cut(&positioned_rect, part.thickness, &tool, rpm, &cam_config);
            println!("    - Profile cut: {:.3}\" x {:.3}\" through {:.3}\"",
                positioned_rect.width, positioned_rect.height, part.thickness);
            sheet_toolpaths.push(tp);
        }

        // Emit G-code for this sheet
        let gcode = emitter.emit(&sheet_toolpaths);
        let filename = if nesting_result.sheet_count > 1 {
            format!("sheet-{}.nc", sheet.sheet_index + 1)
        } else {
            "program.nc".into()
        };
        let output_path = cli.output_dir.join(&filename);
        fs::write(&output_path, &gcode)?;

        println!("  G-code written to: {} ({} lines, {} toolpaths)",
            output_path.display(),
            gcode.lines().count(),
            sheet_toolpaths.len(),
        );
    }

    // Write cut list
    let cutlist_path = cli.output_dir.join("cutlist.txt");
    let mut cutlist = String::new();
    cutlist.push_str(&format!("Cut List: {}\n", project.project.name));
    cutlist.push_str(&format!("Material: {} ({:.3}\" thick)\n", project.material.name, project.material.thickness));
    cutlist.push_str(&format!("Sheets required: {}\n\n", nesting_result.sheet_count));
    cutlist.push_str(&format!("{:<20} {:>10} {:>10} {:>5} {:>5}\n", "Part", "Width", "Height", "Qty", "Ops"));
    cutlist.push_str(&format!("{:-<55}\n", ""));
    for part in &parts {
        cutlist.push_str(&format!(
            "{:<20} {:>9.3}\" {:>9.3}\" {:>5} {:>5}\n",
            part.label, part.rect.width, part.rect.height, part.quantity, part.operations.len(),
        ));
    }

    // Add nesting layout summary
    cutlist.push_str(&format!("\nNesting Layout:\n"));
    for sheet in &nesting_result.sheets {
        cutlist.push_str(&format!("  Sheet {} ({:.1}% utilization):\n", sheet.sheet_index + 1, sheet.utilization));
        for placed in &sheet.parts {
            cutlist.push_str(&format!(
                "    {:<20} at ({:>7.3}\", {:>7.3}\")\n",
                placed.id, placed.rect.origin.x, placed.rect.origin.y,
            ));
        }
    }

    fs::write(&cutlist_path, &cutlist)?;
    println!("\nCut list written to: {}", cutlist_path.display());
    println!("Overall utilization: {:.1}%", nesting_result.overall_utilization);

    Ok(())
}
