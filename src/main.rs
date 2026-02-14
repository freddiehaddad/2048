#![allow(unused)]

mod board;
mod event;
mod game;

use anyhow::Result;
use ratatui::crossterm::event::{KeyCode, read};
use tokio::{
    sync::mpsc::{Receiver, Sender, channel},
    task::{spawn, spawn_blocking},
};

use crate::event::Event;
use crate::game::{ActionOutcome, Game, GameAction};

const BUFSIZE: usize = 1;

fn render(move_result: ActionOutcome) {
    println!("RENDER: {move_result:#?}");
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

async fn run_event_loop(mut rx: Receiver<Event>) -> Result<()> {
    let mut game = Game::new();
    render(ActionOutcome::default());

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

        render(move_result);
    }
    Ok(())
}

async fn run(mut terminal: ratatui::DefaultTerminal) -> Result<()> {
    let (tx, mut rx): (Sender<Event>, Receiver<Event>) = channel(BUFSIZE);

    let handle = spawn_blocking(move || input_loop(tx));
    run_event_loop(rx).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let terminal = ratatui::init();
    run(terminal).await?;
    ratatui::restore();
    Ok(())
}
