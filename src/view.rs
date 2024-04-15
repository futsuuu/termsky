mod home;
mod loading;
mod login;

use std::fmt;

use ratatui::{prelude::*, widgets::*};
use tracing::{event, Level};

use crate::prelude::*;

pub use home::Home;
pub use loading::Loading;
pub use login::Login;

pub enum View {
    Home(Home),
    Loading(Loading),
    Login(Login),
}

impl View {
    pub fn update<V: Into<View>>(&mut self, new: V) {
        *self = new.into();
        event!(Level::DEBUG, "set view: {self:?}");
    }
}

impl WidgetRef for View {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        match self {
            View::Home(v) => v.render_ref(area, buf),
            View::Loading(v) => v.render_ref(area, buf),
            View::Login(v) => v.render_ref(area, buf),
        }
    }
}

impl AppHandler for View {
    fn tui_event(&mut self, app: &mut App, ev: TuiEvent) {
        match self {
            View::Home(v) => v.tui_event(app, ev),
            View::Loading(v) => v.tui_event(app, ev),
            View::Login(v) => v.tui_event(app, ev),
        }
    }

    fn atp_response(&mut self, app: &mut App, res: AtpResponse) {
        match self {
            View::Home(v) => v.atp_response(app, res),
            View::Loading(v) => v.atp_response(app, res),
            View::Login(v) => v.atp_response(app, res),
        }
    }
}

macro_rules! impl_from {
    ($s:ident) => {
        impl From<$s> for View {
            fn from(value: $s) -> Self {
                Self::$s(value)
            }
        }
    };
}

impl_from!(Home);
impl_from!(Loading);
impl_from!(Login);

impl fmt::Debug for View {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::Home(_) => "Home",
            Self::Loading(_) => "Loading",
            Self::Login(_) => "Login",
        })
    }
}
