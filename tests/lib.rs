// Crate Dependencies ---------------------------------------------------------
use debug_stub_derive::DebugStub;
use std::fmt::Debug;

struct StructWithoutDebug;

// Struct Tests ---------------------------------------------------------------
#[test]
fn test_struct_empty() {
    #[derive(DebugStub)]
    struct TestStruct;

    let s = TestStruct;

    assert_eq!(format!("{:?}", s), "TestStruct");
    assert_eq!(format!("{:#?}", s), "TestStruct");
}

#[test]
fn test_struct_compare_std_empty() {
    mod a {
        use debug_stub_derive::DebugStub;

        #[derive(DebugStub)]
        pub struct TestStruct;
    }

    mod b {
        #[derive(Debug)]
        pub struct TestStruct;
    }

    assert_eq!(
        format!("{:?}", a::TestStruct),
        format!("{:?}", b::TestStruct)
    );

    assert_eq!(
        format!("{:#?}", a::TestStruct),
        format!("{:#?}", b::TestStruct)
    );
}

#[test]
fn test_struct() {
    #[derive(Debug)]
    struct StructWithDebug {
        number: u64,
    }

    #[derive(DebugStub)]
    struct TestStruct {
        value: bool,
        a: StructWithDebug,
        #[allow(dead_code)]
        #[debug_stub = "StructWithoutDebugReplaceValue"]
        b: StructWithoutDebug,
    }

    let s = TestStruct {
        value: true,
        a: StructWithDebug { number: 42 },
        b: StructWithoutDebug,
    };

    assert_eq!(format!("{:?}", s), "TestStruct { value: true, a: StructWithDebug { number: 42 }, b: StructWithoutDebugReplaceValue }");
    assert_eq!(
        format!("{:#?}", s),
        r#"TestStruct {
    value: true,
    a: StructWithDebug {
        number: 42,
    },
    b: StructWithoutDebugReplaceValue,
}"#
    );
}

#[test]
fn test_struct_dyn_fields() {
    trait Trait: Debug {}

    #[derive(Debug)]
    struct TraitImpl;

    impl Trait for TraitImpl {}

    #[derive(DebugStub)]
    struct Struct<'a> {
        a: Box<dyn Trait>,
        #[allow(dead_code)]
        #[debug_stub = "_"]
        b: Box<dyn Trait>,
        c: &'a dyn Trait,
        #[allow(dead_code)]
        #[debug_stub = "_"]
        d: &'a dyn Trait,
    }

    let trait_impl = TraitImpl;
    assert_eq!(
        format!(
            "{:?}",
            Struct {
                a: Box::new(TraitImpl),
                b: Box::new(TraitImpl),
                c: &trait_impl,
                d: &trait_impl,
            },
        ),
        "Struct { a: TraitImpl, b: _, c: TraitImpl, d: _ }",
    );
}

#[test]
fn test_struct_compare_std() {
    mod a {
        use debug_stub_derive::DebugStub;

        #[derive(Debug)]
        pub struct InternalStruct {
            pub a: bool,
        }

        #[derive(DebugStub)]
        pub struct TestStruct {
            pub a: InternalStruct,
            pub b: u64,
        }
    }

    mod b {
        #[derive(Debug)]
        pub struct InternalStruct {
            pub a: bool,
        }

        #[derive(Debug)]
        pub struct TestStruct {
            pub a: InternalStruct,
            pub b: u64,
        }
    }

    let struct_a = a::TestStruct {
        a: a::InternalStruct { a: true },
        b: 42,
    };

    let struct_b = b::TestStruct {
        a: b::InternalStruct { a: true },
        b: 42,
    };

    assert_eq!(format!("{:?}", struct_a), format!("{:?}", struct_b));

    assert_eq!(format!("{:#?}", struct_a), format!("{:#?}", struct_b));
}

#[test]
fn test_struct_with_type_parameters() {
    use std::fmt::{Debug, Display};

    #[derive(DebugStub)]
    struct TestStruct<T: Display + Debug> {
        t: T,
    }

    assert_eq!(
        format!("{:?}", TestStruct { t: 42 }),
        "TestStruct { t: 42 }"
    );

    assert_eq!(
        format!("{:#?}", TestStruct { t: 42 }),
        r#"TestStruct {
    t: 42,
}"#
    );
}

