#![allow(unused)]

mod board;
mod event;
mod game;

use anyhow::Result;
use ratatui::crossterm::event::{KeyCode, read};
use ratatui::layout::{Margin, Rect};
use ratatui::{DefaultTerminal, Frame};
use tokio::{
    sync::mpsc::{Receiver, Sender, channel},
    task::spawn_blocking,
};

use crate::event::Event;
use crate::game::{ActionOutcome, Game, GameAction};

const BUFSIZE: usize = 1;

const CELL_WIDTH: u16 = 11;
const CELL_HEIGHT: u16 = 5;
const SCORE_HEIGHT: u16 = 1;
const CELL_PADDING_X: u16 = 1;
const BORDER_WIDTH: u16 = 1;

fn render(move_result: ActionOutcome, frame: &mut Frame) {
    use crate::board::{BOARD_SIZE, Board};
    use crate::game::TITLE;
    use Constraint::Length;
    use ratatui::layout::{Constraint, Layout};
    use ratatui::widgets::{Block, BorderType, Paragraph};

    const MAIN_WIDTH: u16 = BOARD_SIZE as u16 * (CELL_WIDTH + CELL_PADDING_X)
        + CELL_PADDING_X
        + (BORDER_WIDTH * 2);
    const MAIN_HEIGHT: u16 =
        BOARD_SIZE as u16 * CELL_HEIGHT + SCORE_HEIGHT + (BORDER_WIDTH * 2);

    // Center the game area within the terminal frame
    let game_rect = frame
        .area()
        .centered(Length(MAIN_WIDTH), Length(MAIN_HEIGHT));

    // Split the game area into the tiles area and the score area
    let game_layout =
        Layout::vertical([Constraint::Fill(1), Length(SCORE_HEIGHT)]);
    let [tiles_rect, scores_rect] = game_layout.areas(game_rect);

    // Render the border and title around the tiles area
    frame.render_widget(
        Block::bordered()
            .border_type(BorderType::Thick)
            .title(TITLE),
        tiles_rect,
    );

    // Split the tiles area into rows
    let rows_layout = Layout::vertical([Constraint::Fill(1); BOARD_SIZE]);
    let rows_rects: [Rect; BOARD_SIZE] = rows_layout.areas(
        tiles_rect
            .inner(Margin::new(BORDER_WIDTH + CELL_PADDING_X, BORDER_WIDTH)),
    );

    // Each row is split into columns, with spacing between them
    let cols_layout = Layout::horizontal((0..BOARD_SIZE).flat_map(|i| {
        if i < BOARD_SIZE - 1 {
            [Constraint::Fill(1), Constraint::Length(1)].iter()
        } else {
            [Constraint::Fill(1)].iter()
        }
    }));

    // Render each cell with its coordinates for demonstration purposes
    for (row, row_rect) in rows_rects.into_iter().enumerate() {
        let col_rects: [Rect; BOARD_SIZE * 2 - 1] = cols_layout.areas(row_rect);

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
            frame.render_widget(Block::bordered(), col_rect);
            let cell = col_rect.inner(Margin::new(2, 2));
            let text = format!("({},{})", row, col);
            frame.render_widget(Paragraph::new(text).centered(), cell);
        }
    }

    frame.render_widget(Paragraph::new("Score").right_aligned(), scores_rect);
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
                println!("SEND: {:?}", Event::MoveUp);
                tx.blocking_send(Event::MoveUp)?;
            }
            KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('j') => {
                println!("SEND: {:?}", Event::MoveDown);
                tx.blocking_send(Event::MoveDown)?;
            }
            KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('h') => {
                println!("SEND: {:?}", Event::MoveLeft);
                tx.blocking_send(Event::MoveLeft)?;
            }
            KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('l') => {
                println!("SEND: {:?}", Event::MoveRight);
                tx.blocking_send(Event::MoveRight)?;
            }
            KeyCode::Char('r') => {
                println!("SEND: {:?}", Event::Restart);
                tx.blocking_send(Event::Restart)?;
            }
            KeyCode::Char('q') => {
                println!("SEND: {:?}", Event::Quit);
                tx.blocking_send(Event::Quit)?;
                break;
            }
            _ => continue,
        };
    }
    Ok(())
}

async fn run_event_loop(
    mut rx: Receiver<Event>,
    mut terminal: DefaultTerminal,
) -> Result<()> {
    let mut game = Game::new();
    terminal.draw(|frame| render(ActionOutcome::default(), frame))?;

    while let Some(e) = rx.recv().await {
        let move_result = match e {
            Event::MoveUp => game.apply_move(GameAction::Up),
            Event::MoveDown => game.apply_move(GameAction::Down),
            Event::MoveLeft => game.apply_move(GameAction::Left),
            Event::MoveRight => game.apply_move(GameAction::Right),
            Event::Restart => game.restart(),
            Event::Quit => {
                println!("EVENT: {:?}", Event::Quit);
                break;
            }
        };

        terminal.draw(|frame| render(move_result, frame))?;
    }
    Ok(())
}

async fn run(terminal: DefaultTerminal) -> Result<()> {
    let (tx, rx): (Sender<Event>, Receiver<Event>) = channel(BUFSIZE);

    let handle = spawn_blocking(move || input_loop(tx));
    run_event_loop(rx, terminal).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let terminal = ratatui::init();
    run(terminal).await?;
    ratatui::restore();
    Ok(())
}
