//! PostGIS geometry type support for sqlx
//!
//! Provides a wrapper around geo_types::Point that implements sqlx's
//! Type, Encode, and Decode traits for PostGIS GEOMETRY(POINT) columns.

#[cfg(feature = "postgres")]
use geo_types::Point;
#[cfg(feature = "postgres")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "postgres")]
use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef};
#[cfg(feature = "postgres")]
use sqlx::{Decode, Encode, Postgres, Type};

/// Wrapper around geo_types::Point for PostGIS GEOMETRY(POINT, 4326)
///
/// This type handles the encoding and decoding of PostGIS points in WKB (Well-Known Binary) format.
/// It uses SRID 4326 (WGS84) which is the standard for GPS coordinates.
#[cfg(feature = "postgres")]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PgPoint(pub Point<f64>);

#[cfg(feature = "postgres")]
impl PgPoint {
    /// Create a new PostGIS point from x (longitude) and y (latitude) coordinates
    pub fn new(x: f64, y: f64) -> Self {
        Self(Point::new(x, y))
    }

    /// Get the inner geo_types::Point
    pub fn inner(&self) -> &Point<f64> {
        &self.0
    }

    /// Get the longitude (x coordinate)
    pub fn lon(&self) -> f64 {
        self.0.x()
    }

    /// Get the latitude (y coordinate)
    pub fn lat(&self) -> f64 {
        self.0.y()
    }

    /// Encode to PostGIS WKB (Well-Known Binary) format with SRID
    ///
    /// Format: [byte_order(1)] [wkb_type(4)] [srid(4)] [x(8)] [y(8)]
    /// Total: 21 bytes for POINT with SRID
    fn to_wkb(self) -> Vec<u8> {
        let mut wkb = Vec::with_capacity(21);

        // Byte order: 1 = little endian
        wkb.push(1u8);

        // WKB type: 0x20000001 = POINT with SRID (0x20000000 flag + 0x00000001 for POINT)
        wkb.extend_from_slice(&0x20000001u32.to_le_bytes());

        // SRID: 4326 (WGS84)
        wkb.extend_from_slice(&4326u32.to_le_bytes());

        // X coordinate (longitude)
        wkb.extend_from_slice(&self.0.x().to_le_bytes());

        // Y coordinate (latitude)
        wkb.extend_from_slice(&self.0.y().to_le_bytes());

        wkb
    }

    /// Decode from PostGIS WKB (Well-Known Binary) format
    fn from_wkb(wkb: &[u8]) -> Result<Self, String> {
        if wkb.len() < 21 {
            return Err(format!(
                "WKB too short: expected at least 21 bytes, got {}",
                wkb.len()
            ));
        }

        // Check byte order (we only support little endian for now)
        let byte_order = wkb[0];
        if byte_order != 1 {
            return Err(format!("Unsupported byte order: {}", byte_order));
        }

        // Read WKB type
        let wkb_type = u32::from_le_bytes([wkb[1], wkb[2], wkb[3], wkb[4]]);

        // Check if it's a POINT (type 1) or POINT with SRID (type 0x20000001)
        let has_srid = (wkb_type & 0x20000000) != 0;
        let base_type = wkb_type & 0x1FFFFFFF;

        if base_type != 1 {
            return Err(format!("Not a POINT geometry: type {}", base_type));
        }

        let mut offset = 5;

        // Skip SRID if present
        if has_srid {
            offset += 4;
        }

        if wkb.len() < offset + 16 {
            return Err(format!(
                "WKB too short for coordinates: expected at least {} bytes",
                offset + 16
            ));
        }

        // Read X coordinate (longitude)
        let x = f64::from_le_bytes([
            wkb[offset],
            wkb[offset + 1],
            wkb[offset + 2],
            wkb[offset + 3],
            wkb[offset + 4],
            wkb[offset + 5],
            wkb[offset + 6],
            wkb[offset + 7],
        ]);

        // Read Y coordinate (latitude)
        let y = f64::from_le_bytes([
            wkb[offset + 8],
            wkb[offset + 9],
            wkb[offset + 10],
            wkb[offset + 11],
            wkb[offset + 12],
            wkb[offset + 13],
            wkb[offset + 14],
            wkb[offset + 15],
        ]);

        Ok(Self::new(x, y))
    }
}

#[cfg(feature = "postgres")]
impl Type<Postgres> for PgPoint {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("geometry")
    }
}

#[cfg(feature = "postgres")]
impl<'q> Encode<'q, Postgres> for PgPoint {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let wkb = (*self).to_wkb();
        buf.extend_from_slice(&wkb);
        Ok(sqlx::encode::IsNull::No)
    }
}

#[cfg(feature = "postgres")]
impl<'r> Decode<'r, Postgres> for PgPoint {
    fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let bytes = <&[u8] as Decode<Postgres>>::decode(value)?;
        Self::from_wkb(bytes).map_err(|e| e.into())
    }
}

#[cfg(feature = "postgres")]
impl From<Point<f64>> for PgPoint {
    fn from(point: Point<f64>) -> Self {
        Self(point)
    }
}

#[cfg(feature = "postgres")]
impl From<PgPoint> for Point<f64> {
    fn from(pg_point: PgPoint) -> Self {
        pg_point.0
    }
}

#[cfg(test)]
#[cfg(feature = "postgres")]
mod tests {
    use super::*;

    #[test]
    fn test_wkb_encoding_decoding() {
        let point = PgPoint::new(121.5, 31.2);
        let wkb = point.to_wkb();

        assert_eq!(wkb.len(), 25);
        assert_eq!(wkb[0], 1); // Little endian

        let decoded = PgPoint::from_wkb(&wkb).unwrap();
        assert_eq!(decoded.lon(), 121.5);
        assert_eq!(decoded.lat(), 31.2);
    }

    #[test]
    fn test_from_geo_types() {
        let geo_point = Point::new(120.0, 30.0);
        let pg_point = PgPoint::from(geo_point);
        assert_eq!(pg_point.lon(), 120.0);
        assert_eq!(pg_point.lat(), 30.0);
    }
}
