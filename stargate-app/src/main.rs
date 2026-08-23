//! Native Rust frontend: winit + OpenGL/glow + Dear ImGui.

use std::{
    env,
    num::{NonZeroU16, NonZeroU32},
    path::PathBuf,
};

use glow::HasContext;
use glutin::{
    config::{Config, ConfigTemplateBuilder},
    context::{
        ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentContext,
        PossiblyCurrentGlContext,
    },
    display::{GetGlDisplay, GlDisplay},
    surface::{GlSurface, Surface, SurfaceAttributesBuilder, WindowSurface},
};
use grl_sources::SourceDatabase;
use imgui::{Condition, MouseButton, MouseCursor, StyleColor, TreeNodeFlags, WindowFlags};
use imgui_glow_renderer::Renderer;
use imgui_winit_support::winit::{
    dpi::LogicalSize,
    event_loop::EventLoop,
    window::{Window, WindowAttributes},
};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use raw_window_handle::HasWindowHandle;
use rodio::{buffer::SamplesBuffer, DeviceSinkBuilder, MixerDeviceSink, Player};
use stargate_emu::{StargateInput, StargateMachine};
use wms_sound::{HarnessConfig, SoundHarness};

mod syntax;
use syntax::{GazmSyntax, HighlightKind};

struct Audio {
    _device: MixerDeviceSink,
    player: Player,
}

/// The game picture is part of the emulator display, not an ImGui widget.
/// Keeping this tiny textured quad outside ImGui also means it remains visible
/// when the debugger is closed.
struct GameQuad {
    program: glow::NativeProgram,
    vao: glow::NativeVertexArray,
}

struct DebuggerSurface {
    window: Window,
    surface: Surface<WindowSurface>,
    context: PossiblyCurrentContext,
    gl: glow::Context,
    imgui: imgui::Context,
    platform: WinitPlatform,
    textures: imgui::Textures<glow::Texture>,
    renderer: Renderer,
    mono_font: imgui::FontId,
    theme: DebuggerTheme,
}

/// Colors for the debugger UI.  Keep this as a named theme so alternate
/// palettes can be added without spreading ImGui color literals through the
/// rendering code.
#[derive(Clone, Copy)]
struct DebuggerTheme {
    text: [f32; 4],
    muted: [f32; 4],
    accent: [f32; 4],
    current_line: [f32; 4],
    current_line_bg: [f32; 4],
}

fn syntax_color(theme: DebuggerTheme, kind: HighlightKind) -> [f32; 4] {
    match kind {
        HighlightKind::Text => theme.text,
        HighlightKind::Comment => theme.muted,
        HighlightKind::Keyword => [0.796, 0.651, 0.969, 1.0], // Mauve
        HighlightKind::Mnemonic => [0.463, 0.843, 0.725, 1.0], // Green
        HighlightKind::Label => [0.580, 0.773, 0.988, 1.0],   // Sapphire
        HighlightKind::Number => [0.980, 0.702, 0.529, 1.0],  // Peach
        HighlightKind::String => [0.651, 0.890, 0.631, 1.0],  // Green
        HighlightKind::Register => [0.706, 0.776, 0.980, 1.0], // Lavender
        HighlightKind::Error => [0.957, 0.541, 0.518, 1.0],   // Red
    }
}

impl DebuggerTheme {
    fn catppuccin_mocha() -> Self {
        Self {
            text: [0.804, 0.839, 0.957, 1.0],            // Text #CDD6F4
            muted: [0.424, 0.439, 0.533, 1.0],           // Overlay0 #6C7086
            accent: [0.537, 0.706, 0.980, 1.0],          // Blue #89B4FA
            current_line: [0.976, 0.886, 0.686, 1.0],    // Yellow #F9E2AF
            current_line_bg: [0.275, 0.286, 0.416, 1.0], // Surface1 #45475A
        }
    }

