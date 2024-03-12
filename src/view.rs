mod home;
mod login;

use ratatui::terminal::Frame;
use tracing::{event, Level};

pub use home::Home;
pub use login::Login;

#[derive(Clone, Debug)]
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
        event!(Level::INFO, "set view: {self:?}");
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
