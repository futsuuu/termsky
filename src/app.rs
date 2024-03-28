use anyhow::Result;
use atrium_api::app::bsky;
use crossterm::event::{Event as TuiEvent, KeyCode, KeyEventKind};
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

    atp_tx.send(atp::Request::GetSession)?;

    event!(Level::INFO, "start main loop");
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
                    } else if key_event == KeyCode::Enter.into() && login.textarea().is_some() {
                        atp_tx.send(atp::Request::Login {
                            ident: login.ident(),
                            passwd: login.passwd(),
                        })?;
                        login.block_input();
                        continue;
                    }
                }

                if let Some(ref mut textarea) = login.textarea() {
                    textarea.input(tui_textarea::Input::from(tui_event));
                }

                continue;
            }

            if let Response::Atp(atp::Response::Session(ref session)) = res {
                if session.is_none() {
                    login.unblock_input();
                } else {
                    view.update(view::Home::new());
                }

                continue;
            }

            if let Response::Atp(atp::Response::Login(result)) = res {
                if let Err(_e) = result {
                    login.unblock_input();
                } else {
                    view.update(view::Home::new());
                }

                continue;
            }

            continue;
        }

        if let View::Home(ref mut home) = view {
            if home.new_posts_required() {
                atp_tx.send(atp::Request::GetTimeline(
                    bsky::feed::get_timeline::Parameters {
                        algorithm: None,
                        cursor: None,
                        limit: 20.try_into().ok(),
                    },
                ))?;
                home.wait_response();
            }

            if let Response::Tui(tui::Response::Event(TuiEvent::Key(key_event))) = res {
                if key_event.code == KeyCode::Esc {
                    break;
                } else if key_event.kind == KeyEventKind::Release {
                    continue;
                }
                if key_event.code == KeyCode::Char('k') {
                    home.scroll_up();
                }
                if key_event.code == KeyCode::Char('j') {
                    home.scroll_down();
                }
                continue;
            }

            if let Response::Atp(atp::Response::Timeline(Ok(timeline))) = res {
                for post in timeline.feed.into_iter().rev() {
                    home.add_received_post(post, true);
                }
            }
        }
    }

    event!(Level::INFO, "stop main loop");
    Ok(())
}
