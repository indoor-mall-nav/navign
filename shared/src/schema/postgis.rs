//! PostGIS geometry type support for sqlx
//!
//! Provides a wrapper around geo_types::Point that implements sqlx's
//! Type, Encode, and Decode traits for PostGIS GEOMETRY(POINT) columns.

#[cfg(all(feature = "postgres", feature = "geo"))]
use geo_traits::to_geo::{ToGeoPoint, ToGeoPolygon};
#[cfg(all(feature = "postgres", feature = "geo"))]
use geo_traits::{GeometryTrait, GeometryType};
#[cfg(feature = "geo")]
use geo_types::{Point, Polygon};
#[cfg(feature = "postgres")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "postgres")]
use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef};
#[cfg(feature = "postgres")]
use sqlx::{Decode, Encode, Postgres, Type};
use wkt::TryFromWkt;

/// Wrapper around geo_types::Point for PostGIS GEOMETRY(POINT, 4326)
///
/// This type handles the encoding and decoding of PostGIS points in WKB (Well-Known Binary) format.
/// It uses SRID 4326 (WGS84) which is the standard for GPS coordinates.
#[cfg(feature = "postgres")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PgPoint(pub Point<f64>);

#[cfg(all(feature = "serde", feature = "postgres"))]
impl Serialize for PgPoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Serialize to `[x, y]` array
        let coords = [self.lon(), self.lat()];
        coords.serialize(serializer)
    }
}

#[cfg(all(feature = "serde", feature = "postgres"))]
impl<'de> Deserialize<'de> for PgPoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Deserialize from `[x, y]` array
        let coords: [f64; 2] = Deserialize::deserialize(deserializer)?;
        Ok(PgPoint::new(coords[0], coords[1]))
    }
}

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

    #[cfg(feature = "geo")]
    pub fn to_wkb(&self) -> Result<Vec<u8>, wkb::error::WkbError> {
        let mut buffer = Vec::new();
        wkb::writer::write_point(
            &mut buffer,
            &self.0,
            &wkb::writer::WriteOptions {
                endianness: wkb::Endianness::LittleEndian,
            },
        )?;
        Ok(buffer)
    }

    #[cfg(feature = "geo")]
    pub fn from_wkb(bytes: &[u8]) -> Result<Self, wkb::error::WkbError> {
        let data = wkb::reader::read_wkb(bytes)?;
        let point = data.as_type();
        if let GeometryType::Point(pt) = point {
            let pt = pt.to_point();
            Ok(Self(pt))
        } else {
            Err(wkb::error::WkbError::General(
                "WKB does not represent a Point".to_string(),
            ))
        }
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
        let wkb = (*self).to_wkb()?;
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

#[cfg(feature = "postgres")]
#[derive(Debug, Clone, PartialEq)]
pub struct PgPolygon(pub Polygon<f64>);

#[cfg(all(feature = "serde", feature = "postgres"))]
impl Serialize for PgPolygon {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Serialize to [(x1, y1), (x2, y2), ...] array
        let coords: Vec<(f64, f64)> = self
            .0
            .exterior()
            .0
            .iter()
            .map(|coord| (coord.x, coord.y))
            .collect();
        coords.serialize(serializer)
    }
}

#[cfg(all(feature = "serde", feature = "postgres"))]
impl<'de> Deserialize<'de> for PgPolygon {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Deserialize from [(x1, y1), (x2, y2), ...] array
        let coords: Vec<(f64, f64)> = Deserialize::deserialize(deserializer)?;
        Ok(PgPolygon::new(coords))
    }
}

#[cfg(feature = "postgres")]
impl PgPolygon {
    pub fn new(points: Vec<(f64, f64)>) -> Self {
        let exterior = geo_types::LineString::new(
            points
                .into_iter()
                .map(|(x, y)| geo_types::Coord { x, y })
                .collect(),
        );
        let polygon = Polygon::new(exterior, vec![]);
        Self(polygon)
    }

    pub fn inner(&self) -> &Polygon<f64> {
        &self.0
    }

    #[cfg(feature = "geo")]
    pub fn to_wkb(self) -> Result<Vec<u8>, wkb::error::WkbError> {
        let mut buffer = Vec::new();
        wkb::writer::write_polygon(
            &mut buffer,
            &self.0,
            &wkb::writer::WriteOptions {
                endianness: wkb::Endianness::LittleEndian,
            },
        )?;
        Ok(buffer)
    }

    #[cfg(feature = "geo")]
    pub fn from_wkb(bytes: &[u8]) -> Result<Self, wkb::error::WkbError> {
        let data = wkb::reader::read_wkb(bytes)?;
        let polygon = data.as_type();
        if let GeometryType::Polygon(pg) = polygon {
            let pg = pg.to_polygon();
            Ok(Self(pg))
        } else {
            Err(wkb::error::WkbError::General(
                "WKB does not represent a Polygon".to_string(),
            ))
        }
    }
}

#[cfg(feature = "postgres")]
impl Type<Postgres> for PgPolygon {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("geometry")
    }
}

