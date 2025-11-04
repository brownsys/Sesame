use erased_serde::Serialize;
use sesame::context::Context;
use sesame::critical::{CriticalRegion, Signature};
use sesame::policy::{NoPolicy, Policy, RefPolicy};
use sesame_derive::PConRender;

type RefPCon<'a> = sesame::pcon::PCon<&'a dyn Serialize, RefPolicy<'a, dyn Policy + 'a>>;

#[derive(PConRender)]
struct Nested {
    pub v: Vec<sesame::pcon::PCon<u8, NoPolicy>>,
}

#[derive(PConRender)]
struct Simple {
    t1: sesame::pcon::PCon<String, NoPolicy>,
    t2: sesame::pcon::PCon<u8, NoPolicy>,
    t3: String,
    t4: Nested,
}
impl Simple {
    pub fn new() -> Self {
        Simple {
            t1: sesame::pcon::PCon::new(String::from("hello"), NoPolicy {}),
            t2: sesame::pcon::PCon::new(10, NoPolicy {}),
            t3: String::from("unprotected"),
            t4: Nested {
                v: vec![
                    sesame::pcon::PCon::new(100, NoPolicy {}),
                    sesame::pcon::PCon::new(110, NoPolicy {}),
                    sesame::pcon::PCon::new(200, NoPolicy {}),
                ],
            },
        }
    }
}

// Helper: turn Vec<u8> to String.
fn to_string(v: &Vec<u8>) -> String {
    std::str::from_utf8(v.as_slice()).unwrap().to_string()
}

// Helper: serializes PCons.
fn pcon_to_string<'a>(pcon: &'a RefPCon<'_>) -> Result<String, ()> {
    let context = Context::test(());
    let result = pcon.critical(
        context,
        CriticalRegion::new(
            |t: &&'a dyn Serialize, _| *t,
            Signature {
                username: "",
                signature: "",
            },
        ),
        (),
    );
    serialize_to_string(result.unwrap())
}

fn serialize_to_string(data: &dyn erased_serde::Serialize) -> Result<String, ()> {
    use serde::ser::Serialize;
    use serde_json::Serializer;

    let mut buf: Vec<u8> = Vec::new();
    let json = &mut Serializer::new(Box::new(&mut buf));
    match data.serialize(json) {
        Err(_) => Err(()),
        Ok(_) => Ok(to_string(&buf)),
    }
}

#[test]
fn simple_render_struct() {
    use sesame_rocket::render::{PConRender, Renderable};

    let simple = Simple::new();
    let renderable = simple.render();
    assert!(matches!(renderable, Renderable::Dict(_)));

    if let Renderable::Dict(map) = renderable {
        assert_eq!(map.len(), 4);
        assert!(matches!(map.get("t1"), Option::Some(_)));
        assert!(matches!(map.get("t2"), Option::Some(_)));
        assert!(matches!(map.get("t3"), Option::Some(_)));
        assert!(matches!(map.get("t4"), Option::Some(_)));

        let t1 = map.get("t1").unwrap();
        let t2 = map.get("t2").unwrap();
        let t3 = map.get("t3").unwrap();
        let t4 = map.get("t4").unwrap();
        assert!(matches!(t1, Renderable::PCon(_)));
        assert!(matches!(t2, Renderable::PCon(_)));
        assert!(matches!(t3, Renderable::Serialize(_)));
        assert!(matches!(t4, Renderable::Dict(_)));

        if let Renderable::PCon(t1) = t1 {
            assert_eq!(pcon_to_string(t1), Ok(String::from("\"hello\"")));
        }
        if let Renderable::PCon(t2) = t2 {
            assert_eq!(pcon_to_string(t2), Ok(String::from("10")));
        }
        if let Renderable::Serialize(t3) = t3 {
            assert_eq!(serialize_to_string(t3), Ok(String::from("\"unprotected\"")));
        }
        if let Renderable::Dict(t4) = t4 {
            matches!(t4.get("v"), Option::Some(Renderable::Array(_)));
            if let Renderable::Array(v) = t4.get("v").unwrap() {
                assert_eq!(v.len(), 3);
                assert!(
                    matches!(&v[0], Renderable::PCon(a) if pcon_to_string(a) == Ok(String::from("100")))
                );
                assert!(
                    matches!(&v[1], Renderable::PCon(a) if pcon_to_string(a) == Ok(String::from("110")))
                );
                assert!(
                    matches!(&v[2], Renderable::PCon(a) if pcon_to_string(a) == Ok(String::from("200")))
                );
            }
        }
    }
}
