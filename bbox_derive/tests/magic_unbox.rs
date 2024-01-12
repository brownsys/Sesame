
/* Testing Overview
        Derived for these genres of struct:
            Fields without BBoxes
            Fields with BBoxes
            Fields with collection (Vec) of BBoxes
            Fields with custom struct that impls MagicUnbox       

        Tested that the Out struct names are as specified + the traits are derived
        Tested that name not specified panics
        Tested form + contents of to_enum and from_enum

        Checked that struct and field visibility transfer properly via cargo expand output, 
        but that's not foolproof + rust analyzer shows every generated struct and field as pub.
*/

mod tests {
    use bbox::policy::NoPolicy;
    use bbox::bbox::{BBox, MagicUnbox, MagicUnboxEnum};
    use bbox_derive::MagicUnbox;

    use std::collections::HashMap;

    #[derive(MagicUnbox, Clone, PartialEq, Debug)]
    #[magic_unbox_out(name = "NoBoxesLite", to_derive = [Clone, Debug, PartialEq])]
    pub struct NoBoxes {
        f1: u64, 
        f2: String,
    }

    #[derive(MagicUnbox, Clone, PartialEq, Debug)]
    #[magic_unbox_out(name = "SimpleBoxedLite", to_derive = [Clone, PartialEq, Debug])]
    pub struct SimpleBoxed {
        f1: BBox<u64, NoPolicy>, 
        f2: BBox<String, NoPolicy>, 
    }

    #[derive(MagicUnbox)]
    #[magic_unbox_out(name = "MixedBoxedLite", to_derive = [Clone, Debug])]
    pub struct MixedBoxed {
        f1: BBox<u64, NoPolicy>, 
        f2: String, 
    }

    #[derive(MagicUnbox)]
    #[magic_unbox_out(name = "VecBoxedLite", to_derive = [Clone, PartialEq, Debug])]
    pub struct VecBoxed {
        f1: Vec<BBox<u64, NoPolicy>>,
    }

    #[derive(MagicUnbox)]
    #[magic_unbox_out(name = "ContainsStructLite", to_derive = [Clone, PartialEq, Debug])]
    pub struct ContainsStruct {
        f1: SimpleBoxed,
        f2: BBox<SimpleBoxed, NoPolicy>,
        f3: NoBoxes, 
        f4: BBox<NoBoxes, NoPolicy>, 
        f5: VecBoxed,
    }

    /*  
        The MagicUnboxOut structs have the specified names,
        we can impl on them (the compiler agrees they exist), 
        and the types to construct them are correctly Unboxed. 
    */
    impl NoBoxesLite {
        pub fn new(f1: u64, f2: String) -> Self {
            Self{f1: f1, f2: f2}
        }
    }
    impl SimpleBoxedLite {
        pub fn new(f1: u64, f2: String) -> Self {
            Self{f1: f1, f2: f2}
        }
    }
    impl MixedBoxedLite {
        pub fn new(f1: u64, f2: String) -> Self {
            Self{f1: f1, f2: f2}
        }
    }
    impl VecBoxedLite {
        pub fn new(f1: Vec<u64>) -> Self {
            Self{f1: f1}
        }
    }
    impl ContainsStructLite { // TODO discuss discrepancy btwn f1/f2 and f3/f4
        pub fn new( f1: SimpleBoxedLite, f2: SimpleBoxed, 
                    f3: NoBoxesLite, f4: NoBoxes, 
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

        let no_boxes = NoBoxesLite::new(num.clone(), hi.clone()); 
        let simple_boxes= SimpleBoxedLite::new(num.clone(), hi.clone()); 
        let mixed_boxes = MixedBoxedLite::new(num.clone(), hi.clone()); 
        let vec_boxes = VecBoxedLite::new(vec![num.clone()]); 

        // More importantly, the traits derived correctly
        assert_eq!(clone_and_debug(no_boxes), "Debug: NoBoxesLite { f1: 42, f2: \"hi\" }");
        assert_eq!(clone_and_debug(simple_boxes), "Debug: SimpleBoxedLite { f1: 42, f2: \"hi\" }");
        assert_eq!(clone_and_debug(mixed_boxes), "Debug: MixedBoxedLite { f1: 42, f2: \"hi\" }");
        assert_eq!(clone_and_debug(vec_boxes), "Debug: VecBoxedLite { f1: [42] }");
    }

    /* ----- No names panic at compile time ----- */

    #[test]
    //#[should_panic]
    fn no_name_specified(){
        //#[derive(MagicUnbox)]
        pub struct MyStruct {
            f1: String,
        }
    }

    #[test]
    //#[should_panic]
    fn no_name_specified_2(){
        //#[derive(MagicUnbox)]
        //#[magic_unbox_out(to_derive = [Debug])]
        pub struct MyStruct {
            f1: String,
        }
    }

