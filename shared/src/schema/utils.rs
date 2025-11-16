#[cfg(feature = "mongodb")]
use super::ConnectedArea;

#[cfg(all(feature = "mongodb", feature = "serde"))]
use bson::oid::ObjectId;

/// Serialize Option<ObjectId> as Option<hex_string>
#[cfg(all(feature = "mongodb", feature = "serde"))]
pub fn serialize_option_object_id_as_hex_string<S>(
    val: &Option<ObjectId>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match val {
        Some(oid) => bson::serde_helpers::serialize_object_id_as_hex_string(oid, serializer),
        None => serializer.serialize_none(),
    }
}

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
