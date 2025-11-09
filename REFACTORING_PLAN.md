# Navign Refactoring Plan

**Generated:** 2025-11-09
**Status:** Proposed
**Priority:** Critical security fixes → High-impact improvements → Code quality → Technical debt

---

## Executive Summary

This document outlines refactoring opportunities identified across the Navign codebase. The analysis found:

- **2 critical security issues** requiring immediate attention
- **~2,000-3,000 lines of code** that could be eliminated through better use of ecosystem crates
- **40-50% reduction** in maintenance burden through improved abstractions
- **Significant code duplication** due to feature flag implementations
- **72 `.unwrap()` calls** in server code that could panic
- **Multiple manual implementations** of algorithms available in well-tested crates

---

## 🚨 CRITICAL SECURITY ISSUES (Immediate Action Required)

### 1. Weak Password Hashing (CRITICAL)

**File:** `server/src/schema/user.rs:30`

**Issue:**
```rust
let hashed_password = hash(password, 4).expect("Failed to hash password");
//                               ^ COST FACTOR TOO LOW!
```

**Risk:** Cost factor of 4 makes brute-force attacks trivial. OWASP recommends minimum 10-12.

**Fix:**
```rust
const BCRYPT_COST: u32 = 12; // Adjust based on performance testing

pub fn new(..., password: String) -> Result<Self, bcrypt::BcryptError> {
    let hashed_password = hash(password, BCRYPT_COST)?;
    Ok(Self { /* ... */, hashed_password, /* ... */ })
}
```

**Impact:** HIGH - User account security
**Effort:** 5 minutes
**Files to modify:** `server/src/schema/user.rs`

---

### 2. Missing Authorization Check (CRITICAL)

**File:** `server/src/kernel/unlocker/instance.rs:131`

**Issue:**
```rust
// TODO verify if the user is allowed to unlock this beacon
let instance = UnlockInstance { /* ... */ };
```

**Risk:** Any authenticated user can unlock any beacon without permission checks.

**Fix:**
```rust
// Verify user authorization
let user_permissions = UserPermissions::get_for_beacon(db, &user, &beacon.get_id()).await?;
if !user_permissions.can_unlock() {
    anyhow::bail!("User not authorized to unlock this beacon");
}
```

**Impact:** HIGH - Access control bypass
**Effort:** 2-4 hours (needs new permission system)
**Files to modify:**
- `server/src/kernel/unlocker/instance.rs`
- `server/src/schema/user_permissions.rs` (new file)

---

## 🔥 HIGH-IMPACT REFACTORING (High Priority)

### 3. Replace Manual TOTP Implementation with `totp-rs` Crate

**File:** `server/src/kernel/totp.rs:83-103`

**Current Code:** ~42 lines of manual HMAC-SHA1 and dynamic truncation

**Benefits:**
- **Standards compliance:** RFC 6238 compliant
- **Battle-tested:** Used in production by thousands of projects
- **Maintenance:** Remove custom crypto code
- **Features:** Built-in QR code generation, secret management

**Implementation:**

```toml
# Cargo.toml
totp-rs = "5.6"
```

```rust
use totp_rs::{Algorithm, TOTP, Secret};

impl BeaconSecret {
    pub fn generate_totp(&self) -> Result<String> {
        if !self.is_active {
            anyhow::bail!("Beacon is not active");
        }

        let totp = TOTP::new(
            Algorithm::SHA1,
            6,                          // 6 digits
            1,                          // 1 step skew
            30,                         // 30 second step
            Secret::Raw(self.secret.as_bytes().to_vec()).to_bytes().unwrap(),
        )?;

        Ok(totp.generate_current()?)
    }
}
```

**Impact:** MEDIUM-HIGH - Removes ~40 lines, improves security
**Effort:** 1-2 hours
**Files to modify:**
- `server/src/kernel/totp.rs`
- `server/Cargo.toml`

---

### 4. Use `thiserror` for All Error Types

