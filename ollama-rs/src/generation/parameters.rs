use schemars::{gen::SchemaSettings, schema::RootSchema};
pub use schemars::{schema_for, JsonSchema};
use serde::{Serialize, Deserialize, Serializer};

/// The format to return a response in
#[derive(Debug, Clone)]
pub enum FormatType {
    Json,

    /// Requires Ollama 0.5.0 or greater.
    StructuredJson(JsonStructure),
}

impl Serialize for FormatType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            FormatType::Json => serializer.serialize_str("json"),
            FormatType::StructuredJson(s) => s.schema.serialize(serializer),
        }
    }
}

impl Deserialize for FormatType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s == "json" {
            Ok(FormatType::Json)
        } else {
            let schema = serde_json::from_str::<RootSchema>(&s)
                .map_err(serde::de::Error::custom)?;
            Ok(FormatType::StructuredJson(JsonStructure { schema }))
        }
    }
}

/// Represents a serialized JSON schema. You can create this by converting
/// a JsonSchema:
/// ```rust
/// let json_schema = schema_for!(Output);
/// let serialized: SerializedJsonSchema = json_schema.into();
/// ```
#[derive(Debug, Clone)]
pub struct JsonStructure {
    schema: RootSchema,
}

impl JsonStructure {
    pub fn new<T: JsonSchema>() -> Self {
        // Need to do this because Ollama doesn't support $refs (references in the schema)
        // So we have to explicitly turn them off
        let mut settings = SchemaSettings::draft07();
        settings.inline_subschemas = true;
        let generator = settings.into_generator();
        let schema = generator.into_root_schema_for::<T>();

        Self { schema }
    }
}

/// Used to control how long a model stays loaded in memory, by default models are unloaded after 5 minutes of inactivity
#[derive(Debug, Clone)]
pub enum KeepAlive {
    Indefinitely,
    UnloadOnCompletion,
    Until { time: u64, unit: TimeUnit },
}

impl Serialize for KeepAlive {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            KeepAlive::Indefinitely => serializer.serialize_i8(-1),
            KeepAlive::UnloadOnCompletion => serializer.serialize_i8(0),
            KeepAlive::Until { time, unit } => {
                let mut s = String::new();
                s.push_str(&time.to_string());
                s.push_str(unit.to_symbol());
                serializer.serialize_str(&s)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum TimeUnit {
    Seconds,
    Minutes,
    Hours,
}

impl TimeUnit {
    pub fn to_symbol(&self) -> &'static str {
        match self {
            TimeUnit::Seconds => "s",
            TimeUnit::Minutes => "m",
            TimeUnit::Hours => "hr",
        }
    }
}
