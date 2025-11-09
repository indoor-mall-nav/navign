<script setup lang="ts">
/**
 * This is an i18n-enabled version of LoginView.vue
 * It demonstrates how to use vue-i18n in the Navign mobile app
 *
 * To replace the current LoginView.vue with this version:
 * 1. Backup the original: mv src/views/LoginView.vue src/views/LoginView.old.vue
 * 2. Rename this file: mv src/views/LoginView.i18n.vue src/views/LoginView.vue
 */
import { Form } from '@/components/ui/form'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { useRouter } from 'vue-router'
import { ref } from 'vue'
import { Icon } from '@iconify/vue'
import { Label } from '@/components/ui/label'
import { Checkbox } from '@/components/ui/checkbox'
import { login, register, guestLogin } from '@/lib/api/tauri'
import { useSessionStore } from '@/states/session'
import { info } from '@tauri-apps/plugin-log'
import { useI18n } from 'vue-i18n'
import LanguageSwitcher from '@/components/LanguageSwitcher.vue'

const { t } = useI18n()
const router = useRouter()
const session = useSessionStore()

const email = ref('')
const username = ref('')
const password = ref('')
const confirmPassword = ref('')
const isRegister = ref(false)
const loading = ref(false)
const errorMessage = ref('')
const acceptTerms = ref(false)

async function handleLogin() {
  if (!email.value || !password.value) {
    errorMessage.value = t('auth.errors.emailPasswordRequired')
    return
  }

  loading.value = true
  errorMessage.value = ''

  try {
    const result = await login(email.value, password.value)
    if (result.status === 'success') {
      if (result.token) {
        localStorage.setItem('auth_token', result.token)
        session.setUserToken(result.token)
      }
      router.push('/')
    } else {
      errorMessage.value = result.message || t('auth.errors.loginFailed')
    }
  } catch (error) {
    errorMessage.value = `${t('common.error')}: ${error}`
  } finally {
    loading.value = false
  }
}

async function handleRegister() {
  if (!email.value || !username.value || !password.value) {
    errorMessage.value = t('auth.errors.fillAllFields')
    return
  }

  if (password.value !== confirmPassword.value) {
    errorMessage.value = t('auth.errors.passwordsNotMatch')
    return
  }

  if (!acceptTerms.value) {
    errorMessage.value = t('auth.errors.acceptTermsRequired')
    return
  }

  loading.value = true
  errorMessage.value = ''

  try {
    const result = await register(email.value, username.value, password.value)
    if (result.status === 'success') {
      errorMessage.value = t('auth.registrationSuccess')
      isRegister.value = false
      password.value = ''
      confirmPassword.value = ''
    } else {
      errorMessage.value = result.message || t('auth.errors.registrationFailed')
    }
  } catch (error) {
    errorMessage.value = `${t('common.error')}: ${error}`
  } finally {
    loading.value = false
  }
}

async function handleGuestLogin() {
  loading.value = true
  errorMessage.value = ''

  try {
    const result = await guestLogin()
    if (result.status === 'success') {
      session.setUserToken('guest')
      session.setUserId('guest')
      await info('Guest login successful')
      await router.push('/')
    } else {
      errorMessage.value = result.message || t('auth.errors.guestLoginFailed')
    }
  } catch (error) {
    errorMessage.value = `${t('common.error')}: ${error}`
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="mx-4 mt-16">
    <!-- Language Switcher at the top -->
    <div class="flex justify-end mb-4">
      <LanguageSwitcher />
    </div>

    <div v-if="isRegister">
      <p class="text-2xl text-center">{{ t('auth.registerTitle') }}</p>
      <Form class="mt-6 mx-4" @submit.prevent="handleRegister">
        <Input
          v-model="email"
          type="email"
          :placeholder="t('common.email')"
          class="my-4"
          :disabled="loading"
        />
        <Input
          v-model="username"
          type="text"
          :placeholder="t('common.username')"
          class="my-4"
          :disabled="loading"
        />
        <Input
          v-model="password"
          type="password"
          :placeholder="t('common.password')"
          class="my-4"
          :disabled="loading"
        />
        <Input
          v-model="confirmPassword"
          type="password"
          :placeholder="t('common.confirmPassword')"
          class="my-4"
          :disabled="loading"
        />
        <div class="flex mt-2">
          <Checkbox
            id="terms"
            v-model:checked="acceptTerms"
            :disabled="loading"
          />
          <Label
            for="terms"
            class="ml-2 text-sm text-gray-800 dark:text-gray-100"
          >
            {{ t('auth.acceptTerms') }}
          </Label>
        </div>
        <p v-if="errorMessage" class="text-red-500 text-sm mt-2">
          {{ errorMessage }}
        </p>
        <Button class="w-full my-4" type="submit" :disabled="loading">
          {{ loading ? t('auth.registering') : t('auth.registerButton') }}
        </Button>
      </Form>
    </div>
    <div v-else>
      <p class="text-2xl text-center">{{ t('auth.loginTitle') }}</p>
      <p class="text-sm ml-2 mt-2 text-gray-800 dark:text-gray-100">
        {{ t('auth.loginDescription') }}
      </p>
      <Form class="mt-6 mx-4" @submit.prevent="handleLogin">
        <Input
          v-model="email"
          type="email"
          :placeholder="t('common.email')"
          class="my-4"
          :disabled="loading"
        />
        <Input
          v-model="password"
          type="password"
          :placeholder="t('common.password')"
          class="my-4"
          :disabled="loading"
        />
        <p v-if="errorMessage" class="text-red-500 text-sm mt-2">
          {{ errorMessage }}
        </p>
        <Button class="w-full my-4" type="submit" :disabled="loading">
          {{ loading ? t('auth.loggingIn') : t('auth.loginButton') }}
        </Button>
      </Form>
    </div>
    <Separator class="my-4 mx-2" />
    <div class="text-center">
      <Button variant="ghost" size="icon">
        <Icon icon="tabler:brand-google" class="w-8 h-8" />
      </Button>
      <Button variant="ghost" size="icon">
        <Icon icon="tabler:brand-github" class="w-8 h-8" />
      </Button>
      <Button variant="ghost" size="icon">
        <Icon icon="tabler:brand-wechat" class="w-8 h-8" />
      </Button>
    </div>
    <Button
      variant="outline"
      class="w-full my-4"
      @click="handleGuestLogin"
      :disabled="loading"
    >
      {{ t('auth.continueAsGuest') }}
    </Button>
    <Button variant="link" class="w-full" @click="isRegister = !isRegister">
      <Icon icon="tabler:arrow-right" class="w-4 h-4 mr-2" />
      {{ isRegister ? t('auth.alreadyHaveAccount') : t('auth.dontHaveAccount') }}
    </Button>
  </div>
</template>

<style scoped></style>
