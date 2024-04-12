mod home;
mod loading;
mod login;

use std::fmt;

use ratatui::{prelude::*, widgets::*};
use tracing::{event, Level};

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
            View::Home(home) => home.render(area, buf),
            View::Loading(loading) => loading.render(area, buf),
            View::Login(login) => login.render(area, buf),
        }
    }
}

impl From<Home> for View {
    fn from(value: Home) -> Self {
        Self::Home(value)
    }
}

impl From<Loading> for View {
    fn from(value: Loading) -> Self {
        Self::Loading(value)
    }
}

impl From<Login> for View {
    fn from(value: Login) -> Self {
        Self::Login(value)
    }
}

impl fmt::Debug for View {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::Home(_) => "Home",
            Self::Loading(_) => "Loading",
            Self::Login(_) => "Login",
        })
    }
}
