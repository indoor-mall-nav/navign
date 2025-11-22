#!/usr/bin/env python3
"""
MongoDB to PostgreSQL Migration Script for Navign Indoor Navigation System

This script migrates data from MongoDB collections to PostgreSQL tables,
handling data type conversions, UUID generation, and PostGIS geometry types.

Requirements:
    pip install pymongo psycopg2-binary python-dotenv

Usage:
    python mongodb_to_postgres_migration.py
"""

import os
import sys
from datetime import datetime
from typing import Dict, List, Any, Optional, Tuple
import json

try:
    from pymongo import MongoClient
    import psycopg2
    from psycopg2.extras import execute_batch
    from psycopg2.extensions import AsIs
except ImportError:
    print("Error: Required packages not installed")
    print("Please run: pip install pymongo psycopg2-binary")
    sys.exit(1)


class MongoToPostgresMigration:
    """Handles migration from MongoDB to PostgreSQL"""

    def __init__(
        self, mongo_uri: str, pg_connection_string: str, batch_size: int = 100
    ):
        """
        Initialize migration handler

        Args:
            mongo_uri: MongoDB connection URI
            pg_connection_string: PostgreSQL connection string
            batch_size: Number of records to process in each batch
        """
        self.mongo_client = MongoClient(mongo_uri)
        self.pg_conn = psycopg2.connect(pg_connection_string)
        self.pg_cursor = self.pg_conn.cursor()
        self.batch_size = batch_size

        # ID mapping from MongoDB ObjectId to PostgreSQL UUID/INT
        self.entity_id_map: Dict[str, str] = {}  # MongoDB _id -> PostgreSQL UUID
        self.area_id_map: Dict[str, int] = {}  # MongoDB _id -> PostgreSQL INT
        self.merchant_id_map: Dict[str, int] = {}  # MongoDB _id -> PostgreSQL INT
        self.connection_id_map: Dict[str, int] = {}  # MongoDB _id -> PostgreSQL INT
        self.beacon_id_map: Dict[str, int] = {}  # MongoDB _id -> PostgreSQL INT
        self.user_id_map: Dict[str, str] = {}  # MongoDB _id -> PostgreSQL UUID

        self.stats = {
            "entities": 0,
            "users": 0,
            "areas": 0,
            "merchants": 0,
            "beacons": 0,
            "connections": 0,
            "errors": [],
        }

    def log(self, message: str):
        """Print timestamped log message"""
        print(f"[{datetime.now().strftime('%Y-%m-%d %H:%M:%S')}] {message}")

    def convert_timestamp(self, mongo_timestamp: Any) -> Optional[datetime]:
        """Convert MongoDB timestamp to PostgreSQL TIMESTAMP WITH TIME ZONE"""
        if not mongo_timestamp:
            return None

        # Handle $numberLong format
        if isinstance(mongo_timestamp, dict) and "$numberLong" in mongo_timestamp:
            timestamp_ms = int(mongo_timestamp["$numberLong"])
        elif isinstance(mongo_timestamp, (int, float)):
            timestamp_ms = int(mongo_timestamp)
        else:
            return None

        # Convert milliseconds to datetime
        return datetime.fromtimestamp(timestamp_ms / 1000.0)

    def convert_oid_to_string(self, oid: Any) -> Optional[str]:
        """Convert MongoDB ObjectId to string"""
        if not oid:
            return None

        if isinstance(oid, dict) and "$oid" in oid:
            return oid["$oid"]
        elif hasattr(oid, "__str__"):
            return str(oid)
        return None

    def point_to_postgis(self, coordinates: List[float]) -> str:
        """Convert [x, y] coordinates to PostGIS POINT format"""
        if not coordinates or len(coordinates) < 2:
            return None
        return f"SRID=4326;POINT({coordinates[0]} {coordinates[1]})"

    def polygon_to_postgis(self, coordinates: List[List[float]]) -> Optional[str]:
        """Convert list of [x, y] coordinates to PostGIS POLYGON format"""
        if not coordinates or len(coordinates) < 3:
            return None

        # Close the polygon if not already closed
        polygon_coords = coordinates.copy()
        if polygon_coords[0] != polygon_coords[-1]:
            polygon_coords.append(polygon_coords[0])

        # Format as WKT: POLYGON((x1 y1, x2 y2, ...))
        points_str = ", ".join([f"{coord[0]} {coord[1]}" for coord in polygon_coords])
        return f"SRID=4326;POLYGON(({points_str}))"

    def migrate_entities(self, db_name: str):
        """Migrate entities from MongoDB to PostgreSQL"""
        self.log("Migrating entities...")

        mongo_db = self.mongo_client[db_name]
        entities_collection = mongo_db["entities"]

        total = entities_collection.count_documents({})
        self.log(f"Found {total} entities to migrate")

        batch = []
        processed = 0

        for entity_doc in entities_collection.find():
            try:
                entity_id_str = self.convert_oid_to_string(entity_doc["_id"])

                # Extract longitude and latitude ranges
                lon_range = entity_doc.get("longitude_range", [0, 0])
                lat_range = entity_doc.get("latitude_range", [0, 0])
                alt_range = entity_doc.get("altitude_range", [0, 0])

                # Create point_min and point_max
                point_min = self.point_to_postgis([lon_range[0], lat_range[0]])
                point_max = self.point_to_postgis([lon_range[1], lat_range[1]])

                record = {
                    "type": entity_doc.get("type", ""),
                    "name": entity_doc.get("name", ""),
                    "description": entity_doc.get("description", ""),
                    "point_min": point_min,
                    "point_max": point_max,
                    "altitude_min": alt_range[0] if len(alt_range) > 0 else None,
                    "altitude_max": alt_range[1] if len(alt_range) > 1 else None,
                    "nation": entity_doc.get("nation"),
                    "region": entity_doc.get("region"),
                    "city": entity_doc.get("city"),
                    "tags": entity_doc.get("tags", []),
                    "created_at": self.convert_timestamp(entity_doc.get("created_at")),
                    "updated_at": self.convert_timestamp(entity_doc.get("updated_at")),
                }

                batch.append((entity_id_str, record))

                if len(batch) >= self.batch_size:
                    self._insert_entities_batch(batch)
                    processed += len(batch)
                    self.log(f"Processed {processed}/{total} entities")
                    batch = []

            except Exception as e:
                error_msg = f"Error processing entity {entity_doc.get('_id')}: {str(e)}"
                self.log(error_msg)
                self.stats["errors"].append(error_msg)

        # Process remaining batch
        if batch:
            self._insert_entities_batch(batch)
            processed += len(batch)

        self.log(f"Completed: {processed} entities migrated")
        self.stats["entities"] = processed

    def _insert_entities_batch(self, batch: List[Tuple[str, Dict]]):
        """Insert batch of entities into PostgreSQL"""
        query = """
            INSERT INTO entities (
                type, name, description, point_min, point_max,
                altitude_min, altitude_max, nation, region, city, tags,
                created_at, updated_at
            ) VALUES (
                %(type)s, %(name)s, %(description)s, ST_GeomFromEWKT(%(point_min)s), 
                ST_GeomFromEWKT(%(point_max)s), %(altitude_min)s, %(altitude_max)s,
                %(nation)s, %(region)s, %(city)s, %(tags)s,
                %(created_at)s, %(updated_at)s
            ) RETURNING id
        """

        for mongo_id, record in batch:
            try:
                self.pg_cursor.execute(query, record)
                pg_uuid = self.pg_cursor.fetchone()[0]
                self.entity_id_map[mongo_id] = str(pg_uuid)
            except Exception as e:
                self.log(f"Error inserting entity {mongo_id}: {str(e)}")
                self.pg_conn.rollback()
                raise

        self.pg_conn.commit()

    def migrate_users(self, db_name: str):
        """Migrate users from MongoDB to PostgreSQL"""
        self.log("Migrating users...")

        mongo_db = self.mongo_client[db_name]
        users_collection = mongo_db.get_collection("users")

        if users_collection is None:
            self.log("No users collection found, skipping...")
            return

        total = users_collection.count_documents({})
        self.log(f"Found {total} users to migrate")

        batch = []
        processed = 0

        for user_doc in users_collection.find():
            try:
                user_id_str = self.convert_oid_to_string(user_doc["_id"])

                record = {
                    "username": user_doc.get("username", ""),
                    "email": user_doc.get("email", ""),
                    "phone": user_doc.get("phone"),
                    "google": user_doc.get("google"),
                    "wechat": user_doc.get("wechat"),
                    "hashed_password": user_doc.get(
                        "hashed_password", user_doc.get("password", "")
                    ),
                    "activated": user_doc.get("activated", False),
                    "privileged": user_doc.get("privileged", False),
                    "created_at": self.convert_timestamp(user_doc.get("created_at")),
                    "updated_at": self.convert_timestamp(user_doc.get("updated_at")),
                }

                batch.append((user_id_str, record))

                if len(batch) >= self.batch_size:
                    self._insert_users_batch(batch)
                    processed += len(batch)
                    self.log(f"Processed {processed}/{total} users")
                    batch = []

            except Exception as e:
                error_msg = f"Error processing user {user_doc.get('_id')}: {str(e)}"
                self.log(error_msg)
                self.stats["errors"].append(error_msg)

        if batch:
            self._insert_users_batch(batch)
            processed += len(batch)

        self.log(f"Completed: {processed} users migrated")
        self.stats["users"] = processed

    def _insert_users_batch(self, batch: List[Tuple[str, Dict]]):
        """Insert batch of users into PostgreSQL"""
        query = """
            INSERT INTO users (
                username, email, phone, google, wechat, hashed_password,
                activated, privileged, created_at, updated_at
            ) VALUES (
                %(username)s, %(email)s, %(phone)s, %(google)s, %(wechat)s,
                %(hashed_password)s, %(activated)s, %(privileged)s,
                %(created_at)s, %(updated_at)s
            ) RETURNING id
        """

        for mongo_id, record in batch:
            try:
                self.pg_cursor.execute(query, record)
                pg_uuid = self.pg_cursor.fetchone()[0]
                self.user_id_map[mongo_id] = str(pg_uuid)
            except Exception as e:
                self.log(f"Error inserting user {mongo_id}: {str(e)}")
                self.pg_conn.rollback()
                raise

        self.pg_conn.commit()

    def migrate_areas(self, db_name: str):
        """Migrate areas from MongoDB to PostgreSQL"""
        self.log("Migrating areas...")

        mongo_db = self.mongo_client[db_name]
        areas_collection = mongo_db["areas"]

        total = areas_collection.count_documents({})
        self.log(f"Found {total} areas to migrate")

        batch = []
        processed = 0

        for area_doc in areas_collection.find():
            try:
                area_id_str = self.convert_oid_to_string(area_doc["_id"])
                entity_id_str = self.convert_oid_to_string(area_doc.get("entity"))

                # Get PostgreSQL entity UUID
                entity_uuid = self.entity_id_map.get(entity_id_str)
                if not entity_uuid:
                    self.log(
                        f"Warning: Entity {entity_id_str} not found for area {area_id_str}, skipping..."
                    )
                    continue

                # Extract floor information
                floor_info = area_doc.get("floor", {})
                floor_type = (
                    floor_info.get("type") if isinstance(floor_info, dict) else None
                )
                floor_name = (
                    floor_info.get("name") if isinstance(floor_info, dict) else None
                )

                # Convert polygon
                polygon = area_doc.get("polygon", [])
                polygon_wkt = self.polygon_to_postgis(polygon)

                if not polygon_wkt:
                    self.log(
                        f"Warning: Invalid polygon for area {area_id_str}, skipping..."
                    )
                    continue

                record = {
                    "entity_id": entity_uuid,
                    "name": area_doc.get("name", ""),
                    "description": area_doc.get("description"),
                    "floor_type": floor_type,
                    "floor_name": floor_name,
                    "beacon_code": area_doc.get("beacon_code", ""),
                    "polygon": polygon_wkt,
                    "created_at": self.convert_timestamp(area_doc.get("created_at")),
                    "updated_at": self.convert_timestamp(area_doc.get("updated_at")),
                }

                batch.append((area_id_str, record))

                if len(batch) >= self.batch_size:
                    self._insert_areas_batch(batch)
                    processed += len(batch)
                    self.log(f"Processed {processed}/{total} areas")
                    batch = []

            except Exception as e:
                error_msg = f"Error processing area {area_doc.get('_id')}: {str(e)}"
                self.log(error_msg)
                self.stats["errors"].append(error_msg)

        if batch:
            self._insert_areas_batch(batch)
            processed += len(batch)

        self.log(f"Completed: {processed} areas migrated")
        self.stats["areas"] = processed

    def _insert_areas_batch(self, batch: List[Tuple[str, Dict]]):
        """Insert batch of areas into PostgreSQL"""
        query = """
            INSERT INTO areas (
                entity_id, name, description, floor_type, floor_name,
                beacon_code, polygon, created_at, updated_at
            ) VALUES (
                %(entity_id)s::uuid, %(name)s, %(description)s, %(floor_type)s,
                %(floor_name)s, %(beacon_code)s, ST_GeomFromEWKT(%(polygon)s),
                %(created_at)s, %(updated_at)s
            ) RETURNING id
        """

        for mongo_id, record in batch:
            try:
                self.pg_cursor.execute(query, record)
                pg_id = self.pg_cursor.fetchone()[0]
                self.area_id_map[mongo_id] = pg_id
            except Exception as e:
                self.log(f"Error inserting area {mongo_id}: {str(e)}")
                self.pg_conn.rollback()
                raise

        self.pg_conn.commit()

    def migrate_connections(self, db_name: str):
        """Migrate connections from MongoDB to PostgreSQL"""
        self.log("Migrating connections...")

        mongo_db = self.mongo_client[db_name]
        connections_collection = mongo_db["connections"]

        total = connections_collection.count_documents({})
        self.log(f"Found {total} connections to migrate")

        batch = []
        processed = 0

        for conn_doc in connections_collection.find():
            try:
                conn_id_str = self.convert_oid_to_string(conn_doc["_id"])
                entity_id_str = self.convert_oid_to_string(conn_doc.get("entity"))

                entity_uuid = self.entity_id_map.get(entity_id_str)
                if not entity_uuid:
                    self.log(
                        f"Warning: Entity {entity_id_str} not found for connection {conn_id_str}, skipping..."
                    )
                    continue

                # Convert connected_areas format
                # MongoDB: [[area_oid, x, y, boolean], ...]
                # PostgreSQL: [[area_id, x, y, boolean], ...]
                connected_areas_mongo = conn_doc.get("connected_areas", [])
                connected_areas_pg = []

                for area_conn in connected_areas_mongo:
                    if len(area_conn) >= 4:
                        area_oid_str = self.convert_oid_to_string(area_conn[0])
                        area_pg_id = self.area_id_map.get(area_oid_str)

                        if area_pg_id:
                            connected_areas_pg.append(
                                [area_pg_id, area_conn[1], area_conn[2], area_conn[3]]
                            )

                # Handle gnd (ground point) if exists
                gnd = conn_doc.get("gnd")
                gnd_wkt = self.point_to_postgis(gnd) if gnd else None

                record = {
                    "entity_id": entity_uuid,
                    "name": conn_doc.get("name", ""),
                    "description": conn_doc.get("description"),
                    "type": conn_doc.get("type", ""),
                    "connected_areas": json.dumps(connected_areas_pg),
                    "available_period": json.dumps(
                        conn_doc.get("available_period", [])
                    ),
                    "tags": json.dumps(conn_doc.get("tags", [])),
                    "gnd": gnd_wkt,
                    "created_at": self.convert_timestamp(conn_doc.get("created_at")),
                    "updated_at": self.convert_timestamp(conn_doc.get("updated_at")),
                }

                batch.append((conn_id_str, record))

                if len(batch) >= self.batch_size:
                    self._insert_connections_batch(batch)
                    processed += len(batch)
                    self.log(f"Processed {processed}/{total} connections")
                    batch = []

            except Exception as e:
                error_msg = (
                    f"Error processing connection {conn_doc.get('_id')}: {str(e)}"
                )
                self.log(error_msg)
                self.stats["errors"].append(error_msg)

        if batch:
            self._insert_connections_batch(batch)
            processed += len(batch)

        self.log(f"Completed: {processed} connections migrated")
        self.stats["connections"] = processed

    def _insert_connections_batch(self, batch: List[Tuple[str, Dict]]):
        """Insert batch of connections into PostgreSQL"""
        query = """
            INSERT INTO connections (
                entity_id, name, description, type, connected_areas,
                available_period, tags, gnd, created_at, updated_at
            ) VALUES (
                %(entity_id)s::uuid, %(name)s, %(description)s, %(type)s,
                %(connected_areas)s::jsonb, %(available_period)s::jsonb,
                %(tags)s::jsonb, 
                CASE WHEN %(gnd)s IS NOT NULL THEN ST_GeomFromEWKT(%(gnd)s) ELSE NULL END,
                %(created_at)s, %(updated_at)s
            ) RETURNING id
        """

        for mongo_id, record in batch:
            try:
                self.pg_cursor.execute(query, record)
                pg_id = self.pg_cursor.fetchone()[0]
                self.connection_id_map[mongo_id] = pg_id
            except Exception as e:
                self.log(f"Error inserting connection {mongo_id}: {str(e)}")
                self.pg_conn.rollback()
                raise

        self.pg_conn.commit()

    def migrate_merchants(self, db_name: str):
        """Migrate merchants from MongoDB to PostgreSQL"""
        self.log("Migrating merchants...")

        mongo_db = self.mongo_client[db_name]
        merchants_collection = mongo_db["merchants"]

        total = merchants_collection.count_documents({})
        self.log(f"Found {total} merchants to migrate")

        batch = []
        processed = 0

        for merchant_doc in merchants_collection.find():
            try:
                merchant_id_str = self.convert_oid_to_string(merchant_doc["_id"])
                entity_id_str = self.convert_oid_to_string(merchant_doc.get("entity"))
                area_id_str = self.convert_oid_to_string(merchant_doc.get("area"))

                entity_uuid = self.entity_id_map.get(entity_id_str)
                area_pg_id = self.area_id_map.get(area_id_str)

                if not entity_uuid or not area_pg_id:
                    self.log(
                        f"Warning: Entity or Area not found for merchant {merchant_id_str}, skipping..."
                    )
                    continue

                # Convert location point
                location = merchant_doc.get("location", [])
                location_wkt = self.point_to_postgis(location)

                if not location_wkt:
                    self.log(
                        f"Warning: Invalid location for merchant {merchant_id_str}, skipping..."
                    )
                    continue

                # Convert polygon (may be empty)
                polygon = merchant_doc.get("polygon", [])
                polygon_wkt = self.polygon_to_postgis(polygon) if polygon else None

                # If no polygon, create a small box around the location point
                if not polygon_wkt and len(location) >= 2:
                    # Create a 1x1 box around the point
                    x, y = location[0], location[1]
                    box_coords = [
                        [x - 0.5, y - 0.5],
                        [x + 0.5, y - 0.5],
                        [x + 0.5, y + 0.5],
                        [x - 0.5, y + 0.5],
                        [x - 0.5, y - 0.5],
                    ]
                    polygon_wkt = self.polygon_to_postgis(box_coords)

                # Handle merchant type - can be string or object in MongoDB
                merchant_type = f'"{merchant_doc.get("type", "other")}"'

                record = {
                    "entity_id": entity_uuid,
                    "area_id": area_pg_id,
                    "name": merchant_doc.get("name", ""),
                    "description": merchant_doc.get("description"),
                    "chain": merchant_doc.get("chain"),
                    "beacon_code": merchant_doc.get("beacon_code", ""),
                    "type": merchant_type,
                    "color": merchant_doc.get("color"),
                    "tags": json.dumps(merchant_doc.get("tags", [])),
                    "location": location_wkt,
                    "style": merchant_doc.get("style"),
                    "polygon": polygon_wkt,
                    "available_period": json.dumps(
                        merchant_doc.get("available_period", [])
                    ),
                    "opening_hours": json.dumps(merchant_doc.get("opening_hours"))
                    if merchant_doc.get("opening_hours")
                    else None,
                    "email": merchant_doc.get("email"),
                    "phone": merchant_doc.get("phone"),
                    "website": merchant_doc.get("website"),
                    "social_media": json.dumps(merchant_doc.get("social_media", [])),
                    "created_at": self.convert_timestamp(
                        merchant_doc.get("created_at")
                    ),
                    "updated_at": self.convert_timestamp(
                        merchant_doc.get("updated_at")
                    ),
                }

                batch.append((merchant_id_str, record))

                if len(batch) >= self.batch_size:
                    self._insert_merchants_batch(batch)
                    processed += len(batch)
                    self.log(f"Processed {processed}/{total} merchants")
                    batch = []

            except Exception as e:
                error_msg = (
                    f"Error processing merchant {merchant_doc.get('_id')}: {str(e)}"
                )
                self.log(error_msg)
                self.stats["errors"].append(error_msg)

        if batch:
            self._insert_merchants_batch(batch)
            processed += len(batch)

        self.log(f"Completed: {processed} merchants migrated")
        self.stats["merchants"] = processed

    def _insert_merchants_batch(self, batch: List[Tuple[str, Dict]]):
        """Insert batch of merchants into PostgreSQL"""
        query = """
            INSERT INTO merchants (
                entity_id, area_id, name, description, chain, beacon_code,
                type, color, tags, location, style, polygon, available_period,
                opening_hours, email, phone, website, social_media,
                created_at, updated_at
            ) VALUES (
                %(entity_id)s::uuid, %(area_id)s, %(name)s, %(description)s,
                %(chain)s, %(beacon_code)s, %(type)s::jsonb, %(color)s,
                %(tags)s::jsonb, ST_GeomFromEWKT(%(location)s), %(style)s,
                ST_GeomFromEWKT(%(polygon)s), %(available_period)s::jsonb,
                %(opening_hours)s::jsonb, %(email)s, %(phone)s, %(website)s,
                %(social_media)s::jsonb, %(created_at)s, %(updated_at)s
            ) RETURNING id
        """

        for mongo_id, record in batch:
            try:
                self.pg_cursor.execute(query, record)
                pg_id = self.pg_cursor.fetchone()[0]
                self.merchant_id_map[mongo_id] = pg_id
            except Exception as e:
                self.log(f"Error inserting merchant {mongo_id}: {str(e)}")
                self.pg_conn.rollback()
                raise

        self.pg_conn.commit()

    def migrate_beacons(self, db_name: str):
        """Migrate beacons from MongoDB to PostgreSQL"""
        self.log("Migrating beacons...")

        mongo_db = self.mongo_client[db_name]
        beacons_collection = mongo_db["beacons"]

        total = beacons_collection.count_documents({})
        self.log(f"Found {total} beacons to migrate")

        batch = []
        processed = 0

        for beacon_doc in beacons_collection.find():
            try:
                beacon_id_str = self.convert_oid_to_string(beacon_doc["_id"])
                entity_id_str = self.convert_oid_to_string(beacon_doc.get("entity"))
                area_id_str = self.convert_oid_to_string(beacon_doc.get("area"))
                merchant_id_str = self.convert_oid_to_string(beacon_doc.get("merchant"))
                connection_id_str = self.convert_oid_to_string(
                    beacon_doc.get("connection")
                )

                entity_uuid = self.entity_id_map.get(entity_id_str)
                area_pg_id = self.area_id_map.get(area_id_str)

                if not entity_uuid or not area_pg_id:
                    self.log(
                        f"Warning: Entity or Area not found for beacon {beacon_id_str}, skipping..."
                    )
                    continue

                # Optional foreign keys
                merchant_pg_id = (
                    self.merchant_id_map.get(merchant_id_str)
                    if merchant_id_str
                    else None
                )
                connection_pg_id = (
                    self.connection_id_map.get(connection_id_str)
                    if connection_id_str
                    else None
                )

                # Convert location
                location = beacon_doc.get("location", [])
                location_wkt = self.point_to_postgis(location)

                if not location_wkt:
                    self.log(
                        f"Warning: Invalid location for beacon {beacon_id_str}, skipping..."
                    )
                    continue

                record = {
                    "entity_id": entity_uuid,
                    "area_id": area_pg_id,
                    "merchant_id": merchant_pg_id,
                    "connection_id": connection_pg_id,
                    "name": beacon_doc.get("name", ""),
                    "description": beacon_doc.get("description"),
                    "type": beacon_doc.get("type", ""),
                    "location": location_wkt,
                    "device": beacon_doc.get("device", ""),
                    "mac": beacon_doc.get("mac", ""),
                    "created_at": self.convert_timestamp(beacon_doc.get("created_at")),
                    "updated_at": self.convert_timestamp(beacon_doc.get("updated_at")),
                }

                batch.append((beacon_id_str, record))

                if len(batch) >= self.batch_size:
                    self._insert_beacons_batch(batch)
                    processed += len(batch)
                    self.log(f"Processed {processed}/{total} beacons")
                    batch = []

            except Exception as e:
                error_msg = f"Error processing beacon {beacon_doc.get('_id')}: {str(e)}"
                self.log(error_msg)
                self.stats["errors"].append(error_msg)

        if batch:
            self._insert_beacons_batch(batch)
            processed += len(batch)

        self.log(f"Completed: {processed} beacons migrated")
        self.stats["beacons"] = processed

    def _insert_beacons_batch(self, batch: List[Tuple[str, Dict]]):
        """Insert batch of beacons into PostgreSQL"""
        query = """
            INSERT INTO beacons (
                entity_id, area_id, merchant_id, connection_id,
                name, description, type, location, device, mac,
                created_at, updated_at
            ) VALUES (
                %(entity_id)s::uuid, %(area_id)s, %(merchant_id)s, %(connection_id)s,
                %(name)s, %(description)s, %(type)s, ST_GeomFromEWKT(%(location)s),
                %(device)s, %(mac)s, %(created_at)s, %(updated_at)s
            ) RETURNING id
        """

        for mongo_id, record in batch:
            try:
                self.pg_cursor.execute(query, record)
                pg_id = self.pg_cursor.fetchone()[0]
                self.beacon_id_map[mongo_id] = pg_id
            except Exception as e:
                self.log(f"Error inserting beacon {mongo_id}: {str(e)}")
                self.pg_conn.rollback()
                raise

        self.pg_conn.commit()

    def run_migration(self, mongo_db_name: str):
        """Run full migration process"""
        self.log("=" * 60)
        self.log("Starting MongoDB to PostgreSQL Migration")
        self.log("=" * 60)

        try:
            # Migration order matters due to foreign key constraints
            self.migrate_entities(mongo_db_name)
            self.migrate_users(mongo_db_name)
            self.migrate_areas(mongo_db_name)
            self.migrate_connections(mongo_db_name)
            self.migrate_merchants(mongo_db_name)
            self.migrate_beacons(mongo_db_name)

            self.log("=" * 60)
            self.log("Migration Summary:")
            self.log(f"  Entities:    {self.stats['entities']}")
            self.log(f"  Users:       {self.stats['users']}")
            self.log(f"  Areas:       {self.stats['areas']}")
            self.log(f"  Connections: {self.stats['connections']}")
            self.log(f"  Merchants:   {self.stats['merchants']}")
            self.log(f"  Beacons:     {self.stats['beacons']}")
            self.log(f"  Errors:      {len(self.stats['errors'])}")

            if self.stats["errors"]:
                self.log("\nErrors encountered:")
                for error in self.stats["errors"][:10]:  # Show first 10 errors
                    self.log(f"  - {error}")
                if len(self.stats["errors"]) > 10:
                    self.log(f"  ... and {len(self.stats['errors']) - 10} more")

            self.log("=" * 60)
            self.log("Migration completed successfully!")

        except Exception as e:
            self.log(f"Fatal error during migration: {str(e)}")
            raise
        finally:
            self.close()

    def close(self):
        """Close database connections"""
        if self.pg_cursor:
            self.pg_cursor.close()
        if self.pg_conn:
            self.pg_conn.close()
        if self.mongo_client:
            self.mongo_client.close()


