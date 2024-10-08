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
    let context: &<PortfolioContext as SesameType>::Out = context.downcast_ref().unwrap();
    return context.file_path == format!("{}{}.txt", self.student, self.year);
  }
}

#[derive(SesameType, Clone)]
struct PortfolioContext {
  file_path: PCon<String, AnyPolicy>,
}

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
      |decision: AdmissionDecision, context: <PortfolioContext as SesameType>::Out| {
        decision.write_to_file(context.file_path);
      },
      sesame::Signature { username: "KinanBab", signature: "LS0tLS1CRUdJTiBTU0ggU0lHTkFUVVJFLS0tLS0KVTFOSVUwbEhBQUFBQVFBQUFETUFBQUFMYzNOb0xXVmtNalUxTVRrQUFBQWdRRVVMUGFSOEVlZk53WGtvc2RhZFJDZU14Zwp3MnEvMlY3dzk4VndneUZiTUFBQUFFWm1sc1pRQUFBQUFBQUFBR2MyaGhOVEV5QUFBQVV3QUFBQXR6YzJndFpXUXlOVFV4Ck9RQUFBRUNNeFpvWXRETGJQQ01PTzRXWHg4SElhZzQ3Tzc4bWV0dmUrQStHQTFwS2llUHluVkFoMzZzT0JlSlZsUHVNbWsKWG1GL2hKK3BDUUNiYWtyaWRtWGVFQwotLS0tLUVORCBTU0ggU0lHTkFUVVJFLS0tLS0K" },
      sesame::Signature { username: "KinanBab", signature: "LS0tLS1CRUdJTiBTU0ggU0lHTkFUVVJFLS0tLS0KVTFOSVUwbEhBQUFBQVFBQUFETUFBQUFMYzNOb0xXVmtNalUxTVRrQUFBQWdRRVVMUGFSOEVlZk53WGtvc2RhZFJDZU14Zwp3MnEvMlY3dzk4VndneUZiTUFBQUFFWm1sc1pRQUFBQUFBQUFBR2MyaGhOVEV5QUFBQVV3QUFBQXR6YzJndFpXUXlOVFV4Ck9RQUFBRUNNeFpvWXRETGJQQ01PTzRXWHg4SElhZzQ3Tzc4bWV0dmUrQStHQTFwS2llUHluVkFoMzZzT0JlSlZsUHVNbWsKWG1GL2hKK3BDUUNiYWtyaWRtWGVFQwotLS0tLUVORCBTU0ggU0lHTkFUVVJFLS0tLS0K" },
      sesame::Signature { username: "KinanBab", signature: "LS0tLS1CRUdJTiBTU0ggU0lHTkFUVVJFLS0tLS0KVTFOSVUwbEhBQUFBQVFBQUFETUFBQUFMYzNOb0xXVmtNalUxTVRrQUFBQWdRRVVMUGFSOEVlZk53WGtvc2RhZFJDZU14Zwp3MnEvMlY3dzk4VndneUZiTUFBQUFFWm1sc1pRQUFBQUFBQUFBR2MyaGhOVEV5QUFBQVV3QUFBQXR6YzJndFpXUXlOVFV4Ck9RQUFBRUFxanpLcXR3bzg3eU9RSXQrM3dPWkxzTE93Wms3OU5SYkdYMnhYYm5LWFh1TVk4S1BPbkpxUnlLUmZLcUtQUmsKQXdwQ3NoOCtTejZLbk9LUXdpelN3QgotLS0tLUVORCBTU0ggU0lHTkFUVVJFLS0tLS0K" },
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
