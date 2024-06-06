use atrium_api::app::bsky;

#[derive(Debug)]
pub struct Account {
    pub name: String,
    pub opt_name: Option<String>,
}

impl Account {
    fn new(display_name: Option<String>, handle: &str) -> Self {
        let handle = format!("@{handle}");
        match display_name.filter(|s| !s.is_empty()) {
            Some(display_name) => Self {
                name: display_name,
                opt_name: Some(handle),
            },
            None => Self {
                name: handle,
                opt_name: None,
            },
        }
    }
}

impl From<bsky::actor::defs::ProfileView> for Account {
    fn from(value: bsky::actor::defs::ProfileView) -> Self {
        Self::new(value.display_name, value.handle.as_str())
    }
}

impl From<&bsky::actor::defs::ProfileView> for Account {
    fn from(value: &bsky::actor::defs::ProfileView) -> Self {
        Self::new(value.display_name.clone(), value.handle.as_str())
    }
}

impl From<bsky::actor::defs::ProfileViewBasic> for Account {
    fn from(value: bsky::actor::defs::ProfileViewBasic) -> Self {
        Self::new(value.display_name, value.handle.as_str())
    }
}

impl From<&bsky::actor::defs::ProfileViewBasic> for Account {
    fn from(value: &bsky::actor::defs::ProfileViewBasic) -> Self {
        Self::new(value.display_name.clone(), value.handle.as_str())
    }
}
