// The policy carried by every protected leaf of the chat payload.
//
// It stores the name of the template it is willing to be rendered into, and
// checks that against the `Reason::TemplateRender(name)` that
// `PConTemplate::render` threads down to every leaf. The check is therefore a
// real per-leaf string comparison exercising live plumbing, rather than a stub
// the optimizer can fold away.

use sesame::context::UnprotectedContext;
use sesame::policy::{Reason, SimplePolicy};

#[derive(Clone)]
pub struct TemplatePolicy {
    template: String,
}

impl TemplatePolicy {
    pub fn new(template: &str) -> Self {
        TemplatePolicy {
            template: String::from(template),
        }
    }
}

impl SimplePolicy for TemplatePolicy {
    fn simple_name(&self) -> String {
        String::from("TemplatePolicy")
    }

    fn simple_check(&self, _context: &UnprotectedContext, reason: Reason<'_>) -> bool {
        match reason {
            Reason::TemplateRender(name) => name == self.template.as_str(),
            _ => false,
        }
    }

    // Two leaves only agree if they name the same template. Joining leaves that
    // disagree yields a policy that permits no template at all.
    fn simple_join_direct(&mut self, other: &mut Self) {
        if self.template != other.template {
            self.template = String::new();
        }
    }
}