#[test]
fn test_struct_with_type_where_clause() {
    use std::fmt::{Debug, Display};

    #[derive(DebugStub)]
    struct TestStruct<T>
    where
        T: Display + Debug,
    {
        t: T,
    }

    assert_eq!(
        format!("{:?}", TestStruct { t: 42 }),
        "TestStruct { t: 42 }"
    );

    assert_eq!(
        format!("{:#?}", TestStruct { t: 42 }),
        r#"TestStruct {
    t: 42,
}"#
    );
}

#[test]
fn test_struct_optional() {
    #[derive(DebugStub)]
    struct TestStruct {
        #[debug_stub(some = "StructWithoutDebugReplaceValue")]
        s: Option<StructWithoutDebug>,
    }

    assert_eq!(
        format!("{:?}", TestStruct { s: None }),
        "TestStruct { s: None }"
    );

    assert_eq!(
        format!("{:#?}", TestStruct { s: None }),
        r#"TestStruct {
    s: None,
}"#
    );

    assert_eq!(
        format!(
            "{:?}",
            TestStruct {
                s: Some(StructWithoutDebug)
            }
        ),
        "TestStruct { s: Some(StructWithoutDebugReplaceValue) }"
    );

    assert_eq!(
        format!(
            "{:#?}",
            TestStruct {
                s: Some(StructWithoutDebug)
            }
        ),
        r#"TestStruct {
    s: Some(
        StructWithoutDebugReplaceValue,
    ),
}"#
    );
}

#[test]
fn test_struct_result_both() {
    #[derive(DebugStub)]
    struct TestStruct {
        #[debug_stub(
            ok = "StructWithoutDebugReplaceValue",
            err = "ErrorWithoutDebugReplaceValue"
        )]
        s: Result<StructWithoutDebug, StructWithoutDebug>,
    }

    assert_eq!(
        format!(
            "{:?}",
            TestStruct {
                s: Err(StructWithoutDebug)
            }
        ),
        "TestStruct { s: Err(ErrorWithoutDebugReplaceValue) }"
    );

    assert_eq!(
        format!(
            "{:#?}",
            TestStruct {
                s: Err(StructWithoutDebug)
            }
        ),
        r#"TestStruct {
    s: Err(
        ErrorWithoutDebugReplaceValue,
    ),
}"#
    );

    assert_eq!(
        format!(
            "{:?}",
            TestStruct {
                s: Ok(StructWithoutDebug)
            }
        ),
        "TestStruct { s: Ok(StructWithoutDebugReplaceValue) }"
    );

    assert_eq!(
        format!(
            "{:#?}",
            TestStruct {
                s: Ok(StructWithoutDebug)
            }
        ),
        r#"TestStruct {
    s: Ok(
        StructWithoutDebugReplaceValue,
    ),
}"#
    );
}

#[test]
fn test_struct_result_ok() {
    #[derive(DebugStub)]
    struct TestStruct {
        #[debug_stub(ok = "StructWithoutDebugReplaceValue")]
        s: Result<StructWithoutDebug, ()>,
    }

    assert_eq!(
        format!(
            "{:?}",
            TestStruct {
                s: Ok(StructWithoutDebug)
            }
        ),
        "TestStruct { s: Ok(StructWithoutDebugReplaceValue) }"
    );

    assert_eq!(
        format!(
            "{:#?}",
            TestStruct {
                s: Ok(StructWithoutDebug)
            }
        ),
        r#"TestStruct {
    s: Ok(
        StructWithoutDebugReplaceValue,
    ),
}"#
    );
}

#[test]
fn test_struct_result_err() {
    struct ErrorWithoutDebug;

    #[derive(DebugStub)]
    struct TestStruct {
        #[debug_stub(err = "ErrorWithoutDebugReplaceValue")]
        s: Result<(), ErrorWithoutDebug>,
    }

    assert_eq!(
        format!(
            "{:?}",
            TestStruct {
                s: Err(ErrorWithoutDebug)
            }
        ),
        "TestStruct { s: Err(ErrorWithoutDebugReplaceValue) }"
    );

    assert_eq!(
        format!(
            "{:#?}",
            TestStruct {
                s: Err(ErrorWithoutDebug)
            }
        ),
        r#"TestStruct {
    s: Err(
        ErrorWithoutDebugReplaceValue,
    ),
}"#
    );
}