    fn apply_to(&self, imgui: &mut imgui::Context) {
        let style = imgui.style_mut();
        style.colors[StyleColor::Text as usize] = self.text;
        style.colors[StyleColor::TextDisabled as usize] = self.muted;
        style.colors[StyleColor::WindowBg as usize] = [0.118, 0.118, 0.180, 1.0]; // Base
        style.colors[StyleColor::ChildBg as usize] = [0.094, 0.094, 0.145, 1.0]; // Mantle
        style.colors[StyleColor::PopupBg as usize] = [0.067, 0.067, 0.106, 1.0]; // Crust
        style.colors[StyleColor::Border as usize] = [0.192, 0.204, 0.286, 1.0];
        style.colors[StyleColor::FrameBg as usize] = [0.192, 0.204, 0.286, 1.0];
        style.colors[StyleColor::FrameBgHovered as usize] = [0.267, 0.278, 0.392, 1.0];
        style.colors[StyleColor::FrameBgActive as usize] = [0.345, 0.361, 0.490, 1.0];
        style.colors[StyleColor::Header as usize] = [0.118, 0.314, 0.506, 1.0];
        style.colors[StyleColor::HeaderHovered as usize] = [0.243, 0.439, 0.643, 1.0];
        style.colors[StyleColor::HeaderActive as usize] = [0.345, 0.514, 0.725, 1.0];
        style.colors[StyleColor::Button as usize] = [0.118, 0.314, 0.506, 1.0];
        style.colors[StyleColor::ButtonHovered as usize] = [0.243, 0.439, 0.643, 1.0];
        style.colors[StyleColor::ButtonActive as usize] = [0.345, 0.514, 0.725, 1.0];
        style.colors[StyleColor::ScrollbarBg as usize] = [0.067, 0.067, 0.106, 1.0];
        style.colors[StyleColor::ScrollbarGrab as usize] = [0.306, 0.322, 0.416, 1.0];
        style.colors[StyleColor::ScrollbarGrabHovered as usize] = self.accent;
        style.colors[StyleColor::CheckMark as usize] = self.accent;
        style.window_rounding = 0.0;
        style.child_rounding = 0.0;
        style.frame_rounding = 3.0;
    }
}

fn source_file(
    source_db: &SourceDatabase,
    pc: u16,
) -> Option<(String, u64, usize, usize, Vec<(usize, String)>)> {
    let info = source_db
        .get_source_info_from_physical_address(pc as usize)
        .or_else(|| source_db.get_source_info_from_address(pc as usize))?;
    let file = info.file.clone();
    let file_id = info.file_id;
    let line = info.line_number;
    let source = source_db.get_source_file(file_id)?;
    let first = 0;
    let lines = (0..source.num_of_lines())
        .filter_map(|number| source.get_line(number).map(|entry| (number, entry.text)))
        .collect();
    Some((
        file.to_string_lossy().into_owned(),
        file_id,
        line,
        first,
        lines,
    ))
}

impl GameQuad {
    unsafe fn new(gl: &glow::Context) -> Result<Self, String> {
        let vertex = gl.create_shader(glow::VERTEX_SHADER)?;
        gl.shader_source(
            vertex,
            r#"#version 150
in vec2 a_pos;
in vec2 a_uv;
out vec2 v_uv;
void main() {
    gl_Position = vec4(a_pos, 0.0, 1.0);
    v_uv = a_uv;
}"#,
        );
        gl.compile_shader(vertex);
        if !gl.get_shader_compile_status(vertex) {
            let error = gl.get_shader_info_log(vertex);
            gl.delete_shader(vertex);
            return Err(format!("game vertex shader: {error}"));
        }

        let fragment = gl.create_shader(glow::FRAGMENT_SHADER)?;
        gl.shader_source(
            fragment,
            r#"#version 150
uniform sampler2D u_tex;
in vec2 v_uv;
out vec4 color;
void main() { color = texture(u_tex, v_uv); }"#,
        );
        gl.compile_shader(fragment);
        if !gl.get_shader_compile_status(fragment) {
            let error = gl.get_shader_info_log(fragment);
            gl.delete_shader(vertex);
            gl.delete_shader(fragment);
            return Err(format!("game fragment shader: {error}"));
        }

        let program = gl.create_program()?;
        gl.attach_shader(program, vertex);
        gl.attach_shader(program, fragment);
        gl.bind_attrib_location(program, 0, "a_pos");
        gl.bind_attrib_location(program, 1, "a_uv");
        gl.link_program(program);
        gl.delete_shader(vertex);
        gl.delete_shader(fragment);
        if !gl.get_program_link_status(program) {
            let error = gl.get_program_info_log(program);
            gl.delete_program(program);
            return Err(format!("game shader link: {error}"));
        }

        // position (x,y), texture coordinate (u,v), triangle strip
        let vertices: [f32; 16] = [
            -1.0, -1.0, 0.0, 1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0,
        ];
        let vao = gl.create_vertex_array()?;
        let vbo = gl.create_buffer()?;
        gl.bind_vertex_array(Some(vao));
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        let bytes = std::slice::from_raw_parts(
            vertices.as_ptr().cast::<u8>(),
            vertices.len() * std::mem::size_of::<f32>(),
        );
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytes, glow::STATIC_DRAW);
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 16, 0);
        gl.enable_vertex_attrib_array(1);
        gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 16, 8);
        gl.bind_vertex_array(None);
        gl.bind_buffer(glow::ARRAY_BUFFER, None);
        gl.delete_buffer(vbo);
        Ok(Self { program, vao })
    }

    unsafe fn draw(
        &self,
        gl: &glow::Context,
        texture: glow::NativeTexture,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) {
        gl.viewport(x, y, width, height);
        gl.use_program(Some(self.program));
        gl.active_texture(glow::TEXTURE0);
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));
        if let Some(location) = gl.get_uniform_location(self.program, "u_tex") {
            gl.uniform_1_i32(Some(&location), 0);
        }
        gl.bind_vertex_array(Some(self.vao));
        gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
        gl.bind_vertex_array(None);
        gl.use_program(None);
    }
}