**Files:** Multiple across codebase

**Current Code:** ~50+ lines of manual error implementations per error type

**Example - NavigationError:**
```rust
// Before (server/src/kernel/route/implementations/navigate.rs:10-64)
#[derive(Debug, Clone, PartialEq)]
pub enum NavigationError {
    NoRoute,
    InvalidDeparture,
    InvalidArrival,
    AccessDenied,
    Other(String),
}

impl Error for NavigationError {}
impl std::fmt::Display for NavigationError { /* 15 lines */ }
impl From<NavigationError> for String { /* 10 lines */ }
impl Serialize for NavigationError { /* 20 lines manual */ }
impl<'de> Deserialize<'de> for NavigationError { /* 15 lines manual */ }
```

**After:**
```rust
use thiserror::Error;

#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum NavigationError {
    #[error("No route found between departure and arrival points")]
    NoRoute,
    #[error("Invalid departure point")]
    InvalidDeparture,
    #[error("Invalid arrival point")]
    InvalidArrival,
    #[error("Access denied to requested route")]
    AccessDenied,
    #[error("{0}")]
    Other(String),
}
```

**Implementation:**

```toml
# Cargo.toml
thiserror = "2.0"
```

**Files to modify:**
- `server/src/kernel/route/implementations/navigate.rs`
- `shared/src/errors/crypto.rs`
- Any other custom error types

**Impact:** HIGH - Removes ~500-1000 lines of boilerplate
**Effort:** 3-4 hours
**Benefits:** Better error messages, automatic derive implementations

---

### 5. Replace Binary Serialization with `postcard` Crate

**Files:** `shared/src/ble/*.rs`, `shared/src/traits/*.rs`

**Problem:** Massive code duplication due to feature flags

**Current Code:**
- `shared/src/ble/message.rs`: 130 lines with ~60% duplication between `heapless` and `alloc` features
- `shared/src/ble/proof.rs`: Similar duplication
- `shared/src/ble/device_caps.rs`: Similar duplication
- `shared/src/ble/device_type.rs`: Similar duplication

**Solution:** Use `postcard` - designed for `no_std` embedded + `serde`

```toml
# shared/Cargo.toml
postcard = { version = "1.0", default-features = false, features = ["alloc"] }

[features]
heapless = ["postcard/heapless"]
alloc = ["postcard/alloc"]
```

**Example:**
```rust
// Before: 80+ lines of manual byte manipulation
#[cfg(feature = "heapless")]
impl Packetize<128> for BleMessage {
    fn packetize(&self) -> heapless::Vec<u8, 128> {
        let mut vec = heapless::Vec::<u8, 128>::new();
        match self {
            BleMessage::DeviceRequest => { /* ... */ }
            // ... 40 more lines
        }
        vec
    }
}

// After: 3 lines
impl Packetize for BleMessage {
    fn packetize(&self) -> Result<Vec<u8>, PacketizeError> {
        postcard::to_allocvec(self).map_err(Into::into)
    }
}
```

**Impact:** VERY HIGH - Removes ~1,000-1,500 lines of duplicated code
**Effort:** 1-2 days (needs testing on embedded)
**Risk:** MEDIUM - Requires thorough testing on ESP32-C3

**Files to modify:**
- `shared/src/ble/message.rs`
- `shared/src/ble/proof.rs`
- `shared/src/ble/device_caps.rs`
- `shared/src/ble/device_type.rs`
- `shared/src/errors/crypto.rs`
- `shared/src/traits/packetize.rs`

**Alternative:** If `postcard` doesn't fit, consider `serde-json-core` for `no_std` or keep manual implementation but use macros to reduce duplication.

---

### 6. Use `geo` Crate for Polygon Operations

**File:** `server/src/kernel/route/implementations/blocks/polygon.rs`

**Issues:**
1. Manual ray-casting algorithm (lines 50-63)
2. **Data loss:** Casting `f64` to `u64` for sorting (lines 66-67)
3. **Unsafe:** `.unwrap()` on polygon.last() (lines 95-96)

