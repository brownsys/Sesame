

// TODO: this should just be a default for the trait
#[macro_export]
macro_rules! default_policy_join {
    () => {
        /// Default Join implementation
        fn join(&self, other: alohomora::policy::AnyPolicy) -> Result<alohomora::policy::AnyPolicy, ()> {
            if other.is::<Self>() {
                // Policies are combinable
                let other = other.specialize::<Self>().unwrap();
                Ok(alohomora::policy::AnyPolicy::new(self.join_logic(other)?))
            } else if other.is::<alohomora::policy::NoPolicy>() {
                // Other is NoPolicy anyway
                Ok(alohomora::policy::AnyPolicy::new(self.clone()))
            } else {
                // Policies must be stacked
                Ok(alohomora::policy::AnyPolicy::new(alohomora::policy::PolicyAnd::new(
                    alohomora::policy::AnyPolicy::new(self.clone()),
                    other,
                )))
            }
        }
    }
}

// join_logic() depends on specific policy struct