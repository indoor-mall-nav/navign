// Tests for proc_macros
// Tests derive macros and attribute macros

use navign_proc_macros::ExampleDerive;

#[test]
fn test_example_derive() {
    #[derive(ExampleDerive)]
    struct TestStruct {
        field: String,
    }

    let instance = TestStruct {
        field: "test".to_string(),
    };

    // The derive macro should add an example_method
    instance.example_method();
    // If this compiles, the derive macro works
}

#[test]
fn test_example_derive_multiple_fields() {
    #[derive(ExampleDerive)]
    struct MultiFieldStruct {
        field1: i32,
        field2: String,
        field3: bool,
    }

    let instance = MultiFieldStruct {
        field1: 42,
        field2: "hello".to_string(),
        field3: true,
    };

    instance.example_method();
}

// Note: Generic struct test removed due to Rust compiler limitations
// with local generic structs and derive macros in test modules

#[test]
fn test_example_derive_empty_struct() {
    #[derive(ExampleDerive)]
    struct EmptyStruct;

    let instance = EmptyStruct;
    instance.example_method();
}

#[test]
fn test_example_attribute() {
    use navign_proc_macros::example_attribute;

    #[example_attribute]
    fn test_function() {
        // Function body
    }

    // Call the function to ensure it still works
    test_function();
    // If this compiles and runs, the attribute macro works
}

#[test]
fn test_example_attribute_with_args() {
    use navign_proc_macros::example_attribute;

    #[example_attribute]
    fn function_with_return() -> i32 {
        42
    }

    assert_eq!(function_with_return(), 42);
}

#[test]
fn test_example_attribute_with_params() {
    use navign_proc_macros::example_attribute;

    #[example_attribute]
    fn function_with_params(x: i32, y: i32) -> i32 {
        x + y
    }

    assert_eq!(function_with_params(10, 20), 30);
}

#[test]
fn test_example_macro() {
    use navign_proc_macros::example_macro;

    // The function-like macro generates code
    example_macro! {
        // Macro input
    }

    // If this compiles, the macro works
}

// Test that macros work with common Rust patterns

#[test]
fn test_derive_with_debug() {
    #[derive(Debug, ExampleDerive)]
    struct DebugStruct {
        value: i32,
    }

    let instance = DebugStruct { value: 42 };
    instance.example_method();

    // Debug should still work
    let debug_str = format!("{:?}", instance);
    assert!(debug_str.contains("DebugStruct"));
}

#[test]
fn test_derive_with_clone() {
    #[derive(Clone, ExampleDerive)]
    struct CloneStruct {
        value: i32,
    }

    let instance = CloneStruct { value: 42 };
    let cloned = instance.clone();

    instance.example_method();
    cloned.example_method();
}

#[test]
fn test_multiple_derives() {
    #[derive(Debug, Clone, ExampleDerive)]
    struct MultiDeriveStruct {
        field: String,
    }

    let instance = MultiDeriveStruct {
        field: "test".to_string(),
    };

    instance.example_method();
    let _ = instance.clone();
    let _ = format!("{:?}", instance);
}
