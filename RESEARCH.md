# Cabinet Maker Software — Opportunity & Architecture Research

## The Core Opportunity

The woodworking/cabinet making software market has a massive gap: **there is no affordable, integrated, open-source tool that takes a user from design intent to machine-ready G-code**. The workflow today looks like this for most small-to-mid shops:

```
Idea → SketchUp/FreeCAD (design) → Export DXF/SVG
    → Vectric/Fusion360 (CAM) → Configure toolpaths manually
    → Nesting software (optimize sheets) → another manual step
    → Post-processor (machine-specific G-code) → pray it works
    → CNC controller (Mach4/GRBL/LinuxCNC) → cut
```

That's 4-6 separate tools, $2K-$40K in software, and hours of manual translation between steps. Every handoff is a place where errors creep in.

### Who We'd Serve

The **"missing middle"**: small-to-mid cabinet shops (1-10 people), serious hobbyists, independent furniture makers, and makerspaces. These users:

- Can't afford $10K-$40K for Cabinet Vision / Microvellum
- Need more than SketchUp + a cutlist plugin
- Own or have access to a CNC router (OneFinity, Avid, ShopBot, Shapeoko, or similar)
- Want to go from "I need a kitchen with these cabinets" to "parts on the machine" fast
- Currently waste 30-50% of their time on file format translation and manual CAM setup

### What Victory Looks Like

A user describes or draws what they want → the software understands it as a **parametric assembly of wood parts with joinery** → automatically generates **cut lists, nesting layouts, toolpaths, and machine-specific G-code** → the user loads files onto their CNC and cuts.

---

## Product Vision: What It Should Do

### Level 1: Parametric Cabinet/Furniture Design
- Design cabinets, shelves, tables, desks, and custom furniture
- Parametric: change one dimension and everything updates (cut lists, joinery, nesting, G-code)
- Built-in knowledge of cabinet construction: face-frame, Euro/frameless (32mm system), both
- Standard cabinet types as templates: base, wall, tall, drawer banks, lazy susans, etc.
- Room layout: place cabinets in a kitchen/office floorplan, handle fillers, scribe panels, end panels
- Material library: plywood, MDF, hardwoods, melamine, with real thicknesses and properties
- Hardware library: hinges (cup hinges, butt hinges), slides (undermount, side-mount), shelf pins, etc.

### Level 2: Intelligent Joinery
- Automatic joinery suggestions based on joint type and material
- Supported joinery: dados, rabbets, butt joints, pocket holes, dowels, dominos, mortise-and-tenon, dovetails, biscuits, box joints
- Each joinery type maps to specific CNC operations (tool, depth, path)
- User can override or customize joinery per joint
- Joinery rules engine: "use dados for shelf-to-side joints in plywood, use dominos for face-frame to carcass"

### Level 3: Cut Lists & Nesting
- Automatic bill of materials with grain direction
- Sheet goods optimization (nesting) with support for grain-direction constraints
- Solid wood optimization (linear cutting)
- Waste tracking and reporting
- Edge banding specification

### Level 4: CAM / Toolpath Generation
- Generate toolpaths directly from the 3D model — no export/import step
- Operations: profile cuts, pocket cuts, dados, rabbets, drilling (shelf pin holes, hinge cups, dowel holes), V-carving for decorative elements
- Tool library: router bits, drill bits, V-bits with real dimensions
- Automatic tab generation for hold-downs
- Feed/speed recommendations based on material + tool
- Toolpath simulation / preview (material removal visualization)

### Level 5: Post-Processing & Machine Support
- Abstraction layer that knows the dialect of common CNC controllers
- Target machines: GRBL, grblHAL, FluidNC, Mach3/Mach4, LinuxCNC, ShopBot, OneFinity, Avid
- Machine profile: work area, number of axes, tool change capability, homing behavior
- Post-processor generates correct G-code header, tool changes, coolant, safe retracts, etc.
- Community-contributed machine profiles

### Level 6: AI-Assisted Design (Future)
- "I want a shaker-style kitchen, 10ft x 12ft L-shape, 30" uppers, 36" base" → generates a parametric layout
- Style libraries: shaker, modern/slab, raised panel, beadboard, etc.
- "Suggest joinery for this assembly given my machine capabilities"
- Material optimization recommendations

---

## Technical Architecture

