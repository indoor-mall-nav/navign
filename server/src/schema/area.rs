use crate::schema::metadata::{PaginationResponse, PaginationResponseMetadata};
use crate::schema::service::{SearchQueryParams, Service};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

// Re-export from navign-shared
pub use navign_shared::{Area, Floor, FloorType};

fn parse_wkt_polygon(wkt: &str) -> Vec<(f64, f64)> {
    // Parse WKT POLYGON format: "POLYGON((x1 y1, x2 y2, ...))"
    let coords_str = wkt
        .trim_start_matches("POLYGON((")
        .trim_end_matches("))")
        .trim();

    coords_str
        .split(',')
        .filter_map(|pair| {
            let mut parts = pair.trim().split_whitespace();
            let x = parts.next()?.parse::<f64>().ok()?;
            let y = parts.next()?.parse::<f64>().ok()?;
            Some((x, y))
        })
        .collect()
}

fn polygon_to_wkt(polygon: &[(f64, f64)]) -> String {
    let coords: Vec<String> = polygon
        .iter()
        .map(|(x, y)| format!("{} {}", x, y))
        .collect();
    format!("POLYGON(({}))", coords.join(", "))
}

#[async_trait]
impl Service for Area {
    type Id = i64;

    fn get_id(&self) -> i64 {
        self.id.expect("Area must have an ID")
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn set_name(&mut self, name: String) {
        self.name = name;
    }

    fn get_description(&self) -> Option<String> {
        self.description.clone()
    }

    fn set_description(&mut self, description: Option<String>) {
        self.description = description;
    }

    fn get_table_name() -> &'static str {
        "areas"
    }

    fn require_unique_name() -> bool {
        false
    }

