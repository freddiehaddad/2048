mod board;
mod event;
mod game;

use anyhow::Result;
use event::GameEvent;
use ratatui::layout::Rect;
use tokio::sync::mpsc;

// ============================================================================
// UI Constants
// ============================================================================

const CELL_WIDTH: u16 = 10;
const CELL_HEIGHT: u16 = 5;
const CELL_GAP_X: u16 = 1;
const OUTER_MARGIN_X: u16 = 1;
const SCORE_HEIGHT: u16 = 1;

// ============================================================================
// UI Rendering
// ============================================================================

fn render(
    frame: &mut ratatui::Frame,
    game: &game::Game,
    report: &game::TurnResult,
) {
    use crate::board::BOARD_SIZE;
    use ratatui::layout::{Constraint, Layout};

    let board_size = BOARD_SIZE as u16;

    // Total game width: cells + gaps between cells + outer margin on both sides
    let grid_width =
        (CELL_WIDTH * board_size) + (CELL_GAP_X * (board_size - 1));
    let board_width = grid_width + (OUTER_MARGIN_X * 2) + 2; // +2 for outer border

    // Total board height: cells + border
    let grid_height = CELL_HEIGHT * board_size;
    let board_height = grid_height + 2; // +2 for outer border

    // Total game height: board + score line
    let game_height = board_height + SCORE_HEIGHT;

    // Create a centered game area (board + score)
    let area = frame.area();
    let game_area = area.centered(
        Constraint::Length(board_width),
        Constraint::Length(game_height),
    );

    // Split into board area and score area
    let [board_area, score_area] = Layout::vertical([
        Constraint::Length(board_height),
        Constraint::Length(SCORE_HEIGHT),
    ])
    .areas(game_area);

    render_board(frame, board_area, game, report);
    render_score(frame, score_area, game);
}

fn render_board(
    frame: &mut ratatui::Frame,
    area: Rect,
    game: &game::Game,
    report: &game::TurnResult,
) {
    use crate::board::BOARD_SIZE;
    use crate::game::GAME_TITLE;
    use ratatui::{
        layout::{Constraint, Flex, Layout},
        style::Color,
        widgets::{Block, BorderType, Borders, Paragraph},
    };

    // Render the main border around the board
    let main_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Color::Gray)
        .title_style(Color::Yellow)
        .title(GAME_TITLE);
    frame.render_widget(&main_block, area);

    // Create the grid layout for each tile
    let row_layout =
        Layout::vertical([Constraint::Length(CELL_HEIGHT); BOARD_SIZE])
            .horizontal_margin(OUTER_MARGIN_X);

    let mut col_constraints = Vec::new();
    for col in 0..BOARD_SIZE {
        col_constraints.push(Constraint::Length(CELL_WIDTH));
        if col + 1 < BOARD_SIZE {
            col_constraints.push(Constraint::Length(CELL_GAP_X));
        }
    }
    let col_layout = Layout::horizontal(col_constraints);

    let inner_area = main_block.inner(area); // Area inside borders
    let rows = inner_area.layout_vec(&row_layout);

    let cell_areas = rows.iter().flat_map(|row_area| {
        row_area
            .layout_vec(&col_layout)
            .into_iter()
            .enumerate()
            .filter_map(|(idx, area)| (idx % 2 == 0).then_some(area))
    });

    let game_over = game.is_game_over();

    for (i, (cell_area, value)) in
        cell_areas.zip(game.board().iter_tiles()).enumerate()
    {
        let row = i / BOARD_SIZE;
        let col = i % BOARD_SIZE;

        let mut cell_block = Block::default().borders(Borders::ALL);
        let mut style = Color::DarkGray;

        if !game_over && report.merged[row][col] {
            // Highlight the merged tile
            style = Color::Green;
        }

        cell_block = cell_block.style(style);
        frame.render_widget(&cell_block, cell_area);

        let cell_inner = cell_block.inner(cell_area);

        // Center the cell value horizontally in the cell
        let vertical_layout =
            Layout::vertical([Constraint::Length(1)]).flex(Flex::Center);
        let [text_area] = vertical_layout.areas(cell_inner);

        // Get the cell value and format it for display
        let (text, style) = if game_over {
            match (row, col) {
                (1, 0) => ("G".to_string(), Color::Red),
                (1, 1) => ("A".to_string(), Color::Red),
                (1, 2) => ("M".to_string(), Color::Red),
                (1, 3) => ("E".to_string(), Color::Red),
                (2, 0) => ("O".to_string(), Color::Red),
                (2, 1) => ("V".to_string(), Color::Red),
                (2, 2) => ("E".to_string(), Color::Red),
                (2, 3) => ("R".to_string(), Color::Red),
                _ => (
                    value.map_or(String::default(), |v| v.to_string()),
                    Color::LightCyan,
                ),
            }
        } else {
            (
                value.map_or(String::default(), |v| v.to_string()),
                Color::LightCyan,
            )
        };
        let paragraph = Paragraph::new(text).centered().style(style);
        frame.render_widget(paragraph, text_area);
    }
}

fn render_score(frame: &mut ratatui::Frame, area: Rect, game: &game::Game) {
    use ratatui::widgets::Paragraph;

    // Render score below the board
    let score_text = format!("Score: {:>6} ", game.score());
    let score_paragraph = Paragraph::new(score_text).right_aligned();
    frame.render_widget(score_paragraph, area);
}

// ============================================================================
// Input Handling
// ============================================================================

fn handle_input_events(
    input_tx: mpsc::UnboundedSender<GameEvent>,
) -> Result<()> {
    use crossterm::event::KeyCode::*;
    use event::GameEvent::*;

    loop {
        let event = crossterm::event::read()?;

        let key = match event.as_key_press_event() {
            Some(key_event) => key_event,
            None => continue,
        };

        match key.code {
            Char('q') => {
                input_tx.send(Quit)?;
                break;
            }
            Char('r') => input_tx.send(Restart)?,
            Up | Char('k') | Char('w') => input_tx.send(MoveUp)?,
            Down | Char('j') | Char('s') => input_tx.send(MoveDown)?,
            Left | Char('h') | Char('a') => input_tx.send(MoveLeft)?,
            Right | Char('l') | Char('d') => input_tx.send(MoveRight)?,
            _ => (),
        };
    }
    Ok(())
}

// ============================================================================
// Main Game Loop
// ============================================================================

async fn run(mut terminal: ratatui::DefaultTerminal) -> Result<()> {
    use event::GameEvent::*;

    let mut game = game::Game::new();
    let mut last_report = game::TurnResult::default();

    let (input_tx, mut input_rx) = mpsc::unbounded_channel();

    tokio::task::spawn_blocking(move || handle_input_events(input_tx));

    terminal.draw(|frame| render(frame, &game, &last_report))?;
    while let Some(e) = input_rx.recv().await {
        match !game.is_game_over() {
            true => match e {
                Quit => break,
                Restart => {
                    game.reset();
                    last_report = game::TurnResult::default();
                }
                MoveUp => last_report = game.move_up(),
                MoveDown => last_report = game.move_down(),
                MoveLeft => last_report = game.move_left(),
                MoveRight => last_report = game.move_right(),
            },

            false => match e {
                Quit => break,
                Restart => {
                    game.reset();
                    last_report = game::TurnResult::default();
                }
                _ => (),
            },
        }

        terminal.draw(|frame| render(frame, &game, &last_report))?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let terminal = ratatui::init();
    let result = run(terminal).await;
    ratatui::restore();
    result
}
