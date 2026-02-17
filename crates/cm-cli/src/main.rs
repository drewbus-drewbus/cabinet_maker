use std::fs;
use std::path::PathBuf;

use clap::Parser;

use cm_cabinet::project::Project;
use cm_cabinet::part::PartOperation;
use cm_cam::ops::{generate_profile_cut, generate_dado_toolpath, CamConfig};
use cm_cam::Toolpath;
use cm_core::tool::Tool;
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

    // Generate toolpaths for each unique part
    let mut all_toolpaths: Vec<Toolpath> = Vec::new();

    for part in &parts {
        println!("\nGenerating toolpaths for: {} (qty {})", part.label, part.quantity);

        // First: operations on the part face (dados, rabbets, drills)
        for op in &part.operations {
            match op {
                PartOperation::Dado(dado) => {
                    let horizontal = dado.orientation == cm_cabinet::part::DadoOrientation::Horizontal;
                    let tp = generate_dado_toolpath(
                        &part.rect,
                        dado.position,
                        dado.width,
                        dado.depth,
                        horizontal,
                        &tool,
                        rpm,
                        &cam_config,
                    );
                    println!("  - Dado at {:.3}\", width {:.3}\", depth {:.3}\"",
                        dado.position, dado.width, dado.depth);
                    all_toolpaths.push(tp);
                }
                PartOperation::Rabbet(_rabbet) => {
                    // TODO: implement rabbet toolpath generation
                    println!("  - Rabbet (not yet implemented, skipping)");
                }
                PartOperation::Drill(_drill) => {
                    // TODO: implement drill patterns
                    println!("  - Drill (not yet implemented, skipping)");
                }
            }
        }

        // Then: profile cut to cut the part from the sheet
        let tp = generate_profile_cut(&part.rect, part.thickness, &tool, rpm, &cam_config);
        println!("  - Profile cut: {:.3}\" x {:.3}\" through {:.3}\"",
            part.rect.width, part.rect.height, part.thickness);
        all_toolpaths.push(tp);
    }

    // Emit G-code
    let emitter = GCodeEmitter::new(&machine, project.project.units);
    let gcode = emitter.emit(&all_toolpaths);

    // Write output
    fs::create_dir_all(&cli.output_dir)?;
    let output_path = cli.output_dir.join("program.nc");
    fs::write(&output_path, &gcode)?;

    println!("\nG-code written to: {}", output_path.display());
    println!("Total toolpaths: {}", all_toolpaths.len());
    println!(
        "Total lines: {}",
        gcode.lines().count()
    );

    // Also write a cut list
    let cutlist_path = cli.output_dir.join("cutlist.txt");
    let mut cutlist = String::new();
    cutlist.push_str(&format!("Cut List: {}\n", project.project.name));
    cutlist.push_str(&format!("Material: {} ({:.3}\" thick)\n\n", project.material.name, project.material.thickness));
    cutlist.push_str(&format!("{:<15} {:>10} {:>10} {:>5} {:>5}\n", "Part", "Width", "Height", "Qty", "Ops"));
    cutlist.push_str(&format!("{:-<50}\n", ""));
    for part in &parts {
        cutlist.push_str(&format!(
            "{:<15} {:>9.3}\" {:>9.3}\" {:>5} {:>5}\n",
            part.label, part.rect.width, part.rect.height, part.quantity, part.operations.len(),
        ));
    }
    fs::write(&cutlist_path, &cutlist)?;
    println!("Cut list written to: {}", cutlist_path.display());

    Ok(())
}
