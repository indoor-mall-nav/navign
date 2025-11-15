# navign-proc-macros

Procedural macros for the Navign indoor navigation system.

## Overview

This crate provides custom derive macros, attribute macros, and function-like macros used across the Navign project. Procedural macros enable compile-time code generation for reducing boilerplate and enforcing patterns.

## Usage

Add this crate as a dependency in your `Cargo.toml`:

```toml
[dependencies]
navign-proc-macros = { path = "../proc_macros" }
```

## Available Macros

### Derive Macros

Currently includes example macros that should be replaced with actual implementations:

- `#[derive(ExampleDerive)]` - Example derive macro (replace with actual implementation)

### Attribute Macros

- `#[example_attribute]` - Example attribute macro (replace with actual implementation)

### Function-like Macros

- `example_macro!` - Example function-like macro (replace with actual implementation)

## Development

### Adding New Macros

1. Add the macro implementation to `src/lib.rs`
2. Use appropriate `proc_macro` attributes:
   - `#[proc_macro_derive(Name)]` for derive macros
   - `#[proc_macro_attribute]` for attribute macros
   - `#[proc_macro]` for function-like macros
3. Document with doc comments and examples
4. Add tests in the dependent crates

### Dependencies

- **syn** - Parsing Rust code
- **quote** - Generating Rust code
- **proc-macro2** - Procedural macro utilities

### Testing

Procedural macros are typically tested via integration tests in dependent crates:

```bash
# Test in a dependent crate
cd ../shared
cargo test
```

## Examples

### Derive Macro

```rust
use navign_proc_macros::ExampleDerive;

#[derive(ExampleDerive)]
struct MyStruct {
    field: String,
}
```

### Attribute Macro

```rust
use navign_proc_macros::example_attribute;

#[example_attribute]
fn my_function() {
    // function body
}
```

## Contributing

Follow the project's Rust coding conventions:

- Use `rustfmt` for formatting
- Run `clippy` for linting
- Document all public items
- Provide usage examples

## License

MIT - See LICENSE file for details
