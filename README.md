# Playout
playout is a Rust library implementing a domain-specific language (DSL) for defining pipeline layouts and descriptor set layouts in a shared format usable by both shading languages and host-side programming languages.

Currently, playout supports generating code for GLSL and Rust.


## Benefits
- Shared Format: Define pipeline and descriptor set layouts once and use them in both shading languages and your host-side code.
- Improved Readability: Write layouts in a human-readable format instead of boilerplate code.
- Reduced Errors: Avoid typos and inconsistencies by defining layouts in a single place.


## Compile playout to GLSL
```rs
let playout_str = "...";
let module = PlayoutModule::try_from(playout_str).unwrap();


let mut writer = String::new();
module.show(&mut writer);
println!("{}", writer)
```

## Compile playout to Rust as a procedural macro
```rs
use ash::vk;
// Extract descriptor set 3 from the playout file
let out = playout_macro::layout!("./example.playout", 3);
```
