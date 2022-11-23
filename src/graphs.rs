use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use crate::Path;

pub struct Relationship {
    pub paths: Vec<Path>
}

impl Display for Relationship {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Relations")?;
        for p in &self.paths {
            write!(f, " {} |", p)?;
        }
        Ok(())
    }
}

pub type Relationships = HashMap<String, Relationship>;