### Design Principles
1. **Local-first**: All computation runs locally. No cloud dependency for core function.
2. **Open-source core**: The geometry engine, joinery engine, CAM engine, and post-processors are all open-source.
3. **Modular**: Each subsystem (CAD kernel, joinery, nesting, CAM, post-processing) is a separate library that can be used independently.
4. **Parametric from the ground up**: Every dimension is a parameter. Change one, everything downstream updates.
5. **File format agnostic**: Import/export STEP, DXF, SVG, STL. Native format is human-readable.
6. **Cross-platform**: Runs on Linux, macOS, Windows. Browser version possible via WASM.

### Technology Choices

#### Core Language: Rust

Why Rust:
- Performance-critical geometry and toolpath computation at native speed
- Compiles to WebAssembly for browser deployment (proven by CADmium, Truck, Fornjot)
- Memory safety without GC — important for complex geometric operations
- Strong type system catches design errors at compile time
- Excellent ecosystem for the building blocks we need
- Can expose Python bindings (via PyO3) for scripting/automation
- Cross-compiles to all target platforms

#### Geometry Kernel: Phased Approach

**Key Insight**: Cabinet making at the CNC level is fundamentally a **2D problem**. Every
part is a rectangular panel cut from a sheet. Every operation (dado, rabbet, hole, profile
cut) is a 2D toolpath at a specified depth. The 3D model is just extruded rectangles for
visualization.

**Phase 1 (no kernel)**: Custom parametric 2D geometry — panels are parameterized
rectangles, joints are parameterized modifications (grooves, holes), toolpaths are generated
directly from parametric descriptions. 3D visualization uses simple extruded boxes in
Three.js (no B-rep kernel needed). This gets us to working G-code fastest.

