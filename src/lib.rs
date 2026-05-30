use clap::Parser;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
    },
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io, panic, time::Duration};

use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Layout, Rect},
    style::Color,
    widgets::Block,
};

use crate::{
    error::Result,
    git::kit::GRepo,
    metrics::{RenderMetric, cadence::CadenceMetric, overview::OverviewMetric},
    tui::{
        state::State,
        ui::{
            Page::{self},
            nav,
        },
    },
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct GKitArgs {
    #[arg(default_value = ".")]
    target_path: String,
}

pub mod error;
pub mod git;
pub mod metrics;
pub mod tui;

pub fn run(args: GKitArgs) -> Result<()> {
    let repo = GRepo::open(args.target_path)?;
    let mut state = State::new();

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    terminal::enable_raw_mode()?;
    ratatui::crossterm::execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    panic::set_hook(Box::new(move |panic| {
        let _ = terminal::disable_raw_mode();
        let _ =
            ratatui::crossterm::execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        eprintln!("Panic??: {}", panic);
    }));
    terminal.hide_cursor()?;
    terminal.clear()?;

    let tui_result = tui(&mut terminal, &mut state, &repo);

    let _ = terminal.show_cursor();
    let _ = ratatui::crossterm::execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
    let _ = terminal::disable_raw_mode();

    tui_result // returns once drawing stops
}

pub fn tui(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &mut State,
    repo: &GRepo,
) -> Result<()> {
    terminal.clear()?;

    state.is_quit = false;

    let cadence = CadenceMetric::full_report(repo)?;
    state.cadence = Some(cadence);

    let overview = OverviewMetric::default();
    state.overview = Some(overview);

    while !state.is_quit {
        terminal.draw(|frame| render(frame, &state))?;

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Tab => state.tab(),
                        _ => match state.page {
                            Page::Overview => {}
                            Page::Cadence => {
                                if let Some(cadence) = &mut state.cadence {
                                    cadence.update(key.code);
                                }
                            }
                            Page::Todo => {}
                        },
                    };
                };
                let is_ctrl_c =
                    key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL);
                let is_q = key.code == KeyCode::Char('q');

                if is_ctrl_c || is_q {
                    state.is_quit = true;
                    break;
                }
            }
        }
    }

    Ok(())
}

pub fn render(frame: &mut Frame, state: &State) {
    let chunks = Layout::vertical([Constraint::Length(Page::size() as u16), Constraint::Min(0)])
        .horizontal_margin(2)
        .vertical_margin(1)
        .split(frame.area());

    render_tabs(frame, &state, chunks[0]);

    match state.page {
        Page::Overview => match &state.overview {
            Some(overview) => overview.render(frame, chunks[1], state),
            _ => {}
        },
        Page::Cadence => match &state.cadence {
            Some(cadence) => cadence.render(frame, chunks[1], state),
            _ => {}
        },
        Page::Todo => {}
    }
}

fn render_tabs(frame: &mut Frame, state: &State, chunk: Rect) {
    let nav_block = Block::bordered()
        .border_style(Color::Gray)
        .title("< Ghkit: {Version} >")
        .title_alignment(ratatui::layout::HorizontalAlignment::Center);

    let nav_tabs = nav(state).block(nav_block);

    frame.render_widget(nav_tabs, chunk);
}
