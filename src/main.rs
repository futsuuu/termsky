use anyhow::Result;
use atrium_api::app::bsky;
use crossterm::event::{KeyCode, KeyEventKind};
use tracing::{event, Level};

use termsky::{
    atp::{Atp, Request as AtpRequest, Response as AtpResponse},
    tui::{self, Event as TuiEvent, Tui},
    utils,
    view::{self, View},
};

fn main() -> Result<()> {
    utils::init()?;
    tui::enter()?;
    main_async()?;
    tui::exit()?;
    Ok(())
}

enum Event {
    Atp(AtpResponse),
    Tui(TuiEvent),
}

#[tokio::main]
async fn main_async() -> Result<()> {
    let atp = Atp::new()?;
    let mut tui = Tui::new()?;
    let mut view = View::Login(view::Login::new());

    atp.send(AtpRequest::GetSession)?;

    event!(Level::INFO, "start main loop");
    loop {
        tui.render(&view)?;
        let ev = tokio::select! {
            Some(event) = tui.event() => Event::Tui(event),
            Some(res) = atp.recv() => Event::Atp(res),
            else => break,
        };

        if let View::Login(ref mut login) = view {
            if let Event::Tui(tui_event) = ev {
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
                        atp.send(AtpRequest::Login {
                            ident: login.ident(),
                            passwd: login.passwd(),
                        })?;
                        login.block_input();
                        continue;
                    }
                }

                if let Some(ref mut textarea) = login.textarea() {
                    textarea.input(tui_event);
                }

                continue;
            }

            if let Event::Atp(AtpResponse::Session(ref session)) = ev {
                if session.is_none() {
                    login.unblock_input();
                } else {
                    view.update(view::Home::new());
                }

                continue;
            }

            if let Event::Atp(AtpResponse::Login(result)) = ev {
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
                atp.send(AtpRequest::GetTimeline(
                    bsky::feed::get_timeline::Parameters {
                        algorithm: None,
                        cursor: None,
                        limit: 20.try_into().ok(),
                    },
                ))?;
                home.wait_response();
            }

            if let Event::Tui(TuiEvent::Key(key_event)) = ev {
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

            if let Event::Atp(AtpResponse::Timeline(Ok(timeline))) = ev {
                for post in timeline.feed.into_iter().rev() {
                    home.add_received_post(post, true);
                }
            }
        }
    }

    event!(Level::INFO, "stop main loop");
    Ok(())
}
