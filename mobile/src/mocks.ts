import type { Area, Merchant } from '@/schema'

export const mockMerchants: Merchant[] = [
  {
    _id: { $oid: '507f1f77bcf86cd799439011' },
    name: 'Brew & Beans Coffee',
    description:
      'Artisanal coffee shop serving locally roasted beans and fresh pastries',
    chain: null,
    entity: { $oid: '507f1f77bcf86cd799439001' },
    beacon_code: 'BB-COFFEE-001',
    area: { $oid: '507f1f77bcf86cd799439101' },
    type: { food: { cuisine: 'american', type: 'cafe' } },
    tags: ['coffee', 'breakfast', 'pastries', 'wifi', 'study-space'],
    location: [-74.006, 40.7128], // NYC coordinates
    style: 'store',
    polygon: [
      [-74.0061, 40.7129],
      [-74.0059, 40.7129],
      [-74.0059, 40.7127],
      [-74.0061, 40.7127],
      [-74.0061, 40.7129],
    ],
    website: 'https://brewandbeans.com',
    phone: '+1-212-555-0123',
    email: 'hello@brewandbeans.com',
    opening_hours: [
      [21600000, 79200000], // Monday: 6:00-22:00
      [21600000, 79200000], // Tuesday: 6:00-22:00
      [21600000, 79200000], // Wednesday: 6:00-22:00
      [21600000, 79200000], // Thursday: 6:00-22:00
      [21600000, 82800000], // Friday: 6:00-23:00
      [25200000, 82800000], // Saturday: 7:00-23:00
      [25200000, 75600000], // Sunday: 7:00-21:00
    ],
    images: [
      'https://example.com/images/brew-beans-exterior.jpg',
      'https://example.com/images/brew-beans-interior.jpg',
      'https://example.com/images/brew-beans-coffee.jpg',
    ],
    social_media: [
      {
        platform: 'instagram',
        handle: '@brewandbeansnyc',
        url: 'https://instagram.com/brewandbeansnyc',
      },
      {
        platform: 'facebook',
        handle: 'BrewAndBeansNYC',
        url: 'https://facebook.com/BrewAndBeansNYC',
      },
    ],
    created_at: Date.now() - Math.floor(Math.random() * 86400000 * 30),
    updated_at: Date.now() - Math.floor(Math.random() * 86400000),
  },
  {
    _id: { $oid: '507f1f77bcf86cd799439012' },
    name: 'Starbucks',
    description:
      "World's largest coffeehouse chain serving coffee, tea, and light snacks",
    chain: 'Starbucks',
    entity: { $oid: '507f1f77bcf86cd799439002' },
    beacon_code: 'SB-TIMES-001',
    area: { $oid: '507f1f77bcf86cd799439101' },
    type: { food: { cuisine: 'american', type: 'cafe' } },
    tags: ['coffee', 'chain', 'quick-service', 'wifi', 'mobile-order'],
    location: [-73.9857, 40.7589], // Times Square
    style: 'store',
    polygon: [
      [-73.9858, 40.759],
      [-73.9856, 40.759],
      [-73.9856, 40.7588],
      [-73.9858, 40.7588],
      [-73.9858, 40.759],
    ],
    website: 'https://starbucks.com',
    phone: '+1-212-555-0456',
    email: null,
    opening_hours: [
      [18000000, 86400000], // Monday: 5:00-24:00
      [18000000, 86400000], // Tuesday: 5:00-24:00
      [18000000, 86400000], // Wednesday: 5:00-24:00
      [18000000, 86400000], // Thursday: 5:00-24:00
      [18000000, 86400000], // Friday: 5:00-24:00
      [21600000, 86400000], // Saturday: 6:00-24:00
      [21600000, 82800000], // Sunday: 6:00-23:00
    ],
    images: [
      'https://example.com/images/starbucks-exterior.jpg',
      'https://example.com/images/starbucks-menu.jpg',
    ],
    social_media: [
      {
        platform: 'instagram',
        handle: '@starbucks',
        url: 'https://instagram.com/starbucks',
      },
      {
        platform: 'twitter',
        handle: '@Starbucks',
        url: 'https://twitter.com/Starbucks',
      },
    ],
    created_at: Date.now() - Math.floor(Math.random() * 86400000 * 30),
    updated_at: Date.now() - Math.floor(Math.random() * 86400000),
  },
  {
    _id: { $oid: '507f1f77bcf86cd799439013' },
    name: 'TechGear Electronics',
    description:
      'Your one-stop shop for the latest smartphones, laptops, and tech accessories',
    chain: null,
    entity: { $oid: '507f1f77bcf86cd799439003' },
    beacon_code: 'TG-ELEC-001',
    area: { $oid: '507f1f77bcf86cd799439102' },
    type: {
      electronics: {
        is_mobile: true,
        is_computer: true,
        is_accessories: true,
      },
    },
    tags: ['electronics', 'smartphones', 'laptops', 'accessories', 'repairs'],
    location: [-118.2437, 34.0522], // LA coordinates
    style: 'store',
    polygon: [
      [-118.244, 34.0525],
      [-118.2434, 34.0525],
      [-118.2434, 34.0519],
      [-118.244, 34.0519],
      [-118.244, 34.0525],
    ],
    website: 'https://techgearelectronics.com',
    phone: '+1-323-555-0789',
    email: 'info@techgearelectronics.com',
    opening_hours: [
      [36000000, 75600000], // Monday: 10:00-21:00
      [36000000, 75600000], // Tuesday: 10:00-21:00
      [36000000, 75600000], // Wednesday: 10:00-21:00
      [36000000, 75600000], // Thursday: 10:00-21:00
      [36000000, 79200000], // Friday: 10:00-22:00
      [32400000, 79200000], // Saturday: 9:00-22:00
      [39600000, 72000000], // Sunday: 11:00-20:00
    ],
    images: [
      'https://example.com/images/techgear-storefront.jpg',
      'https://example.com/images/techgear-interior.jpg',
      'https://example.com/images/techgear-products.jpg',
    ],
    social_media: [
      {
        platform: 'youtube',
        handle: '@TechGearElectronics',
        url: 'https://youtube.com/@TechGearElectronics',
      },
      {
        platform: 'tiktok',
        handle: '@techgearla',
        url: 'https://tiktok.com/@techgearla',
      },
    ],
    created_at: Date.now() - Math.floor(Math.random() * 86400000 * 30),
    updated_at: Date.now() - Math.floor(Math.random() * 86400000),
  },
  {
    _id: { $oid: '507f1f77bcf86cd799439014' },
    name: 'Golden Dragon Restaurant',
    description:
      'Authentic Cantonese cuisine with traditional dim sum and modern Chinese dishes',
    chain: null,
    entity: { $oid: '507f1f77bcf86cd799439004' },
    beacon_code: 'GD-REST-001',
    area: { $oid: '507f1f77bcf86cd799439103' },
    type: {
      food: {
        cuisine: {
          chinese: {
            cuisine: 'cantonese',
            specific: 'Dim Sum',
          },
        },
        type: {
          restaurant: {
            chinese: { cuisine: 'cantonese', specific: 'Dim Sum' },
          },
        },
      },
    },
    tags: ['chinese', 'dim-sum', 'authentic', 'family-dining', 'takeout'],
    location: [-122.4194, 37.7749], // San Francisco Chinatown
    style: 'store',
    polygon: [
      [-122.4197, 37.7752],
      [-122.4191, 37.7752],
      [-122.4191, 37.7746],
      [-122.4197, 37.7746],
      [-122.4197, 37.7752],
    ],
    website: 'https://goldendragonrestaurant.com',
    phone: '+1-415-555-0234',
    email: 'reservations@goldendragonrestaurant.com',
    opening_hours: [
      [39600000, 79200000], // Monday: 11:00-22:00
      [39600000, 79200000], // Tuesday: 11:00-22:00
      [39600000, 79200000], // Wednesday: 11:00-22:00
      [39600000, 79200000], // Thursday: 11:00-22:00
      [39600000, 82800000], // Friday: 11:00-23:00
      [36000000, 82800000], // Saturday: 10:00-23:00
      [36000000, 79200000], // Sunday: 10:00-22:00
    ],
    images: [
      'https://example.com/images/golden-dragon-exterior.jpg',
      'https://example.com/images/golden-dragon-dimsum.jpg',
      'https://example.com/images/golden-dragon-dining.jpg',
    ],
    social_media: [
      {
        platform: 'wechat',
        handle: 'GoldenDragonSF',
        url: 'https://weixin.qq.com/goldendragon',
      },
    ],
    created_at: Date.now() - Math.floor(Math.random() * 86400000 * 30),
    updated_at: Date.now() - Math.floor(Math.random() * 86400000),
  },
  {
    _id: { $oid: '507f1f77bcf86cd799439015' },
    name: 'Zen Wellness Spa',
    description:
      'Luxury spa offering massages, facials, and holistic wellness treatments',
    chain: null,
    entity: { $oid: '507f1f77bcf86cd799439005' },
    beacon_code: 'ZW-SPA-001',
    area: { $oid: '507f1f77bcf86cd799439104' },
    type: 'service',
    tags: ['spa', 'wellness', 'massage', 'facial', 'relaxation', 'luxury'],
    location: [-87.6298, 41.8781], // Chicago coordinates
    style: 'store',
    polygon: [
      [-87.6301, 41.8784],
      [-87.6295, 41.8784],
      [-87.6295, 41.8778],
      [-87.6301, 41.8778],
      [-87.6301, 41.8784],
    ],
    website: 'https://zenwellnessspa.com',
    phone: '+1-312-555-0567',
    email: 'bookings@zenwellnessspa.com',
    opening_hours: [
      [32400000, 75600000], // Monday: 9:00-21:00
      [32400000, 75600000], // Tuesday: 9:00-21:00
      [32400000, 75600000], // Wednesday: 9:00-21:00
      [32400000, 75600000], // Thursday: 9:00-21:00
      [28800000, 79200000], // Friday: 8:00-22:00
      [28800000, 79200000], // Saturday: 8:00-22:00
      [36000000, 72000000], // Sunday: 10:00-20:00
    ],
    images: [
      'https://example.com/images/zen-spa-entrance.jpg',
      'https://example.com/images/zen-spa-treatment-room.jpg',
      'https://example.com/images/zen-spa-relaxation.jpg',
    ],
    social_media: [
      {
        platform: 'instagram',
        handle: '@zenwellnessspa',
        url: 'https://instagram.com/zenwellnessspa',
      },
      {
        platform: 'facebook',
        handle: 'ZenWellnessSpaChicago',
        url: 'https://facebook.com/ZenWellnessSpaChicago',
      },
    ],
    created_at: Date.now() - Math.floor(Math.random() * 86400000 * 30),
    updated_at: Date.now() - Math.floor(Math.random() * 86400000),
  },
  {
    _id: { $oid: '507f1f77bcf86cd799439016' },
    name: 'GameZone Arcade',
    description:
      'Classic and modern arcade games, pinball machines, and competitive gaming',
    chain: null,
    entity: { $oid: '507f1f77bcf86cd799439006' },
    beacon_code: 'GZ-ARC-001',
    area: { $oid: '507f1f77bcf86cd799439105' },
    type: 'entertainment',
    tags: [
      'arcade',
      'gaming',
      'pinball',
      'entertainment',
      'family-fun',
      'tournaments',
    ],
    location: [-71.0589, 42.3601], // Boston coordinates
    style: 'store',
    polygon: [
      [-71.0592, 42.3604],
      [-71.0586, 42.3604],
      [-71.0586, 42.3598],
      [-71.0592, 42.3598],
      [-71.0592, 42.3604],
    ],
    website: 'https://gamezone-arcade.com',
    phone: '+1-617-555-0890',
    email: 'events@gamezone-arcade.com',
    opening_hours: [
      [50400000, 82800000], // Monday: 14:00-23:00
      [50400000, 82800000], // Tuesday: 14:00-23:00
      [50400000, 82800000], // Wednesday: 14:00-23:00
      [50400000, 86400000], // Thursday: 14:00-24:00
      [50400000, 7200000], // Friday: 14:00-02:00 (next day)
      [43200000, 7200000], // Saturday: 12:00-02:00 (next day)
      [43200000, 82800000], // Sunday: 12:00-23:00
    ],
    images: [
      'https://example.com/images/gamezone-exterior.jpg',
      'https://example.com/images/gamezone-arcade.jpg',
      'https://example.com/images/gamezone-pinball.jpg',
    ],
    social_media: [
      {
        platform: 'discord',
        handle: 'GameZoneArcade',
        url: 'https://discord.gg/gamezonearc',
      },
    ],
    created_at: Date.now() - Math.floor(Math.random() * 86400000 * 30),
    updated_at: Date.now() - Math.floor(Math.random() * 86400000),
  },
  {
    _id: { $oid: '507f1f77bcf86cd799439017' },
    name: 'Fashion Forward Boutique',
    description: 'Trendy clothing and accessories for the modern fashionista',
    chain: null,
    entity: { $oid: '507f1f77bcf86cd799439007' },
    beacon_code: 'FF-BOUT-001',
    area: { $oid: '507f1f77bcf86cd799439106' },
    type: {
      clothing: {
        is_menswear: false,
        is_womenswear: true,
        is_childrenswear: false,
      },
    },
    tags: ['fashion', 'women', 'trendy', 'accessories', 'boutique'],
    location: [-77.0369, 38.9072], // Washington DC coordinates
    style: 'store',
    polygon: [
      [-77.0372, 38.9075],
      [-77.0366, 38.9075],
      [-77.0366, 38.9069],
      [-77.0372, 38.9069],
      [-77.0372, 38.9075],
    ],
    website: 'https://fashionforwardboutique.com',
    phone: '+1-202-555-0345',
    email: 'info@fashionforwardboutique.com',
    opening_hours: [
      [36000000, 75600000], // Monday: 10:00-21:00
      [36000000, 75600000], // Tuesday: 10:00-21:00
      [36000000, 75600000], // Wednesday: 10:00-21:00
      [36000000, 75600000], // Thursday: 10:00-21:00
      [36000000, 79200000], // Friday: 10:00-22:00
      [32400000, 79200000], // Saturday: 9:00-22:00
      [43200000, 72000000], // Sunday: 12:00-20:00
    ],
    images: [
      'https://example.com/images/fashion-forward-exterior.jpg',
      'https://example.com/images/fashion-forward-interior.jpg',
      'https://example.com/images/fashion-forward-clothes.jpg',
    ],
    social_media: [
      {
        platform: 'instagram',
        handle: '@fashionforwarddc',
        url: 'https://instagram.com/fashionforwarddc',
      },
    ],
    created_at: Date.now() - Math.floor(Math.random() * 86400000 * 30),
    updated_at: Date.now() - Math.floor(Math.random() * 86400000),
  },
  {
    _id: { $oid: '507f1f77bcf86cd799439018' },
    name: 'Fresh Market',
    description:
      'Local supermarket with fresh produce, organic options, and everyday essentials',
    chain: null,
    entity: { $oid: '507f1f77bcf86cd799439008' },
    beacon_code: 'FM-SUPER-001',
    area: { $oid: '507f1f77bcf86cd799439107' },
    type: 'supermarket',
    tags: ['grocery', 'fresh-produce', 'organic', 'essentials', 'local'],
    location: [-122.3321, 47.6062], // Seattle coordinates
    style: 'store',
    polygon: [
      [-122.3325, 47.6065],
      [-122.3317, 47.6065],
      [-122.3317, 47.6059],
      [-122.3325, 47.6059],
      [-122.3325, 47.6065],
    ],
    website: 'https://freshmarket-seattle.com',
    phone: '+1-206-555-0678',
    email: 'customerservice@freshmarket-seattle.com',
    opening_hours: [
      [21600000, 79200000], // Monday: 6:00-22:00
      [21600000, 79200000], // Tuesday: 6:00-22:00
      [21600000, 79200000], // Wednesday: 6:00-22:00
      [21600000, 79200000], // Thursday: 6:00-22:00
      [21600000, 82800000], // Friday: 6:00-23:00
      [21600000, 82800000], // Saturday: 6:00-23:00
      [25200000, 75600000], // Sunday: 7:00-21:00
    ],
    images: [
      'https://example.com/images/fresh-market-exterior.jpg',
      'https://example.com/images/fresh-market-produce.jpg',
      'https://example.com/images/fresh-market-aisles.jpg',
    ],
    social_media: [
      {
        platform: 'facebook',
        handle: 'FreshMarketSeattle',
        url: 'https://facebook.com/freshmarketseattle',
      },
      {
        platform: 'instagram',
        handle: '@freshmarketseattle',
        url: 'https://instagram.com/freshmarketseattle',
      },
    ],
    created_at: Date.now() - Math.floor(Math.random() * 86400000 * 30),
    updated_at: Date.now() - Math.floor(Math.random() * 86400000),
  },
  {
    _id: { $oid: '507f1f77bcf86cd799439019' },
    name: 'Sakura Sushi Bar',
    description:
      'Traditional Japanese sushi and sashimi prepared by master chefs',
    chain: null,
    entity: { $oid: '507f1f77bcf86cd799439009' },
    beacon_code: 'SS-SUSHI-001',
    area: { $oid: '507f1f77bcf86cd799439108' },
    type: {
      food: {
        cuisine: 'japanese',
        type: { restaurant: 'japanese' },
      },
    },
    tags: ['japanese', 'sushi', 'sashimi', 'traditional', 'omakase'],
    location: [-104.9903, 39.7392], // Denver coordinates
    style: 'store',
    polygon: [
      [-104.9906, 39.7395],
      [-104.99, 39.7395],
      [-104.99, 39.7389],
      [-104.9906, 39.7389],
      [-104.9906, 39.7395],
    ],
    website: 'https://sakurasushibar.com',
    phone: '+1-303-555-0901',
    email: 'reservations@sakurasushibar.com',
    opening_hours: [
      [], // Monday: Closed
      [64800000, 79200000], // Tuesday: 18:00-22:00
      [64800000, 79200000], // Wednesday: 18:00-22:00
      [64800000, 79200000], // Thursday: 18:00-22:00
      [64800000, 82800000], // Friday: 18:00-23:00
      [64800000, 82800000], // Saturday: 18:00-23:00
      [64800000, 79200000], // Sunday: 18:00-22:00
    ],
    images: [
      'https://example.com/images/sakura-exterior.jpg',
      'https://example.com/images/sakura-sushi-bar.jpg',
      'https://example.com/images/sakura-sashimi.jpg',
    ],
    social_media: [
      {
        platform: 'instagram',
        handle: '@sakurasushibardenver',
        url: 'https://instagram.com/sakurasushibardenver',
      },
    ],
    created_at: Date.now() - Math.floor(Math.random() * 86400000 * 30),
    updated_at: Date.now() - Math.floor(Math.random() * 86400000),
  },
  {
    _id: { $oid: '507f1f77bcf86cd799439020' },
    name: 'HealthFirst Clinic',
    description:
      'Comprehensive healthcare services including general practice and specialty care',
    chain: null,
    entity: { $oid: '507f1f77bcf86cd799439010' },
    beacon_code: 'HF-CLINIC-001',
    area: { $oid: '507f1f77bcf86cd799439109' },
    type: 'health',
    tags: [
      'healthcare',
      'clinic',
      'general-practice',
      'specialists',
      'insurance-accepted',
    ],
    location: [-80.1918, 25.7617], // Miami coordinates
    style: 'store',
    polygon: [
      [-80.1921, 25.762],
      [-80.1915, 25.762],
      [-80.1915, 25.7614],
      [-80.1921, 25.7614],
      [-80.1921, 25.762],
    ],
    website: 'https://healthfirstclinic.com',
    phone: '+1-305-555-0234',
    email: 'appointments@healthfirstclinic.com',
    opening_hours: [
      [28800000, 61200000], // Monday: 8:00-17:00
      [28800000, 61200000], // Tuesday: 8:00-17:00
      [28800000, 61200000], // Wednesday: 8:00-17:00
      [28800000, 61200000], // Thursday: 8:00-17:00
      [28800000, 57600000], // Friday: 8:00-16:00
      [32400000, 50400000], // Saturday: 9:00-14:00
      [], // Sunday: Closed
    ],
    images: [
      'https://example.com/images/healthfirst-exterior.jpg',
      'https://example.com/images/healthfirst-lobby.jpg',
      'https://example.com/images/healthfirst-exam-room.jpg',
    ],
    social_media: [
      {
        platform: 'facebook',
        handle: 'HealthFirstClinicMiami',
        url: 'https://facebook.com/healthfirstmiami',
      },
      {
        platform: 'linkedin',
        handle: 'HealthFirst Clinic',
        url: 'https://linkedin.com/company/healthfirst-clinic',
      },
    ],
    created_at: Date.now() - Math.floor(Math.random() * 86400000 * 30),
    updated_at: Date.now() - Math.floor(Math.random() * 86400000),
  },
]

