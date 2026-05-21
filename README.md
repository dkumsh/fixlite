# fixlite
fixlite is a Rust crate for parsing and building FIX (Financial Information eXchange) protocol messages. It provides a procedural macro, FixDeserialize, to automatically generate deserialization implementations for your structs, a macro, fix_tag_registry!, to define registries that map FIX tags to their corresponding Rust types, and a FixBuilder that encodes values via FixValue and tag-bound FixTaggedValue types.

## Features
 * Automatic deserialization of FIX messages into Rust structs using #[derive(FixDeserialize)].
 * Support for FIX components and repeating groups via attributes.
 * Customizable tag-to-type mappings through registries defined with fix_tag_registry!.
 * Compile-time validation of tag-type associations to ensure correctness.
 * Zero-copy deserialization for string fields defined as &str, enhancing performance by avoiding unnecessary allocations.
 * Message building with FixBuilder chaining (including field_tagged/field_tagged_ref) and the build_fix! macro.
 * Trait-based field encoding via FixValue and AsFixStr, plus FixTaggedValue for tag-bound types.
 * Common tag constants in the `tags` module (not exhaustive).
 * Optional BodyLength/CheckSum validation during parsing when the `checksum` feature is enabled.

## Usage
### Defining a Registry
Use the fix_tag_registry! macro to define a registry that maps FIX tags to their corresponding Rust types. This registry is used during deserialization to validate and parse tag values correctly.

```rust
use fixlite::fix_tag_registry;

fix_tag_registry! {
    MyRegistry {
        35 => [fixlite::enums::MsgType],
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
    msg_type: fixlite::enums::MsgType,

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

Decode a FIX message without calling the trait method directly:

```rust
let msg: TestMessage = fixlite::decode(bytes)?;
```

`fixlite::decode` (and `FixDeserialize::from_fix`) validate that the input is ASCII before parsing — FIX 4.x is by spec an ASCII protocol, and the check guarantees the parser can safely treat field values as `&str` internally. Non-ASCII input returns `FixError::Malformed(MalformedFix::NonAsciiByte)`.

If you have already verified your input (for example, because it comes from a trusted upstream that ASCII-validates), use the `unsafe` variants to skip the check:

```rust
// SAFETY: caller guarantees `bytes` contains only valid UTF-8 (ASCII is sufficient).
let msg: TestMessage = unsafe { fixlite::decode_unchecked(bytes) }?;
```

Violating the safety contract is undefined behavior because the parser uses unchecked UTF-8 conversion internally.

## Building FIX Messages
Use FixBuilder directly or via the build_fix! macro. Types that implement FixValue can be encoded, and FIX enums implement AsFixStr and FixTaggedValue automatically. begin_with returns a chainable message builder: use `field` for owned values, `field_ref` for borrowed values, `field_tagged`/`field_tagged_ref` for tag-bound types, and the `str`/`bytes` helpers for string/byte fields. For fallible encoding (currently only `f64` rejects NaN/inf), use `try_field`/`try_field_ref`/`try_field_tagged`/`try_field_tagged_ref` or `try_fields`, which return `Result`.

Tagged values (FixTaggedValue) let the type supply its FIX tag so you do not have to:

```rust
use fixlite::FixBuilder;
use fixlite::enums::{HandlInst, MsgType, OrdType, Side};

let mut builder = FixBuilder::new("FIX.4.2", "BUYER", "SELLER");
let dt = chrono::Utc::now();

let msg = builder
    .begin_with(&2u64, &dt, &MsgType::NewOrderSingle)
    .field_tagged(HandlInst::Automated)
    .field_tagged(Side::Buy)
    .field_tagged(OrdType::Limit)
    .finish();
```

```rust
use chrono::Utc;
use fixlite::{FixBuilder, tags};
use fixlite::enums::{HandlInst, MsgType, OrdType, Side, TimeInForce};

let mut builder = FixBuilder::new("FIX.4.2", "BUYER", "SELLER");
let dt = Utc::now();

let extras = &[(tags::TEXT, "note"), (tags::EX_DESTINATION, "XNAS")];

let msg = builder
    .begin_with(&2u64, &dt, &MsgType::NewOrderSingle)
    .str(tags::CL_ORD_ID, "123")
    .field_tagged(HandlInst::Automated)
    .str(tags::SYMBOL, "IBM")
    .field_tagged(Side::Buy)
    .field(tags::ORDER_QTY, 100u32)
    .field_tagged(OrdType::Limit)
    .str(tags::PRICE, "150.25")
    .field_tagged(TimeInForce::Day)
    .fields(|m| {
        for &(tag, val) in extras {
            m.str(tag, val);
        }
    })
    .finish();

// The tags module is a convenience list of common tags; it is not exhaustive.
```

### Encoding `f64`

`f64` does not implement `FixValue` — encoding can fail for `NaN`/`inf`, and a silent infallible path would emit a malformed FIX field. Use the fallible builder methods (`try_field`, `try_field_ref`) or the `?tag => value` macro arm:

```rust
let price = 150.25_f64;

let msg = builder
    .begin_with(&2u64, &dt, &MsgType::NewOrderSingle)
    .try_field(44, price)?
    .finish();
