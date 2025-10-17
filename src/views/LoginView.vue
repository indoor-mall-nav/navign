<script setup lang="ts">
import { Form } from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { useRouter } from "vue-router";
import { ref } from "vue";
import { Icon } from "@iconify/vue";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";
import { login, register, guestLogin } from "@/lib/api/tauri";
import { useSessionStore } from "@/states/session";

const router = useRouter();
const session = useSessionStore();

const email = ref("");
const username = ref("");
const password = ref("");
const confirmPassword = ref("");
const isRegister = ref(false);
const loading = ref(false);
const errorMessage = ref("");
const acceptTerms = ref(false);

async function handleLogin() {
  if (!email.value || !password.value) {
    errorMessage.value = "Please enter email and password";
    return;
  }

  loading.value = true;
  errorMessage.value = "";

  try {
    const result = await login(email.value, password.value);
    if (result.status === "success") {
      // Store token in session
      if (result.token) {
        localStorage.setItem("auth_token", result.token);
        session.setUserToken(result.token);
      }
      router.push("/");
    } else {
      errorMessage.value = result.message || "Login failed";
    }
  } catch (error) {
    errorMessage.value = `Error: ${error}`;
  } finally {
    loading.value = false;
  }
}

async function handleRegister() {
  if (!email.value || !username.value || !password.value) {
    errorMessage.value = "Please fill in all fields";
    return;
  }

  if (password.value !== confirmPassword.value) {
    errorMessage.value = "Passwords do not match";
    return;
  }

  if (!acceptTerms.value) {
    errorMessage.value = "Please accept the terms of service";
    return;
  }

  loading.value = true;
  errorMessage.value = "";

  try {
    const result = await register(email.value, username.value, password.value);
    if (result.status === "success") {
      errorMessage.value = "Registration successful! Please login.";
      isRegister.value = false;
      // Clear password fields
      password.value = "";
      confirmPassword.value = "";
    } else {
      errorMessage.value = result.message || "Registration failed";
    }
  } catch (error) {
    errorMessage.value = `Error: ${error}`;
  } finally {
    loading.value = false;
  }
}

async function handleGuestLogin() {
  loading.value = true;
  errorMessage.value = "";

  try {
    const result = await guestLogin();
    if (result.status === "success") {
      session.setUserToken("guest");
      session.setUserId("guest");
      console.log("Guest login successful");
      await router.push("/");
    } else {
      errorMessage.value = result.message || "Guest login failed";
    }
  } catch (error) {
    errorMessage.value = `Error: ${error}`;
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <div class="mx-4 mt-16">
    <div v-if="isRegister">
      <p class="text-2xl text-center">Register an Account</p>
      <Form class="mt-6 mx-4" @submit.prevent="handleRegister">
        <Input
          v-model="email"
          type="email"
          placeholder="Email"
          class="my-4"
          :disabled="loading"
        />
        <Input
          v-model="username"
          type="text"
          placeholder="Username"
          class="my-4"
          :disabled="loading"
        />
        <Input
          v-model="password"
          type="password"
          placeholder="Password"
          class="my-4"
          :disabled="loading"
        />
        <Input
          v-model="confirmPassword"
          type="password"
          placeholder="Confirm Password"
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
            I agree to the Terms of Service and Privacy Policy.
          </Label>
        </div>
        <p v-if="errorMessage" class="text-red-500 text-sm mt-2">
          {{ errorMessage }}
        </p>
        <Button class="w-full my-4" type="submit" :disabled="loading">
          {{ loading ? "Registering..." : "Register" }}
        </Button>
      </Form>
    </div>
    <div v-else>
      <p class="text-2xl text-center">Login to Proceed</p>
      <p class="text-sm ml-2 mt-2 text-gray-800 dark:text-gray-100">
        You may use any email and password to login, or proceed as a guest. You
        may also use Google or GitHub to login. To ensure a smooth experience,
        we recommend you to login to create a customized experience.
      </p>
      <Form class="mt-6 mx-4" @submit.prevent="handleLogin">
        <Input
          v-model="email"
          type="email"
          placeholder="Email"
          class="my-4"
          :disabled="loading"
        />
        <Input
          v-model="password"
          type="password"
          placeholder="Password"
          class="my-4"
          :disabled="loading"
        />
        <p v-if="errorMessage" class="text-red-500 text-sm mt-2">
          {{ errorMessage }}
        </p>
        <Button class="w-full my-4" type="submit" :disabled="loading">
          {{ loading ? "Logging in..." : "Login" }}
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
      Continue as Guest
    </Button>
    <Button variant="link" class="w-full" @click="isRegister = !isRegister">
      <Icon icon="tabler:arrow-right" class="w-4 h-4 mr-2" />
      {{
        isRegister
          ? "Already have an account? Login"
          : "Don't have an account? Register"
      }}
    </Button>
  </div>
</template>

<style scoped></style>
