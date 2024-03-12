use anyhow::Result;
use crossterm::event::{Event as TuiEvent, KeyCode};
use tracing::{event, Level};

use crate::{
    command::{Command, CommandTx},
    event::{recv, Event, EventRx},
    view::{self, View},
};

pub async fn start(mut event_rx: EventRx, command_tx: CommandTx) -> Result<()> {
    let mut view = View::Login(view::Login::new());
    command_tx.send(Command::Render(view.clone()))?;

    command_tx.send(Command::GetSession)?;
    let session = recv!(event_rx, Event::Session(_));
    let Some(Event::Session(session)) = session else {
        return Ok(());
    };

    if session.is_some() {
        view.update(view::Home::new());
    }

    loop {
        command_tx.send(Command::Render(view.clone()))?;
        let Some(event) = event_rx.recv().await else {
            break;
        };

        if let View::Login(ref mut login) = view {
            if let Event::Tui(tui_event) = event {
                if let TuiEvent::Key(key_event) = tui_event {
                    if key_event == KeyCode::Esc.into() {
                        if login.has_focus() {
                            login.lose_focus();
                        } else {
                            break;
                        }
                        continue;
                    } else if key_event == KeyCode::Tab.into() {
                        login.switch_focus();
                        continue;
                    } else if key_event == KeyCode::Enter.into() {
                        command_tx.send(Command::Login {
                            ident: login.get_ident(),
                            passwd: login.get_passwd(),
                        })?;
                        continue;
                    }
                }

                if let Some(ref mut textarea) = login.textarea() {
                    textarea.input(tui_textarea::Input::from(tui_event));
                }

                continue;
            }

            if let Event::Login { err } = event {
                if let Some(msg) = err {
                    event!(Level::WARN, "show error message");
                    login.set_error(msg);
                } else {
                    view.update(view::Home::new());
                }

                continue;
            }

            continue;
        }

        if let View::Home(ref mut _home) = view {
            if let Event::Tui(tui_event) = event {
                if tui_event == TuiEvent::Key(KeyCode::Esc.into()) {
                    break;
                }
            }
        }
    }

    Ok(())
}
