//! Integration tests for the `ConstBuilder` derive macro.

use myrtio_macros::ConstBuilder;

// -----------------------------------------------------------------------------
// Test 1: Simple struct with scalar fields
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, ConstBuilder)]
pub struct SimpleEntity {
    pub name: &'static str,
    pub count: u32,
}

#[test]
fn simple_entity_builder_works() {
    const ENTITY: SimpleEntity = SimpleEntity::builder().name("test").count(42).build();

    assert_eq!(ENTITY.name, "test");
    assert_eq!(ENTITY.count, 42);
}

#[test]
fn simple_entity_builder_chaining() {
    let entity = SimpleEntity::builder().count(10).name("chained").build();

    assert_eq!(entity.name, "chained");
    assert_eq!(entity.count, 10);
}

// -----------------------------------------------------------------------------
// Test 2: Struct with Option fields (optional fields)
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, ConstBuilder)]
pub struct EntityWithOptions {
    pub required_field: u32,
    pub optional_field: Option<u8>,
    pub another_optional: Option<&'static str>,
}

#[test]
fn optional_fields_default_to_none() {
    const ENTITY: EntityWithOptions = EntityWithOptions::builder().required_field(100).build();

    assert_eq!(ENTITY.required_field, 100);
    assert_eq!(ENTITY.optional_field, None);
    assert_eq!(ENTITY.another_optional, None);
}

#[test]
fn optional_fields_can_be_set_to_some() {
    const ENTITY: EntityWithOptions = EntityWithOptions::builder()
        .required_field(200)
        .optional_field(Some(42))
        .another_optional(Some("hello"))
        .build();

    assert_eq!(ENTITY.required_field, 200);
    assert_eq!(ENTITY.optional_field, Some(42));
    assert_eq!(ENTITY.another_optional, Some("hello"));
}

#[test]
fn optional_fields_can_be_explicitly_none() {
    const ENTITY: EntityWithOptions = EntityWithOptions::builder()
        .required_field(300)
        .optional_field(None)
        .build();

    assert_eq!(ENTITY.required_field, 300);
    assert_eq!(ENTITY.optional_field, None);
}

// -----------------------------------------------------------------------------
// Test 3: Struct with lifetime parameters
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, ConstBuilder)]
pub struct EntityWithLifetime<'a> {
    pub name: &'a str,
    pub value: u32,
    pub description: Option<&'a str>,
}

#[test]
fn lifetime_entity_const_construction() {
    const ENTITY: EntityWithLifetime<'static> = EntityWithLifetime::builder()
        .name("lifetime test")
        .value(999)
        .description(Some("with description"))
        .build();

    assert_eq!(ENTITY.name, "lifetime test");
    assert_eq!(ENTITY.value, 999);
    assert_eq!(ENTITY.description, Some("with description"));
}

#[test]
fn lifetime_entity_optional_defaults() {
    const ENTITY: EntityWithLifetime<'static> = EntityWithLifetime::builder()
        .name("no desc")
        .value(1)
        .build();

    assert_eq!(ENTITY.name, "no desc");
    assert_eq!(ENTITY.value, 1);
    assert_eq!(ENTITY.description, None);
}

// -----------------------------------------------------------------------------
// Test 4: Struct similar to LightEntity (complex example)
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Rgb,
    ColorTemp,
    Brightness,
}

#[derive(Debug, Clone, PartialEq, ConstBuilder)]
pub struct LightEntityLike<'a> {
    pub name: &'a str,
    pub device_id: &'static str,
    pub icon: Option<&'a str>,
    pub brightness: bool,
    pub color_modes: &'a [ColorMode],
    pub effects: Option<&'a [&'a str]>,
    pub min_mireds: Option<u16>,
    pub max_mireds: Option<u16>,
    pub optimistic: bool,
}

static COLOR_MODES: [ColorMode; 2] = [ColorMode::Rgb, ColorMode::Brightness];
static EFFECTS: [&str; 2] = ["rainbow", "pulse"];

#[test]
fn light_entity_like_full_construction() {
    const ENTITY: LightEntityLike<'static> = LightEntityLike::builder()
        .name("Living Room Light")
        .device_id("living_room_01")
        .icon(Some("mdi:lightbulb"))
        .brightness(true)
        .color_modes(&COLOR_MODES)
        .effects(Some(&EFFECTS))
        .min_mireds(Some(153))
        .max_mireds(Some(500))
        .optimistic(false)
        .build();

    assert_eq!(ENTITY.name, "Living Room Light");
    assert_eq!(ENTITY.device_id, "living_room_01");
    assert_eq!(ENTITY.icon, Some("mdi:lightbulb"));
    assert!(ENTITY.brightness);
    assert_eq!(ENTITY.color_modes.len(), 2);
    assert_eq!(ENTITY.effects, Some(EFFECTS.as_slice()));
    assert_eq!(ENTITY.min_mireds, Some(153));
    assert_eq!(ENTITY.max_mireds, Some(500));
    assert!(!ENTITY.optimistic);
}

#[test]
fn light_entity_like_minimal_construction() {
    static EMPTY_MODES: [ColorMode; 0] = [];

    const ENTITY: LightEntityLike<'static> = LightEntityLike::builder()
        .name("Simple Light")
        .device_id("simple_01")
        .brightness(false)
        .color_modes(&EMPTY_MODES)
        .optimistic(true)
        .build();

    assert_eq!(ENTITY.name, "Simple Light");
    assert_eq!(ENTITY.device_id, "simple_01");
    assert_eq!(ENTITY.icon, None);
    assert!(!ENTITY.brightness);
    assert!(ENTITY.color_modes.is_empty());
    assert_eq!(ENTITY.effects, None);
    assert_eq!(ENTITY.min_mireds, None);
    assert_eq!(ENTITY.max_mireds, None);
    assert!(ENTITY.optimistic);
}

// -----------------------------------------------------------------------------
// Test 5: Builder can be used at runtime too
// -----------------------------------------------------------------------------

#[test]
fn builder_works_at_runtime() {
    let name = String::from("runtime");
    let entity = SimpleEntity::builder()
        .name(Box::leak(name.into_boxed_str()))
        .count(123)
        .build();

    assert_eq!(entity.count, 123);
}

// -----------------------------------------------------------------------------
// Test 6: Verify builder type is accessible and has expected methods
// -----------------------------------------------------------------------------

#[test]
fn builder_type_is_public() {
    let _builder: SimpleEntityBuilder = SimpleEntity::builder();
    let _builder2: EntityWithOptionsBuilder = EntityWithOptions::builder();
    let _builder3: EntityWithLifetimeBuilder<'static> = EntityWithLifetime::builder();
}