impl Audio {
    fn new() -> Option<Self> {
        let device = DeviceSinkBuilder::open_default_sink().ok()?;
        let player = Player::connect_new(device.mixer());
        Some(Self {
            _device: device,
            player,
        })
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rom_dir = PathBuf::from(
        env::args()
            .nth(1)
            .ok_or("usage: stargate-app <rom-directory>")?,
    );
    let mut machine = StargateMachine::from_rom_dir(&rom_dir)?;
    machine.reset()?;
    let source_db = [rom_dir.join("stargate.map"), rom_dir.join("sound.map")]
        .iter()
        .find_map(|path| {
            let text = std::fs::read_to_string(path).ok()?;
            let mut source_db = serde_json::from_str::<SourceDatabase>(&text).ok()?;
            source_db.rebuild_indexes();
            Some(source_db)
        });
    let sound_path = env::args()
        .nth(2)
        .map(PathBuf::from)
        .unwrap_or_else(|| rom_dir.join("sound.bin"));
    let mut sound = std::fs::read(sound_path)
        .ok()
        .map(|rom| SoundHarness::from_rom_fast(&rom, HarnessConfig::default()));
    let audio = Audio::new();
    let (event_loop, window, config, surface, context) = create_window();
    let gl = unsafe {
        glow::Context::from_loader_function_cstr(|s| context.display().get_proc_address(s).cast())
    };
    let game_quad = unsafe { GameQuad::new(&gl)? };
    let game_gl_texture = unsafe { gl.create_texture()? };
    unsafe {
        gl.bind_texture(glow::TEXTURE_2D, Some(game_gl_texture));
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            glow::NEAREST as _,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            glow::NEAREST as _,
        );
    }
    let mut input = StargateInput::default();
    let mut debugger_window: Option<DebuggerSurface> = None;
    let mut paused = false;
    let mut step_requested = false;
    let mut source_height = 260.0_f32;
    let mut source_follow_pc = true;
    let mut source_syntax: Option<(u64, GazmSyntax)> = None;

