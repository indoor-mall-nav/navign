#!/usr/bin/env python3
"""
Migration Validation Script

Validates that data was correctly migrated from MongoDB to PostgreSQL
by comparing record counts and sampling data integrity.

Usage:
    python validate_migration.py
"""

import os
import sys
from typing import Dict

try:
    from pymongo import MongoClient
    import psycopg2
except ImportError:
    print("Error: Required packages not installed")
    print("Please run: pip install pymongo psycopg2-binary")
    sys.exit(1)


class MigrationValidator:
    """Validates MongoDB to PostgreSQL migration"""

    def __init__(self, mongo_uri: str, mongo_db_name: str, pg_connection_string: str):
        self.mongo_client = MongoClient(mongo_uri)
        self.mongo_db = self.mongo_client[mongo_db_name]
        self.pg_conn = psycopg2.connect(pg_connection_string)
        self.pg_cursor = self.pg_conn.cursor()

        self.results = {"passed": [], "failed": [], "warnings": []}

    def log(self, message: str, level: str = "INFO"):
        """Print formatted log message"""
        prefix = {"INFO": "ℹ️ ", "PASS": "✅", "FAIL": "❌", "WARN": "⚠️ "}.get(
            level, "  "
        )
        print(f"{prefix} {message}")

    def validate_counts(self):
        """Compare record counts between MongoDB and PostgreSQL"""
        self.log("Validating record counts...", "INFO")

        collections_to_tables = {
            "entities": "entities",
            "users": "users",
            "areas": "areas",
            "merchants": "merchants",
            "beacons": "beacons",
            "connections": "connections",
        }

        all_match = True

        for collection, table in collections_to_tables.items():
            try:
                # Get MongoDB count
                mongo_count = self.mongo_db[collection].count_documents({})

                # Get PostgreSQL count
                self.pg_cursor.execute(f"SELECT COUNT(*) FROM {table}")
                pg_count = self.pg_cursor.fetchone()[0]

                if mongo_count == pg_count:
                    self.log(
                        f"{collection:12} → {table:12}: {mongo_count:6} records ✓",
                        "PASS",
                    )
                    self.results["passed"].append(
                        f"{collection} count matches ({mongo_count})"
                    )
                else:
                    self.log(
                        f"{collection:12} → {table:12}: MongoDB={mongo_count}, PostgreSQL={pg_count} ✗",
                        "FAIL",
                    )
                    self.results["failed"].append(
                        f"{collection} count mismatch: MongoDB={mongo_count}, PostgreSQL={pg_count}"
                    )
                    all_match = False

            except Exception as e:
                self.log(f"Error validating {collection}: {str(e)}", "FAIL")
                self.results["failed"].append(
                    f"Error validating {collection}: {str(e)}"
                )
                all_match = False

        return all_match

    def validate_foreign_keys(self):
        """Validate foreign key relationships"""
        self.log("\nValidating foreign key relationships...", "INFO")

        fk_checks = [
            ("areas", "entity_id", "entities", "id"),
            ("beacons", "entity_id", "entities", "id"),
            ("beacons", "area_id", "areas", "id"),
            ("merchants", "entity_id", "entities", "id"),
            ("merchants", "area_id", "areas", "id"),
            ("connections", "entity_id", "entities", "id"),
        ]

        all_valid = True

        for child_table, child_fk, parent_table, parent_pk in fk_checks:
            try:
                # Check for orphaned records
                query = f"""
                    SELECT COUNT(*) 
                    FROM {child_table} c
                    WHERE NOT EXISTS (
                        SELECT 1 FROM {parent_table} p 
                        WHERE c.{child_fk} = p.{parent_pk}
                    )
                """
                self.pg_cursor.execute(query)
                orphaned_count = self.pg_cursor.fetchone()[0]

                if orphaned_count == 0:
                    self.log(
                        f"{child_table}.{child_fk} → {parent_table}.{parent_pk}: No orphaned records ✓",
                        "PASS",
                    )
                    self.results["passed"].append(f"{child_table}.{child_fk} FK valid")
                else:
                    self.log(
                        f"{child_table}.{child_fk} → {parent_table}.{parent_pk}: {orphaned_count} orphaned records ✗",
                        "FAIL",
                    )
                    self.results["failed"].append(
                        f"{child_table}.{child_fk} has {orphaned_count} orphaned records"
                    )
                    all_valid = False

            except Exception as e:
                self.log(
                    f"Error validating FK {child_table}.{child_fk}: {str(e)}", "FAIL"
                )
                self.results["failed"].append(f"Error validating FK: {str(e)}")
                all_valid = False

        return all_valid

    def validate_geometry(self):
        """Validate PostGIS geometry data"""
        self.log("\nValidating PostGIS geometries...", "INFO")

        geometry_checks = [
            ("entities", "point_min", "POINT"),
            ("entities", "point_max", "POINT"),
            ("areas", "polygon", "POLYGON"),
            ("beacons", "location", "POINT"),
            ("merchants", "location", "POINT"),
            ("merchants", "polygon", "POLYGON"),
        ]

        all_valid = True

        for table, column, geom_type in geometry_checks:
            try:
                # Check for invalid geometries
                query = f"""
                    SELECT 
                        COUNT(*) as total,
                        COUNT(*) FILTER (WHERE {column} IS NOT NULL) as non_null,
                        COUNT(*) FILTER (WHERE ST_IsValid({column})) as valid,
                        COUNT(*) FILTER (WHERE ST_GeometryType({column}) = 'ST_{geom_type}') as correct_type
                    FROM {table}
                """
                self.pg_cursor.execute(query)
                result = self.pg_cursor.fetchone()
                total, non_null, valid, correct_type = result

                if non_null == valid == correct_type:
                    self.log(
                        f"{table}.{column:12} ({geom_type:7}): {valid}/{total} valid geometries ✓",
                        "PASS",
                    )
                    self.results["passed"].append(f"{table}.{column} geometries valid")
                else:
                    self.log(
                        f"{table}.{column:12} ({geom_type:7}): {valid}/{non_null} valid, {correct_type}/{non_null} correct type ✗",
                        "FAIL",
                    )
                    self.results["failed"].append(
                        f"{table}.{column} has invalid geometries"
                    )
                    all_valid = False

                # Check if null geometries exist where they shouldn't
                if (
                    total > non_null and table != "connections"
                ):  # connections.gnd can be null
                    self.log(
                        f"{table}.{column}: {total - non_null} NULL geometries (should not be null)",
                        "WARN",
                    )
                    self.results["warnings"].append(
                        f"{table}.{column} has NULL geometries"
                    )

            except Exception as e:
                self.log(
                    f"Error validating geometry {table}.{column}: {str(e)}", "FAIL"
                )
                self.results["failed"].append(f"Error validating geometry: {str(e)}")
                all_valid = False

        return all_valid

    def validate_unique_constraints(self):
        """Validate unique constraints"""
        self.log("\nValidating unique constraints...", "INFO")

        unique_checks = [
            ("users", "username"),
            ("users", "email"),
            ("beacons", "mac"),
        ]

        all_valid = True

        for table, column in unique_checks:
            try:
                # Check for duplicates
                query = f"""
                    SELECT {column}, COUNT(*) as count
                    FROM {table}
                    GROUP BY {column}
                    HAVING COUNT(*) > 1
                """
                self.pg_cursor.execute(query)
                duplicates = self.pg_cursor.fetchall()

                if not duplicates:
                    self.log(f"{table}.{column}: No duplicates ✓", "PASS")
                    self.results["passed"].append(
                        f"{table}.{column} unique constraint valid"
                    )
                else:
                    self.log(
                        f"{table}.{column}: {len(duplicates)} duplicate values ✗",
                        "FAIL",
                    )
                    self.results["failed"].append(
                        f"{table}.{column} has {len(duplicates)} duplicates"
                    )
                    all_valid = False
                    for value, count in duplicates[:5]:  # Show first 5
                        self.log(
                            f"  Duplicate: '{value}' appears {count} times", "FAIL"
                        )

            except Exception as e:
                self.log(
                    f"Error validating unique constraint {table}.{column}: {str(e)}",
                    "FAIL",
                )
                self.results["failed"].append(
                    f"Error validating unique constraint: {str(e)}"
                )
                all_valid = False

        return all_valid

    def validate_sample_data(self):
        """Sample and compare specific records"""
        self.log("\nValidating sample data integrity...", "INFO")

        all_valid = True

        # Sample an entity
        try:
            mongo_entity = self.mongo_db["entities"].find_one()
            if mongo_entity:
                entity_name = mongo_entity.get("name")

                self.pg_cursor.execute(
                    "SELECT COUNT(*) FROM entities WHERE name = %s", (entity_name,)
                )
                pg_count = self.pg_cursor.fetchone()[0]

                if pg_count > 0:
                    self.log(
                        f"Sample entity '{entity_name}' found in PostgreSQL ✓", "PASS"
                    )
                    self.results["passed"].append("Sample entity data verified")
                else:
                    self.log(
                        f"Sample entity '{entity_name}' NOT found in PostgreSQL ✗",
                        "FAIL",
                    )
                    self.results["failed"].append("Sample entity missing in PostgreSQL")
                    all_valid = False
        except Exception as e:
            self.log(f"Error validating sample entity: {str(e)}", "FAIL")
            all_valid = False

        # Sample a beacon
        try:
            mongo_beacon = self.mongo_db["beacons"].find_one()
            if mongo_beacon:
                beacon_mac = mongo_beacon.get("mac")

                self.pg_cursor.execute(
                    "SELECT COUNT(*) FROM beacons WHERE mac = %s", (beacon_mac,)
                )
                pg_count = self.pg_cursor.fetchone()[0]

                if pg_count > 0:
                    self.log(
                        f"Sample beacon MAC '{beacon_mac}' found in PostgreSQL ✓",
                        "PASS",
                    )
                    self.results["passed"].append("Sample beacon data verified")
                else:
                    self.log(
                        f"Sample beacon MAC '{beacon_mac}' NOT found in PostgreSQL ✗",
                        "FAIL",
                    )
                    self.results["failed"].append("Sample beacon missing in PostgreSQL")
                    all_valid = False
        except Exception as e:
            self.log(f"Error validating sample beacon: {str(e)}", "FAIL")
            all_valid = False

        return all_valid

    def run_validation(self):
        """Run all validation checks"""
        print("=" * 70)
        print("🔍 MongoDB to PostgreSQL Migration Validation")
        print("=" * 70)

        checks = [
            ("Record Counts", self.validate_counts),
            ("Foreign Keys", self.validate_foreign_keys),
            ("Geometry Data", self.validate_geometry),
            ("Unique Constraints", self.validate_unique_constraints),
            ("Sample Data", self.validate_sample_data),
        ]

        all_passed = True

        for check_name, check_func in checks:
            try:
                result = check_func()
                if not result:
                    all_passed = False
            except Exception as e:
                self.log(f"Error during {check_name} validation: {str(e)}", "FAIL")
                all_passed = False

        # Print summary
        print("\n" + "=" * 70)
        print("📊 Validation Summary")
        print("=" * 70)
        print(f"✅ Passed:   {len(self.results['passed'])}")
        print(f"❌ Failed:   {len(self.results['failed'])}")
        print(f"⚠️  Warnings: {len(self.results['warnings'])}")

        if self.results["failed"]:
            print("\n❌ Failed Checks:")
            for failure in self.results["failed"]:
                print(f"  - {failure}")

        if self.results["warnings"]:
            print("\n⚠️  Warnings:")
            for warning in self.results["warnings"]:
                print(f"  - {warning}")

        print("=" * 70)

        if all_passed:
            print("🎉 All validation checks PASSED!")
            return 0
        else:
            print("⚠️  Some validation checks FAILED. Please review the issues above.")
            return 1

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
    # Configuration
    MONGO_URI = os.getenv("MONGO_URI", "mongodb://localhost:27017/")
    MONGO_DB_NAME = os.getenv("MONGO_DB_NAME", "indoor-mall-nav")

    PG_HOST = os.getenv("PG_HOST", "localhost")
    PG_PORT = os.getenv("PG_PORT", "5432")
    PG_DATABASE = os.getenv("PG_DATABASE", "navign")
    PG_USER = os.getenv("PG_USER", "ethangoh")
    PG_PASSWORD = os.getenv("PG_PASSWORD", "")

    PG_CONNECTION_STRING = f"host={PG_HOST} port={PG_PORT} dbname={PG_DATABASE} user={PG_USER} password={PG_PASSWORD}"

    validator = MigrationValidator(
        mongo_uri=MONGO_URI,
        mongo_db_name=MONGO_DB_NAME,
        pg_connection_string=PG_CONNECTION_STRING,
    )

    try:
        exit_code = validator.run_validation()
        validator.close()
        sys.exit(exit_code)
    except KeyboardInterrupt:
        print("\n\nValidation interrupted by user")
        validator.close()
        sys.exit(1)
    except Exception as e:
        print(f"\n\nValidation failed: {str(e)}")
        validator.close()
        sys.exit(1)


if __name__ == "__main__":
    main()