#[test]
fn test_struct_optional_compare_std() {
    mod a {
        use debug_stub_derive::DebugStub;

        pub struct Internal;

        #[derive(DebugStub)]
        pub struct TestStruct {
            pub a: Option<String>,
            pub b: Option<u64>,
            #[debug_stub(some = "Internal")]
            pub c: Option<Internal>,
        }
    }

    mod b {
        #[derive(Debug)]
        pub struct Internal;

        #[derive(Debug)]
        pub struct TestStruct {
            pub a: Option<String>,
            pub b: Option<u64>,
            pub c: Option<Internal>,
        }
    }

    let struct_a = a::TestStruct {
        a: Some("Foo".to_string()),
        b: None,
        c: Some(a::Internal),
    };

    let struct_b = b::TestStruct {
        a: Some("Foo".to_string()),
        b: None,
        c: Some(b::Internal),
    };

    assert_eq!(format!("{:?}", struct_a), format!("{:?}", struct_b));

    assert_eq!(format!("{:#?}", struct_a), format!("{:#?}", struct_b));
}

#[test]
fn test_struct_result_compare_std() {
    mod a {
        use debug_stub_derive::DebugStub;

        pub struct Internal;

        #[derive(DebugStub)]
        pub struct TestStruct {
            pub a: Result<String, bool>,
            pub b: Result<u64, ()>,
            #[debug_stub(ok = "Internal")]
            pub c: Result<Internal, ()>,
            #[debug_stub(err = "Internal")]
            pub d: Result<(), Internal>,
        }
    }

    mod b {
        #[derive(Debug)]
        pub struct Internal;

        #[derive(Debug)]
        pub struct TestStruct {
            pub a: Result<String, bool>,
            pub b: Result<u64, ()>,
            pub c: Result<Internal, ()>,
            pub d: Result<(), Internal>,
        }
    }

    let struct_a = a::TestStruct {
        a: Ok("Foo".to_string()),
        b: Err(()),
        c: Ok(a::Internal),
        d: Err(a::Internal),
    };

    let struct_b = b::TestStruct {
        a: Ok("Foo".to_string()),
        b: Err(()),
        c: Ok(b::Internal),
        d: Err(b::Internal),
    };

    assert_eq!(format!("{:?}", struct_a), format!("{:?}", struct_b));

    assert_eq!(format!("{:#?}", struct_a), format!("{:#?}", struct_b));
}

