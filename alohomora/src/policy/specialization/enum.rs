use crate::policy::{NoPolicy, OwnedReflection, PolicyReflection, Specializable, Specialize};

pub type SpecializationEnum = OwnedReflection<'static>;

impl Specializable for SpecializationEnum {
    fn to_specialization_enum(self) -> SpecializationEnum {
        self
    }
    fn to_specialization_enum_box(self: Box<Self>) -> SpecializationEnum {
        self.to_specialization_enum()
    }

    // Visitor pattern.
    fn specialize<P: Specialize>(self) -> Result<P, Self> {
        match self {
            Self::NoReflection(_) => panic!("Specializing something without reflection"),
            Self::Leaf(b) => {
                P::specialize_leaf(b).map_err(Self::Leaf)
            },
            Self::PolicyAnd(p1, p2) => {
                match P::specialize_and(p1, p2) {
                    Ok(p) => Ok(p),
                    Err((p1, p2)) => {
                        match P::specialize_and(p2, p1) {
                            Ok(p) => Ok(p),
                            Err((mut p2, mut p1)) => {
                                if p1.is_no_policy() {
                                    match p2.specialize::<P>() {
                                        Ok(p2) => { return Ok(p2); },
                                        Err(e) => { p2 = Box::new(e); }
                                    }
                                }
                                if p2.is_no_policy() {
                                    match p1.specialize::<P>() {
                                        Ok(p1) => { return Ok(p1); },
                                        Err(e) => { p1 = Box::new(e); }
                                    }
                                }
                                Err(Self::PolicyAnd(p1, p2))
                            }
                        }
                    },
                }
            },
            Self::PolicyOr(p1, p2) => {
                match P::specialize_or(p1, p2) {
                    Ok(p) => Ok(p),
                    Err((p1, p2)) => {
                        match P::specialize_or(p2, p1) {
                            Ok(p2) => Ok(p2),
                            Err((mut p2, mut p1)) => {
                                match p1.specialize::<P>() {
                                    Ok(p1) => { return Ok(p1); },
                                    Err(e) => { p1 = Box::new(e); }
                                }
                                match p2.specialize::<P>() {
                                    Ok(p2) => { return Ok(p2); },
                                    Err(e) => { p2 = Box::new(e); }
                                }
                                Err(Self::PolicyOr(p1, p2))
                            },
                        }
                    },
                }
            },
            Self::OptionPolicy(p) => {
                let p = match P::specialize_option(p) {
                    Ok(p) => { return Ok(p); },
                    Err(p) => p,
                };
                match p {
                    None => {
                        let p = Box::new(NoPolicy {});
                        match P::specialize_leaf(p) {
                            Ok(p) => Ok(p),
                            Err(_) => Err(Self::OptionPolicy(None)),
                        }
                    },
                    Some(p) => {
                        match p.specialize::<P>() {
                            Ok(p) => Ok(p),
                            Err(e) => Err(Self::OptionPolicy(Some(Box::new(e)))),
                        }
                    }
                }
            },
            Self::AnyPolicy(_) => panic!("use normalize() first"),
            Self::TestPolicy(_) => panic!("use normalize() first"),
            PolicyReflection::_Unreachable(inf, _) => match inf {},
        }
    }
}