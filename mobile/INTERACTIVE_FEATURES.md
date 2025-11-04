# Interactive Area Description and Merchant Explanation Features

This document describes the interactive features added to the mobile client for displaying detailed information about areas and merchants.

## Overview

Users can now click on areas and merchants in the map view to see detailed information in interactive dialogs. This enhances the user experience by providing quick access to location details without navigation away from the map.

## Features

### Area Details Dialog

When a user clicks on an area boundary in the map, a dialog opens displaying:

- **Area Name**: The name of the area
- **Description**: Detailed description of the area (if available)
- **Floor Information**: Floor type and number (Level, Floor, or Basement)
- **Boundary Points**: Number of coordinate points and preview of coordinates
- **Beacon Code**: Unique identifier for the area's beacons
- **Entity ID**: Parent entity reference

### Merchant Details Dialog

When a user clicks on a merchant in the map, a dialog opens displaying:

- **Merchant Name and Chain**: Store name and chain affiliation (if applicable)
- **Type and Style**: Merchant type (Food, Electronics, Clothing, etc.) and style (Store, Kiosk, etc.)
- **Description**: Detailed description of the merchant
- **Tags**: Categorization tags for filtering and search
- **Contact Information**:
  - Email address (clickable mailto link)
  - Phone number (clickable tel link)
  - Website (clickable external link)
- **Social Media**: Links to social media profiles with platform-specific icons
  - Supported platforms: Facebook, Instagram, Twitter, LinkedIn, TikTok, WeChat, Weibo, RedNote, Bluesky, Reddit, Discord, WhatsApp, Telegram
- **Location**: Coordinate position on the map
- **Beacon Code**: Unique identifier for the merchant's beacon

## Technical Implementation

### Backend (Rust/Tauri)

#### New API Handlers

Two new Tauri command handlers were added to `src-tauri/src/api/map.rs`:

```rust
#[tauri::command]
pub async fn get_area_details_handler(
    _app: AppHandle,
    entity: String,
    area: String,
) -> Result<String, String>

#[tauri::command]
pub async fn get_merchant_details_handler(
    _app: AppHandle,
    entity: String,
    merchant: String,
) -> Result<String, String>
```

These handlers:
1. Fetch detailed information from the server using the existing API
2. Return JSON responses with status and data/error information
3. Are registered in the main Tauri application builder

#### Server API Integration

The handlers make HTTP requests to:
- `GET /api/entities/{entity}/areas/{area}` - For area details
- `GET /api/entities/{entity}/merchants/{merchant}` - For merchant details

#### SVG Map Enhancements

The SVG map generation was enhanced to make elements visually clickable:
- Area boundaries have `cursor: pointer` style
- Merchant polygons have `cursor: pointer` style

### Frontend (Vue/TypeScript)

#### API Layer

New functions in `src/lib/api/tauri.ts`:

```typescript
export async function getAreaDetails(
  entity: string,
  area: string,
): Promise<ApiResponse<AreaDetails>>

export async function getMerchantDetails(
  entity: string,
  merchant: string,
): Promise<ApiResponse<MerchantDetails>>
```

Complete TypeScript interfaces were defined for both response types.

#### Vue Components

**AreaDetailsDialog.vue** (`src/components/map/AreaDetailsDialog.vue`):
- Displays area information in a clean, organized dialog
- Shows loading skeleton while fetching data
- Handles errors gracefully with user-friendly messages
- Uses existing UI components (Dialog, Card, Skeleton, Icon)

**MerchantDetailsDialog.vue** (`src/components/map/MerchantDetailsDialog.vue`):
- Comprehensive merchant information display
- Formats merchant type using existing utility functions
- Clickable contact information (email, phone, website)
- Social media links with platform-specific icons
- Responsive layout for mobile and desktop

#### Integration

**MapDisplay.vue** updates:
- Added `areaClick` event emission when area boundary is clicked
- Enhanced click handler to detect area boundary clicks
- Maintained existing beacon and merchant click handling

**NavigationView.vue** updates:
- Imported both dialog components
- Added state management for dialog visibility and selected IDs
- Implemented click handlers to open appropriate dialogs
- Integrated dialogs into the view template

## Usage

### For Users