#[cfg(feature = "postgres")]
impl<'q> Encode<'q, Postgres> for PgPolygon {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, Box<dyn core::error::Error + Send + Sync>> {
        let wkb = self.clone().to_wkb()?;
        buf.extend_from_slice(&wkb);
        Ok(sqlx::encode::IsNull::No)
    }
}

#[cfg(feature = "postgres")]
impl<'r> Decode<'r, Postgres> for PgPolygon {
    fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn core::error::Error + Send + Sync>> {
        let bytes = <&[u8] as Decode<Postgres>>::decode(value)?;
        Self::from_wkb(bytes).map_err(|e| e.into())
    }
}

#[cfg(feature = "postgres")]
impl From<Polygon<f64>> for PgPolygon {
    fn from(polygon: Polygon<f64>) -> Self {
        Self(polygon)
    }
}

#[cfg(feature = "postgres")]
impl From<PgPolygon> for Polygon<f64> {
    fn from(pg_polygon: PgPolygon) -> Self {
        pg_polygon.0
    }
}

// Helper functions for SQLite WKB conversion (works with raw tuples)
#[cfg(feature = "geo")]
pub fn point_to_wkb(point: (f64, f64)) -> Result<Vec<u8>, wkb::error::WkbError> {
    let geo_point = geo_types::Point::new(point.0, point.1);
    let mut buffer = Vec::new();
    wkb::writer::write_point(
        &mut buffer,
        &geo_point,
        &wkb::writer::WriteOptions {
            endianness: wkb::Endianness::LittleEndian,
        },
    )?;
    Ok(buffer)
}

#[cfg(feature = "geo")]
pub fn wkb_to_point(bytes: &[u8]) -> Result<(f64, f64), wkb::error::WkbError> {
    use geo_traits::to_geo::ToGeoPoint;
    use geo_traits::{GeometryTrait, GeometryType};

    let data = wkb::reader::read_wkb(bytes)?;
    let geom = data.as_type();
    if let GeometryType::Point(pt) = geom {
        let point = pt.to_point();
        Ok((point.x(), point.y()))
    } else {
        Err(wkb::error::WkbError::General(
            "WKB does not represent a Point".to_string(),
        ))
    }
}

#[cfg(feature = "geo")]
pub fn polygon_to_wkb(points: &[(f64, f64)]) -> Result<Vec<u8>, wkb::error::WkbError> {
    use geo_types::Coord;

    if points.is_empty() {
        return Err(wkb::error::WkbError::General("Empty polygon".to_string()));
    }

    let coords: Vec<Coord<f64>> = points.iter().map(|(x, y)| Coord { x: *x, y: *y }).collect();

    let line_string = geo_types::LineString::new(coords);
    let polygon = geo_types::Polygon::new(line_string, vec![]);

    let mut buffer = Vec::new();
    wkb::writer::write_polygon(
        &mut buffer,
        &polygon,
        &wkb::writer::WriteOptions {
            endianness: wkb::Endianness::LittleEndian,
        },
    )?;
    Ok(buffer)
}

#[cfg(feature = "geo")]
pub fn wkb_to_polygon(bytes: &[u8]) -> Result<Vec<(f64, f64)>, wkb::error::WkbError> {
    use geo_traits::to_geo::ToGeoPolygon;
    use geo_traits::{GeometryTrait, GeometryType};

    let data = wkb::reader::read_wkb(bytes)?;
    let geom = data.as_type();
    if let GeometryType::Polygon(pg) = geom {
        let polygon = pg.to_polygon();
        let coords: Vec<(f64, f64)> = polygon
            .exterior()
            .0
            .iter()
            .map(|coord| (coord.x, coord.y))
            .collect();
        Ok(coords)
    } else {
        Err(wkb::error::WkbError::General(
            "WKB does not represent a Polygon".to_string(),
        ))
    }
}

#[cfg(feature = "geo")]
pub fn wkt_to_point(wkt_str: &str) -> Result<(f64, f64), wkt::geo_types_from_wkt::Error> {
    Point::<f64>::try_from_wkt_str(wkt_str).map(|pt| (pt.x(), pt.y()))
}

#[cfg(feature = "geo")]
pub fn point_to_wkt(point: (f64, f64)) -> String {
    let pt = Point::new(point.0, point.1);
    format!("POINT({} {})", pt.x(), pt.y())
}

#[cfg(feature = "geo")]
pub fn wkt_to_polygon(wkt_str: &str) -> Result<Vec<(f64, f64)>, wkt::geo_types_from_wkt::Error> {
    let polygon = Polygon::<f64>::try_from_wkt_str(wkt_str)?;
    let coords: Vec<(f64, f64)> = polygon
        .exterior()
        .0
        .iter()
        .map(|coord| (coord.x, coord.y))
        .collect();
    Ok(coords)
}

#[cfg(feature = "geo")]
pub fn polygon_to_wkt(points: &[(f64, f64)]) -> String {
    let coords_str: Vec<String> = points.iter().map(|(x, y)| format!("{} {}", x, y)).collect();
    format!("POLYGON(({}))", coords_str.join(", "))
}
