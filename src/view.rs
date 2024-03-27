mod home;
mod login;

use std::fmt;

use ratatui::terminal::Frame;
use tracing::{event, Level};

pub use home::Home;
pub use login::Login;

#[derive(Clone)]
pub enum View {
    Login(Login),
    Home(Home),
}

impl View {
    pub fn render(&self, frame: &mut Frame) {
        match self {
            View::Login(login) => {
                frame.render_widget(login, frame.size());
            }
            View::Home(home) => {
                frame.render_widget(home, frame.size());
            }
        }
    }

    pub fn update<V: Into<View>>(&mut self, new: V) {
        *self = new.into();
        event!(Level::DEBUG, "set view: {self:?}");
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
