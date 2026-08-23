use std::{env, io, time::Duration};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    DefaultTerminal, Frame,
};
use stargate_emu::{StargateInput, StargateMachine};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Game,
    Debugger,
}

struct App {
    machine: StargateMachine,
    input: StargateInput,
    running: bool,
    mode: Mode,
    message: String,
}

impl App {
    fn draw(&self, frame: &mut Frame) {
        match self.mode {
            Mode::Game => self.draw_game(frame),
            Mode::Debugger => self.draw_debugger(frame),
        }
    }

    fn draw_game(&self, frame: &mut Frame) {
        let areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(2)])
            .split(frame.area());
        let video = self
            .machine
            .video_ascii(100, 38)
            .into_iter()
            .map(Line::from)
            .collect::<Vec<_>>();
        frame.render_widget(
            Paragraph::new(video).block(Block::default().title("Stargate").borders(Borders::ALL)),
            areas[0],
        );
        let status = Paragraph::new(Line::from(vec![
            Span::styled(
                if self.running {
                    " RUNNING "
                } else {
                    " PAUSED "
                },
                Style::default().fg(Color::Yellow),
            ),
            Span::raw(
                " arrows move  Z fire  X thrust  C bomb  Space hyperspace  F12 debugger  Q quit",
            ),
        ]))
        .block(Block::default().borders(Borders::TOP));
        frame.render_widget(status, areas[1]);
    }

    fn draw_debugger(&self, frame: &mut Frame) {
        let areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),
                Constraint::Min(1),
                Constraint::Length(2),
            ])
            .split(frame.area());
        let regs = Paragraph::new(vec![
            Line::from(format!(
                "PC ${:04X}    A ${:02X}  B ${:02X}    X ${:04X}  Y ${:04X}",
                self.machine.regs.pc,
                self.machine.regs.a,
                self.machine.regs.b,
                self.machine.regs.x,
                self.machine.regs.y
            )),
            Line::from(format!(
                "S  ${:04X}    U ${:04X} DP ${:02X}    CC ${:02X}",
                self.machine.regs.s,
                self.machine.regs.u,
                self.machine.regs.dp,
                self.machine.regs.flags.bits()
            )),
            Line::from(format!(
                "instructions {:>12}    cycles {:>12}",
                self.machine.instructions, self.machine.cycles
            )),
        ])
        .block(
            Block::default()
                .title("6809 CPU debugger")
                .borders(Borders::ALL),
        );
        frame.render_widget(regs, areas[0]);
        let video = self
            .machine
            .video_ascii(80, 30)
            .into_iter()
            .map(Line::from)
            .collect::<Vec<_>>();
        frame.render_widget(
            Paragraph::new(video).block(Block::default().title("Video RAM").borders(Borders::ALL)),
            areas[1],
        );
        let status = Paragraph::new(Line::from(vec![
            Span::styled(
                if self.running {
                    " RUNNING "
                } else {
                    " PAUSED "
                },
                Style::default().fg(Color::Yellow),
            ),
            Span::raw(format!(
                " {}   F12 game  R reset  S step  Space run/pause",
                self.message
            )),
        ]))
        .block(Block::default().borders(Borders::TOP));
        frame.render_widget(status, areas[2]);
    }

    fn tick(&mut self) {
        if self.running {
            if let Err(error) = self.machine.run_instructions(10_000) {
                self.running = false;
                self.message = error.to_string();
            }
        }
    }

    fn key(&mut self, key: KeyEvent) {
        let pressed = key.kind != KeyEventKind::Release;
        if pressed && key.code == KeyCode::F(12) {
            self.mode = if self.mode == Mode::Game {
                Mode::Debugger
            } else {
                Mode::Game
            };
            if self.mode == Mode::Debugger {
                self.running = false;
            }
            return;
        }
        if pressed && (key.code == KeyCode::Char(' ') || key.code == KeyCode::Char('r')) {
            self.running = !self.running;
        }
        if pressed && self.mode == Mode::Debugger {
            match key.code {
                KeyCode::Char('s') => {
                    if let Err(error) = self.machine.step() {
                        self.message = error.to_string();
                    }
                }
                KeyCode::Char('R') => {
                    if let Err(error) = self.machine.reset() {
                        self.message = error.to_string();
                    }
                }
                _ => {}
            }
        }
        let mut input = self.input;
        match key.code {
            KeyCode::Up => input.up = pressed,
            KeyCode::Down => input.down = pressed,
            KeyCode::Char('z') => input.fire = pressed,
            KeyCode::Char('x') => input.thrust = pressed,
            KeyCode::Char('c') => input.smart_bomb = pressed,
            KeyCode::Char(' ') => input.hyperspace = pressed,
            KeyCode::Char('a') => input.reverse = pressed,
            KeyCode::Char('s') => input.inviso = pressed,
            KeyCode::Char('1') => input.start1 = pressed,
            KeyCode::Char('2') => input.start2 = pressed,
            KeyCode::Char('5') => input.coin1 = pressed,
            _ => {}
        }
        self.input = input;
        self.machine.set_input(input);
    }
}

fn run(mut terminal: DefaultTerminal, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|frame| app.draw(frame))?;
        app.tick();
        if event::poll(Duration::from_millis(30))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    break;
                }
                app.key(key);
            }
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rom_dir = env::args()
        .nth(1)
        .ok_or("usage: stargate-debug <rom-directory>")?;
    let mut machine = StargateMachine::from_rom_dir(rom_dir)?;
    machine.reset()?;
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let terminal = ratatui::init();
    let result = run(
        terminal,
        App {
            machine,
            input: StargateInput::default(),
            running: true,
            mode: Mode::Game,
            message: "ready".into(),
        },
    );
    ratatui::restore();
    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen)?;
    result?;
    Ok(())
}