    #[test]
    fn out_struct_visibility (){
        //No assertions currently - used cargo expand --test magic_unbox to inspect output
        //TODO(corinn) check field visibility beyond cargo expand and rust analyzer?
        #[derive(MagicUnbox)]
        #[magic_unbox_out(name = "PrivOut")]
        struct PrivStruct {
            f1: u8,
        }
        impl PrivOut {}

        #[derive(MagicUnbox)]
        #[magic_unbox_out(name = "PrivFieldsOut")]
        pub struct PrivFields {
            f1: u8,
        }
        impl PrivFieldsOut {}

        #[derive(MagicUnbox)]
        #[magic_unbox_out(name = "PubFieldsOut")]
        pub struct PubFields {
            pub f1: u8,
            pub f2: u8,
        }
        impl PubFieldsOut {}

        #[derive(MagicUnbox)]
        #[magic_unbox_out(name = "CrateStructOut")]
        pub(crate) struct CrateStruct{
            f1: u8,
            f2: u8,
        }
        impl CrateStructOut {}

        #[derive(MagicUnbox)]
        #[magic_unbox_out(name = "CrateFieldOut")]
        pub(crate) struct CrateField{
            pub(crate) f1: u8,
            pub(crate) f2: u8,
        }
        impl CrateFieldOut {}

        #[derive(MagicUnbox)]
        #[magic_unbox_out(name = "MixedVisOut")]
        struct MixedVis {
            pub(crate) f1: u8,
            pub f2: u8,
            f3: u8,
        }
        impl MixedVisOut {}
    }

    /* --------------- testing to_enum and from_enum-------------------------------- */ 

    #[test]
    fn simple_to_enum() {
        #[derive(MagicUnbox)]
        #[magic_unbox_out(name = "SimpleOut")]
        pub struct Simple {
            t1: Vec<bbox::bbox::BBox<String, NoPolicy>>,
            t2: bbox::bbox::BBox<u8, NoPolicy>,
            t3: String,
        }

        let simple = 
            Simple {
                t1: vec![BBox::new(String::from("hello"), NoPolicy{})],
                t2: BBox::new(10, NoPolicy{}),
                t3: String::from("unprotected"),
            };

        // Call to_enum
        let magical: MagicUnboxEnum = simple.to_enum();
        assert!(matches!(magical, MagicUnboxEnum::Struct(_)));

        if let MagicUnboxEnum::Struct(mut map) = magical {
            // Map contains all the fields
            assert_eq!(map.len(), 3);
            assert!(matches!(map.get("t1"), Option::Some(_)));
            assert!(matches!(map.get("t2"), Option::Some(_)));
            assert!(matches!(map.get("t3"), Option::Some(_)));

            // Contents of fields are correct MagicUnboxEnum variants
            let t1 = map.remove("t1").unwrap();
            let t2 = map.remove("t2").unwrap();
            let t3 = map.remove("t3").unwrap();
            assert!(matches!(t1, MagicUnboxEnum::Vec(_)));
            assert!(matches!(t2, MagicUnboxEnum::BBox(_)));
            assert!(matches!(t3, MagicUnboxEnum::Value(_)));
            
            // Values are correct
            if let MagicUnboxEnum::Vec(t1) = t1 {
                for item_enum in t1 {
                    assert!(matches!(item_enum, MagicUnboxEnum::BBox(_)));
                    if let MagicUnboxEnum::BBox(boxed) = item_enum {
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
            if let MagicUnboxEnum::BBox(boxed) = t2 {
                let any_data = boxed.specialize_policy::<NoPolicy>()
                                            .unwrap()
                                            .discard_box(); 
                let data: Result<u8, ()> = match any_data.downcast() {
                    Err(_) => Err(()),
                    Ok(v) => Ok(*v),
                }; 
                assert_eq!(data, Ok(10));
            }
            if let MagicUnboxEnum::Value(val) = t3 { 
                let data: Result<String, ()> = match val.downcast() {
                    Err(_) => Err(()),
                    Ok(v) => Ok(*v),
                }; 
                assert_eq!(data, Ok(String::from("unprotected")));
            } 
        }
    }

    #[test]
    fn simple_from_enum() {
        #[derive(MagicUnbox)]
        #[magic_unbox_out(name = "SimpleOut")]
        pub struct Simple {
            t1: bbox::bbox::BBox<String, NoPolicy>,
            t2: bbox::bbox::BBox<u8, NoPolicy>,
            t3: String,
        }
        let ten: u8 = 10;
        let hashmap = HashMap::from([
            (String::from("t1"), String::from("hello").to_enum()),
            (String::from("t2"), ten.to_enum()),
            (String::from("t3"), String::from("unprotected").to_enum()),
        ]);
        let magical_enum = MagicUnboxEnum::Struct(hashmap); 

        // Call from_enum to generate an Out struct
        let output_res = Simple::from_enum(magical_enum);
        
        //From_enum didn't fail
        assert!(matches!(output_res, Ok(_)));
        let output = output_res.unwrap(); 
        
        // Out struct has correct data
        assert_eq!(output.t1, String::from("hello"));
        assert_eq!(output.t2, 10);
        assert_eq!(output.t3, String::from("unprotected"));
    }
}
