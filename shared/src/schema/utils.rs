#[cfg(feature = "mongodb")]
use super::ConnectedArea;

#[cfg(feature = "mongodb")]
/// Serialize result: (String, f64, f64, bool),
/// regardless it's ObjectId or String at the first element
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
        #[cfg(feature = "mongodb")]
        {
            seq.serialize_element(&area.0.to_hex())?;
            seq.serialize_element(&area.1)?;
            seq.serialize_element(&area.2)?;
            seq.serialize_element(&area.3)?;
        }
        #[cfg(not(feature = "mongodb"))]
        {
            seq.serialize_element(&area.0)?;
            seq.serialize_element(&area.1)?;
            seq.serialize_element(&area.2)?;
            seq.serialize_element(&area.3)?;
        }
    }
    seq.end()
}

#[cfg(feature = "mongodb")]
/// Deserialize result: (ObjectId, f64, f64, bool),
/// regardless it's ObjectId or String at the first element
pub fn deserialize_connected_areas<'de, D>(deserializer: D) -> Result<Vec<ConnectedArea>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{SeqAccess, Visitor};
    use std::fmt;

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
            use bson::oid::ObjectId;
            use std::str::FromStr;
            while let Some(id_str) = seq.next_element::<String>()? {
                let id = ObjectId::from_str(&id_str).map_err(serde::de::Error::custom)?;
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
            Ok(areas)
        }
    }

    deserializer.deserialize_seq(ConnectedAreasVisitor)
}
