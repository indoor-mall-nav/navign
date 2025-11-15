<script setup lang="ts">
import { onErrorCaptured, ref } from 'vue'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Icon } from '@iconify/vue'
import { error as logError } from '@tauri-apps/plugin-log'

interface Props {
  fallbackComponent?: any
}

defineProps<Props>()

const hasError = ref(false)
const errorMessage = ref('')
const errorStack = ref('')

onErrorCaptured((err, _instance, info) => {
  hasError.value = true
  errorMessage.value = err.message
  errorStack.value = err.stack || ''

  // Log error
  logError(`Component error: ${err.message}\nStack: ${err.stack}\nInfo: ${info}`)

  // Return false to prevent error propagation
  return false
})

function reset() {
  hasError.value = false
  errorMessage.value = ''
  errorStack.value = ''
}

function reload() {
  window.location.reload()
}
</script>

<template>
  <div v-if="hasError">
    <component v-if="fallbackComponent" :is="fallbackComponent" @reset="reset" />
    <div v-else class="min-h-screen flex items-center justify-center p-6 bg-gray-50 dark:bg-gray-900">
      <Card class="max-w-2xl w-full">
        <CardHeader>
          <div class="flex items-center gap-4 mb-4">
            <div class="w-16 h-16 rounded-full bg-red-100 dark:bg-red-900/20 flex items-center justify-center">
              <Icon icon="mdi:alert-circle" class="w-8 h-8 text-red-600 dark:text-red-400" />
            </div>
            <div>
              <CardTitle class="text-2xl text-red-600 dark:text-red-400">Something Went Wrong</CardTitle>
              <CardDescription>An unexpected error occurred in the application</CardDescription>
            </div>
          </div>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md p-4">
            <p class="font-mono text-sm text-red-900 dark:text-red-200">{{ errorMessage }}</p>
          </div>

          <details class="text-sm">
            <summary class="cursor-pointer text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200">
              <Icon icon="mdi:code-braces" class="inline w-4 h-4 mr-1" />
              View Stack Trace
            </summary>
            <pre class="mt-2 p-3 bg-gray-100 dark:bg-gray-800 rounded text-xs overflow-auto max-h-60">{{ errorStack }}</pre>
          </details>

          <div class="flex gap-3 pt-4">
            <Button @click="reset" class="flex-1">
              <Icon icon="mdi:refresh" class="w-4 h-4 mr-2" />
              Try Again
            </Button>
            <Button variant="outline" @click="reload" class="flex-1">
              <Icon icon="mdi:reload" class="w-4 h-4 mr-2" />
              Reload App
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  </div>
  <slot v-else />
</template>
