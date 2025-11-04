# Interactive Features - Usage Examples

This document provides usage examples and expected behavior for the interactive area description and merchant explanation features.

## User Interaction Flow

### Viewing Area Details

1. **Navigate to Map View**
   - User opens the Navigation View in the mobile app
   - Map displays with areas, merchants, and beacons

2. **Click on Area Boundary**
   - User clicks anywhere on the gray area boundary polygon
   - Area boundary has `cursor: pointer` style for visual feedback

3. **Area Details Dialog Opens**
   - Dialog slides in from the center of the screen
   - Shows loading skeleton while fetching data from server

4. **View Area Information**
   - **Header**: Area name with map marker icon
   - **Subtitle**: Beacon code (e.g., "AREA001")
   - **Description Card**: Detailed description of the area (if available)
   - **Floor Card**: Shows floor type and number (e.g., "Floor 2", "Basement 1")
   - **Boundary Points Card**: Number of polygon points and coordinate preview
   - **Entity ID Card**: Technical ID for reference

5. **Close Dialog**
   - Click X button in header
   - Click outside the dialog
   - Press ESC key (desktop)

### Viewing Merchant Details

1. **Navigate to Map View**
   - User sees colored merchant polygons with labels on the map

2. **Click on Merchant**
   - User clicks on a merchant polygon or label
   - Merchant polygon has `cursor: pointer` style

3. **Merchant Details Dialog Opens**
   - Dialog appears with loading state
   - Fetches complete merchant information from server

4. **View Merchant Information**
   - **Header**: 
     - Merchant name with store icon
     - Chain name (if part of a chain) with link icon
   
   - **Type & Style Card**:
     - Merchant type (formatted): "Italian Restaurant", "Electronics Store", etc.
     - Merchant style: "Store", "Kiosk", "Pop-Up", etc.
   
   - **Description Card** (if available):
     - Full text description of the merchant
   
   - **Tags Card**:
     - Colored tag pills for categorization
     - Examples: "food", "coffee", "electronics", "clothing"
   
   - **Contact Card** (if available):
     - Email: Clickable mailto: link
     - Phone: Clickable tel: link  
     - Website: Clickable external link with button
   
   - **Social Media Card** (if available):
     - Platform icons (Facebook, Instagram, Twitter, etc.)
     - Handles and clickable URLs
   
   - **Location Card**:
     - X, Y coordinates on the map
   
   - **Beacon Code Card**:
     - Unique identifier for the merchant's beacon

5. **Interact with Links**
   - Click email to open default mail client
   - Click phone to initiate call (on mobile)
   - Click website to open in external browser
   - Click social media links to open profiles

## Example Scenarios

### Scenario 1: Finding a Restaurant

**User Goal**: Find information about a restaurant in the mall

1. User navigates to map view
2. User sees "Pasta Palace" labeled on the map
3. User clicks on "Pasta Palace"
4. Dialog opens showing:
   - Name: "Pasta Palace"
   - Type: "Italian Restaurant"
   - Style: "Store"
   - Description: "Authentic Italian cuisine with fresh pasta made daily"
   - Tags: "food", "italian", "restaurant"
   - Contact:
     - Email: info@pastapalace.com
     - Phone: +1-555-0123
     - Website: www.pastapalace.com
   - Social Media:
     - Instagram: @pastapalace
     - Facebook: /pastapalace
5. User clicks phone number to call for reservations
6. Dialog closes after call is initiated

### Scenario 2: Exploring an Area

**User Goal**: Learn about the main shopping area

1. User navigates to map view
2. User clicks on the light gray area boundary
3. Dialog opens showing:
   - Name: "Central Plaza"
   - Description: "The main shopping area featuring luxury brands and dining options"
   - Floor: "Floor 2"
   - Beacon Code: "PLAZA01"
   - Boundary: "8 coordinate points"
4. User reads the description
5. User closes dialog to continue browsing

### Scenario 3: Mobile Store with Social Media

**User Goal**: Check out an electronics store and follow them on social media

1. User clicks on "TechHub" on the map
2. Dialog shows:
   - Name: "TechHub"
   - Chain: "TechHub Electronics Chain"
   - Type: "Mobile Devices & Accessories Store"
   - Tags: "electronics", "mobile", "accessories"
   - Contact:
     - Email: support@techhub.com
     - Phone: +1-555-0456
     - Website: www.techhub.com
   - Social Media:
     - Instagram: @techhub
     - Twitter: @techhub_official
     - Facebook: /techhubelectronics
3. User clicks Instagram link
4. Instagram app/website opens
5. User follows the store
6. User returns to app and closes dialog

## Error Handling Examples

### Area Not Found

**Situation**: User clicks on an area that no longer exists in the database

**Behavior**:
1. Dialog opens with loading state
2. After request completes, shows error state:
   - Red alert icon
   - Message: "Failed to load area details"
   - Specific error: "Area not found"
3. User can close dialog and try another area

### Merchant Not Found

**Situation**: Merchant was removed but still shown on cached map

**Behavior**:
1. Dialog opens with loading state
2. Shows error state:
   - Red alert icon
   - Message: "Merchant not found"
3. User closes dialog
4. Map should refresh to remove deleted merchant

### Network Error

**Situation**: Device loses internet connection

