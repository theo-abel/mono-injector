use serde::Serialize;

use crate::error::{Error, Result};

#[derive(Debug, Clone, Copy)]
pub(crate) struct Context {
    json: bool,
}

impl Context {
    #[must_use]
    pub(crate) const fn new(json: bool) -> Self {
        Self { json }
    }

    #[must_use]
    pub(crate) const fn json(self) -> bool {
        self.json
    }

    pub(crate) fn print_json(self, value: &impl Serialize) -> Result<()> {
        debug_assert!(self.json);
        let content = serde_json::to_string_pretty(value).map_err(Error::OutputSerialize)?;
        println!("{content}");
        Ok(())
    }
}
