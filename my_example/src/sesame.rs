pub use alohomora::policy::{AnyPolicy, Policy, Reason};
pub use alohomora::bbox::BBox as PCon;
pub use alohomora::AlohomoraType as SesameType;
pub use alohomora::context::UnprotectedContext;
pub use alohomora::pure::execute_pure as privacy_region;
pub use alohomora::pure::PrivacyPureRegion as PrivacyRegion;
pub use alohomora::pcr::Signature as Signature;
pub use alohomora::pcr::PrivacyCriticalRegion as CriticalRegion;

use alohomora::context::ContextData;

pub fn critical_region<T, P: Policy + Clone + 'static, D: ContextData + SesameType + Clone + Sync + 'static, F: FnOnce(T, D::Out)>(
    bbox: PCon<T, P>,
    context: D,
    region: CriticalRegion<F>,
) {
    bbox.into_unbox2(context, region);
}