#[test]
fn test_struct_tuple() {
    #[derive(DebugStub)]
    struct A();

    #[derive(DebugStub)]
    struct B((), u32, #[debug_stub = "replacement"] StructWithoutDebug);

    #[derive(DebugStub)]
    struct C(
        #[debug_stub(some = "replacement")] Option<StructWithoutDebug>,
        #[debug_stub(ok = "MyOk", err = "MyErr")] Result<StructWithoutDebug, StructWithoutDebug>,
    );

    assert_eq!(format!("{:?}", A()), "A");
    assert_eq!(
        format!("{:?}", B((), 5, StructWithoutDebug)),
        "B((), 5, replacement)"
    );
    assert_eq!(
        format!("{:?}", C(Some(StructWithoutDebug), Err(StructWithoutDebug))),
        "C(Some(replacement), Err(MyErr))"
    );
    assert_eq!(
        format!("{:?}", C(None, Ok(StructWithoutDebug))),
        "C(None, Ok(MyOk))"
    );
}

#[test]
fn test_struct_generic() {
    use std::marker::PhantomData;

    #[derive(DebugStub)]
    struct A<T>(T);

    #[derive(DebugStub)]
    #[debug_stub(ignore_generics)]
    struct B<T>(PhantomData<T>);

    assert_eq!(format!("{:?}", A(5)), "A(5)");
    assert_eq!(
        format!("{:?}", B(PhantomData::<StructWithoutDebug>)),
        "B(PhantomData)"
    );
}

// Enum Tests -----------------------------------------------------------------

#[test]
fn test_enum_empty() {
    #[derive(DebugStub)]
    enum TestEnum {
        VariantC {},
        VariantD,
    }

    assert_eq!(format!("{:?}", TestEnum::VariantC {}), "VariantC");

    assert_eq!(format!("{:?}", TestEnum::VariantC {}), "VariantC");

    assert_eq!(format!("{:#?}", TestEnum::VariantD), "VariantD");

    assert_eq!(format!("{:#?}", TestEnum::VariantD), "VariantD");
}

#[test]
fn test_enum_compare_std_empty() {
    mod a {
        use debug_stub_derive::DebugStub;

        #[derive(DebugStub)]
        pub enum TestEnum {
            VariantA,
            VariantB {},
        }
    }

    mod b {
        #[derive(Debug)]
        pub enum TestEnum {
            VariantA,
            VariantB {},
        }
    }

    assert_eq!(
        format!("{:?}", a::TestEnum::VariantA),
        format!("{:?}", b::TestEnum::VariantA)
    );

    assert_eq!(
        format!("{:#?}", a::TestEnum::VariantA),
        format!("{:#?}", b::TestEnum::VariantA)
    );

    assert_eq!(
        format!("{:?}", a::TestEnum::VariantB {}),
        format!("{:?}", b::TestEnum::VariantB {})
    );

    assert_eq!(
        format!("{:#?}", a::TestEnum::VariantB {}),
        format!("{:#?}", b::TestEnum::VariantB {})
    );
}

#[test]
fn test_enum() {
    #[derive(Debug)]
    struct StructWithDebug {
        number: u64,
    }

    #[derive(DebugStub)]
    enum TestEnum {
        VariantA(
            StructWithDebug,
            #[debug_stub = "StructWithoutDebugReplaceValue"] StructWithoutDebug,
            bool,
        ),
        VariantB {
            a: StructWithDebug,
            #[allow(dead_code)]
            #[debug_stub = "StructWithoutDebugReplaceValue"]
            b: StructWithoutDebug,
            c: bool,
        },
    }

    assert_eq!(
        format!(
            "{:?}",
            TestEnum::VariantA(StructWithDebug { number: 42 }, StructWithoutDebug, true)
        ),
        "VariantA(StructWithDebug { number: 42 }, StructWithoutDebugReplaceValue, true)"
    );

    assert_eq!(
        format!("{:?}", TestEnum::VariantB {
            a: StructWithDebug {
                number: 42
            },
            b: StructWithoutDebug,
            c: true

        }), "VariantB { a: StructWithDebug { number: 42 }, b: StructWithoutDebugReplaceValue, c: true }"
    );
}

#[test]
fn test_enum_dyn_fields() {
    trait Trait: Debug {}

    #[derive(Debug)]
    struct TraitImpl;

    impl Trait for TraitImpl {}

    #[derive(DebugStub)]
    enum Enum<'a> {
        A(
            Box<dyn Trait>,
            #[allow(dead_code)]
            #[debug_stub = "_"]
            Box<dyn Trait>,
            &'a dyn Trait,
            #[allow(dead_code)]
            #[debug_stub = "_"]
            &'a dyn Trait,
        ),
        B {
            a: Box<dyn Trait>,
            #[allow(dead_code)]
            #[debug_stub = "_"]
            b: Box<dyn Trait>,
            c: &'a dyn Trait,
            #[allow(dead_code)]
            #[debug_stub = "_"]
            d: &'a dyn Trait,
        },
    }

    let trait_impl = TraitImpl;

    assert_eq!(
        format!(
            "{:?}",
            Enum::A(
                Box::new(TraitImpl),
                Box::new(TraitImpl),
                &trait_impl,
                &trait_impl,
            ),
        ),
        "A(TraitImpl, _, TraitImpl, _)",
    );

    assert_eq!(
        format!(
            "{:?}",
            Enum::B {
                a: Box::new(TraitImpl),
                b: Box::new(TraitImpl),
                c: &trait_impl,
                d: &trait_impl,
            },
        ),
        "B { a: TraitImpl, b: _, c: TraitImpl, d: _ }",
    );
}

#[test]
fn test_enum_compare_std() {
    mod a {
        use debug_stub_derive::DebugStub;

        #[derive(Debug)]
        pub struct InternalStruct {
            pub a: bool,
        }

        #[derive(DebugStub)]
        pub enum TestEnum {
            VariantA(InternalStruct, bool, u64),
            VariantB { a: InternalStruct, b: bool, c: u64 },
        }
    }

    mod b {
        #[derive(Debug)]
        pub struct InternalStruct {
            pub a: bool,
        }

        #[derive(Debug)]
        pub enum TestEnum {
            VariantA(InternalStruct, bool, u64),
            VariantB { a: InternalStruct, b: bool, c: u64 },
        }
    }

    let enum_a_a = a::TestEnum::VariantA(a::InternalStruct { a: true }, false, 42);

    let enum_a_b = a::TestEnum::VariantB {
        a: a::InternalStruct { a: true },
        b: false,
        c: 42,
    };

    let enum_b_a = b::TestEnum::VariantA(b::InternalStruct { a: true }, false, 42);

    let enum_b_b = b::TestEnum::VariantB {
        a: b::InternalStruct { a: true },
        b: false,
        c: 42,
    };

    assert_eq!(format!("{:?}", enum_a_a), format!("{:?}", enum_b_a));

    assert_eq!(format!("{:#?}", enum_a_a), format!("{:#?}", enum_b_a));

    assert_eq!(format!("{:?}", enum_a_b), format!("{:?}", enum_b_b));

    assert_eq!(format!("{:#?}", enum_a_b), format!("{:#?}", enum_b_b));
}