```

When you know finiteness statically (typical for prices), prefer `FixedPrice<W, F>` (or its alias `Price`), which encodes without allocation and has no NaN concerns.

### `build_fix!` macro

You can also use the build_fix! macro. It supports four arms:
- `tag => value` — infallible field
- `?tag => value` — fallible field (currently `f64`); expands to `try_field_ref(...)?` and requires a `Result`-returning context
- `@value` — tagged value (the type provides its FIX tag)
- legacy `tag, value` — equivalent to `tag => value`

```rust
use chrono::Utc;
use fixlite::build_fix;
use fixlite::{FixBuilder, tags};
use fixlite::enums::{HandlInst, MsgType, OrdType, Side, TimeInForce};

let mut builder = FixBuilder::new("FIX.4.2", "BUYER", "SELLER");
let dt = Utc::now();

let price: fixlite::fix::Price = "150.25".parse().unwrap();

let msg = build_fix!(
    builder,
    2u64,
    dt,
    MsgType::NewOrderSingle,
    tags::ORDER_QTY => 100u32,
    tags::PRICE => price,
    @HandlInst::Automated,
    @Side::Buy,
    @OrdType::Limit,
    @TimeInForce::Day,
);
```

**Note on string fields in the macro.** The `tag => value` arm expands to `field_ref(tag, &value)`. The unsized `str` implements `FixValue`, and so does `String`, but `&str` (a reference) does *not*. Two patterns work:

```rust
// (a) Owned String works directly.
let cl_ord_id = String::from("123");
let msg = build_fix!(
    builder, 2u64, dt, MsgType::NewOrderSingle,
    tags::CL_ORD_ID => cl_ord_id,
    @Side::Buy,
);

// (b) For &str, deref so the value position is `str` (unsized) rather than `&str`.
let symbol: &str = "IBM";
let msg = build_fix!(
    builder, 2u64, dt, MsgType::NewOrderSingle,
    tags::SYMBOL => *symbol,
    @Side::Buy,
);
```

For mixed string handling, the chained `FixBuilder` syntax with `.str(tag, "literal")` is often clearer than the macro.
### Attributes
 * `#[fix(tag = N)]`: Maps the field to FIX tag N.
 * `#[fix(component)]`: Indicates that the field is a nested component.
 * `#[fix_group(tag = N)]`: Indicates that the field is a repeating group starting with tag N.
 * `#[fix_registry(RegistryName)]`: Specifies the registry to use for tag-type validation. Defaults to DefaultRegistry if not specified.

### Repeating Groups

`#[fix_group(tag = N)]` declares a repeating-group field of type `Vec<T>`, where `N` is the FIX "NoXxx" counter tag (for example, `453` for `NoPartyIDs`). Element boundaries within the group are detected heuristically rather than from a schema. Two assumptions apply:

1. **All elements start with the same first tag.** When the parser sees the same first tag a second time, it treats that as the start of a new element. If your dialect allows an element's leading tag to be omitted, the parser will not detect element boundaries correctly.
2. **A top-level (outer-struct) tag terminates the group.** If a tag known to the outer struct appears while parsing group elements, the group is considered finished. If an element's tag overlaps with an outer-struct tag, the group will be truncated at that element.

**Nested groups are not supported.** Groups whose elements themselves contain `#[fix_group]` fields will not parse correctly; the inner group's boundaries collide with the outer group's heuristic. For nested-group support you currently need a hand-written `FixDeserialize` impl.

### Components

`#[fix(component)]` declares that a field is a nested component — a sub-struct whose tags are inlined into the message body. The derive routes incoming tags to the component using its `FixDeserialize::is_known_tag` set:

```rust
if component_field.is_none() && Inner::is_known_tag(tag) {
    component_field = Some(Inner::deserialize_fields(...)?);
}
```

This implies two constraints:

1. **Component and outer-struct tag sets must be disjoint.** A tag known to both the outer struct and the component is consumed by the outer struct first; the component never sees it.
2. **First match wins.** If two components share an overlapping tag set, the *field declaration order* determines which component captures the tag.

Optional components (`Option<Component<'a>>`) are allowed and reported missing-via-`None` rather than as `MissingComponent`.

## Zero-Copy Deserialization
For string fields defined as &str, fixlite supports zero-copy deserialization. This means that during deserialization, the string slices in the FIX message are borrowed directly, avoiding unnecessary allocations and enhancing performance.

Ensure that the lifetime annotations are correctly specified to take advantage of this feature.

## Example
Given a FIX message:
``` 
8=FIX.4.2|9=176|35=D|49=BUYER|56=SELLER|34=2|52=20190605-19:45:32.123|11=123|21=1|55=IBM|54=1|38=100|40=2|44=150.25|59=0|10=128|
```
You can deserialize using `fixlite::decode` for the SOH delimiter:

```rust
let raw = b"8=FIX.4.2|9=176|35=D|49=BUYER|56=SELLER|34=2|52=20190605-19:45:32.123|11=123|21=1|55=IBM|54=1|38=100|40=2|44=150.25|59=0|10=128|";
let message = raw
    .iter()
    .map(|&b| if b == b'|' { b'\x01' } else { b })
    .collect::<Vec<u8>>();
let parsed: TestMessage = fixlite::decode(&message)?;
```
## License
This project is dual-licensed under MIT OR Apache-2.0, at your option. See the [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE-2.0](LICENSE-APACHE-2.0) files for details.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
