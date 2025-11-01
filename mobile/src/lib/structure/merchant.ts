import type {
  MerchantType,
  FoodCuisine,
  ChineseFoodCuisine,
  FoodType,
} from '@/schema'

/**
 * Formats a MerchantType for English display
 * @param type The merchant type to format
 * @returns A human-readable English string representation
 */
export function formatMerchantType(type: MerchantType): string {
  if (typeof type === 'string') {
    return formatSimpleType(type)
  }

  if ('food' in type) {
    return formatFoodType(type.food)
  }

  if ('electronics' in type) {
    return formatElectronicsType(type.electronics)
  }

  if ('clothing' in type) {
    return formatClothingType(type.clothing)
  }

  return 'Unknown'
}

/**
 * Formats simple string merchant types
 */
function formatSimpleType(type: string): string {
  const typeMap: Record<string, string> = {
    supermarket: 'Supermarket',
    health: 'Health & Medical',
    entertainment: 'Entertainment',
    service: 'Service',
    room: 'Room',
    other: 'Other',
  }

  return typeMap[type] || type.charAt(0).toUpperCase() + type.slice(1)
}

/**
 * Formats food merchant types with cuisine and type details
 */
function formatFoodType(food: {
  cuisine: FoodCuisine | null
  type: FoodType
}): string {
  const foodTypeStr = formatFoodTypeOnly(food.type)
  const cuisineStr = food.cuisine ? formatCuisine(food.cuisine) : null

  if (cuisineStr && foodTypeStr.toLowerCase() !== 'restaurant') {
    return `${cuisineStr} ${foodTypeStr}`
  } else if (cuisineStr) {
    return `${cuisineStr} Restaurant`
  } else {
    return foodTypeStr
  }
}

/**
 * Formats the food type portion only
 */
function formatFoodTypeOnly(foodType: FoodType): string {
  if (typeof foodType === 'string') {
    const typeMap: Record<string, string> = {
      cafe: 'Cafe',
      fastFood: 'Fast Food',
      bakery: 'Bakery',
      bar: 'Bar',
      other: 'Restaurant',
    }
    return typeMap[foodType] || 'Restaurant'
  }

  if ('restaurant' in foodType) {
    return 'Restaurant'
  }

  return 'Restaurant'
}

/**
 * Formats cuisine types with specific details
 */
function formatCuisine(cuisine: FoodCuisine): string {
  if (typeof cuisine === 'string') {
    const cuisineMap: Record<string, string> = {
      italian: 'Italian',
      indian: 'Indian',
      american: 'American',
      japanese: 'Japanese',
      korean: 'Korean',
      french: 'French',
      thai: 'Thai',
      mexican: 'Mexican',
      mediterranean: 'Mediterranean',
      spanish: 'Spanish',
      vietnamese: 'Vietnamese',
      middleEastern: 'Middle Eastern',
      african: 'African',
    }
    return (
      cuisineMap[cuisine] || cuisine.charAt(0).toUpperCase() + cuisine.slice(1)
    )
  }

  if ('chinese' in cuisine) {
    return formatChineseCuisine(cuisine.chinese)
  }

  if ('other' in cuisine) {
    return cuisine.other
  }

  return 'International'
}

/**
 * Formats Chinese cuisine with regional specifics
 */
function formatChineseCuisine(chinese: {
  cuisine: ChineseFoodCuisine
  specific: string | null
}): string {
  const regionMap: Record<ChineseFoodCuisine, string> = {
    cantonese: 'Cantonese',
    sichuan: 'Sichuan',
    hunan: 'Hunan',
    jiangxi: 'Jiangxi',
    shanghai: 'Shanghai',
    hangzhou: 'Hangzhou',
    ningbo: 'Ningbo',
    northern: 'Northern Chinese',
    other: 'Chinese',
  }

  const regionStr = regionMap[chinese.cuisine]

  if (chinese.specific) {
    return `${regionStr} (${chinese.specific})`
  }

  return `${regionStr} Chinese`
}

/**
 * Formats electronics merchant types with category details
 */
function formatElectronicsType(electronics: {
  is_mobile: boolean
  is_computer: boolean
  is_accessories: boolean
}): string {
  const categories: string[] = []

  if (electronics.is_mobile) categories.push('Mobile Devices')
  if (electronics.is_computer) categories.push('Computers')
  if (electronics.is_accessories) categories.push('Accessories')

  if (categories.length === 0) {
    return 'Electronics Store'
  } else if (categories.length === 1) {
    return `${categories[0]} Store`
  } else if (categories.length === 2) {
    return `${categories.join(' & ')} Store`
  } else {
    return 'Electronics Store'
  }
}

/**
 * Formats clothing merchant types with target demographic details
 */
function formatClothingType(clothing: {
  is_menswear: boolean
  is_womenswear: boolean
  is_childrenswear: boolean
}): string {
  const demographics: string[] = []

  if (clothing.is_menswear) demographics.push("Men's")
  if (clothing.is_womenswear) demographics.push("Women's")
  if (clothing.is_childrenswear) demographics.push("Children's")

  if (demographics.length === 0) {
    return 'Clothing Store'
  } else if (demographics.length === 1) {
    return `${demographics[0]} Clothing`
  } else if (demographics.length === 2) {
    return `${demographics.join(' & ')} Clothing`
  } else {
    return 'Family Clothing Store'
  }
}

/**
 * Gets a short category label for the merchant type (useful for tags, filters)
 * @param type The merchant type to categorize
 * @returns A short category string
 */
export function getMerchantCategory(type: MerchantType): string {
  if (typeof type === 'string') {
    return type.charAt(0).toUpperCase() + type.slice(1)
  }

  if ('food' in type) {
    return 'Food & Dining'
  }

  if ('electronics' in type) {
    return 'Electronics'
  }

  if ('clothing' in type) {
    return 'Fashion'
  }

  return 'Other'
}

/**
 * Gets detailed type information as an object (useful for filtering/search)
 * @param type The merchant type to analyze
 * @returns An object with detailed type information
 */
export function getMerchantTypeDetails(type: MerchantType): {
  category: string
  subcategory?: string
  details?: string[]
  displayName: string
} {
  const displayName = formatMerchantType(type)

  if (typeof type === 'string') {
    return {
      category: formatSimpleType(type),
      displayName,
    }
  }

  if ('food' in type) {
    const details: string[] = []
    if (type.food.cuisine) {
      details.push(`Cuisine: ${formatCuisine(type.food.cuisine)}`)
    }
    details.push(`Type: ${formatFoodTypeOnly(type.food.type)}`)

    return {
      category: 'Food & Dining',
      subcategory: formatFoodTypeOnly(type.food.type),
      details,
      displayName,
    }
  }

  if ('electronics' in type) {
    const details: string[] = []
    if (type.electronics.is_mobile) details.push('Mobile Devices')
    if (type.electronics.is_computer) details.push('Computers')
    if (type.electronics.is_accessories) details.push('Accessories')

    return {
      category: 'Electronics',
      subcategory: 'Electronics Store',
      details:
        details.length > 0 ? [`Categories: ${details.join(', ')}`] : undefined,
      displayName,
    }
  }

  if ('clothing' in type) {
    const details: string[] = []
    if (type.clothing.is_menswear) details.push("Men's")
    if (type.clothing.is_womenswear) details.push("Women's")
    if (type.clothing.is_childrenswear) details.push("Children's")

    return {
      category: 'Fashion',
      subcategory: 'Clothing Store',
      details:
        details.length > 0 ? [`Target: ${details.join(', ')}`] : undefined,
      displayName,
    }
  }

  return {
    category: 'Other',
    displayName,
  }
}