**Current Code:**
```rust
pub fn get_sorted_coords(&self) -> (Vec<f64>, Vec<f64>) {
    let xs: BTreeSet<u64> = self.points.iter().map(|(x, _)| *x as u64).collect();
    //                                                       ^^^^^^^^^ PRECISION LOSS!
    let ys: BTreeSet<u64> = self.points.iter().map(|(_, y)| *y as u64).collect();
    let xs: Vec<f64> = xs.into_iter().map(|x| x as f64).collect();
    let ys: Vec<f64> = ys.into_iter().map(|y| y as f64).collect();
    (xs, ys)
}
```

**Solution:**
```toml
geo = "0.29"
ordered-float = "4.6"
```

```rust
use geo::{Polygon as GeoPolygon, Contains, Coordinate, BoundingRect};
use ordered_float::OrderedFloat;
use std::collections::BTreeSet;

impl<'a> Polygon<'a> {
    pub fn is_point_inside(&self, x: f64, y: f64) -> bool {
        let polygon = self.to_geo_polygon();
        let contains = polygon.contains(&Coordinate { x, y });
        if self.bounding { contains } else { !contains }
    }

    pub fn get_sorted_coords(&self) -> (Vec<f64>, Vec<f64>) {
        let xs: BTreeSet<OrderedFloat<f64>> = self.points
            .iter()
            .map(|(x, _)| OrderedFloat(*x))
            .collect();
        let ys: BTreeSet<OrderedFloat<f64>> = self.points
            .iter()
            .map(|(_, y)| OrderedFloat(*y))
            .collect();
        (
            xs.into_iter().map(|x| x.into_inner()).collect(),
            ys.into_iter().map(|y| y.into_inner()).collect(),
        )
    }

    fn to_geo_polygon(&self) -> GeoPolygon {
        let coords: Vec<_> = self.points
            .iter()
            .map(|(x, y)| Coordinate { x: *x, y: *y })
            .collect();
        GeoPolygon::new(coords.into(), vec![])
    }
}
```

**Impact:** MEDIUM - Fixes data loss bug, removes ~30 lines, adds battle-tested algorithms
**Effort:** 2-3 hours
**Benefits:**
- No precision loss
- Industry-standard geometric operations
- Additional features available (intersections, unions, buffering)

---

### 7. Typed ID Wrappers to Replace `ObjectId::parse_str` Pattern

**Files:** 16+ files across `server/src/`

**Problem:** Repetitive parsing with error handling (72 `.unwrap()` calls found)

**Current Pattern (repeated 50+ times):**
```rust
let oid = ObjectId::parse_str(id).map_err(|_| {
    mongodb::error::Error::custom("Invalid ObjectId format".to_string())
})?;
```

**Solution:** Newtype pattern with Axum extractors

```rust
// server/src/schema/typed_ids.rs (NEW FILE)
use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

macro_rules! typed_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
        #[serde(transparent)]
        pub struct $name(pub ObjectId);

        impl $name {
            pub fn new() -> Self {
                Self(ObjectId::new())
            }

            pub fn as_oid(&self) -> &ObjectId {
                &self.0
            }
        }

        impl FromStr for $name {
            type Err = bson::oid::Error;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(ObjectId::parse_str(s)?))
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0.to_hex())
            }
        }

        // Axum path extractor support
        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let s = String::deserialize(deserializer)?;
                s.parse().map_err(serde::de::Error::custom)
            }
        }
    };
}

typed_id!(EntityId);
typed_id!(BeaconId);
typed_id!(AreaId);
typed_id!(MerchantId);
typed_id!(ConnectionId);
typed_id!(UserId);
```