    async fn get_one_by_id(pool: &PgPool, id: i64) -> Result<Option<Self>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT
                id,
                entity,
                name,
                description,
                beacon_code,
                floor_type,
                floor_name,
                ST_AsText(polygon) as polygon_wkt,
                created_at,
                updated_at
            FROM areas WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|r| {
            let floor = match (r.floor_type, r.floor_name) {
                (Some(ft), Some(fn_)) => {
                    let floor_type = match ft.as_str() {
                        "level" => FloorType::Level,
                        "floor" => FloorType::Floor,
                        "basement" => FloorType::Basement,
                        _ => FloorType::Floor,
                    };
                    Some(Floor {
                        r#type: floor_type,
                        name: fn_ as u32,
                    })
                }
                _ => None,
            };

            let polygon = parse_wkt_polygon(&r.polygon_wkt.unwrap_or_default());

            Area {
                id: Some(r.id),
                entity: r.entity,
                name: r.name,
                description: r.description,
                beacon_code: r.beacon_code,
                floor,
                polygon,
                created_at: r.created_at,
                updated_at: r.updated_at,
            }
        }))
    }

    async fn get_one_by_name(pool: &PgPool, name: &str) -> Result<Option<Self>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT
                id,
                entity,
                name,
                description,
                beacon_code,
                floor_type,
                floor_name,
                ST_AsText(polygon) as polygon_wkt,
                created_at,
                updated_at
            FROM areas WHERE name = $1 LIMIT 1
            "#,
            name
        )
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|r| {
            let floor = match (r.floor_type, r.floor_name) {
                (Some(ft), Some(fn_)) => {
                    let floor_type = match ft.as_str() {
                        "level" => FloorType::Level,
                        "floor" => FloorType::Floor,
                        "basement" => FloorType::Basement,
                        _ => FloorType::Floor,
                    };
                    Some(Floor {
                        r#type: floor_type,
                        name: fn_ as u32,
                    })
                }
                _ => None,
            };

            let polygon = parse_wkt_polygon(&r.polygon_wkt.unwrap_or_default());

            Area {
                id: Some(r.id),
                entity: r.entity,
                name: r.name,
                description: r.description,
                beacon_code: r.beacon_code,
                floor,
                polygon,
                created_at: r.created_at,
                updated_at: r.updated_at,
            }
        }))
    }

    async fn get_all(pool: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT
                id,
                entity,
                name,
                description,
                beacon_code,
                floor_type,
                floor_name,
                ST_AsText(polygon) as polygon_wkt,
                created_at,
                updated_at
            FROM areas
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| {
                let floor = match (r.floor_type, r.floor_name) {
                    (Some(ft), Some(fn_)) => {
                        let floor_type = match ft.as_str() {
                            "level" => FloorType::Level,
                            "floor" => FloorType::Floor,
                            "basement" => FloorType::Basement,
                            _ => FloorType::Floor,
                        };
                        Some(Floor {
                            r#type: floor_type,
                            name: fn_ as u32,
                        })
                    }
                    _ => None,
                };

                let polygon = parse_wkt_polygon(&r.polygon_wkt.unwrap_or_default());

                Area {
                    id: Some(r.id),
                    entity: r.entity,
                    name: r.name,
                    description: r.description,
                    beacon_code: r.beacon_code,
                    floor,
                    polygon,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                }
            })
            .collect())
    }

    async fn get_with_pagination(
        pool: &PgPool,
        page: i64,
        limit: i64,
        sort: Option<&str>,
        asc: bool,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let offset = page * limit;
        let sort_column = sort.unwrap_or("name");
        let order = if asc { "ASC" } else { "DESC" };

        let query = format!(
            r#"
            SELECT
                id, entity, name, description,
                beacon_code, floor_type, floor_name,
                ST_AsText(polygon) as polygon_wkt,
                created_at, updated_at
            FROM areas
            ORDER BY {} {}
            LIMIT $1 OFFSET $2
            "#,
            sort_column, order
        );

        let rows = sqlx::query(&query)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let id: i64 = row.get("id");
                let entity: Uuid = row.get("entity");
                let name: String = row.get("name");
                let description: Option<String> = row.get("description");
                let beacon_code: String = row.get("beacon_code");
                let floor_type: Option<String> = row.get("floor_type");
                let floor_name: Option<i32> = row.get("floor_name");
                let polygon_wkt: String = row.get("polygon_wkt");
                let created_at: i64 = row.get("created_at");
                let updated_at: i64 = row.get("updated_at");

                let floor = match (floor_type, floor_name) {
                    (Some(ft), Some(fn_)) => {
                        let floor_type = match ft.as_str() {
                            "level" => FloorType::Level,
                            "floor" => FloorType::Floor,
                            "basement" => FloorType::Basement,
                            _ => FloorType::Floor,
                        };
                        Some(Floor {
                            r#type: floor_type,
                            name: fn_ as u32,
                        })
                    }
                    _ => None,
                };

                let polygon = parse_wkt_polygon(&polygon_wkt);

                Area {
                    id: Some(id),
                    entity,
                    name,
                    description,
                    beacon_code,
                    floor,
                    polygon,
                    created_at,
                    updated_at,
                }
            })
            .collect())
    }

    async fn create(&self, pool: &PgPool) -> Result<i64, sqlx::Error> {
        let now = chrono::Utc::now().timestamp_millis();
        let wkt = polygon_to_wkt(&self.polygon);

        let (floor_type, floor_name) = match &self.floor {
            Some(floor) => {
                let ft = match floor.r#type {
                    FloorType::Level => "level",
                    FloorType::Floor => "floor",
                    FloorType::Basement => "basement",
                };
                (Some(ft), Some(floor.name as i32))
            }
            None => (None, None),
        };

        let result = sqlx::query!(
            r#"
            INSERT INTO areas (
                entity, name, description, beacon_code,
                floor_type, floor_name, polygon,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, ST_GeomFromText($7, 4326), $8, $9)
            RETURNING id
            "#,
            self.entity,
            self.name,
            self.description,
            self.beacon_code,
            floor_type,
            floor_name,
            wkt,
            now,
            now
        )
        .fetch_one(pool)
        .await?;

        Ok(result.id)
    }

    async fn update(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        let now = chrono::Utc::now().timestamp_millis();
        let id = self.id.expect("Area must have an ID for update");
        let wkt = polygon_to_wkt(&self.polygon);

        let (floor_type, floor_name) = match &self.floor {
            Some(floor) => {
                let ft = match floor.r#type {
                    FloorType::Level => "level",
                    FloorType::Floor => "floor",
                    FloorType::Basement => "basement",
                };
                (Some(ft), Some(floor.name as i32))
            }
            None => (None, None),
        };

        sqlx::query!(
            r#"
            UPDATE areas SET
                entity = $1, name = $2, description = $3,
                beacon_code = $4, floor_type = $5, floor_name = $6,
                polygon = ST_GeomFromText($7, 4326), updated_at = $8
            WHERE id = $9
            "#,
            self.entity,
            self.name,
            self.description,
            self.beacon_code,
            floor_type,
            floor_name,
            wkt,
            now,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn delete_by_id(pool: &PgPool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM areas WHERE id = $1", id)
            .execute(pool)
            .await?;
        Ok(())
    }

    async fn delete_by_name(pool: &PgPool, name: &str) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM areas WHERE name = $1", name)
            .execute(pool)
            .await?;
        Ok(())
    }

    async fn search_and_page_by_name_pattern(
        pool: &PgPool,
        params: SearchQueryParams<'_>,
    ) -> Result<PaginationResponse<Self>, sqlx::Error> {
        let sort_column = params.sort.unwrap_or("name");
        let order = if params.asc { "ASC" } else { "DESC" };
        let pattern = if params.case_insensitive {
            format!("%{}%", params.pattern.to_lowercase())
        } else {
            format!("%{}%", params.pattern)
        };

        let entity_uuid = Uuid::parse_str(params.entity)
            .map_err(|e| sqlx::Error::Protocol(format!("Invalid entity UUID: {}", e)))?;

        let count_query = if params.case_insensitive {
            "SELECT COUNT(*) as count FROM areas WHERE entity = $1 AND LOWER(name) LIKE $2"
        } else {
            "SELECT COUNT(*) as count FROM areas WHERE entity = $1 AND name LIKE $2"
        };

        let total: i64 = sqlx::query_scalar(count_query)
            .bind(entity_uuid)
            .bind(&pattern)
            .fetch_one(pool)
            .await?;

        let query = format!(
            r#"
            SELECT
                id, entity, name, description,
                beacon_code, floor_type, floor_name,
                ST_AsText(polygon) as polygon_wkt,
                created_at, updated_at
            FROM areas
            WHERE entity = $1 AND {} LIKE $2
            ORDER BY {} {}
            LIMIT $3 OFFSET $4
            "#,
            if params.case_insensitive {
                "LOWER(name)"
            } else {
                "name"
            },
            sort_column,
            order
        );

        let rows = sqlx::query(&query)
            .bind(entity_uuid)
            .bind(&pattern)
            .bind(params.limit)
            .bind(params.offset)
            .fetch_all(pool)
            .await?;

        let items = rows
            .into_iter()
            .map(|row| {
                let id: i64 = row.get("id");
                let entity: Uuid = row.get("entity");
                let name: String = row.get("name");
                let description: Option<String> = row.get("description");
                let beacon_code: String = row.get("beacon_code");
                let floor_type: Option<String> = row.get("floor_type");
                let floor_name: Option<i32> = row.get("floor_name");
                let polygon_wkt: String = row.get("polygon_wkt");
                let created_at: i64 = row.get("created_at");
                let updated_at: i64 = row.get("updated_at");

                let floor = match (floor_type, floor_name) {
                    (Some(ft), Some(fn_)) => {
                        let floor_type = match ft.as_str() {
                            "level" => FloorType::Level,
                            "floor" => FloorType::Floor,
                            "basement" => FloorType::Basement,
                            _ => FloorType::Floor,
                        };
                        Some(Floor {
                            r#type: floor_type,
                            name: fn_ as u32,
                        })
                    }
                    _ => None,
                };

                let polygon = parse_wkt_polygon(&polygon_wkt);

                Area {
                    id: Some(id),
                    entity,
                    name,
                    description,
                    beacon_code,
                    floor,
                    polygon,
                    created_at,
                    updated_at,
                }
            })
            .collect();

        Ok(PaginationResponse {
            items,
            metadata: PaginationResponseMetadata {
                total,
                offset: params.offset,
                limit: params.limit,
            },
        })
    }

    async fn search_by_description_pattern(
        pool: &PgPool,
        pattern: &str,
        case_insensitive: bool,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let pattern = if case_insensitive {
            format!("%{}%", pattern.to_lowercase())
        } else {
            format!("%{}%", pattern)
        };

        let query = if case_insensitive {
            r#"
            SELECT
                id, entity, name, description,
                beacon_code, floor_type, floor_name,
                ST_AsText(polygon) as polygon_wkt,
                created_at, updated_at
            FROM areas
            WHERE LOWER(description) LIKE $1
            "#
        } else {
            r#"
            SELECT
                id, entity, name, description,
                beacon_code, floor_type, floor_name,
                ST_AsText(polygon) as polygon_wkt,
                created_at, updated_at
            FROM areas
            WHERE description LIKE $1
            "#
        };

        let rows = sqlx::query(query).bind(&pattern).fetch_all(pool).await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let id: i64 = row.get("id");
                let entity: Uuid = row.get("entity");
                let name: String = row.get("name");
                let description: Option<String> = row.get("description");
                let beacon_code: String = row.get("beacon_code");
                let floor_type: Option<String> = row.get("floor_type");
                let floor_name: Option<i32> = row.get("floor_name");
                let polygon_wkt: String = row.get("polygon_wkt");
                let created_at: i64 = row.get("created_at");
                let updated_at: i64 = row.get("updated_at");

                let floor = match (floor_type, floor_name) {
                    (Some(ft), Some(fn_)) => {
                        let floor_type = match ft.as_str() {
                            "level" => FloorType::Level,
                            "floor" => FloorType::Floor,
                            "basement" => FloorType::Basement,
                            _ => FloorType::Floor,
                        };
                        Some(Floor {
                            r#type: floor_type,
                            name: fn_ as u32,
                        })
                    }
                    _ => None,
                };

                let polygon = parse_wkt_polygon(&polygon_wkt);

                Area {
                    id: Some(id),
                    entity,
                    name,
                    description,
                    beacon_code,
                    floor,
                    polygon,
                    created_at,
                    updated_at,
                }
            })
            .collect())
    }

    async fn bulk_create(pool: &PgPool, areas: Vec<Self>) -> Result<Vec<i64>, sqlx::Error> {
        let mut ids = Vec::with_capacity(areas.len());

        for area in areas {
            let id = area.create(pool).await?;
            ids.push(id);
        }

        Ok(ids)
    }
}
