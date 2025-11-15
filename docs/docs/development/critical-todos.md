# Critical TODOs and FIXMEs in Navign Codebase

**Generated:** 2025-11-09
**Total Found:** 27 actionable items (excluding WeChat mini-program type definitions)

---

## 🚨 SECURITY & FUNCTIONALITY CRITICAL

### 1. Missing Authorization Check
**File:** `server/src/kernel/unlocker/instance.rs:131`
```rust
// TODO verify if the user is allowed to unlock this beacon
```
**Impact:** HIGH - Any user can unlock any beacon
**Priority:** IMMEDIATE
**Effort:** 2-4 hours

---

### 2. Merchant Data Loading Issue
**File:** `mobile/src-tauri/src/unlocker/pipeline.rs:94`
```rust
// FIXME merchant not loaded properly from DB?
```
**Impact:** MEDIUM - UI may show incorrect merchant info
**Priority:** HIGH
**Effort:** 2-3 hours

---

## 🔧 MISSING IMPLEMENTATIONS (Documented but Not Implemented)

### 3. BluFi WiFi Provisioning
**Files:**
- `mobile/src-tauri/src/blufi/mod.rs` (multiple TODOs)
- `mobile/src/components/beacon/BluFiProvisioning.vue:8`
- `beacon/src/bin/main.rs:182`

**Missing Functions:**
```rust
// TODO: Implement BLE scanning (line 47)
pub fn scan_blufi_devices() -> Result<Vec<BlufiDevice>> { todo!() }

// TODO: Implement BLE connection (line 63)
pub fn connect_blufi(address: String) -> Result<BlufiConnection> { todo!() }

// TODO: Implement WiFi network scanning (line 79)
pub fn scan_wifi_networks(connection: &BlufiConnection) -> Result<Vec<WifiNetwork>> { todo!() }

// TODO: Implement provisioning (line 97)
pub fn provision_wifi(...) -> Result<ProvisionResult> { todo!() }

// TODO: Implement disconnection (line 122)
pub fn disconnect_blufi(connection: BlufiConnection) -> Result<()> { todo!() }
```

**Impact:** HIGH - WiFi provisioning for beacons not working
**Priority:** HIGH (if beacons need WiFi for OTA)
**Effort:** 2-3 days
**Dependencies:** BLE stack, BluFi protocol implementation

---

### 4. OTA Update Handler
**File:** `beacon/src/bin/main.rs:181`
```rust
// TODO: Handle the OTA update process here, possibly restarting the device if an update was applied.
```
**Impact:** MEDIUM - OTA updates won't work properly
**Priority:** MEDIUM
**Effort:** 4-6 hours
**Note:** OTA manager exists, just needs integration

---

