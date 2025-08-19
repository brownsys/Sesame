        use rocket::serde::Serialize;
use crate::context::UnprotectedContext;
use crate::policy::{FrontendPolicy, Policy, Reason, SchemaPolicy, Specializable, SpecializationEnum, Specialize};
use crate::Unjoinable;

#[derive(Clone, Serialize, PartialEq, Eq, Debug)]
pub struct PolicyOr<P1: Policy, P2: Policy> {
    p1: P1,
    p2: P2,
}
impl<P1: Policy, P2: Policy> PolicyOr<P1, P2> {
    pub fn new(p1: P1, p2: P2) -> Self {
        Self { p1, p2 }
    }
    pub fn policy1(&self) -> &P1 {
        &self.p1
    }
    pub fn policy2(&self) -> &P2 {
        &self.p2
    }
    pub fn policies(&self) -> (&P1, &P2) {
        (&self.p1, &self.p2)
    }
}

impl<P1: Policy, P2: Policy> Policy for PolicyOr<P1, P2> {
    fn name(&self) -> String {
        format!("PolicyOr({} OR {})", self.p1.name(), self.p2.name())
    }
    fn check(&self, context: &UnprotectedContext, reason: Reason) -> bool {
        self.p1.check(context, reason.clone()) || self.p2.check(context, reason)
    }
    Unjoinable!(!Any);
}

impl<P1: Specializable, P2: Specializable> Specializable for PolicyOr<P1, P2> {
    fn to_specialization_enum(self) -> SpecializationEnum {
        SpecializationEnum::PolicyOr(
            Box::new(self.p1.to_specialization_enum()),
            Box::new(self.p2.to_specialization_enum()),
        )
    }
    fn to_specialization_enum_box(self: Box<Self>) -> SpecializationEnum {
        self.to_specialization_enum()
    }
}
impl<P1: Specialize, P2: Specialize> Specialize for PolicyOr<P1, P2> {
    fn specialize_or(b1: Box<SpecializationEnum>, b2: Box<SpecializationEnum>) -> Result<Self, (Box<SpecializationEnum>, Box<SpecializationEnum>)> {
        let r1 = b1.specialize::<P1>();
        let r2 = b2.specialize::<P2>();
        match (r1, r2) {
            (Ok(p1), Ok(p2)) => Ok(PolicyOr { p1, p2 }),
            (Err(e1), Err(e2)) => Err((Box::new(e1), Box::new(e2))),
            (Ok(p1), Err(e2)) =>
                Err((Box::new(p1.to_specialization_enum().normalize()), Box::new(e2))),
            (Err(e1), Ok(p2)) =>
                Err((Box::new(e1), Box::new(p2.to_specialization_enum().normalize()))),
        }
    }
}

impl<P1: SchemaPolicy, P2: SchemaPolicy> SchemaPolicy for PolicyOr<P1, P2> {
    fn from_row(table_name: &str, row: &Vec<mysql::Value>) -> Self {
        Self {
            p1: P1::from_row(table_name, row),
            p2: P2::from_row(table_name, row),
        }
    }
}
impl<P1: FrontendPolicy, P2: FrontendPolicy> FrontendPolicy for PolicyOr<P1, P2> {
    fn from_request(request: &rocket::Request<'_>) -> Self {
        Self {
            p1: P1::from_request(request),
            p2: P2::from_request(request),
        }
    }
    fn from_cookie<'a, 'r>(
        name: &str,
        cookie: &'a rocket::http::Cookie<'static>,
        request: &'a rocket::Request<'r>,
    ) -> Self {
        Self {
            p1: P1::from_cookie(name, cookie, request),
            p2: P2::from_cookie(name, cookie, request),
        }
    }
}
