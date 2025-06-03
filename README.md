# fixlite
fixlite is a Rust crate designed to facilitate the deserialization of FIX (Financial Information eXchange) protocol messages. It provides a procedural macro, FixDeserialize, to automatically generate deserialization implementations for your structs, and a macro, fix_tag_registry!, to define registries that map FIX tags to their corresponding Rust types.

## Features
 * Automatic deserialization of FIX messages into Rust structs using #[derive(FixDeserialize)].
 * Support for FIX components and repeating groups via attributes.
 * Customizable tag-to-type mappings through registries defined with fix_tag_registry!.
 * Compile-time validation of tag-type associations to ensure correctness.
 * Zero-copy deserialization for string fields defined as &str, enhancing performance by avoiding unnecessary allocations.

## Usage
### Defining a Registry
Use the fix_tag_registry! macro to define a registry that maps FIX tags to their corresponding Rust types. This registry is used during deserialization to validate and parse tag values correctly.

```rust
use fixlite::fix_tag_registry;

fix_tag_registry! {
    MyRegistry {
        35 => [fixlite::MsgType],
        31 => [f64], // LastPx
        8001 => [f64],
    }
}
```
You can also define an empty registry:

```rust
fix_tag_registry!(EmptyRegistry);
```
## Deserializing FIX Messages
Annotate your struct with #[derive(FixDeserialize)] and use the provided attributes to specify how each field corresponds to FIX tags.

```rust
use fixlite::FixDeserialize;

#[derive(FixDeserialize, Debug)]
#[fix_registry(MyRegistry)]
struct TestMessage<'a> {
#[fix(tag = 35)]
msg_type: fixlite::MsgType,

    #[fix(tag = 31)]
    last_px: Option<f64>,

    #[fix(component)]
    header: Header<'a>,

    #[fix_group(tag = 453)]
    parties: Vec<Party<'a>>,

    #[fix(tag = 55)]
    symbol: &'a str, // Zero-copy deserialization
}
```
### Attributes
 * `#[fix(tag = N)]`: Maps the field to FIX tag N.
 * `#[fix(component)]`: Indicates that the field is a nested component.
 * `#[fix_group(tag = N)]`: Indicates that the field is a repeating group starting with tag N.
 * `#[fix_registry(RegistryName)]`: Specifies the registry to use for tag-type validation. Defaults to DefaultRegistry if not specified.

## Zero-Copy Deserialization
For string fields defined as &str, fixlite supports zero-copy deserialization. This means that during deserialization, the string slices in the FIX message are borrowed directly, avoiding unnecessary allocations and enhancing performance.

Ensure that the lifetime annotations are correctly specified to take advantage of this feature.

## Example
Given a FIX message:
``` 
8=FIX.4.2|9=176|35=D|49=BUYER|56=SELLER|34=2|52=20190605-19:45:32.123|11=123|21=1|55=IBM|54=1|38=100|40=2|44=150.25|59=0|10=128|
```
You can deserialize using from_fix() for SOH delimiter or for custom delimiters use from_fix_with_separator it as follows:

```rust
let raw = b"8=FIX.4.2|9=176|35=D|49=BUYER|56=SELLER|34=2|52=20190605-19:45:32.123|11=123|21=1|55=IBM|54=1|38=100|40=2|44=150.25|59=0|10=128|";
let message = TestMessage::from_fix_with_separator(raw,'|')?;
```
## License
This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
