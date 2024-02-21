/* 
Testing Overview
        Derived for these genres of struct:
            Fields w/o boxes + doesn't provide a new struct name

            Fields without BBoxes
            Fields with BBoxes
            Fields with collection (Vec) of BBoxes
            Fields with custom struct that impls MagicUnbox    


        Tested that the Out struct names are as specified + the traits are derived
        Tested that name not specified panics
        
        For both impls with a generated struct and without, 
            tested form + contents of to_enum and from_enum
 */

// TODO(babman): simplify these tests and test the from_enum/to_enum/fold pipeline
mod tests {
    use alohomora::policy::NoPolicy;
    use alohomora::bbox::BBox;
    use alohomora::r#type::{AlohomoraType, AlohomoraTypeEnum};
    use std::collections::HashMap;

    #[derive(AlohomoraType, Clone, PartialEq, Debug)] // Keep same out type
    pub struct NoBoxes {
        f1: u64, 
        f2: String,
    }

    #[derive(AlohomoraType, Clone, Debug, PartialEq)]
    #[alohomora_out_type(name = "SimpleBoxedLite", to_derive = [Clone, PartialEq, Debug])]
    pub struct SimpleBoxed {
        f1: BBox<u64, NoPolicy>, 
        f2: BBox<String, NoPolicy>, 
    }

    #[derive(AlohomoraType)]
    #[alohomora_out_type(name = "MixedBoxedLite", to_derive = [Clone, Debug])]
    pub struct MixedBoxed {
        f1: BBox<u64, NoPolicy>, 
        f2: String, 
    }

    #[derive(AlohomoraType)]
    #[alohomora_out_type(name = "VecBoxedLite", to_derive = [Clone, PartialEq, Debug])]
    pub struct VecBoxed {
        f1: Vec<BBox<u64, NoPolicy>>,
    }

    #[derive(AlohomoraType)]
    #[alohomora_out_type(name = "ContainsStructLite", to_derive = [Clone, PartialEq, Debug])]
    pub struct ContainsStruct {
        f1: SimpleBoxed,
        f2: BBox<SimpleBoxed, NoPolicy>,
        f3: NoBoxes, // when not preserving the Out type for NoBoxes, this becomes NoBoxesLite
        f4: BBox<NoBoxes, NoPolicy>, 
        f5: VecBoxed,
    }

    /*  The MagicUnboxOut structs have the specified names,
        we can impl on them (the compiler agrees they exist), 
        and the types to construct them are correctly Unboxed. 
    */
    impl NoBoxes {
        pub fn new(f1: u64, f2: String) -> Self {
        Self { f1: f1, f2: f2 }
    }
    }
    impl SimpleBoxedLite {
        pub fn new(f1: u64, f2: String) -> Self {
            Self { f1: f1, f2: f2 }
        }
    }
    impl MixedBoxedLite {
        pub fn new(f1: u64, f2: String) -> Self {
            Self { f1: f1, f2: f2 }
        }
    }
    impl VecBoxedLite {
        pub fn new(f1: Vec<u64>) -> Self {
            Self { f1: f1 }
        }
    }
    // TODO(babman): discuss discrepancy btwn f1/f2 (still funky but acceptable) and f3/f4 (now fixed with "same" Out type)
    #[allow(dead_code)]
    impl ContainsStructLite {
        pub fn new( f1: SimpleBoxedLite, f2: SimpleBoxed,
                    f3: NoBoxes,     f4: NoBoxes,
                    f5: VecBoxedLite) -> Self {
            Self{f1: f1, f2: f2, f3: f3, f4: f4, f5: f5}
        }
    }

    #[test]
    fn traits_derived() {
        let num: u64 = 42; 
        let hi = String::from("hi"); 

        fn clone_and_debug<T: Clone + core::fmt::Debug>(item: T) -> String {
            let item_clone = item.clone();
            format!("Debug: {item_clone:?}")
        }

        let no_boxes = NoBoxes::new(num.clone(), hi.clone());
        let simple_boxes= SimpleBoxedLite::new(num.clone(), hi.clone()); 
        let mixed_boxes = MixedBoxedLite::new(num.clone(), hi.clone()); 
        let vec_boxes = VecBoxedLite::new(vec![num.clone()]); 

        // More importantly, the traits derived correctly
        assert_eq!(clone_and_debug(no_boxes), "Debug: NoBoxes { f1: 42, f2: \"hi\" }");
        assert_eq!(clone_and_debug(simple_boxes), "Debug: SimpleBoxedLite { f1: 42, f2: \"hi\" }");
        assert_eq!(clone_and_debug(mixed_boxes), "Debug: MixedBoxedLite { f1: 42, f2: \"hi\" }");
        assert_eq!(clone_and_debug(vec_boxes), "Debug: VecBoxedLite { f1: [42] }");
    }

    /* ----------- Tests for perfunctory/no-name AlohomoraType derive ------------ */

