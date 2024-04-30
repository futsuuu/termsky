use anyhow::Result;
use tracing::{event, Level};

use crate::{prelude::*, widgets::pages};

pub struct App {
    running: bool,
    new_view: Option<View>,
    atp: Atp,
    tui: Tui,
}

impl App {
    fn new() -> Result<Self> {
        Ok(Self {
            running: true,
            new_view: None,
            atp: Atp::new()?,
            tui: Tui::new()?,
        })
    }

    pub fn update_view(&mut self, view: impl Into<View>) {
        self.new_view = Some(view.into());
    }

    pub fn exit(&mut self) {
        self.running = false;
    }

    pub fn send(&mut self, req: AtpRequest) {
        if self.atp.send(req).is_err() {
            self.exit();
        }
    }
}

pub trait Handler {
    #![allow(unused_variables)]
    fn tui_event(&mut self, app: &mut App, ev: TuiEvent) {}
    fn atp_response(&mut self, app: &mut App, res: AtpResponse) {}
}

#[tokio::main]
pub async fn run() -> Result<()> {
    let mut app = App::new()?;
    let mut view = View::Loading(pages::Loading::new());

    app.send(AtpRequest::GetSession);

    event!(Level::INFO, "start main loop");
    while app.running {
        app.tui.render(&view)?;
        tokio::select! {
            Some(event) = app.tui.event() => {
                view.tui_event(&mut app, event);
            }
            Some(res) = app.atp.recv() => {
                view.atp_response(&mut app, res);
            }
            else => break,
        };
        if let Some(new_view) = app.new_view.take() {
            view.update(new_view);
        }
    }

    event!(Level::INFO, "stop main loop");
    Ok(())
}
