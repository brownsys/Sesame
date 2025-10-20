use crate::fold::fold;
use crate::SesameType;

use std::any::Any;

// Context Data must satisfy these requirements.
pub trait ContextData: SesameType + Send + 'static {}
impl<D: SesameType + Send + 'static> ContextData for D {}

// Context is generic over some developer defined data.
#[derive(Debug, Clone)]
pub struct Context<D: ContextData> {
    route: String,
    data: Option<D>,
}
impl<D: ContextData> Context<D> {
    pub fn route(&self) -> &str {
        &self.route
    }

    // Context cannot be manufactured.
    pub fn new(route: String, data: D) -> Self {
        Self {
            route,
            data: Some(data),
        }
    }

    // Can manufacture an empty context for ease of use when dealing with Alohomora APIs without boxes.
    pub fn empty() -> Self {
        Self {
            route: String::from(""),
            data: None,
        }
    }

    // Only for testing.
    pub fn data(&self) -> Option<&D> {
        self.data.as_ref()
    }
}

// Alohomora turns Context into UnprotectedContext before invoking Policy Check.
pub struct UnprotectedContext {
    pub route: String,
    pub data: Box<dyn Any>,
}
impl UnprotectedContext {
    // TODO(babman)-exts: REMEMBER TO MAKE THIS PRIVATE AFTER
    pub fn from<D: ContextData>(context: Context<D>) -> Self {
        Self {
            route: context.route,
            data: match context.data {
                None => Box::new(Option::<()>::None),
                Some(data) => Box::new(fold(data).unwrap().consume().0),
            },
        }
    }
    pub fn downcast_ref<D: 'static>(&self) -> Option<&D> {
        self.data.downcast_ref()
    }
}