    #[allow(deprecated)]
    event_loop.run(move |event, target| {
        if let Some(debugger) = debugger_window.as_mut() {
            debugger
                .platform
                .handle_event(debugger.imgui.io_mut(), &debugger.window, &event);
        }
        match event {
            imgui_winit_support::winit::event::Event::NewEvents(_) => {}
            imgui_winit_support::winit::event::Event::AboutToWait => window.request_redraw(),
            imgui_winit_support::winit::event::Event::WindowEvent {
                event: imgui_winit_support::winit::event::WindowEvent::RedrawRequested,
                window_id,
            } if window_id == window.id() => {
                let before_cycles = machine.cycles;
                if !paused || step_requested {
                    let count = if paused { 1 } else { 8_000 };
                    if let Err(error) = machine.run_instructions(count) {
                        eprintln!(
                            "emulator stopped at PC ${:04X} after {} instructions: {}",
                            machine.regs.pc, machine.instructions, error
                        );
                        paused = true;
                    }
                    step_requested = false;
                }
                let main_cycles = machine.cycles - before_cycles;
                if let Some(sound) = &mut sound {
                    for command in machine.take_sound_commands() {
                        sound.send_pia_value(command);
                    }
                    let sound_cycles =
                        ((main_cycles as u128 * sound.config.cpu_hz as u128) / 1_000_000) as u64;
                    if sound.run_cycles(sound_cycles).is_ok() {
                        let samples = sound.render_audio(sound_cycles);
                        if let Some(audio) = &audio {
                            audio.player.append(SamplesBuffer::new(
                                NonZeroU16::new(1).unwrap(),
                                NonZeroU32::new(sound.config.sample_rate).unwrap(),
                                samples,
                            ));
                        }
                    }
                }
                let pixels = machine.video_rgba_visible();
                unsafe {
                    gl.clear(glow::COLOR_BUFFER_BIT);
                    gl.bind_texture(glow::TEXTURE_2D, Some(game_gl_texture));
                    gl.tex_image_2d(
                        glow::TEXTURE_2D,
                        0,
                        glow::RGBA as _,
                        292,
                        240,
                        0,
                        glow::RGBA,
                        glow::UNSIGNED_BYTE,
                        Some(&pixels),
                    );
                    let size = window.inner_size();
                    let game_aspect = 292.0 / 240.0;
                    let window_aspect = size.width as f32 / size.height.max(1) as f32;
                    let (viewport_width, viewport_height) = if window_aspect > game_aspect {
                        (
                            (size.height as f32 * game_aspect) as i32,
                            size.height as i32,
                        )
                    } else {
                        (size.width as i32, (size.width as f32 / game_aspect) as i32)
                    };
                    let viewport_x = (size.width as i32 - viewport_width) / 2;
                    let viewport_y = (size.height as i32 - viewport_height) / 2;
                    game_quad.draw(
                        &gl,
                        game_gl_texture,
                        viewport_x,
                        viewport_y,
                        viewport_width,
                        viewport_height,
                    );
                }
                if let Some(debugger) = debugger_window.as_mut() {
                    let current_pc = machine.regs.pc;
                    let disassembly = machine.disassembly_around(current_pc, 8, 24);
                    let source_lines = source_db
                        .as_ref()
                        .and_then(|db| source_file(db, current_pc));
                    let source_file_id = source_lines.as_ref().map(|(_, file_id, _, _, _)| *file_id);
                    if let Some((_, file_id, _, _, _)) = &source_lines {
                        if source_syntax
                            .as_ref()
                            .map(|(cached_file_id, _)| cached_file_id != file_id)
                            .unwrap_or(true)
                        {
                            source_syntax = source_db
                                .as_ref()
                                .and_then(|db| db.get_source_file(*file_id))
                                .and_then(|source| {
                                    GazmSyntax::parse(&source.get_entire_source()?)
                                        .ok()
                                        .map(|syntax| (*file_id, syntax))
                                });
                        }
                    } else {
                        source_syntax = None;
                    }
                    debugger
                        .platform
                        .prepare_frame(debugger.imgui.io_mut(), &debugger.window)
                        .expect("debugger imgui frame");
                    if debugger.imgui.io().delta_time <= 0.0 {
                        debugger.imgui.io_mut().delta_time = 1.0 / 60.0;
                    }
                    let display_size = debugger.imgui.io().display_size;
                    let ui = debugger.imgui.frame();
                    ui.window("##debugger_root")
                        .size(display_size, Condition::Always)
                        .position([0.0, 0.0], Condition::Always)
                        .flags(
                            WindowFlags::NO_DECORATION
                                | WindowFlags::NO_MOVE
                                | WindowFlags::NO_RESIZE
                                | WindowFlags::NO_SAVED_SETTINGS
                                | WindowFlags::NO_BACKGROUND,
                        )
                        .build(|| {
                            // One ImGui host window owns the whole client area.
                            // The debugger panes are collapsing headers rather
                            // than independent ImGui windows, so closing one
                            // naturally moves the remaining panes upward.
                            let _scroll = ui
                                .child_window("debugger_content")
                                .size([0.0, 0.0])
                                .border(false)
                                .build(|| {
                                    if ui
                                        .collapsing_header("Registers", TreeNodeFlags::DEFAULT_OPEN)
                                    {
                                        if ui.button(if paused { "Resume" } else { "Pause" }) {
                                            paused = !paused;
                                        }
                                        ui.same_line();
                                        if ui.button("Step") {
                                            paused = true;
                                            step_requested = true;
                                        }
                                        let names = [
                                            "PC", "A", "B", "X", "Y", "S", "U", "CC", "FLAGS",
                                        ];
                                        let values = [
                                            format!("${:04X}", machine.regs.pc),
                                            format!("${:02X}", machine.regs.a),
                                            format!("${:02X}", machine.regs.b),
                                            format!("${:04X}", machine.regs.x),
                                            format!("${:04X}", machine.regs.y),
                                            format!("${:04X}", machine.regs.s),
                                            format!("${:04X}", machine.regs.u),
                                            format!("${:02X}", machine.regs.flags.bits()),
                                        ];
                                        let header_min = ui.cursor_screen_pos();
                                        let header_max = [
                                            header_min[0] + 635.0,
                                            header_min[1] + ui.text_line_height_with_spacing(),
                                        ];
                                        ui.get_window_draw_list()
                                            .add_rect(
                                                header_min,
                                                header_max,
                                                debugger.theme.current_line_bg,
                                            )
                                            .filled(true)
                                            .build();
                                        // Fixed layout: resizing legacy columns can invalidate
                                        // their clipping state while the debugger is redrawing.
                                        ui.columns(9, "registers", false);
                                        for (column, width) in
                                            [65.0, 50.0, 50.0, 75.0, 75.0, 75.0, 75.0, 60.0, 110.0]
                                                .into_iter()
                                                .enumerate()
                                        {
                                            ui.set_column_width(column as i32, width);
                                        }
                                        for name in names {
                                            ui.text_colored(debugger.theme.accent, name);
                                            ui.next_column();
                                        }
                                        for value in values {
                                            ui.text(value);
                                            ui.next_column();
                                        }
                                        // Keep the condition-code letters in the CC column,
                                        // directly below its hexadecimal value.
                                        let flags = ["E", "F", "H", "I", "N", "Z", "V", "C"];
                                        let mask = [0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, 0x01];
                                        for (index, bit) in mask.into_iter().enumerate() {
                                            let set = machine.regs.flags.bits() & bit != 0;
                                            ui.text_colored(
                                                if set { debugger.theme.accent } else { debugger.theme.muted },
                                                flags[index],
                                            );
                                            if index + 1 < flags.len() {
                                                ui.same_line();
                                            }
                                        }
                                        ui.columns(1, "registers_end", false);
                                        ui.text(format!(
                                            "cycles {} instructions {}",
                                            machine.cycles, machine.instructions
                                        ));
                                    }
                                    if ui.collapsing_header("Source", TreeNodeFlags::DEFAULT_OPEN) {
                                        ui.child_window("source_pane")
                                            .size([0.0, source_height])
                                            .border(true)
                                            .build(|| {
                                                if ui.is_window_hovered()
                                                    && (ui.io().mouse_wheel != 0.0
                                                        || ui.io()[MouseButton::Left])
                                                {
                                                    source_follow_pc = false;
                                                }
                                                if let Some((file, _, current_line, _, lines)) =
                                                    &source_lines
                                                {
                                                    ui.text(format!("{file}:{}", current_line + 1));
                                                    let _mono_font =
                                                        ui.push_font(debugger.mono_font);
                                                    for (line, text) in lines {
                                                        // Keep source text in a stable column, like a
                                                        // conventional editor: the line-number gutter
                                                        // is muted and never shifts the code around.
                                                        let is_current = *line == *current_line;
                                                        if is_current {
                                                            let row_min = ui.cursor_screen_pos();
                                                            let row_max = [
                                                                row_min[0] + ui.content_region_avail()[0],
                                                                row_min[1] + ui.text_line_height_with_spacing(),
                                                            ];
                                                            ui.get_window_draw_list()
                                                                .add_rect(
                                                                    row_min,
                                                                    row_max,
                                                                    debugger.theme.current_line_bg,
                                                                )
                                                                .filled(true)
                                                                .build();
                                                            ui.text_colored(
                                                                debugger.theme.current_line,
                                                                format!("> {:04}", line + 1),
                                                            );
                                                        } else {
                                                            ui.text_colored(
                                                                debugger.theme.muted,
                                                                format!("  {:04}", line + 1),
                                                            );
                                                        }
                                                        ui.same_line_with_pos(92.0);
                                                        let highlighted = source_syntax
                                                            .as_ref()
                                                            .filter(|(file_id, _)| {
                                                                Some(*file_id) == source_file_id
                                                            })
                                                            .map(|(_, syntax)| syntax.line_spans(*line));
                                                        if let Some(spans) = highlighted {
                                                            for (index, (segment, kind)) in
                                                                spans.into_iter().enumerate()
                                                            {
                                                                if index != 0 {
                                                                    ui.same_line();
                                                                }
                                                                ui.text_colored(
                                                                    syntax_color(debugger.theme, kind),
                                                                    segment,
                                                                );
                                                            }
                                                            if source_follow_pc && is_current {
                                                                ui.set_scroll_here_y_with_ratio(0.5);
                                                            }
                                                        } else if is_current {
                                                            ui.text_colored(debugger.theme.current_line, text);
                                                            if source_follow_pc {
                                                                ui.set_scroll_here_y_with_ratio(0.5);
                                                            }
                                                        } else {
                                                            ui.text(text);
                                                        }
                                                    }
                                                } else {
                                                    ui.text("No source mapping for the current PC");
                                                }
                                            });
                                        // A small invisible drag handle gives the
                                        // source pane a real vertical splitter
                                        // without creating another top-level window.
                                        let region = ui.window_content_region_max()[0]
                                            - ui.window_content_region_min()[0];
                                        let resize_width = region.max(100.0);
                                        ui.invisible_button("##source_resize", [resize_width, 12.0]);
                                        if ui.is_item_hovered() || ui.is_item_active() {
                                            ui.set_mouse_cursor(Some(MouseCursor::ResizeNS));
                                        }
                                        if ui.is_item_active() {
                                            source_height = (source_height
                                                + ui.io().mouse_delta[1])
                                                .clamp(120.0, 900.0);
                                        }
                                    }
                                    if ui.collapsing_header(
                                        "Disassembly",
                                        TreeNodeFlags::DEFAULT_OPEN,
                                    ) {
                                        let _mono_font = ui.push_font(debugger.mono_font);
                                        for (pc, line) in &disassembly {
                                            if *pc == current_pc {
                                                ui.text_colored(
                                                    debugger.theme.current_line,
                                                    format!("> {line}"),
                                                );
                                            } else {
                                                ui.text(format!("  {line}"));
                                            }
                                        }
                                    }
                                });
                        });
                    debugger.platform.prepare_render(ui, &debugger.window);
                    let draw_data = debugger.imgui.render();
                    if draw_data.draw_lists_count() != 0 {
                        debugger
                            .context
                            .make_current(&debugger.surface)
                            .expect("debugger context");
                        unsafe { debugger.gl.clear(glow::COLOR_BUFFER_BIT) };
                        if let Err(error) =
                            debugger
                                .renderer
                                .render(&debugger.gl, &debugger.textures, draw_data)
                        {
                            eprintln!("imgui render: {error}");
                        }
                        debugger
                            .surface
                            .swap_buffers(&debugger.context)
                            .expect("debugger swap buffers");
                        context.make_current(&surface).expect("game context");
                    }
                }
                surface.swap_buffers(&context).expect("swap buffers");
            }
            imgui_winit_support::winit::event::Event::WindowEvent {
                event: imgui_winit_support::winit::event::WindowEvent::KeyboardInput { event, .. },
                ..
            } => {
                use imgui_winit_support::winit::event::ElementState;
                use imgui_winit_support::winit::keyboard::PhysicalKey;
                let pressed = event.state == ElementState::Pressed;
                match event.physical_key {
                    PhysicalKey::Code(code) => match code {
                        imgui_winit_support::winit::keyboard::KeyCode::KeyP
                            if pressed && !event.repeat =>
                        {
                            paused = !paused;
                        }
                        imgui_winit_support::winit::keyboard::KeyCode::KeyN
                            if pressed && paused =>
                        {
                            step_requested = true;
                        }
                        imgui_winit_support::winit::keyboard::KeyCode::KeyR
                            if pressed && !event.repeat =>
                        {
                            if let Err(error) = machine.reset() {
                                eprintln!("reset emulator: {error}");
                            }
                        }
                        imgui_winit_support::winit::keyboard::KeyCode::KeyD if pressed => {
                            if debugger_window.is_some() {
                                debugger_window = None;
                            } else {
                                let debug_window = target
                                    .create_window(
                                        WindowAttributes::default()
                                            .with_title("Stargate Debugger")
                                            .with_inner_size(LogicalSize::new(900, 620)),
                                    )
                                    .expect("create debugger window");
                                let raw = debug_window.window_handle().unwrap().as_raw();
                                let debug_size = debug_window.inner_size();
                                let debug_surface = unsafe {
                                    config
                                        .display()
                                        .create_window_surface(
                                            &config,
                                            &SurfaceAttributesBuilder::<WindowSurface>::new()
                                                .build(
                                                    raw,
                                                    NonZeroU32::new(debug_size.width.max(1))
                                                        .unwrap(),
                                                    NonZeroU32::new(debug_size.height.max(1))
                                                        .unwrap(),
                                                ),
                                        )
                                        .expect("create debugger surface")
                                };
                                let debug_context = unsafe {
                                    config
                                        .display()
                                        .create_context(
                                            &config,
                                            &ContextAttributesBuilder::new().build(Some(raw)),
                                        )
                                        .expect("create debugger context")
                                        .make_current(&debug_surface)
                                        .expect("make debugger context current")
                                };
                                let debug_gl = unsafe {
                                    glow::Context::from_loader_function_cstr(|s| {
                                        debug_context.display().get_proc_address(s).cast()
                                    })
                                };
                                let mut debug_imgui = imgui::Context::create();
                                debug_imgui.set_ini_filename(None);
                                let theme = DebuggerTheme::catppuccin_mocha();
                                theme.apply_to(&mut debug_imgui);
                                let mut debug_platform = WinitPlatform::new(&mut debug_imgui);
                                debug_platform.attach_window(
                                    debug_imgui.io_mut(),
                                    &debug_window,
                                    HiDpiMode::Rounded,
                                );
                                let hidpi = debug_platform.hidpi_factor() as f32;
                                // The built-in ImGui font is intentionally tiny and
                                // pixel-oriented.  On a Retina display it looks like
                                // a low-resolution emulator font, so rasterize a real
                                // system TTF at the physical pixel size instead.
                                let debugger_font = std::fs::read(
                                    "/Users/garyliddon/Library/Fonts/JetBrainsMonoNerdFontMono-Regular.ttf",
                                )
                                    .or_else(|_| std::fs::read("/System/Library/Fonts/SFNSMono.ttf"))
                                    .or_else(|_| {
                                        std::fs::read(
                                            "/System/Library/Fonts/Menlo.ttc",
                                        )
                                    })
                                    .ok();
                                // One font and one size for the entire debugger keeps the
                                // gutter, source, controls, and disassembly aligned.
                                let debugger_font = if let Some(bytes) = debugger_font {
                                    let bytes: &'static [u8] = Box::leak(bytes.into_boxed_slice());
                                    debug_imgui.fonts().add_font(&[imgui::FontSource::TtfData {
                                        data: bytes,
                                        size_pixels: 18.0 * hidpi,
                                        config: Some(imgui::FontConfig {
                                            oversample_h: 2,
                                            oversample_v: 2,
                                            ..Default::default()
                                        }),
                                    }])
                                } else {
                                    debug_imgui.fonts().add_font(&[
                                        imgui::FontSource::DefaultFontData {
                                            config: Some(imgui::FontConfig {
                                                size_pixels: 18.0 * hidpi,
                                                ..Default::default()
                                            }),
                                        },
                                    ])
                                };
                                debug_imgui.io_mut().font_global_scale = 1.0 / hidpi;
                                let mut debug_textures =
                                    imgui::Textures::<glow::Texture>::default();
                                let debug_renderer = Renderer::new(
                                    &debug_gl,
                                    &mut debug_imgui,
                                    &mut debug_textures,
                                    true,
                                )
                                .expect("create debugger renderer");
                                context
                                    .make_current(&surface)
                                    .expect("restore game context");
                                debugger_window = Some(DebuggerSurface {
                                    window: debug_window,
                                    surface: debug_surface,
                                    context: debug_context,
                                    gl: debug_gl,
                                    imgui: debug_imgui,
                                    platform: debug_platform,
                                    textures: debug_textures,
                                    renderer: debug_renderer,
                                    mono_font: debugger_font,
                                    theme,
                                });
                            }
                        }
                        imgui_winit_support::winit::keyboard::KeyCode::Tab
                            if pressed && !event.repeat =>
                        {
                            source_follow_pc = true;
                        }
                        imgui_winit_support::winit::keyboard::KeyCode::ArrowUp => {
                            input.up = pressed
                        }
                        imgui_winit_support::winit::keyboard::KeyCode::ArrowDown => {
                            input.down = pressed
                        }
                        imgui_winit_support::winit::keyboard::KeyCode::KeyZ => input.fire = pressed,
                        imgui_winit_support::winit::keyboard::KeyCode::KeyX => {
                            input.thrust = pressed
                        }
                        imgui_winit_support::winit::keyboard::KeyCode::KeyC => {
                            input.smart_bomb = pressed
                        }
                        imgui_winit_support::winit::keyboard::KeyCode::Space => {
                            input.hyperspace = pressed
                        }
                        imgui_winit_support::winit::keyboard::KeyCode::KeyA => {
                            input.reverse = pressed
                        }
                        imgui_winit_support::winit::keyboard::KeyCode::KeyS => {
                            input.inviso = pressed
                        }
                        imgui_winit_support::winit::keyboard::KeyCode::Digit1 => {
                            input.start1 = pressed
                        }
                        imgui_winit_support::winit::keyboard::KeyCode::Digit2 => {
                            input.start2 = pressed
                        }
                        imgui_winit_support::winit::keyboard::KeyCode::Digit5 => {
                            input.coin1 = pressed
                        }
                        imgui_winit_support::winit::keyboard::KeyCode::F2 => {
                            input.service = pressed
                        }
                        imgui_winit_support::winit::keyboard::KeyCode::F3 => {
                            input.advance = pressed
                        }
                        _ => {}
                    },
                    _ => {}
                }
                machine.set_input(input);
            }
            imgui_winit_support::winit::event::Event::WindowEvent {
                event: imgui_winit_support::winit::event::WindowEvent::CloseRequested,
                window_id,
            } => {
                if window_id == window.id() {
                    target.exit();
                } else if debugger_window
                    .as_ref()
                    .is_some_and(|debugger| debugger.window.id() == window_id)
                {
                    debugger_window = None;
                }
            }
            imgui_winit_support::winit::event::Event::WindowEvent {
                event: imgui_winit_support::winit::event::WindowEvent::Resized(size),
                window_id,
            } => {
                if size.width > 0 && size.height > 0 {
                    let width = NonZeroU32::new(size.width).unwrap();
                    let height = NonZeroU32::new(size.height).unwrap();
                    if window_id == window.id() {
                        surface.resize(&context, width, height);
                    } else if let Some(debugger) = debugger_window.as_ref() {
                        if debugger.window.id() == window_id {
                            debugger.surface.resize(&debugger.context, width, height);
                        }
                    }
                }
            }
            _other => {}
        }
    })?;
    Ok(())
}