**Phase 2+ (Truck)**: Introduce the [Truck](https://github.com/ricosjp/truck) B-rep kernel
when we need real 3D boolean operations — complex joinery visualization, STEP export,
non-rectangular parts, edge profiles.

**Why Truck over OpenCascade (OCCT) when we do need a kernel**:

| Factor | Truck | OpenCascade |
|--------|-------|-------------|
| Language | Pure Rust | C++ (complex FFI) |
| WASM | Compiles natively | No viable path |
| License | MIT/Apache | LGPL (restrictions) |
| Architecture | Modular crates | Monolithic |
| Maturity | Medium (growing) | Very mature |
| Browser deploy | Yes (proven by CADmium) | No |

Our domain is narrow enough (boxes, holes, grooves) that we can work around any Truck
limitations. CADmium has already proven the Truck + Rust + WASM + Three.js pipeline works.

#### Nesting Engine

**libnest2d** (C++) is proven (used in PrusaSlicer), but for a Rust-native stack, we could:
- Port the core algorithm (NFP-based placement + genetic optimization) to Rust
- Or use libnest2d via FFI initially
- **SVGnest**'s algorithm is well-documented and could be reimplemented
- For rectangular parts (most cabinet panels), simpler bin-packing algorithms work well and are fast

**Recommendation**: Implement a **two-tier nesting system**:
1. Fast rectangular nesting for sheet goods (most cabinet work is rectangular panels)
2. Irregular shape nesting (for curved parts, templates) using NFP algorithm

#### CAM / Toolpath Engine

No good Rust-native CAM library exists. Options:
- **Build our own**, purpose-built for woodworking operations (not general 5-axis machining)
- Wrap **PyCAM** or **FreeCAD Path** via FFI (heavy, complex)
- Study **Generic CAM** and **krabmern/blendercam** for algorithm reference

**Recommendation**: **Build a woodworking-specific CAM engine in Rust.** The operations we need are dramatically simpler than general machining:
- 2D profile cuts (with tabs)
- Pocket operations (rectangular, mostly)
- Drilling operations (shelf pins, hinge cups, dowels)
- Dado/rabbet cuts (linear groove operations)
- V-carve for decorative elements (can add later)

This is a tractable problem. We don't need 5-axis simultaneous, adaptive clearing, or complex surface finishing. A woodworking CAM engine is maybe 20% of the complexity of a general-purpose one.

#### Post-Processor System

**Design**: A declarative, data-driven post-processor system:
```
Machine Profile (TOML/JSON) → Post-Processor Engine → G-code
```

Each machine profile defines:
- G-code dialect (line format, decimal places, line numbers)
- Supported G/M codes
- Tool change sequence
- Homing/parking behavior
- Axis naming and limits
- Safe retract height behavior
- Spindle control commands
- Any machine-specific quirks

This is essentially a template engine with machine-specific knowledge. Community-contributed profiles would cover the long tail of machines.

#### UI Layer

| Option | Pros | Cons |
|--------|------|------|
| **Web (SvelteKit + Three.js + WASM)** | Cross-platform, no install, shareability | Performance ceiling, offline complexity |
| **Tauri (Rust + Web UI)** | Native performance, local-first, web tech for UI | Less mature than Electron |
| **egui / iced (Rust-native GUI)** | Maximum performance, no web tech | Smaller ecosystem, harder to build rich UI |
| **Qt via Rust bindings** | Mature, powerful | Complex bindings, licensing concerns |

**Recommendation**: **Tauri + SvelteKit + Three.js** (same stack as CADmium). Reasons:
- Proven architecture for browser-based CAD
- Rust backend handles all computation (geometry, CAM, nesting, post-processing)
- Web frontend handles UI, 3D visualization, parameter editing
- Can also deploy as a pure web app via WASM if desired
- Tauri bundles are tiny compared to Electron
- Three.js / Threlte is excellent for 3D visualization

#### Data Model / File Format

Native project format: **A directory of human-readable files** (not a binary blob):

```
my-kitchen-project/
├── project.toml              # Project metadata, units, defaults
├── materials/
│   └── materials.toml        # Material definitions (thickness, density, cost)
├── hardware/
│   └── hardware.toml         # Hardware catalog (hinges, slides, pins)
├── room/
│   └── layout.toml           # Room dimensions, obstacles, appliance locations
├── cabinets/
│   ├── base-sink-36.toml     # Each cabinet is a parametric definition
│   ├── wall-left-30.toml
│   └── ...
├── assemblies/
│   └── kitchen.toml          # How cabinets are arranged in the room
├── joinery/
│   └── rules.toml            # Joinery rules and overrides
├── machining/
│   ├── tools.toml            # Tool library
│   ├── machine.toml          # Machine profile
│   └── settings.toml         # Feed/speed defaults per material
└── output/                   # Generated artifacts
    ├── cutlist.csv
    ├── bom.csv
    ├── nesting/
    │   ├── sheet-1.svg
    │   └── sheet-2.svg
    └── gcode/
        ├── sheet-1.nc
        └── sheet-2.nc
```

Benefits:
- Git-friendly (diff, merge, history)
- Human-readable and editable
- Each file is independently understandable
- Easy to build templates and share cabinet designs

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────┐
│                     UI Layer (Svelte + Three.js)         │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌────────────┐ │
│  │ Room     │ │ Cabinet  │ │ Nesting  │ │ Toolpath   │ │
│  │ Layout   │ │ Designer │ │ Viewer   │ │ Preview    │ │
│  │ Editor   │ │          │ │          │ │ & Simulate │ │
│  └──────────┘ └──────────┘ └──────────┘ └────────────┘ │
└──────────────────────┬──────────────────────────────────┘
                       │  Tauri IPC / WASM bridge
┌──────────────────────▼──────────────────────────────────┐
│                   Core Engine (Rust)                      │
│                                                          │
│  ┌─────────────┐  ┌──────────────┐  ┌────────────────┐  │
│  │ Parametric   │  │ Joinery      │  │ Material       │  │
│  │ Model Engine │  │ Engine       │  │ Library        │  │
│  │              │  │              │  │                │  │
│  │ - Cabinets   │  │ - Rules      │  │ - Sheet goods  │  │
│  │ - Parts      │  │ - Dado       │  │ - Solid wood   │  │
│  │ - Assemblies │  │ - Rabbet     │  │ - Hardware     │  │
│  │ - Constraints│  │ - Dowel      │  │ - Costs        │  │
│  │ - Parameters │  │ - Pocket hole│  │                │  │
│  └──────┬───────┘  └──────┬───────┘  └───────┬────────┘  │
│         │                 │                   │          │
│  ┌──────▼─────────────────▼───────────────────▼────────┐ │
│  │         Geometry Layer (cm-geometry)                  │ │
│  │  Phase 1: 2D parametric (rects, lines, arcs)         │ │
│  │  Phase 2+: Truck B-rep (booleans, STEP export)       │ │
│  │  - Panel primitives (board, sheet, edge profile)     │ │
│  │  - Hole patterns (32mm system, shelf pins)           │ │
│  │  - Mesh generation for visualization                 │ │
│  └──────┬───────────────────────────────────────────────┘ │
│         │                                                │
│  ┌──────▼───────┐  ┌──────────────┐  ┌────────────────┐  │
│  │ Nesting      │  │ CAM Engine   │  │ Post-Processor │  │
│  │ Engine       │  │              │  │ System         │  │
│  │              │  │ - Profile    │  │                │  │
│  │ - Rectangular│  │ - Pocket     │  │ - Machine      │  │
│  │ - Irregular  │  │ - Drill      │  │   profiles     │  │
│  │ - Grain-aware│  │ - Dado       │  │ - G-code gen   │  │
│  │ - Multi-sheet│  │ - V-carve    │  │ - Dialect      │  │
│  │              │  │ - Tabs       │  │   translation  │  │
│  │              │  │ - Simulation │  │                │  │
│  └──────────────┘  └──────┬───────┘  └───────┬────────┘  │
│                           │                   │          │
│                    ┌──────▼───────────────────▼────────┐  │
│                    │         G-code Output              │  │
│                    │  - Per-sheet NC files              │  │
│                    │  - Tool change sequences           │  │
│                    │  - Estimated run times             │  │
│                    └───────────────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

### Crate Structure (Rust workspace)

```
cabinet-maker/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── cm-core/                  # Core types: dimensions, units, parameters
│   ├── cm-geometry/              # Geometry kernel wrapper (Truck-based)
│   ├── cm-materials/             # Material and hardware library
│   ├── cm-cabinet/               # Cabinet domain model (parametric cabinets, parts)
│   ├── cm-joinery/               # Joinery rules engine
│   ├── cm-nesting/               # Sheet nesting algorithms
│   ├── cm-cam/                   # Toolpath generation
│   ├── cm-post/                  # Post-processor engine + machine profiles
│   ├── cm-project/               # Project file I/O (TOML serialization)
│   ├── cm-export/                # DXF, SVG, STEP, STL export
│   ├── cm-wasm/                  # WASM bindings for browser
│   └── cm-cli/                   # CLI interface for batch processing
├── app/                          # Tauri + SvelteKit desktop app
│   ├── src-tauri/                # Rust backend (Tauri commands)
│   └── src/                      # Svelte frontend
├── profiles/                     # Machine profiles
│   ├── tormach-24r.toml          # First target (PathPilot/LinuxCNC)
│   ├── tormach-1100m.toml        # Tormach mill variant
│   ├── linuxcnc-default.toml     # Generic LinuxCNC
│   ├── grbl-default.toml         # Future: GRBL machines
│   ├── onefinity-elite.toml      # Future: OneFinity
│   ├── avid-cnc.toml             # Future: Avid
│   ├── shopbot-buddy.toml        # Future: ShopBot
│   └── mach4-default.toml        # Future: Mach4
└── templates/                    # Cabinet templates
    ├── base-cabinet.toml
    ├── wall-cabinet.toml
    ├── drawer-bank.toml
    └── ...
```

### Data Flow

```
User Input (parametric dimensions, cabinet type, material)
    │
    ▼
Parametric Model Engine (cm-cabinet)
    │  → Resolves all parameters
    │  → Generates part list with dimensions
    │
    ▼
Joinery Engine (cm-joinery)
    │  → Applies joinery rules to each joint
    │  → Modifies part geometry (adds dados, holes, etc.)
    │
    ▼
Geometry Kernel (cm-geometry)
    │  → Creates precise 3D B-rep model
    │  → Generates visualization mesh
    │
    ├──────────────────────┐
    ▼                      ▼
Nesting Engine          Cut List Generator
(cm-nesting)            (cm-project)
    │                      │
    │  → Optimizes parts   │  → BOM with costs
    │    onto sheets       │  → Edge banding list
    │  → Respects grain    │  → Hardware list
    │                      │
    ▼
CAM Engine (cm-cam)
    │  → Generates toolpaths for each sheet
    │  → Profile cuts, pockets, drilling, dados
    │  → Adds tabs for hold-downs
    │  → Optimizes tool change order
    │
    ▼
Post-Processor (cm-post)
    │  → Translates to machine-specific G-code
    │  → Adds machine setup/teardown
    │
    ▼
Output: G-code files ready for CNC
```

---

## Build Strategy: What to Build First

### Phase 1: Foundation — "TOML to Tormach"
**Goal: Define a simple cabinet in TOML → generate G-code → cut real parts on the Tormach.**

This is the minimum viable proof that the pipeline works end-to-end.

**Crates to build:**
- `cm-core`: Unit system (inches/mm, fractions), parameter types, dimension types
- `cm-geometry`: 2D parametric geometry — rectangles, lines, arcs, circles (NO B-rep kernel yet)
- `cm-cabinet`: One cabinet type: a basic box (2 sides + top/bottom + back + 1 shelf)
- `cm-cam`: Profile cut + dado toolpath generation only
- `cm-post`: PathPilot/LinuxCNC post-processor (RS274/NGC dialect)
- `cm-cli`: CLI tool that reads a TOML cabinet definition → outputs .nc G-code files

**The "Hello World" test:**
1. Write a TOML file defining a simple bookshelf (e.g., 36"W x 30"H x 12"D, 3/4" plywood, 2 shelves)
2. `cm-cli generate bookshelf.toml --machine tormach-24r`
3. Output: cut list + G-code files for each sheet of plywood
4. Load G-code into PathPilot on the Tormach
5. Cut the parts, assemble the bookshelf
6. If it fits together, Phase 1 is done

**What we explicitly skip in Phase 1:**
- No GUI (CLI only)
- No 3D visualization
- No nesting optimization (manual layout or simple grid)
- No joinery rules engine (hardcoded dado joinery)
- No hardware library
- Only one material: 3/4" plywood
- Only one machine: Tormach 24R / PathPilot

### Phase 2: Real Cabinets (Months 3-6)
**Goal: Design and cut a real kitchen base cabinet.**

- `cm-joinery`: Dado, rabbet, shelf pin holes (32mm system)
- `cm-materials`: Plywood, MDF material definitions with real thicknesses
- `cm-nesting`: Rectangular nesting for sheet goods
- `cm-cabinet`: Base cabinet, wall cabinet, drawer bank templates
- `cm-post`: Add Mach4, LinuxCNC, OneFinity profiles
- `cm-export`: DXF and SVG export for manual verification
- Basic Tauri app with 3D preview

### Phase 3: Usable Product (Months 6-12)
**Goal: A small shop can design and cut a full kitchen.**

- Room layout editor (place cabinets in a floor plan)
- Full cabinet library (lazy susan, corner cabinets, tall pantry, etc.)
- Hardware library (hinges, slides, shelf pins) with automatic placement
- Full joinery engine (pocket holes, dowels, dominos)
- Toolpath simulation / material removal preview
- Nesting with grain direction awareness
- Cut list / BOM / cost estimation
- Multiple machine profile support

### Phase 4: Community & Polish (Months 12+)
**Goal: Grow adoption and ecosystem.**

- Community machine profile repository
- Community cabinet template sharing
- AI-assisted design ("generate a shaker kitchen for this room")
- Solid wood operations (not just sheet goods)
- Edge banding specification
- Door and drawer front design (cope and stick, raised panel profiles)
- V-carve for decorative elements
- Plugin/extension system

---

## Key Technical Risks

1. **Geometry kernel maturity**: Truck is still evolving (deferred to Phase 2). May hit limitations requiring workarounds.
2. **CAM correctness**: Generating wrong G-code can damage machines or waste material. Extensive testing against known-good outputs is critical.
3. **Post-processor coverage**: Every machine has quirks. Need community contributions to cover the long tail.
4. **Nesting performance**: Large jobs with many parts and grain constraints can be computationally expensive.
5. **Scope creep**: The temptation to become a general CAD/CAM tool. Must stay focused on woodworking/cabinet making.

## Key Differentiators vs. Existing Tools

| Feature | Cabinet Vision | Mozaik | Vectric | FreeCAD | **This Tool** |
|---------|---------------|--------|---------|---------|---------------|
| Price | $10K-$25K | ~$100/mo | $350-$2K | Free | **Free / Open Source** |
| Integrated CAM | Yes (extra $) | Partial | CAM only | Partial | **Yes, built-in** |
| Parametric | Yes | Partial | No | Yes | **Yes** |
| Automatic joinery | Yes | Basic | No | No | **Yes** |
| Nesting | Add-on | Add-on | No | No | **Built-in** |
| Post-processors | Limited set | Limited | Good | OK | **Community-driven** |
| Open source | No | No | No | Yes | **Yes** |
| Learning curve | Weeks-months | Days-weeks | ~1 week | Weeks | **Target: hours-days** |
| Woodworking-specific | Yes | Yes | No | Plugin | **Yes, core focus** |

---

## Decisions Made

1. **Cabinets first.** More structure and rules = easier to automate, larger market demand.
2. **Target CNC: Tormach (PathPilot).** First post-processor targets PathPilot (LinuxCNC-based, RS274/NGC dialect). Expand to other machines later.
3. **Geometry kernel: Phased.** No kernel in Phase 1 (2D parametric), Truck in Phase 2+.
4. **Business model**: Deferred. Focus on building first.

## Open Questions

1. **Community strategy**: How to attract early users and contributors?
2. **Name**: "Cabinet Maker" is descriptive but generic. May want something more distinctive.
3. **Production router access**: What router(s) will be available for full sheet cutting? (Affects nesting defaults.)

---

## Target Machines

### Machine-Agnostic Design
The software is machine-agnostic from day one. Every machine is described by a TOML
profile containing work envelope, spindle characteristics, supported G/M codes, tool
change behavior, and machine-specific quirks. The CAM and post-processor engines work
against this abstract machine description.

### Development Machine: Tormach PCNC 1100 (Mill)
The primary development and testing machine. NOT suitable for cutting full cabinet
panels (too small), but perfect for validating G-code correctness on test pieces.

| Spec | Value |
|------|-------|
| Type | Vertical milling machine |
| Travel (X/Y/Z) | 18" x 9.5" x 16.25" |
| Table | 34" x 9.5", 3 T-slots |
| Spindle | R8 taper (BT30 retrofit available) |
| RPM | 100-5,140 (low for wood — fine for testing) |
| Power | 1.5 HP brushless AC |
| Controller | PathPilot (LinuxCNC-based) |

**Development workflow**: Design cabinet → generate G-code → cut **individual small
test parts** on the PCNC 1100 to validate toolpaths and fit → once validated, cut
full sheets on a production router.

### PathPilot Controller (Shared Across All Tormach Machines)
- **Based on LinuxCNC** — uses RS274/NGC G-code dialect (Fanuc-style)
- Free, well-documented controller software
- Supports simultaneous motion up to 4 axes
- Has conversational programming built in
- Standard G-code: G00/G01/G02/G03, G17-G19, G20/G21, G40-G43, G54-G59, G73/G80-G89, G90/G91
- Full canned drilling cycles (G73, G81-G89) — perfect for shelf pin hole patterns
- Cutter compensation (G41/G42) supported

### PathPilot G/M-Codes We'll Use
| Code | Function | Use Case |
|------|----------|----------|
| G00 | Rapid move | Repositioning between cuts |
| G01 | Linear feed | Profile cuts, dados, rabbets |
| G02/G03 | Arc CW/CCW | Rounded corners, arc entries |
| G17 | XY plane select | Standard for router/mill work |
| G20/G21 | Inch/mm mode | Unit selection |
| G28/G30 | Return to home/ref | Safe retract |
| G40/G41/G42 | Cutter comp off/left/right | Profile cut compensation |
| G43 | Tool length offset | Multi-tool operations |
| G54-G59 | Work offsets | Multiple fixture setups |
| G73 | Peck drilling | Deep shelf pin holes |
| G81 | Simple drill cycle | Shelf pin holes, dowel holes |
| G90/G91 | Absolute/incremental | Mode selection |
| M03/M05 | Spindle on CW / stop | Essential |
| M06 | Tool change | Multi-tool jobs |
| M08/M09 | Coolant on/off | Air blast for chip clearing |
| M27/M28 | Dust shoe up/down | 24R router specific |
| M30 | Program end & rewind | Standard |
| M98/M99 | Subroutine call/return | Repeated hole patterns |

### Post-Processor Strategy
Since PathPilot is LinuxCNC-based, our work benefits ALL LinuxCNC users (not just
Tormach). The post-processor is layered:

```
LinuxCNC base dialect (RS274/NGC)
  └── Tormach common extensions (ETS, tool change sequences)
       ├── PCNC 1100 profile (mill, R8, small envelope)
       ├── 1100M/1100MX profile (mill, BT30, ATC)
       └── 24R profile (router, ER20, vacuum table, dust shoe)
```

### Production Machine Requirements (Future)
For cutting full cabinet panels from sheet goods, the production machine needs:
- **Bed size**: 4'x4' minimum (4'x8' ideal, but rare in small shops)
- **Spindle**: 2+ HP, 18,000+ RPM (router spindle, not mill spindle)
- **Hold-down**: Vacuum table or T-track + clamps
- Most small shops use 2'x4' or 4'x4' beds and pre-cut sheets on a table saw

**Nesting engine must support bed-size-aware sheet splitting** — this is critical UX:
- A 4x8 plywood sheet on a 2x4 bed = cut in 4 quadrants
- A 4x8 sheet on a 4x4 bed = cut in 2 halves
- The software should tell the user exactly how to pre-cut and which parts go where