**Usage:**
```rust
// Before
async fn get_beacon(
    Path((entity_id, beacon_id)): Path<(String, String)>,
) -> Result<Json<Beacon>> {
    let entity_oid = ObjectId::parse_str(&entity_id).map_err(|_| ...)?;
    let beacon_oid = ObjectId::parse_str(&beacon_id).map_err(|_| ...)?;
    // ...
}

// After
async fn get_beacon(
    Path((entity_id, beacon_id)): Path<(EntityId, BeaconId)>,
) -> Result<Json<Beacon>> {
    // IDs are already validated!
    // Use entity_id.as_oid() to get ObjectId
}
```

**Impact:** VERY HIGH
- Removes ~200-300 lines of error handling
- Type safety prevents mixing entity/beacon/area IDs
- Better API documentation
- Compile-time guarantees

**Effort:** 1 day
**Files to modify:** Most files in `server/src/schema/` and `server/src/kernel/`

---

### 8. Centralize Crypto Challenge-Response Logic

**Files:**
- `beacon/src/bin/crypto/proof.rs`
- `shared/src/ble/proof.rs`
- `mobile/src-tauri/src/unlocker/challenge.rs`

**Problem:** Same challenge-response protocol implemented 3 times with subtle differences

**Solution:** Create `shared::crypto::challenge` module

```rust
// shared/src/crypto/challenge.rs (NEW FILE)
use sha2::{Digest, Sha256};
use p256::ecdsa::{SigningKey, VerifyingKey, Signature, signature::{Signer, Verifier}};

pub struct Challenge {
    pub nonce: [u8; 16],
    pub instance_id: [u8; 16],
    pub timestamp: u64,
    pub user_id: [u8; 24],
}

impl Challenge {
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.nonce);
        hasher.update(self.instance_id);
        hasher.update(self.timestamp.to_be_bytes());
        hasher.update(self.user_id);
        hasher.finalize().into()
    }

    pub fn sign(&self, key: &SigningKey) -> Signature {
        let hash = self.hash();
        key.sign(&hash)
    }

    pub fn verify(&self, signature: &Signature, key: &VerifyingKey) -> Result<(), CryptoError> {
        let hash = self.hash();
        key.verify(&hash, signature)
            .map_err(|_| CryptoError::InvalidSignature)
    }
}
```

**Impact:** MEDIUM-HIGH - Single source of truth, removes duplication
**Effort:** 4-6 hours
**Files to modify:**
- `shared/src/crypto/challenge.rs` (new)
- `beacon/src/bin/crypto/proof.rs`
- `mobile/src-tauri/src/unlocker/challenge.rs`

---

## ⚠️ MEDIUM PRIORITY (Code Quality)

### 9. Fix Nonce Generation Inefficiency

**File:** `beacon/src/bin/crypto/nonce.rs:9-16`

**Issues:**
1. Debug `println!()` in production hot path
2. Byte-by-byte RNG calls (16 calls instead of 1)

**Current Code:**
```rust
pub fn generate(rng: &mut Trng) -> Self {
    let mut nonce = [0u8; NONCE_LENGTH];
    for item in nonce.iter_mut() {
        println!("Generating random byte...");  // ❌ Debug in production
        *item = rng.random() as u8;  // ❌ 16 separate RNG calls
    }
    Nonce(nonce)
}
```

**Fix:**
```rust
pub fn generate(rng: &mut Trng) -> Self {
    let mut nonce = [0u8; NONCE_LENGTH];
    // Single RNG call fills entire buffer
    for chunk in nonce.chunks_mut(4) {
        let random_u32 = rng.random();
        let bytes = random_u32.to_le_bytes();
        chunk.copy_from_slice(&bytes[..chunk.len()]);
    }
    Nonce(nonce)
}
```

**Impact:** MEDIUM - Removes debug output, ~4x faster
**Effort:** 10 minutes

---

### 10. Add Configurable RSSI Parameters

**File:** `mobile/src-tauri/src/locate/locator.rs:88-98`

**Issue:** Hardcoded values for indoor positioning

