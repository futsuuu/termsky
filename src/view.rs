mod home;
mod login;

use std::fmt;

use ratatui::{prelude::*, widgets::*};
use tracing::{event, Level};

pub use home::Home;
pub use login::Login;

pub enum View {
    Login(Login),
    Home(Home),
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
            View::Login(login) => login.render(area, buf),
            View::Home(home) => home.render(area, buf),
        }
    }
}

impl From<Login> for View {
    fn from(value: Login) -> Self {
        Self::Login(value)
    }
}

impl From<Home> for View {
    fn from(value: Home) -> Self {
        Self::Home(value)
    }
}

impl fmt::Debug for View {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::Login(_) => "Login",
            Self::Home(_) => "Home",
        })
    }
}
