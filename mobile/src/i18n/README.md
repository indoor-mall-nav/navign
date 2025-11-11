# Internationalization (i18n) for Navign Mobile

This directory contains the internationalization setup for the Navign mobile application using `vue-i18n`.

## Supported Languages

- **English (en-US)** - Default and fallback language
- **Simplified Chinese (zh-CN)** - 简体中文
- **Traditional Chinese (zh-TW)** - 繁體中文
- **Japanese (ja-JP)** - 日本語
- **French (fr-FR)** - Français

## Directory Structure

```
i18n/
├── index.ts              # i18n configuration and setup
├── locales/              # Translation files
│   ├── en-US.json        # English translations
│   ├── zh-CN.json        # Simplified Chinese translations
│   ├── zh-TW.json        # Traditional Chinese translations
│   ├── ja-JP.json        # Japanese translations
│   └── fr-FR.json        # French translations
└── README.md             # This file
```

## Translation File Structure

All translation files follow the same structure with nested objects:

```json
{
  "common": {
    "email": "Email",
    "username": "Username",
    ...
  },
  "auth": {
    "loginTitle": "Login to Proceed",
    ...
  },
  "home": {
    ...
  },
  ...
}
```

### Main Categories

- **common**: Common UI elements (buttons, labels, actions)
- **auth**: Authentication-related strings (login, register)
- **home**: Home view strings
- **navigation**: Navigation-related strings
- **entityDetails**: Entity details view strings
- **area**: Area-related strings
- **merchant**: Merchant-related strings
- **beacon**: Beacon-related strings
- **units**: Measurement units
- **errors**: Error messages

## Usage in Vue Components

### 1. Import the Composition API

```vue
<script setup lang="ts">
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
</script>
```

### 2. Use Translation Keys in Template

```vue
<template>
  <h1>{{ t('home.title') }}</h1>
  <p>{{ t('home.findLocation.description') }}</p>
  <Button>{{ t('common.login') }}</Button>
</template>
```

### 3. Dynamic Translations with Parameters

```vue
<template>
  <!-- English: "Found 5 entities. Please select one below." -->
  <!-- Chinese: "找到 5 个实体。请在下方选择一个。" -->
  <p>{{ t('home.findLocation.multipleResults', { count: 5 }) }}</p>

  <!-- English: "Selected: Ningbo Mall" -->
  <!-- Chinese: "已选择：宁波商场" -->
  <p>{{ t('home.findLocation.selected', { name: 'Ningbo Mall' }) }}</p>
</template>
```

### 4. Reactive Error Messages

```vue
<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { ref } from 'vue'

const { t } = useI18n()
const errorMessage = ref('')

function handleError() {
  errorMessage.value = t('auth.errors.loginFailed')
}
</script>

<template>
  <p v-if="errorMessage" class="text-red-500">
    {{ errorMessage }}
  </p>
</template>
```

### 5. Conditional Loading States

```vue
<template>
  <Button :disabled="loading">
    {{ loading ? t('auth.loggingIn') : t('auth.loginButton') }}
  </Button>
</template>
```

## Language Switcher Component

A `LanguageSwitcher` component is provided at `@/components/LanguageSwitcher.vue`:

```vue
<script setup lang="ts">
import LanguageSwitcher from '@/components/LanguageSwitcher.vue'
</script>

<template>
  <div class="header">
    <LanguageSwitcher />
  </div>
</template>
```

The component:
- Displays the current language flag and name
- Opens a dialog with all available languages
- Persists the user's choice in localStorage
- Automatically applies the selected language

## Adding a New Language

### 1. Create a New Translation File

Create a new JSON file in `locales/`:

```bash
touch mobile/src/i18n/locales/es-ES.json
```

### 2. Add Translations

Copy the structure from `en-US.json` and translate all strings:

```json
{
  "common": {
    "email": "Correo electrónico",
    "username": "Nombre de usuario",
    ...
  },
  ...
}
```

### 3. Update i18n Configuration

Edit `index.ts`:

```typescript
import esEs from './locales/es-ES.json'

const i18n = createI18n({
  // ...
  messages: {
    'en-US': enUs,
    'zh-CN': zhCn,
    'zh-TW': zhTw,
    'ja-JP': jaJp,
    'fr-FR': frFr,
    'es-ES': esEs,  // Add new language
  },
})
```

### 4. Update Language Switcher

Edit `components/LanguageSwitcher.vue`:

```typescript
const languages: Language[] = [
  // ... existing languages
  {
    code: 'es-ES',
    name: 'Spanish',
    nativeName: 'Español',
    flag: '🇪🇸',
  },
]
```

## Migrating Existing Components

To migrate an existing component to use i18n:

1. **Import the composable**:
   ```typescript
   import { useI18n } from 'vue-i18n'
   const { t } = useI18n()
   ```

2. **Replace hardcoded strings** with translation keys:
   ```vue
   <!-- Before -->
   <h1>Indoor Navigation</h1>

   <!-- After -->
   <h1>{{ t('navigation.title') }}</h1>
   ```

3. **Update error messages**:
   ```typescript
   // Before
   errorMessage.value = 'Login failed'

   // After
   errorMessage.value = t('auth.errors.loginFailed')
   ```

4. **Replace placeholders**:
   ```vue
   <!-- Before -->
   <Input placeholder="Email" />

   <!-- After -->
   <Input :placeholder="t('common.email')" />
   ```

## Example: Migrated LoginView

See `views/LoginView.i18n.vue` for a complete example of a migrated component.

To use it:

```bash
# Backup original
mv src/views/LoginView.vue src/views/LoginView.old.vue

# Activate i18n version
mv src/views/LoginView.i18n.vue src/views/LoginView.vue
```

## Best Practices

### 1. Use Nested Keys

```typescript
// Good
t('home.findLocation.description')

// Avoid
t('home_findLocation_description')
```

### 2. Keep Keys Descriptive

```typescript
// Good
t('auth.errors.passwordsNotMatch')

// Avoid
t('err1')
```

### 3. Use Parameters for Dynamic Content

```typescript
// Good
t('home.findLocation.multipleResults', { count: entities.length })

// Avoid
`Found ${entities.length} entities. Please select one below.`
```

### 4. Organize by Feature

Group translations by the view/feature they belong to, not by type:

```json
{
  "home": {
    "title": "...",
    "button": "...",
    "error": "..."
  }
}
```

### 5. Provide Fallbacks

Always ensure the English translation exists, as it's the fallback language.

## Testing

### Manual Testing

1. Open the app
2. Click the language switcher (usually in the header)
3. Select a different language
4. Verify all text updates correctly
5. Refresh the page to ensure persistence

### Automated Testing

```typescript
import { createI18n } from 'vue-i18n'
import enUs from './locales/en-US.json'

describe('i18n', () => {
  it('should translate auth.loginTitle', () => {
    const i18n = createI18n({
      locale: 'en-US',
      messages: { 'en-US': enUs },
    })
    expect(i18n.global.t('auth.loginTitle')).toBe('Login to Proceed')
  })
})
```

## Common Issues

### Missing Translation Key

**Error**: `Translation key "xyz" not found`

**Solution**: Add the key to all locale files, or use a fallback:

```typescript
t('some.key', 'Default text if key missing')
```

### Language Not Persisting

**Solution**: The LanguageSwitcher component automatically saves to localStorage. Ensure you're using it correctly.

### Translations Not Updating

**Solution**:
1. Check the browser console for errors
2. Verify the locale is changing: `console.log(locale.value)`
3. Clear browser cache and localStorage

## Resources

- [Vue I18n Documentation](https://vue-i18n.intlify.dev/)
- [Vue I18n Composition API](https://vue-i18n.intlify.dev/guide/advanced/composition.html)
- [ICU Message Format](https://unicode-org.github.io/icu/userguide/format_parse/messages/)

## Contributing

When adding new features:

1. Add English translations to `en-US.json`
2. Update all other language files with equivalent translations
3. Use translation keys in your components
4. Test with multiple languages
5. Update this README if you add new categories

## License

Same as the parent Navign project (MIT).