### 5. Ownership Problem in Beacon Execute
**File:** `beacon/src/bin/execute/mod.rs:165`
```rust
todo!("Ownership problem")
```
**Impact:** UNKNOWN - Need to investigate context
**Priority:** HIGH (it's a `todo!()` that will panic)
**Effort:** 1-2 hours

---

## 📚 DOCUMENTATION NEEDED

### 6. Proof Management Documentation
**File:** `beacon/src/bin/crypto/proof.rs:2`
```rust
//! TODO proof management
```
**Impact:** LOW - Documentation only
**Priority:** MEDIUM
**Effort:** 2-3 hours for comprehensive docs

---

## 🔄 REFACTORING / IMPROVEMENTS

### 7. Change Merchant Type to Enum
**File:** `mobile/src-tauri/src/locate/merchant.rs:12`
```rust
/// TODO: Change to enum
```
**Impact:** LOW - Type safety improvement
**Priority:** LOW
**Effort:** 1 hour

---

### 8. Replace Areas Vec with HashMap in Entity
**File:** `server/src/kernel/route/types/entity.rs:11`
```rust
/// TODO: use HashMap instead.
```
**Impact:** MEDIUM - Performance improvement for area lookups
**Priority:** MEDIUM
**Effort:** 2-3 hours
**Benefit:** O(1) lookups instead of O(n)

---

### 9. Future Triangulation-Based Pathfinding
**File:** `shared/src/pathfinding/polygon.rs:169`
```rust
/// TODO: Future implementation for triangulation-based pathfinding
```
**Impact:** LOW - Future enhancement
**Priority:** LOW
**Effort:** 1-2 weeks

---

### 10. Fix SVG Coordinate Swap
**File:** `mobile/src/components/map/MapDisplay.vue:105`
```rust
// FIXME Swap x and y to match SVG coordinate system
```
**Impact:** MEDIUM - May cause coordinate confusion
**Priority:** MEDIUM
**Effort:** 1-2 hours + testing

---

## 🔐 AUTH TOKEN TODOs

### 11-12. Token Implementation Incomplete
**File:** `server/src/kernel/auth/token.rs:16, 57`
```rust
// TODO (appears twice)
```
**Context:** Need to read file to understand context
**Impact:** UNKNOWN
**Priority:** MEDIUM
**Effort:** Unknown until investigated

---

## 📊 PRIORITY MATRIX

### Do Immediately (This Week)
1. ✅ Missing authorization check (#1)
2. ✅ Ownership problem in beacon execute (#5)
3. ✅ Investigate token TODOs (#11-12)

### Do Soon (Next 2 Weeks)
1. Merchant data loading fix (#2)
2. BluFi WiFi provisioning (#3) - if needed for deployment
3. OTA update handler (#4)
4. SVG coordinate swap fix (#10)

### Nice to Have (Future)
1. HashMap for entity areas (#8)
2. Merchant type enum (#7)
3. Proof management docs (#6)
4. Triangulation pathfinding (#9)

---

## 🔍 INVESTIGATION NEEDED

The following TODOs need code review to understand full context:

1. **Token auth TODOs** - `server/src/kernel/auth/token.rs:16, 57`
2. **Ownership problem** - `beacon/src/bin/execute/mod.rs:165`

### Investigation Checklist
- [ ] Read `server/src/kernel/auth/token.rs` fully
- [ ] Read `beacon/src/bin/execute/mod.rs` around line 165
- [ ] Determine if these are blockers for production
- [ ] Create GitHub issues for each TODO
- [ ] Estimate effort for completion

---

## 📝 RECOMMENDED ACTIONS

### Short Term (1-2 Days)
```bash
# 1. Add authorization check
git checkout -b fix/beacon-authorization
# Implement UserPermissions check in server/src/kernel/unlocker/instance.rs:131
# Test thoroughly
# Create PR

# 2. Fix beacon execute ownership
git checkout -b fix/beacon-execute-ownership
# Investigate and fix beacon/src/bin/execute/mod.rs:165
# Test on hardware
# Create PR

# 3. Fix merchant loading
git checkout -b fix/merchant-loading
# Debug mobile/src-tauri/src/unlocker/pipeline.rs:94
# Create PR
```

### Medium Term (1-2 Weeks)
```bash
# 4. Implement BluFi if needed for production
git checkout -b feat/blufi-provisioning
# Implement all BluFi TODOs
# Extensive testing

# 5. Complete OTA integration
git checkout -b feat/ota-complete
# Implement beacon/src/bin/main.rs:181
# Test OTA flow end-to-end
```

### Long Term (Future Sprints)
- Refactor entity areas to use HashMap
- Implement triangulation-based pathfinding
- Comprehensive proof management documentation

---

## 🎯 SUCCESS CRITERIA

**Phase 1 Complete When:**
- [ ] No security-critical TODOs remain
- [ ] No `todo!()` macros in production paths
- [ ] All FIXMEs investigated and documented

**Phase 2 Complete When:**
- [ ] BluFi provisioning works (if required)
- [ ] OTA updates fully functional
- [ ] All medium-priority items resolved

**Phase 3 Complete When:**
- [ ] All TODO comments removed or converted to GitHub issues
- [ ] Documentation complete
- [ ] Performance optimizations implemented

---

**Next Step:** Review this list with the team and create GitHub issues for tracking.