1. Navigate to the map view in the mobile app
2. Click on any area boundary (the gray polygon) to see area details
3. Click on any merchant (colored rectangles with labels) to see merchant details
4. The dialog opens automatically with detailed information
5. Close the dialog by clicking the X button or outside the dialog

### For Developers

#### Adding New Fields

To add new fields to area or merchant details:

1. Update the `AreaResponse` or `MerchantResponse` struct in Rust (`src-tauri/src/api/map.rs`)
2. Update the corresponding TypeScript interface in `src/lib/api/tauri.ts`
3. Add the display logic to the appropriate Vue component
4. Update tests if necessary

#### Customizing Dialogs

The dialogs use the shadcn-vue component library. To customize styling:

1. Edit the Card, CardContent, and other UI components
2. Modify the CSS in the component's style section
3. Adjust Tailwind classes for spacing and colors

## Testing

Comprehensive tests were added to `src/lib/api/tauri.test.ts`:

- Test successful area details fetching
- Test successful merchant details fetching
- Test error handling for not found responses
- Mock all Tauri invoke calls
- Verify correct parameters are passed to handlers

Run tests with:
```bash
pnpm test
```

## Future Enhancements

Potential improvements:

1. **Caching**: Cache area and merchant details to reduce server requests
2. **Images**: Add image gallery support for merchants
3. **Reviews**: Integrate user reviews and ratings
4. **Opening Hours**: Add visual representation of opening hours
5. **Navigation**: Quick "Navigate to" button in merchant dialog
6. **Accessibility**: Enhanced screen reader support
7. **Offline Mode**: Store details locally for offline access
8. **Analytics**: Track which areas/merchants users view most often

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                       User Interface                         │
│  ┌─────────────────┐              ┌────────────────────┐   │
│  │ NavigationView  │              │   MapDisplay       │   │
│  │   (Container)   │──────────────│   (Map + SVG)      │   │
│  └────────┬────────┘              └──────────┬─────────┘   │
│           │                                   │              │
│           │ Opens Dialogs                     │ Emits Events │
│           │                                   │              │
│  ┌────────▼────────┐              ┌──────────▼─────────┐   │
│  │AreaDetails     │              │MerchantDetails     │   │
│  │Dialog.vue      │              │Dialog.vue          │   │
│  └────────┬────────┘              └──────────┬─────────┘   │
└───────────┼───────────────────────────────────┼─────────────┘
            │                                   │
            │ API Calls                         │ API Calls
            │                                   │
┌───────────▼───────────────────────────────────▼─────────────┐
│                   TypeScript API Layer                       │
│         getAreaDetails()    getMerchantDetails()            │
└───────────┬───────────────────────────────────┬─────────────┘
            │                                   │
            │ Tauri Invoke                      │ Tauri Invoke
            │                                   │
┌───────────▼───────────────────────────────────▼─────────────┐
│                      Rust Backend (Tauri)                    │
│   get_area_details_handler()  get_merchant_details_handler()│
└───────────┬───────────────────────────────────┬─────────────┘
            │                                   │
            │ HTTP GET                          │ HTTP GET
            │                                   │
┌───────────▼───────────────────────────────────▼─────────────┐
│                         Server API                           │
│   /api/entities/{entity}/areas/{area}                       │
│   /api/entities/{entity}/merchants/{merchant}               │
└─────────────────────────────────────────────────────────────┘
```

## Security Considerations

- All API requests go through the Rust backend, preventing direct server access from JavaScript
- Entity and area/merchant IDs are validated server-side
- No sensitive information is exposed in error messages
- Rate limiting should be implemented on the server side for these endpoints

## Performance

- Dialogs load data on-demand (only when opened)
- Loading states provide immediate feedback to users
- Error states prevent blocking the UI
- SVG cursor styles provide instant visual feedback
- Future caching would further improve performance

## Accessibility

- Dialogs are keyboard navigable
- Screen reader friendly with proper ARIA labels
- Focus management when dialogs open/close
- Semantic HTML structure
- High contrast colors for readability
- Icon + text combinations for clarity

## Browser Compatibility

Compatible with all modern browsers supported by Tauri 2.0:
- Chrome/Chromium 90+
- Firefox 88+
- Safari 14+
- Edge 90+

Mobile platforms:
- iOS 13+
- Android 7+ (API level 24+)
