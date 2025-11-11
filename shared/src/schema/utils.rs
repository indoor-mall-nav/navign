#[cfg(all(feature = "serde", feature = "alloc"))]
use super::ConnectedArea;

#[cfg(all(feature = "serde", feature = "alloc"))]
use alloc::vec::Vec;

/// Serialize connected areas
/// For SQL: (i64, f64, f64, bool)
/// For non-SQL: (String, f64, f64, bool)
#[cfg(all(feature = "serde", feature = "alloc"))]
pub fn serialize_connected_areas<S>(
    areas: &Vec<ConnectedArea>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeSeq;
    let mut seq = serializer.serialize_seq(Some(areas.len()))?;
    for area in areas {
        // Serialize as tuple directly - works for both i64 and String
        seq.serialize_element(&area.0)?;
        seq.serialize_element(&area.1)?;
        seq.serialize_element(&area.2)?;
        seq.serialize_element(&area.3)?;
    }
    seq.end()
}

/// Deserialize connected areas
/// For SQL: (i64, f64, f64, bool)
/// For non-SQL: (String, f64, f64, bool)
#[cfg(all(feature = "serde", feature = "alloc"))]
pub fn deserialize_connected_areas<'de, D>(deserializer: D) -> Result<Vec<ConnectedArea>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{SeqAccess, Visitor};
    use core::fmt;

    struct ConnectedAreasVisitor;

    impl<'de> Visitor<'de> for ConnectedAreasVisitor {
        type Value = Vec<ConnectedArea>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a sequence of connected areas")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut areas = Vec::new();

            #[cfg(feature = "sql")]
            {
                // For SQL: area ID is i64
                while let Some(id) = seq.next_element::<i64>()? {
                    let x = seq
                        .next_element::<f64>()?
                        .ok_or_else(|| serde::de::Error::custom("missing x"))?;
                    let y = seq
                        .next_element::<f64>()?
                        .ok_or_else(|| serde::de::Error::custom("missing y"))?;
                    let enabled = seq
                        .next_element::<bool>()?
                        .ok_or_else(|| serde::de::Error::custom("missing enabled"))?;
                    areas.push((id, x, y, enabled));
                }
            }

            #[cfg(not(feature = "sql"))]
            {
                use alloc::string::String;
                // For non-SQL: area ID is String
                while let Some(id) = seq.next_element::<String>()? {
                    let x = seq
                        .next_element::<f64>()?
                        .ok_or_else(|| serde::de::Error::custom("missing x"))?;
                    let y = seq
                        .next_element::<f64>()?
                        .ok_or_else(|| serde::de::Error::custom("missing y"))?;
                    let enabled = seq
                        .next_element::<bool>()?
                        .ok_or_else(|| serde::de::Error::custom("missing enabled"))?;
                    areas.push((id, x, y, enabled));
                }
            }

            Ok(areas)
        }
    }

    deserializer.deserialize_seq(ConnectedAreasVisitor)
}
