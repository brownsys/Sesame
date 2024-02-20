use alohomora::context::Context;
use alohomora::policy::{Policy, RefPolicy, NoPolicy};
use alohomora_derive::BBoxRender;
use erased_serde::Serialize;

type RefBBox<'a> = alohomora::bbox::BBox<&'a dyn Serialize, RefPolicy<'a, dyn Policy + 'a>>;

#[derive(BBoxRender)]
struct Simple {
    t1: alohomora::bbox::BBox<String, NoPolicy>,
    t2: alohomora::bbox::BBox<u8, NoPolicy>,
    t3: String,
}
impl Simple {
    pub fn new() -> Self {
        Simple {
            t1: alohomora::bbox::BBox::new(String::from("hello"), NoPolicy {}),
            t2: alohomora::bbox::BBox::new(10, NoPolicy {}),
            t3: String::from("unprotected"),
        }
    }
}

// Helper: turn Vec<u8> to String.
fn to_string(v: &Vec<u8>) -> String {
    std::str::from_utf8(v.as_slice()).unwrap().to_string()
}

// Helper: serializes BBoxes.
fn bbox_to_string(bbox: &RefBBox<'_>) -> Result<String, ()> {
    let context = Context::new(Option::None::<()>, String::from(""), ());
    serialize_to_string(bbox.unbox(&context))
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
    use alohomora::bbox::{BBoxRender, Renderable};

    let simple = Simple::new();
    let renderable = simple.render();
    assert!(matches!(renderable, Renderable::Dict(_)));

    if let Renderable::Dict(map) = renderable {
        assert_eq!(map.len(), 3);
        assert!(matches!(map.get("t1"), Option::Some(_)));
        assert!(matches!(map.get("t2"), Option::Some(_)));
        assert!(matches!(map.get("t3"), Option::Some(_)));

        let t1 = map.get("t1").unwrap();
        let t2 = map.get("t2").unwrap();
        let t3 = map.get("t3").unwrap();
        assert!(matches!(t1, Renderable::BBox(_)));
        assert!(matches!(t2, Renderable::BBox(_)));
        assert!(matches!(t3, Renderable::Serialize(_)));

        if let Renderable::BBox(t1) = t1 {
            assert_eq!(bbox_to_string(t1), Ok(String::from("\"hello\"")));
        }
        if let Renderable::BBox(t2) = t2 {
            assert_eq!(bbox_to_string(t2), Ok(String::from("10")));
        }
        if let Renderable::Serialize(t3) = t3 {
            assert_eq!(serialize_to_string(t3), Ok(String::from("\"unprotected\"")));
        }
    }
}
