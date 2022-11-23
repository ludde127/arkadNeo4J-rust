use core::fmt::{Formatter, Display};


#[derive(PartialEq)]
pub struct  Action {
    pub(crate) company: String,
    pub(crate) collected: bool
}

impl Action {
    pub fn new(company: String, collected: bool) -> Action {
        Action{company, collected}
    }
}

impl Clone for Action {
    fn clone(&self) -> Self {
        Self { company: self.company.clone(), collected: self.collected.clone() }
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.collected {
            write!(f, "({}:{})", self.company, "Collected")?;
        } else {
            write!(f, "({})", self.company)?;
        }
        Ok(())
    }
}