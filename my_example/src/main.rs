mod sesame;

use sesame::{PCon, Policy, AnyPolicy, Reason, UnprotectedContext, SesameType};

#[derive(SesameType, Clone, Debug)]
pub struct AdmissionDecision {
  pub student: String,
  pub year: u32,
  pub admitted: bool,
}
impl AdmissionDecision {
  pub fn write_to_file(&self, file_path: String) {
    // Assume this writes to a file correctly
    println!("Write {:?} to {}", self, file_path);
  }
}

#[derive(Clone)]
struct AdmissionDecisionPolicy {
  student: String,
  year: u32,
}
impl AdmissionDecisionPolicy {
  pub fn new(student: String, year: u32) -> Self {
    Self { student, year }
  }
}

impl Policy for AdmissionDecisionPolicy {
  fn name(&self) -> String { String::from("AdmissionDecisionPolicy") }
  fn join(&self, other: AnyPolicy) -> Result<AnyPolicy, ()> { todo!() }
  fn join_logic(&self, p2: Self) -> Result<Self, ()> { todo!() }

  // Check function
  fn check(&self, context: &UnprotectedContext, _reason: Reason<'_>) -> bool {
    let context: &PortfolioContextOutType = context.downcast_ref().unwrap();
    return context.file_path == format!("{}{}.txt", self.student, self.year);
  }
}

#[derive(SesameType, Clone)]
struct PortfolioContext {
  file_path: PCon<String, AnyPolicy>,
}
pub type PortfolioContextOutType = <PortfolioContext as SesameType>::Out;

// helper function to endpoint
fn write_decision_letter(decision: PCon<AdmissionDecision, AdmissionDecisionPolicy>) -> Result<(), ()> {
  let context =  PortfolioContext {
    file_path: sesame::privacy_region(
      decision.clone(),
      sesame::PrivacyRegion::new(|decision: AdmissionDecision| {
        format!("{}{}.txt", decision.student, decision.year)
      })
    )?,
  };

  sesame::critical_region(
    decision,
    context,
    sesame::CriticalRegion::new(
      |decision: AdmissionDecision, context: PortfolioContextOutType| {
        decision.write_to_file(context.file_path);
      },
      sesame::Signature { username: "", signature: "" },
      sesame::Signature { username: "", signature: "" },
      sesame::Signature { username: "", signature: "" },
    )
  );

  Ok(())
}

fn main() {
  let decision = PCon::new(
    AdmissionDecision {
      student: String::from("Kinan"),
      year: 10,
      admitted: true,
    },
    AdmissionDecisionPolicy::new(String::from("Kinan"), 10),
  );

  write_decision_letter(decision).unwrap();
}