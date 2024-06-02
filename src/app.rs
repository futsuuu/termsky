use anyhow::Result;

use crate::{prelude::*, widgets::ViewID};

pub struct App {
    running: bool,
    pub atp: Atp,
    view_id: ViewID,
    new_view_id: Option<ViewID>,
}

impl App {
    fn new() -> Result<Self> {
        Ok(Self {
            running: true,
            atp: Atp::new()?,
            view_id: ViewID::default(),
            new_view_id: None,
        })
    }

    pub fn view_id(&self) -> &ViewID {
        &self.view_id
    }
    pub fn set_view_id(&mut self, state: ViewID) {
        self.new_view_id = Some(state);
    }

    pub fn exit(&mut self) {
        self.running = false;
    }
}

pub trait Handler {
    #![allow(unused_variables)]
    fn tui_event(&mut self, app: &mut App, ev: TuiEvent) {}
}

pub async fn run() -> Result<()> {
    let mut app = App::new()?;
    let mut tui = Tui::new()?;

    let mut view = View::default();

    tracing::trace!("start main loop");
    while let Some(event) = tui.event().await {
        view.tui_event(&mut app, event);
        if !app.running {
            break;
        }
        if let Some(s) = app.new_view_id.take() {
            tracing::info!("set view ID: {s:?}");
            app.view_id = s;
        }
        tui.render(&view)?;
    }

    tracing::trace!("stop main loop");
    Ok(())
}
