use anyhow::Result;
use crossterm::event::{Event as TuiEvent, KeyCode};
use tokio::sync::mpsc;
use tracing::{event, Level};

use crate::{
    atp, tui,
    view::{self, View},
};

pub enum Response {
    Atp(atp::Response),
    Tui(tui::Response),
}

pub async fn start(
    mut res_rx: mpsc::UnboundedReceiver<Response>,
    atp_tx: mpsc::UnboundedSender<atp::Request>,
    tui_tx: mpsc::UnboundedSender<tui::Request>,
) -> Result<()> {
    let mut view = View::Login(view::Login::new());
    tui_tx.send(tui::Request::Render(view.clone()))?;

    atp_tx.send(atp::Request::GetSession)?;
    let session = loop {
        if let Response::Atp(atp::Response::Session(session)) = res_rx.recv().await.unwrap() {
            break session;
        }
    };

    if session.is_some() {
        view.update(view::Home::new());
    }

    loop {
        tui_tx.send(tui::Request::Render(view.clone()))?;
        tui_tx.send(tui::Request::GetEvent)?;
        let Some(res) = res_rx.recv().await else {
            break;
        };

        if let View::Login(ref mut login) = view {
            if let Response::Tui(tui::Response::Event(tui_event)) = res {
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
                        atp_tx.send(atp::Request::Login {
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

            if let Response::Atp(atp::Response::Login { err }) = res {
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
            if let Response::Tui(tui::Response::Event(tui_event)) = res {
                if tui_event == TuiEvent::Key(KeyCode::Esc.into()) {
                    break;
                }
            }
        }
    }

    Ok(())
}