fn create_window() -> (
    EventLoop<()>,
    Window,
    Config,
    Surface<WindowSurface>,
    PossiblyCurrentContext,
) {
    let event_loop = EventLoop::new().unwrap();
    let attrs = WindowAttributes::default()
        .with_title("Stargate")
        .with_inner_size(LogicalSize::new(1168, 960));
    let (window, config) = glutin_winit::DisplayBuilder::new()
        .with_window_attributes(Some(attrs))
        .build(&event_loop, ConfigTemplateBuilder::new(), |mut configs| {
            configs.next().unwrap()
        })
        .unwrap();
    let window = window.unwrap();
    let raw = window.window_handle().unwrap().as_raw();
    let context_attrs = ContextAttributesBuilder::new().build(Some(raw));
    let context = unsafe {
        config
            .display()
            .create_context(&config, &context_attrs)
            .unwrap()
    };
    let surface_attrs = SurfaceAttributesBuilder::<WindowSurface>::new()
        .with_srgb(Some(true))
        .build(
            raw,
            NonZeroU32::new(1168).unwrap(),
            NonZeroU32::new(960).unwrap(),
        );
    let surface = unsafe {
        config
            .display()
            .create_window_surface(&config, &surface_attrs)
            .unwrap()
    };
    let context = context.make_current(&surface).unwrap();
    (event_loop, window, config, surface, context)
}