    #[test]
    fn no_name_to_enum() {
        #[derive(AlohomoraType, PartialEq, Debug)]
        #[alohomora_out_type(to_derive = [Debug])] // TODO to_derive doesn't do anything when keeping same struct
        pub struct Boxless {
            f1: u64,
        }

        //Testing to_enum
        let simple = Boxless { f1: 10 };
        let magical_enum: AlohomoraTypeEnum = simple.to_enum();
        assert!(matches!(magical_enum, AlohomoraTypeEnum::Value(_)));
        
        if let AlohomoraTypeEnum::Value(any_data) = magical_enum {
            let data: Result<Boxless, &str> = match any_data.downcast::<Boxless>() {
                Err(_) => Err("This Errored"),
                Ok(v) => Ok(*v),
            }; 
            assert_eq!(data, Ok(Boxless {f1: 10}));
        }
    }

    #[test]
    fn no_name_from_enum() {
        #[derive(AlohomoraType)]
        #[alohomora_out_type(to_derive = [Debug])] //TODO to_derive doesn't do anything when keeping same struct ->
        pub struct Boxless {
            f1: u64,
        }
        let boxless = Boxless { f1: 10}; 
        let magical_enum = AlohomoraTypeEnum::Value(Box::new(boxless));

        // Call from_enum to generate an Out struct
        let output_res: Result<Boxless, ()> = <Boxless>::from_enum(magical_enum);
        
        //From_enum didn't fail
        assert!(matches!(output_res, Ok(_)));
        let output = output_res.unwrap(); 
        
        // Out struct has correct data
        assert_eq!(output.f1, 10);
    }

    /* --------------- Testing discrete Out type -> to_enum and from_enum -------------------------------- */ 

    #[test]
    fn cased_fields_to_enum() {
        #[derive(AlohomoraType)]
        #[alohomora_out_type(name = "SimpleOut")]
        pub struct Simple {
            t1: Vec<BBox<String, NoPolicy>>,
            t2: BBox<u8, NoPolicy>,
            t3: String,
        }

        let simple = 
            Simple {
                t1: vec![BBox::new(String::from("hello"), NoPolicy{})],
                t2: BBox::new(10, NoPolicy{}),
                t3: String::from("unprotected"),
            };

        // Call to_enum
        let magical: AlohomoraTypeEnum = simple.to_enum();
        assert!(matches!(magical, AlohomoraTypeEnum::Struct(_)));

        if let AlohomoraTypeEnum::Struct(mut map) = magical {
            // Map contains all the fields
            assert_eq!(map.len(), 3);
            assert!(matches!(map.get("t1"), Option::Some(_)));
            assert!(matches!(map.get("t2"), Option::Some(_)));
            assert!(matches!(map.get("t3"), Option::Some(_)));

            // Contents of fields are correct AlohomoraTypeEnum variants
            let t1 = map.remove("t1").unwrap();
            let t2 = map.remove("t2").unwrap();
            let t3 = map.remove("t3").unwrap();
            assert!(matches!(t1, AlohomoraTypeEnum::Vec(_)));
            assert!(matches!(t2, AlohomoraTypeEnum::BBox(_)));
            assert!(matches!(t3, AlohomoraTypeEnum::Value(_)));
            
            // Values are correct
            if let AlohomoraTypeEnum::Vec(t1) = t1 {
                for item_enum in t1 {
                    assert!(matches!(item_enum, AlohomoraTypeEnum::BBox(_)));
                    if let AlohomoraTypeEnum::BBox(boxed) = item_enum {
                        let any_data = boxed.specialize_policy::<NoPolicy>()
                                                    .unwrap()
                                                    .discard_box(); 
                        let data = match any_data.downcast() {
                            Err(_) => Err(()),
                            Ok(v) => Ok(*v),
                        }; 
                        assert_eq!(data, Ok(String::from("hello")));
                    }
                }
            }
            if let AlohomoraTypeEnum::BBox(boxed) = t2 {
                let any_data = boxed.specialize_policy::<NoPolicy>()
                                            .unwrap()
                                            .discard_box(); 
                let data: Result<u8, ()> = match any_data.downcast() {
                    Err(_) => Err(()),
                    Ok(v) => Ok(*v),
                }; 
                assert_eq!(data, Ok(10));
            }
            if let AlohomoraTypeEnum::Value(any_data) = t3 {
                let data: Result<String, ()> = match any_data.downcast() {
                    Err(_) => Err(()),
                    Ok(v) => Ok(*v),
                }; 
                assert_eq!(data, Ok(String::from("unprotected")));
            } 
        }
    }

    #[test]
    fn cased_fields_from_enum() {
        #[derive(AlohomoraType)]
        #[alohomora_out_type(name = "SimpleOut")]
        pub struct Simple {
            t1: Vec<BBox<bool, NoPolicy>>,
            t2: BBox<u8, NoPolicy>,
            t3: String,
        }
        impl SimpleOut {}

        let ten: u8 = 10;
        let hashmap = HashMap::from([
            (String::from("t1"), vec![true, true, true].to_enum()),
            (String::from("t2"), ten.to_enum()),
            (String::from("t3"), String::from("unprotected").to_enum()),
        ]);
        let magical_enum = AlohomoraTypeEnum::Struct(hashmap);

        // Call from_enum to generate an Out struct
        let output_res = <Simple as AlohomoraType>::from_enum(magical_enum);
        
        //From_enum didn't fail
        assert!(matches!(output_res, Ok(_)));
        let output = output_res.unwrap(); 
        
        // Out struct has correct data
        for b in output.t1 {
            assert_eq!(b, true);
        }
        assert_eq!(output.t2, 10);
        assert_eq!(output.t3, String::from("unprotected"));
    }
}