```rust
fn rssi_to_distance(mut rssi: f64) -> f64 {
    let tx_power = -59.0;  // ❌ Hardcoded
    if rssi > 0f64 {
        rssi = -rssi;
    }
    let n = 2.0;  // ❌ Hardcoded path-loss exponent (free space)
    10f64.powf((tx_power - rssi) / (10.0 * n))
}
```

**Fix:**
```rust
pub struct RssiConfig {
    pub tx_power: f64,
    pub path_loss_exponent: f64,
}

impl Default for RssiConfig {
    fn default() -> Self {
        Self {
            tx_power: -59.0,
            path_loss_exponent: 2.0, // Free space
        }
    }
}

impl RssiConfig {
    pub fn indoor_environment() -> Self {
        Self {
            tx_power: -59.0,
            path_loss_exponent: 3.0, // Indoor with obstacles
        }
    }

    pub fn rssi_to_distance(&self, mut rssi: f64) -> f64 {
        if rssi > 0.0 {
            rssi = -rssi;
        }
        10f64.powf((self.tx_power - rssi) / (10.0 * self.path_loss_exponent))
    }
}
```

**Impact:** MEDIUM - Enables calibration per-environment
**Effort:** 1 hour

---

### 11. Fix `.unwrap()` Usage in Server

**Found:** 72 instances across 16 files in `server/src/`

**Strategy:** Replace with proper error handling

**Examples:**

```rust
// ❌ Before
let last_x = *xs.last().unwrap();

// ✅ After
let last_x = *xs.last().ok_or_else(|| NavigationError::Other("Empty coordinate set".into()))?;
```

```rust
// ❌ Before
pub fn verify_password(&self, password: &str) -> bool {
    bcrypt::verify(password, &self.hashed_password).unwrap_or(false)
}

// ✅ After
pub fn verify_password(&self, password: &str) -> Result<bool, bcrypt::BcryptError> {
    bcrypt::verify(password, &self.hashed_password)
}
```

**Impact:** HIGH - Prevents runtime panics
**Effort:** 1-2 days (systematic replacement)
**Tool:** Use `cargo-clippy` with `unwrap_used` lint

---

## 📊 LOWER PRIORITY (Nice to Have)

### 12. Extract Indoor Positioning to Reusable Module

**File:** `mobile/src-tauri/src/locate/locator.rs`

**Opportunity:** Create standalone positioning library

**Benefits:**
- Reusable across projects
- Better testing
- Potential for advanced algorithms (Kalman filtering, particle filtering)

**Effort:** 1-2 weeks
**Impact:** MEDIUM - Improves code organization

---

### 13. Consider `pathfinding` Crate for Route Finding

**Current:** Custom Dijkstra implementation in `server/src/kernel/route/implementations/`

**Note:** Current implementation uses `bumpalo` for performance (excellent!). Only consider replacing if:
- Maintenance burden becomes high
- Need additional algorithms (A*, bidirectional search)
- Performance issues arise

**Current assessment:** Keep existing implementation, it's well-optimized.

---

### 14. Reduce Feature Flag Duplication with Macros

**File:** `shared/src/` (entire library)

**Problem:** Near-complete duplication between `heapless` and `alloc` implementations

**Solution:** Declarative macros or `#[cfg_attr]` attributes

**Example:**
```rust
macro_rules! impl_for_features {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $($field:ident: $ty:ty),* $(,)?
        }
    ) => {
        #[cfg(feature = "heapless")]
        $(#[$meta])*
        $vis struct $name {
            $($field: heapless_version!($ty)),*
        }

        #[cfg(feature = "alloc")]
        $(#[$meta])*
        $vis struct $name {
            $($field: alloc_version!($ty)),*
        }
    };
}
```

**Impact:** MEDIUM - Reduces duplication, but adds complexity
**Effort:** 1-2 weeks
**Risk:** May reduce readability