def main():
    """Main entry point"""
    # Configuration - Update these values
    MONGO_URI = os.getenv("MONGO_URI", "mongodb://localhost:27017/")
    MONGO_DB_NAME = os.getenv("MONGO_DB_NAME", "indoor-mall-nav")

    PG_HOST = os.getenv("PG_HOST", "localhost")
    PG_PORT = os.getenv("PG_PORT", "5432")
    PG_DATABASE = os.getenv("PG_DATABASE", "navign")
    PG_USER = os.getenv("PG_USER", "ethangoh")
    PG_PASSWORD = os.getenv("PG_PASSWORD", "")

    PG_CONNECTION_STRING = f"host={PG_HOST} port={PG_PORT} dbname={PG_DATABASE} user={PG_USER} password={PG_PASSWORD}"

    BATCH_SIZE = int(os.getenv("BATCH_SIZE", "100"))

    print("MongoDB to PostgreSQL Migration Script")
    print(f"MongoDB: {MONGO_URI} (database: {MONGO_DB_NAME})")
    print(f"PostgreSQL: {PG_HOST}:{PG_PORT}/{PG_DATABASE}")
    print(f"Batch size: {BATCH_SIZE}")
    print()

    # Confirm before proceeding
    response = input(
        "This will migrate data from MongoDB to PostgreSQL. Continue? (yes/no): "
    )
    if response.lower() not in ["yes", "y"]:
        print("Migration cancelled.")
        return

    # Run migration
    migrator = MongoToPostgresMigration(
        mongo_uri=MONGO_URI,
        pg_connection_string=PG_CONNECTION_STRING,
        batch_size=BATCH_SIZE,
    )

    try:
        migrator.run_migration(MONGO_DB_NAME)
    except KeyboardInterrupt:
        print("\n\nMigration interrupted by user")
        migrator.close()
        sys.exit(1)
    except Exception as e:
        print(f"\n\nMigration failed: {str(e)}")
        migrator.close()
        sys.exit(1)


if __name__ == "__main__":
    main()
