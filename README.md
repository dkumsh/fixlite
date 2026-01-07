# fixlite
fixlite is a Rust crate for parsing and building FIX (Financial Information eXchange) protocol messages. It provides a procedural macro, FixDeserialize, to automatically generate deserialization implementations for your structs, a macro, fix_tag_registry!, to define registries that map FIX tags to their corresponding Rust types, and a FixBuilder that encodes values via the FixValue trait.

## Features
 * Automatic deserialization of FIX messages into Rust structs using #[derive(FixDeserialize)].
 * Support for FIX components and repeating groups via attributes.
 * Customizable tag-to-type mappings through registries defined with fix_tag_registry!.
 * Compile-time validation of tag-type associations to ensure correctness.
 * Zero-copy deserialization for string fields defined as &str, enhancing performance by avoiding unnecessary allocations.
 * Message building with FixBuilder chaining (begin_with().field/field_ref(...).finish()) and the build_fix! macro.
 * Trait-based field encoding via FixValue and AsFixStr for enums.
 * Optional BodyLength/CheckSum validation during parsing when the `checksum` feature is enabled.

## Usage
### Defining a Registry
Use the fix_tag_registry! macro to define a registry that maps FIX tags to their corresponding Rust types. This registry is used during deserialization to validate and parse tag values correctly.

```rust
use fixlite::fix_tag_registry;

fix_tag_registry! {
    MyRegistry {
        35 => [fixlite::fix::MsgType],
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
Enable the derive macro:

```toml
fixlite = { version = "...", features = ["derive"] }
```

If you rename the dependency in `Cargo.toml`, the derive macro will still resolve it:

```toml
fix = { package = "fixlite", version = "...", features = ["derive"] }
```

No extra `use` alias is required in your code.

To validate BodyLength and CheckSum during parsing, enable the `checksum` feature (optionally alongside `derive`):

```toml
fixlite = { version = "...", features = ["derive", "checksum"] }
```

With checksum enabled, malformed frames return `FixError::Malformed(MalformedFix::...)` while semantic parse errors return `FixError::InvalidValue`.

```rust
use fixlite::FixDeserialize;

#[derive(FixDeserialize, Debug)]
#[fix_registry(MyRegistry)]
struct TestMessage<'a> {
    #[fix(tag = 35)]
    msg_type: fixlite::fix::MsgType,

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
## Building FIX Messages
Use FixBuilder directly or via the build_fix! macro. Types that implement FixValue can be encoded, and FIX enums implement AsFixStr automatically. begin_with returns a chainable message builder: use `field` for owned values, `field_ref` for borrowed values, and the `str`/`bytes` helpers for string/byte fields. For fallible encoding (currently only `f64` rejects NaN/inf), use `try_field`/`try_field_ref` or `try_fields`, which return `Result`.

```rust
use chrono::Utc;
use fixlite::fix::{FixBuilder, HandlInst, MsgType, OrdType, Side, TimeInForce};

let mut builder = FixBuilder::new("FIX.4.2", "BUYER", "SELLER");
let dt = Utc::now();

let extras = &[(58, "note"), (100, "XNAS")];

let msg = builder
    .begin_with(&2u64, &dt, &MsgType::NewOrderSingle)
    .str(11, "123")
    .field(21, HandlInst::Automated)
    .str(55, "IBM")
    .field(54, Side::Buy)
    .field(38, 100u32)
    .field(40, OrdType::Limit)
    .str(44, "150.25")
    .field(59, TimeInForce::Day)
    .fields(|m| {
        for &(tag, val) in extras {
            m.str(tag, val);
        }
    })
    .finish();
```

Fallible field example (for `f64` validation, in a `Result`-returning context):

```rust
let price = 150.25_f64;

let msg = builder
    .begin_with(&2u64, &dt, &MsgType::NewOrderSingle)
    .try_field(44, price)?
    .finish();
```

You can also use the build_fix! macro:

```rust
use chrono::Utc;
use fixlite::build_fix;
use fixlite::fix::{FixBuilder, HandlInst, MsgType, OrdType, Side, TimeInForce};

let mut builder = FixBuilder::new("FIX.4.2", "BUYER", "SELLER");
let dt = Utc::now();

let msg = build_fix!(
    builder,
    2u64,
    dt,
    MsgType::NewOrderSingle,
    11, "123",
    21, HandlInst::Automated,
    55, "IBM",
    54, Side::Buy,
    38, 100u32,
    40, OrdType::Limit,
    44, "150.25",
    59, TimeInForce::Day,
);
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
You can deserialize using from_fix() for SOH delimiter. If your input uses another separator, replace it before parsing:

```rust
let raw = b"8=FIX.4.2|9=176|35=D|49=BUYER|56=SELLER|34=2|52=20190605-19:45:32.123|11=123|21=1|55=IBM|54=1|38=100|40=2|44=150.25|59=0|10=128|";
let message = raw
    .iter()
    .map(|&b| if b == b'|' { b'\x01' } else { b })
    .collect::<Vec<u8>>();
let parsed = TestMessage::from_fix(&message)?;
```
## License
This project is dual-licensed under MIT OR LGPL-3.0-or-later. See the [LICENSE-MIT](LICENSE-MIT) and [LICENSE-LGPL-3.0](LICENSE-LGPL-3.0) files for details.