export const polygonMock = [
  {
    points: [
      [0.0, 0.0],
      [0.0, 75.0],
      [5.0, 75.0],
      [5.0, 70.0],
      [45.0, 70.0],
      [45.0, 72.0],
      [48.0, 72.0],
      [48.0, 66.0],
      [5.0, 66.0],
      [5.0, 40.0],
      [30.0, 40.0],
      [30.0, 36.0],
      [5.0, 36.0],
      [5.0, 0.0],
      [5.0, 0.0],
    ],
    fillColor: '#e8f4fd',
    strokeColor: '#2196f3',
    strokeWidth: 1,
  },
]

export const mockAreas = [
  {
    _id: {
      $oid: '68a83067bdfa76608b934ae9',
    },
    entity: {
      $oid: '68a8301fbdfa76608b934ae1',
    },
    name: 'Library & Multimedia Building F2 Main Corridor',
    description: null,
    beacon_code: '02',
    floor: {
      type: 'floor',
      name: 2,
    },
    polygon: [
      [0, 0],
      [0, 75],
      [5, 75],
      [5, 70],
      [45, 70],
      [45, 72],
      [48, 72],
      [48, 66],
      [5, 66],
      [5, 40],
      [30, 40],
      [30, 36],
      [5, 36],
      [5, 0],
      [5, 0],
    ],
  } as Area,
  {
    _id: {
      $oid: '68a83067bdfa76608b934aea',
    },
    entity: {
      $oid: '68a8301fbdfa76608b934ae1',
    },
    name: 'Library & Multimedia Building F3 Main Corridor',
    description: null,
    beacon_code: '03',
    floor: {
      type: 'floor',
      name: 3,
    },
    polygon: [
      [0, 60],
      [0, 75],
      [5, 75],
      [5, 70],
      [45, 70],
      [45, 72],
      [48, 72],
      [48, 66],
      [5, 66],
      [5, 60],
    ],
  } as Area,
  {
    _id: {
      $oid: '68a83067bdfa76608b934aeb',
    },
    entity: {
      $oid: '68a8301fbdfa76608b934ae1',
    },
    name: 'Library & Multimedia Building F4 Main Corridor',
    description: null,
    beacon_code: '04',
    floor: {
      type: 'floor',
      name: 4,
    },
    polygon: [
      [0, 60],
      [0, 75],
      [5, 75],
      [5, 70],
      [45, 70],
      [45, 72],
      [48, 72],
      [48, 66],
      [5, 66],
      [5, 60],
    ],
  } as Area,
]
