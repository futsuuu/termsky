use anyhow::Result;
use atrium_api::app::bsky;
use crossterm::event::{KeyCode, KeyEventKind};
use tracing::{event, Level};
use tui_textarea::Input;

use crate::{
    atp, tui,
    view::{self, View},
};

pub enum Event {
    Atp(atp::Response),
    Tui(tui::Event),
}

pub async fn start() -> Result<()> {
    let tui = tui::Tui::new()?;
    let atp = atp::Atp::new()?;
    let mut view = View::Login(view::Login::new());

    atp.send(atp::Request::GetSession)?;

    event!(Level::INFO, "start main loop");
    loop {
        tui.render(view.clone()).ok();
        let ev = tokio::select! {
            Some(event) = tui.event() => Event::Tui(event),
            Some(res) = atp.recv() => Event::Atp(res),
            else => break,
        };

        if let View::Login(ref mut login) = view {
            if let Event::Tui(tui_event) = ev {
                if let tui::Event::Key(key_event) = tui_event {
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
                        atp.send(atp::Request::Login {
                            ident: login.ident(),
                            passwd: login.passwd(),
                        })?;
                        login.block_input();
                        continue;
                    }
                }

                if let Some(ref mut textarea) = login.textarea() {
                    textarea.input(match tui_event {
                        tui::Event::Key(key) => Input::from(key),
                        tui::Event::Mouse(mouse) => Input::from(mouse),
                        _ => Input::default(),
                    });
                }

                continue;
            }

            if let Event::Atp(atp::Response::Session(ref session)) = ev {
                if session.is_none() {
                    login.unblock_input();
                } else {
                    view.update(view::Home::new());
                }

                continue;
            }

            if let Event::Atp(atp::Response::Login(result)) = ev {
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
                atp.send(atp::Request::GetTimeline(
                    bsky::feed::get_timeline::Parameters {
                        algorithm: None,
                        cursor: None,
                        limit: 20.try_into().ok(),
                    },
                ))?;
                home.wait_response();
            }

            if let Event::Tui(tui::Event::Key(key_event)) = ev {
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

            if let Event::Atp(atp::Response::Timeline(Ok(timeline))) = ev {
                for post in timeline.feed.into_iter().rev() {
                    home.add_received_post(post, true);
                }
            }
        }
    }

    event!(Level::INFO, "stop main loop");
    Ok(())
}