#[test]
fn test_enum_with_type_parameters() {
    use std::fmt::{Debug, Display};

    #[derive(DebugStub)]
    enum TestEnum<T: Display + Debug> {
        VariantA(T),
    }

    assert_eq!(format!("{:?}", TestEnum::VariantA(true)), "VariantA(true)");

    assert_eq!(
        format!("{:#?}", TestEnum::VariantA(true)),
        r#"VariantA(
    true,
)"#
    );
}

#[test]
fn test_enum_with_type_where_clause() {
    use std::fmt::{Debug, Display};

    #[derive(DebugStub)]
    enum TestEnum<T>
    where
        T: Display + Debug,
    {
        VariantA(T),
    }

    assert_eq!(format!("{:?}", TestEnum::VariantA(true)), "VariantA(true)");

    assert_eq!(
        format!("{:#?}", TestEnum::VariantA(true)),
        r#"VariantA(
    true,
)"#
    );
}

#[test]
fn test_enum_optional() {
    #[derive(DebugStub)]
    enum TestEnum {
        VariantA(#[debug_stub(some = "StructWithoutDebugReplaceValue")] Option<StructWithoutDebug>),
    }

    assert_eq!(format!("{:?}", TestEnum::VariantA(None)), "VariantA(None)");

    assert_eq!(
        format!("{:#?}", TestEnum::VariantA(None)),
        r#"VariantA(
    None,
)"#
    );

    assert_eq!(
        format!("{:?}", TestEnum::VariantA(Some(StructWithoutDebug))),
        "VariantA(Some(StructWithoutDebugReplaceValue))"
    );

    assert_eq!(
        format!("{:#?}", TestEnum::VariantA(Some(StructWithoutDebug))),
        r#"VariantA(
    Some(
        StructWithoutDebugReplaceValue,
    ),
)"#
    );
}

#[test]
fn test_enum_result_both() {
    #[derive(DebugStub)]
    enum TestEnum {
        VariantA(
            #[debug_stub(
                ok = "StructWithoutDebugReplaceValue",
                err = "ErrorWithoutDebugReplaceValue"
            )]
            Result<StructWithoutDebug, StructWithoutDebug>,
        ),
    }

    assert_eq!(
        format!("{:?}", TestEnum::VariantA(Err(StructWithoutDebug))),
        "VariantA(Err(ErrorWithoutDebugReplaceValue))"
    );

    assert_eq!(
        format!("{:#?}", TestEnum::VariantA(Err(StructWithoutDebug))),
        r#"VariantA(
    Err(
        ErrorWithoutDebugReplaceValue,
    ),
)"#
    );

    assert_eq!(
        format!("{:?}", TestEnum::VariantA(Ok(StructWithoutDebug))),
        "VariantA(Ok(StructWithoutDebugReplaceValue))"
    );

    assert_eq!(
        format!("{:#?}", TestEnum::VariantA(Ok(StructWithoutDebug))),
        r#"VariantA(
    Ok(
        StructWithoutDebugReplaceValue,
    ),
)"#
    );
}

#[test]
fn test_enum_result_ok() {
    #[derive(DebugStub)]
    enum TestEnum {
        VariantA(
            #[debug_stub(ok = "StructWithoutDebugReplaceValue")] Result<StructWithoutDebug, String>,
        ),
    }

    assert_eq!(
        format!("{:?}", TestEnum::VariantA(Ok(StructWithoutDebug))),
        "VariantA(Ok(StructWithoutDebugReplaceValue))"
    );

    assert_eq!(
        format!("{:#?}", TestEnum::VariantA(Ok(StructWithoutDebug))),
        r#"VariantA(
    Ok(
        StructWithoutDebugReplaceValue,
    ),
)"#
    );

    assert_eq!(
        format!("{:?}", TestEnum::VariantA(Err("Foo".to_string()))),
        "VariantA(Err(\"Foo\"))"
    );

    assert_eq!(
        format!("{:#?}", TestEnum::VariantA(Err("Foo".to_string()))),
        r#"VariantA(
    Err(
        "Foo",
    ),
)"#
    );
}

