use bbox_derive::BBoxRender;

#[derive(BBoxRender)]
struct Simple {
    t1: bbox::bbox::BBox<String>,
    t2: bbox::bbox::BBox<u8>,
    t3: String,
}
impl Simple {
    pub fn new() -> Self {
        Simple {
            t1: bbox::bbox::BBox::new(String::from("hello"), vec![]),
            t2: bbox::bbox::BBox::new(10, vec![]),
            t3: String::from("unprotected"),
        }
    }
}

// Helper: turn Vec<u8> to String.
fn to_string(v: &Vec<u8>) -> String {
    std::str::from_utf8(v.as_slice()).unwrap().to_string()
}

// Helper: serializes BBoxes.
type SBBox<'r> = bbox::bbox::BBox<&'r dyn erased_serde::Serialize>;
fn bbox_to_string(bbox: &SBBox<'_>) -> std::result::Result<String, ()> {
    serialize_to_string(bbox.test_unbox())
}

fn serialize_to_string(data: &dyn erased_serde::Serialize) -> std::result::Result<String, ()> {
    use serde::ser::Serialize;
    use serde_json::Serializer;

    let mut buf: Vec<u8> = Vec::new();
    let json = &mut Serializer::new(Box::new(&mut buf));
    match data.serialize(json) {
        Result::Err(_) => Result::Err(()),
        Result::Ok(_) => Result::Ok(to_string(&buf)),
    }
}

#[test]
fn simple_render_struct() {
    use std::result::Result;

    use bbox::bbox::{BBoxRender, Renderable};

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
            assert_eq!(bbox_to_string(t1), Result::Ok(String::from("\"hello\"")));
        }
        if let Renderable::BBox(t2) = t2 {
            assert_eq!(bbox_to_string(t2), Result::Ok(String::from("10")));
        }
        if let Renderable::Serialize(t3) = t3 {
            assert_eq!(
                serialize_to_string(t3),
                Result::Ok(String::from("\"unprotected\""))
            );
        }
    }
}
