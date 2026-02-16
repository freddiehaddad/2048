mod board;
mod event;
mod game;

use anyhow::Result;
use ratatui::crossterm::event::{KeyCode, read};
use ratatui::layout::{Margin, Rect};
use ratatui::style::Style;
use ratatui::{DefaultTerminal, Frame};
use ratatui::{
    layout::{Constraint, Layout},
    widgets::{Block, BorderType, Paragraph},
};
use tokio::{
    sync::mpsc::{Receiver, Sender, channel},
    task::spawn_blocking,
};

use crate::board::BOARD_SIZE;
use crate::event::Event;
use crate::game::{ActionOutcome, CellResult, Game, GameAction, TITLE};

const BUFSIZE: usize = 1;

const CELL_WIDTH: u16 = 11;
const CELL_HEIGHT: u16 = 5;
const SCORE_HEIGHT: u16 = 1;
const CELL_PADDING_X: u16 = 1;
const CELL_PADDING_Y: u16 = 2;
const BORDER_WIDTH: u16 = 1;

fn calculate_game_dimensions() -> (u16, u16) {
    let width = BOARD_SIZE as u16 * (CELL_WIDTH + CELL_PADDING_X)
        + CELL_PADDING_X
        + (BORDER_WIDTH * 2);
    let height =
        BOARD_SIZE as u16 * CELL_HEIGHT + SCORE_HEIGHT + (BORDER_WIDTH * 2);
    (width, height)
}

// Render the border and title around the tiles area
fn render_board(outcome: &ActionOutcome, area: Rect, frame: &mut Frame) {
    let style = if outcome.game_over {
        Style::new().red()
    } else {
        Style::new()
    };

    frame.render_widget(
        Block::bordered()
            .border_type(BorderType::Thick)
            .border_style(style)
            .title(TITLE),
        area,
    );
}

fn render_tiles(
    board: &[[CellResult; BOARD_SIZE]; BOARD_SIZE],
    area: Rect,
    frame: &mut Frame,
) {
    // Split the tiles area into rows
    let rows_layout = Layout::vertical([Constraint::Fill(1); BOARD_SIZE]);
    let rows_rects: [Rect; BOARD_SIZE] = rows_layout.areas(
        area.inner(Margin::new(BORDER_WIDTH + CELL_PADDING_X, BORDER_WIDTH)),
    );

    // Each row is split into columns, with spacing between them
    let cols_layout = Layout::horizontal((0..BOARD_SIZE).flat_map(|i| {
        if i < BOARD_SIZE - 1 {
            [Constraint::Fill(1), Constraint::Length(1)].iter()
        } else {
            [Constraint::Fill(1)].iter()
        }
    }));

    // Iterate over the row rectangles and render the tiles within each row
    for (row, row_rect) in rows_rects.into_iter().enumerate() {
        let col_rects: [Rect; BOARD_SIZE * 2 - 1] = cols_layout.areas(row_rect);

        // Filter out the spacing rectangles and render the tile rectangles
        for (col, col_rect) in
            col_rects.into_iter().enumerate().filter_map(|(idx, rect)| {
                // Include the tile rectangles (skip the spacing)
                if idx % 2 == 0 {
                    Some((idx / 2, rect))
                } else {
                    None
                }
            })
        {
            // Get the cell result for the current coordinates
            let result = &board[row][col];

            // Determine the style based on whether the cell was merged in the last move
            let style = if result.merged {
                Style::new().green()
            } else {
                Style::new().dim()
            };

            // Render the cell border with the appropriate style
            frame
                .render_widget(Block::bordered().border_style(style), col_rect);

            // Render the cell value centered within the cell rectangle
            let cell = col_rect.inner(Margin::new(0, CELL_PADDING_Y));
            let cell_value =
                result.value.map_or("".to_string(), |v| v.to_string());
            frame.render_widget(Paragraph::new(cell_value).centered(), cell);
        }
    }
}

fn render_score(score: u32, area: Rect, frame: &mut Frame) {
    const MIN_SCORE_WIDTH: usize = 6;
    let score_text = format!("Score: {0:>1$} ", score, MIN_SCORE_WIDTH);
    frame.render_widget(Paragraph::new(score_text).right_aligned(), area);
}

fn render(outcome: &ActionOutcome, frame: &mut Frame) {
    let (main_width, main_height) = calculate_game_dimensions();

    // Center the game area within the terminal frame
    let game_area = frame.area().centered(
        Constraint::Length(main_width),
        Constraint::Length(main_height),
    );

    // Split the game area into the tiles area and the score area
    let game_layout = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(SCORE_HEIGHT),
    ]);
    let [tiles_area, scores_area] = game_layout.areas(game_area);

    render_board(&outcome, tiles_area, frame);
    render_tiles(&outcome.board, tiles_area, frame);
    render_score(outcome.score, scores_area, frame);
}

fn input_loop(tx: Sender<Event>) -> Result<()> {
    loop {
        let event = read()?;

        let key = match event.as_key_press_event() {
            Some(key_event) => key_event,
            None => continue,
        };

        match key.code {
            KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('k') => {
                tx.blocking_send(Event::MoveUp)?
            }
            KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('j') => {
                tx.blocking_send(Event::MoveDown)?
            }
            KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('h') => {
                tx.blocking_send(Event::MoveLeft)?
            }
            KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('l') => {
                tx.blocking_send(Event::MoveRight)?
            }
            KeyCode::Char('r') => tx.blocking_send(Event::Restart)?,
            KeyCode::Char('q') => {
                tx.blocking_send(Event::Quit)?;
                break;
            }
            _ => continue,
        };
    }
    Ok(())
}

async fn event_loop(
    mut rx: Receiver<Event>,
    mut terminal: DefaultTerminal,
) -> Result<()> {
    let mut game = Game::new();
    terminal.draw(|frame| render(&game.outcome(), frame))?;

    while let Some(e) = rx.recv().await {
        if e == Event::Quit {
            break;
        }

        if e == Event::Restart {
            let outcome = game.restart();
            terminal.draw(|frame| render(&outcome, frame))?;
            continue;
        }

        let outcome = match e {
            Event::MoveUp => game.apply_move(GameAction::Up),
            Event::MoveDown => game.apply_move(GameAction::Down),
            Event::MoveLeft => game.apply_move(GameAction::Left),
            Event::MoveRight => game.apply_move(GameAction::Right),
            _ => panic!("Should never happen!"),
        };

        if outcome.changed || outcome.game_over {
            terminal.draw(|frame| render(&outcome, frame))?;
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let terminal = ratatui::init();
    let (tx, rx): (Sender<Event>, Receiver<Event>) = channel(BUFSIZE);
    spawn_blocking(move || input_loop(tx));
    event_loop(rx, terminal).await?;
    ratatui::restore();
    Ok(())
}