#[test]
fn test_enum_result_err() {
    struct ErrorWithoutDebug;

    #[derive(DebugStub)]
    enum TestEnum {
        VariantA(
            #[debug_stub(err = "ErrorWithoutDebugReplaceValue")] Result<String, ErrorWithoutDebug>,
        ),
    }

    assert_eq!(
        format!("{:?}", TestEnum::VariantA(Err(ErrorWithoutDebug))),
        "VariantA(Err(ErrorWithoutDebugReplaceValue))"
    );

    assert_eq!(
        format!("{:#?}", TestEnum::VariantA(Err(ErrorWithoutDebug))),
        r#"VariantA(
    Err(
        ErrorWithoutDebugReplaceValue,
    ),
)"#
    );

    assert_eq!(
        format!("{:?}", TestEnum::VariantA(Ok("Foo".to_string()))),
        "VariantA(Ok(\"Foo\"))"
    );

    assert_eq!(
        format!("{:#?}", TestEnum::VariantA(Ok("Foo".to_string()))),
        r#"VariantA(
    Ok(
        "Foo",
    ),
)"#
    );
}

#[test]
fn test_enum_optional_compare_std() {
    mod a {
        use debug_stub_derive::DebugStub;

        #[derive(DebugStub)]
        pub enum TestEnum {
            VariantA(Option<String>, Option<String>),
            VariantB {
                a: Option<String>,
                b: Option<String>,
            },
        }
    }

    mod b {
        #[derive(Debug)]
        pub enum TestEnum {
            VariantA(Option<String>, Option<String>),
            VariantB {
                a: Option<String>,
                b: Option<String>,
            },
        }
    }

    let enum_a_a = a::TestEnum::VariantA(None, Some("Foo".to_string()));
    let enum_a_b = a::TestEnum::VariantB {
        a: Some("Foo".to_string()),
        b: None,
    };

    let enum_b_a = b::TestEnum::VariantA(None, Some("Foo".to_string()));
    let enum_b_b = b::TestEnum::VariantB {
        a: Some("Foo".to_string()),
        b: None,
    };

    assert_eq!(format!("{:?}", enum_a_a), format!("{:?}", enum_b_a));

    assert_eq!(format!("{:#?}", enum_a_a), format!("{:#?}", enum_b_a));

    assert_eq!(format!("{:?}", enum_a_b), format!("{:?}", enum_b_b));

    assert_eq!(format!("{:#?}", enum_a_b), format!("{:#?}", enum_b_b));
}

#[test]
fn test_enum_result_compare_std() {
    mod a {
        use debug_stub_derive::DebugStub;

        #[derive(DebugStub)]
        pub enum TestEnum {
            VariantA(Result<String, bool>, Result<String, bool>),
            VariantB {
                a: Result<String, bool>,
                b: Result<String, bool>,
            },
        }
    }

    mod b {
        #[derive(Debug)]
        pub enum TestEnum {
            VariantA(Result<String, bool>, Result<String, bool>),
            VariantB {
                a: Result<String, bool>,
                b: Result<String, bool>,
            },
        }
    }

    let enum_a_a = a::TestEnum::VariantA(Err(true), Ok("Foo".to_string()));
    let enum_a_b = a::TestEnum::VariantB {
        a: Ok("Foo".to_string()),
        b: Err(true),
    };

    let enum_b_a = b::TestEnum::VariantA(Err(true), Ok("Foo".to_string()));
    let enum_b_b = b::TestEnum::VariantB {
        a: Ok("Foo".to_string()),
        b: Err(true),
    };

    assert_eq!(format!("{:?}", enum_a_a), format!("{:?}", enum_b_a));

    assert_eq!(format!("{:#?}", enum_a_a), format!("{:#?}", enum_b_a));

    assert_eq!(format!("{:?}", enum_a_b), format!("{:?}", enum_b_b));

    assert_eq!(format!("{:#?}", enum_a_b), format!("{:#?}", enum_b_b));
}

#[test]
fn test_enum_generic() {
    use std::marker::PhantomData;

    #[derive(DebugStub)]
    enum Enum<T> {
        A(T),
    }

    #[derive(DebugStub)]
    #[debug_stub(ignore_generics)]
    struct B<T>(PhantomData<T>);

    assert_eq!(format!("{:?}", Enum::A(5)), "A(5)");
    assert_eq!(
        format!("{:?}", B(PhantomData::<StructWithoutDebug>)),
        "B(PhantomData)"
    );
}
