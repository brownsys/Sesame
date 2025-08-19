use serde::Serialize;
use crate::context::UnprotectedContext;
use crate::policy::{FrontendPolicy, Policy, Reason, ReflexiveJoin, SchemaPolicy, Specializable, SpecializationEnum, Specialize};

#[derive(Clone, Serialize, PartialEq, Eq, Debug)]
pub struct PolicyAnd<P1: Policy, P2: Policy> {
    p1: P1,
    p2: P2,
}
impl<P1: Policy, P2: Policy> PolicyAnd<P1, P2> {
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

impl<P1: Policy, P2: Policy> Policy for PolicyAnd<P1, P2> {
    fn name(&self) -> String {
        format!("PolicyAnd({} AND {})", self.p1.name(), self.p2.name())
    }
    fn check(&self, context: &UnprotectedContext, reason: Reason) -> bool {
        self.p1.check(context, reason.clone()) && self.p2.check(context, reason)
    }
    /*
    fn policy_type_enum(&mut self) -> PolicyTypeEnum<'_> {
        PolicyTypeEnum::PolicyAnd(
            Box::new(self.p1.policy_type_enum()),
            Box::new(self.p2.policy_type_enum()),
        )
    }
    fn can_join_with(&mut self, p: &PolicyTypeEnum<'_>) -> bool {
        match p {
            PolicyTypeEnum::PolicyAnd(left, right) => {
                (self.p1.can_join_with(left) && self.p2.can_join_with(right))
                    || self.p1.can_join_with(p)
                    || self.p2.can_join_with(p)
            },
            _ => {
                self.p1.can_join_with(p) || self.p2.can_join_with(p)
            }
        }
    }
    fn join(&mut self, p: PolicyTypeEnum<'_>) -> bool {
        // Try to join left with left and right with right.
        let p = match p {
            PolicyTypeEnum::PolicyAnd(left, right) => {
                if self.p1.can_join_with(&left) && self.p2.can_join_with(&right) {
                    if !self.p1.join(*left) || !self.p2.join(*right) {
                        panic!("join returned false even though can join returned true");
                    }
                    return true;
                } else {
                    PolicyTypeEnum::PolicyAnd(left, right)
                }
            },
            p => p,
        };
        // Try to join left or then right.
        if self.p1.can_join_with(&p) {
            if !self.p1.join(p) {
                panic!("join returned false even though can join returned true");
            }
            true
        } else {
            self.p2.join(p)
        }
    }
     */
}

// Can specialize an And if both its clauses are also specializable.
impl<P1: Policy + Specializable, P2: Policy + Specializable> Specializable for PolicyAnd<P1, P2> {
    fn to_specialization_enum(self) -> SpecializationEnum {
        SpecializationEnum::PolicyAnd(
            Box::new(self.p1.to_specialization_enum()),
            Box::new(self.p2.to_specialization_enum()),
        )
    }
    fn to_specialization_enum_box(self: Box<Self>) -> SpecializationEnum {
        self.to_specialization_enum()
    }
}
impl<P1: Policy + Specialize, P2: Policy + Specialize> Specialize for PolicyAnd<P1, P2> {
    fn specialize_and(b1: Box<SpecializationEnum>, b2: Box<SpecializationEnum>) -> Result<Self, (Box<SpecializationEnum>, Box<SpecializationEnum>)> {
        let r1 = b1.specialize::<P1>();
        let r2 = b2.specialize::<P2>();
        match (r1, r2) {
            (Ok(p1), Ok(p2)) => Ok(PolicyAnd { p1, p2 }),
            (Err(e1), Err(e2)) => Err((Box::new(e1), Box::new(e2))),
            (Ok(p1), Err(e2)) =>
                Err((Box::new(p1.to_specialization_enum().normalize()), Box::new(e2))),
            (Err(e1), Ok(p2)) =>
                Err((Box::new(e1), Box::new(p2.to_specialization_enum().normalize()))),
        }
    }
}

// Guarantees we can join PolicyAnd with other instances of the same type.
impl<P1: ReflexiveJoin, P2: ReflexiveJoin> ReflexiveJoin for PolicyAnd<P1, P2> {
    fn reflexive_join(&mut self, other: &mut Self) {
        self.p1.reflexive_join(&mut other.p1);
        self.p2.reflexive_join(&mut other.p2);
    }
}

// Can use PolicyAnd with schema and frontend policy associations.
impl<P1: SchemaPolicy, P2: SchemaPolicy> SchemaPolicy for PolicyAnd<P1, P2> {
    fn from_row(table_name: &str, row: &Vec<mysql::Value>) -> Self {
        Self {
            p1: P1::from_row(table_name, row),
            p2: P2::from_row(table_name, row),
        }
    }
}
impl<P1: FrontendPolicy, P2: FrontendPolicy> FrontendPolicy for PolicyAnd<P1, P2> {
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