**Behavior**:
1. Dialog opens with loading state
2. Request times out or fails
3. Shows error state:
   - Red alert icon
   - Message: "Error: Network request failed"
4. User can close dialog and try again when connection is restored

## Loading States

### Fast Network (< 500ms)

- Skeleton loader appears briefly
- Content loads smoothly
- User barely notices loading state

### Slow Network (> 1 second)

- Skeleton loader is clearly visible
- Shows placeholders for:
  - Title and description areas
  - Card sections
  - Button areas
- User knows data is being fetched
- Prevents confusion about empty state

### Very Slow Network (> 5 seconds)

- Skeleton continues to show
- No timeout by default (handled by Tauri HTTP client)
- User can close dialog and try again
- Future enhancement: Add manual refresh button

## Accessibility Examples

### Keyboard Navigation

1. User presses Tab to navigate map
2. Map elements receive focus
3. User presses Enter to open dialog
4. Inside dialog:
   - Tab cycles through interactive elements
   - Shift+Tab goes backward
   - Enter activates buttons/links
   - ESC closes dialog

### Screen Reader

1. Screen reader announces: "Map view with 5 areas and 12 merchants"
2. User navigates to merchant
3. Screen reader announces: "Button: Coffee Bean, restaurant"
4. User activates button
5. Dialog opens, screen reader announces: "Dialog: Coffee Bean details"
6. Screen reader reads all text content
7. Links announced as clickable: "Email: info@coffeebean.com, link"

## Performance Expectations

### Initial Map Load

- Map SVG renders immediately with cached/default data
- Areas and merchants displayed with placeholders if needed
- Click events active even during loading

### Dialog Opening

- Dialog opens instantly (no delay)
- Loading skeleton shows immediately
- API request happens in background

### Subsequent Opens

- Same merchant/area clicked again
- Currently: Fresh fetch from server
- Future: Could use cache if data < 5 minutes old

### Multiple Dialogs

- Only one dialog can be open at a time
- Opening new dialog closes previous one
- State is managed properly to prevent conflicts

## Mobile-Specific Behavior

### Touch Interactions

- Single tap on area/merchant opens dialog
- No need for double-tap or long-press
- Tap outside dialog to close
- Swipe gesture to close (if supported by dialog component)

### Screen Size Adaptations

- Dialog is responsive:
  - Mobile: Full screen or near-full screen
  - Tablet: Centered with max-width
  - Desktop: Centered modal
- Content scrolls if necessary
- Touch-friendly button sizes (min 44x44 px)

### Phone/Email/Website Links

- Phone links: Opens native dialer on mobile
- Email links: Opens native mail app
- Website links: Opens in-app browser or external browser
- Social media links: Opens native app if installed, otherwise browser

## Data Examples

### Complete Area Example

```json
{
  "_id": { "$oid": "507f1f77bcf86cd799439011" },
  "entity": { "$oid": "507f1f77bcf86cd799439012" },
  "name": "Grand Atrium",
  "description": "The central hub of the mall featuring a stunning glass ceiling and water fountain. A perfect meeting point with seating areas and directory kiosks.",
  "beacon_code": "ATRIUM01",
  "floor": {
    "type": "floor",
    "name": 1
  },
  "polygon": [
    [0, 0],
    [100, 0],
    [100, 50],
    [75, 100],
    [25, 100],
    [0, 50]
  ]
}
```

### Complete Merchant Example

```json
{
  "_id": { "$oid": "507f1f77bcf86cd799439013" },
  "name": "The Coffee Bean",
  "description": "Premium artisan coffee shop serving single-origin beans from around the world. Features a cozy seating area perfect for work or meetings.",
  "chain": "Coffee Bean International",
  "entity": { "$oid": "507f1f77bcf86cd799439012" },
  "beacon_code": "MERCH101",
  "area": { "$oid": "507f1f77bcf86cd799439011" },
  "location": [45.5, 32.8],
  "polygon": [
    [40, 30],
    [50, 30],
    [50, 35],
    [40, 35]
  ],
  "tags": ["food", "coffee", "cafe", "wifi"],
  "type": {
    "food": {
      "cuisine": "american",
      "type": "cafe"
    }
  },
  "style": "store",
  "email": "info@coffeebean.com",
  "phone": "+1-555-COFFEE",
  "website": "https://www.coffeebean.com",
  "social_media": [
    {
      "platform": "instagram",
      "handle": "@thecoffeebean",
      "url": "https://instagram.com/thecoffeebean"
    },
    {
      "platform": "facebook",
      "handle": "thecoffeebean",
      "url": "https://facebook.com/thecoffeebean"
    }
  ]
}
```

## Integration with Other Features

### With Navigation

- User views merchant details
- Clicks "Navigate to" button (future feature)
- Dialog closes
- Navigation starts to merchant location

### With Search

- User searches for "coffee"
- Results show "The Coffee Bean"
- User clicks result
- Map zooms to merchant
- Merchant details dialog opens automatically

### With Location Services

- User is located in "Grand Atrium"
- User clicks current area boundary
- Dialog shows area details with note: "You are here"
- Provides context about current location

## Tips for Users

1. **Quick Info**: Click any area or merchant for instant details
2. **Contact**: All contact information is clickable
3. **Share**: Copy beacon codes to share locations with others
4. **Explore**: Click around the map to learn about the space
5. **Close**: Tap outside dialog or press ESC to close quickly