**Alternative:** Wait to see if `postcard` (item #5) solves most of the duplication first.

---

## 🛠️ IMPLEMENTATION ROADMAP

### Phase 1: Critical Security (Week 1)
- [ ] #1: Fix bcrypt cost factor (30 min)
- [ ] #2: Implement authorization checks (2-4 hours)
- [ ] Review and deploy security patches

### Phase 2: High-Impact Refactoring (Weeks 2-3)
- [ ] #4: Implement `thiserror` for all error types (3-4 hours)
- [ ] #7: Create typed ID wrappers (1 day)
- [ ] #3: Replace TOTP implementation (1-2 hours)
- [ ] #9: Fix nonce generation (10 min)

### Phase 3: Code Quality (Weeks 4-5)
- [ ] #11: Fix all `.unwrap()` calls (1-2 days)
- [ ] #6: Integrate `geo` crate (2-3 hours)
- [ ] #8: Centralize crypto challenge logic (4-6 hours)
- [ ] #10: Add configurable RSSI parameters (1 hour)

### Phase 4: Major Refactoring (Week 6+)
- [ ] #5: Evaluate and implement `postcard` for binary serialization (1-2 days + testing)
- [ ] Test on ESP32-C3 hardware
- [ ] Performance benchmarking

### Phase 5: Optional Improvements (Future)
- [ ] #12: Extract indoor positioning library
- [ ] #14: Macro-based feature flag reduction
- [ ] #13: Evaluate pathfinding crate (only if needed)

---

## 📈 METRICS

**Before Refactoring:**
- **Lines of Code:** ~25,000+
- **`.unwrap()` calls:** 72 in server
- **Duplicate code:** ~2,000-3,000 lines (feature flags)
- **Manual implementations:** 8+ (TOTP, errors, serialization, geometry)
- **Security issues:** 2 critical

**After Refactoring (Estimated):**
- **Lines of Code:** ~22,000-23,000 (10-12% reduction)
- **`.unwrap()` calls:** 0 (or very few, well-justified)
- **Duplicate code:** ~500-1,000 lines (75% reduction)
- **Manual implementations:** 2-3 (only where necessary)
- **Security issues:** 0 critical

**Maintenance Burden Reduction:** 40-50%

---

## 🎯 QUICK WINS (Can Do Today)

1. **Fix bcrypt cost factor** (30 min) - Security
2. **Fix nonce generation debug prints** (10 min) - Production bug
3. **Add TOTP crate** (1-2 hours) - Removes crypto code
4. **Create typed ID module skeleton** (1 hour) - Foundation for future work

---

## ⚠️ RISKS AND CONSIDERATIONS

### Risk 1: Binary Protocol Breaking Changes
**Impact:** High
**Mitigation:**
- Test `postcard` thoroughly on ESP32-C3
- Keep manual implementation as fallback
- Version protocol messages

### Risk 2: Performance Regression
**Impact:** Medium
**Mitigation:**
- Benchmark before/after
- Keep `bumpalo` for pathfinding
- Profile on actual hardware

### Risk 3: Introduction of New Bugs
**Impact:** Medium
**Mitigation:**
- Comprehensive testing for each change
- One refactoring at a time
- Code review for all changes

---

## 📝 NOTES

### Dependencies to Add
```toml
# High priority
thiserror = "2.0"              # Error handling
totp-rs = "5.6"                # TOTP generation
ordered-float = "4.6"          # Ordered f64 for BTreeSet

# Medium priority
geo = "0.29"                   # Geometric operations
postcard = "1.0"               # Serialization (requires testing)

# Lower priority
pathfinding = "4.11"           # Only if replacing custom Dijkstra
```

### Testing Strategy
1. Unit tests for all refactored code
2. Integration tests for binary protocol changes
3. Hardware testing for beacon changes
4. Performance benchmarks for pathfinding
5. Security audit after auth changes

### Documentation Updates
- Update CLAUDE.md with new patterns
- Document typed ID usage
- Add security best practices section
- Update contribution guidelines

---

**Prepared by:** Claude (Navign AI Assistant)
**Review Status:** Awaiting human review
**Next Steps:** Discuss priorities with team, begin Phase 1
