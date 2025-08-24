<script setup lang="ts">
import { Form, FormItem } from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { Separator } from "@/components/ui/separator";
import { useRouter } from 'vue-router';
import { ref } from 'vue';
import { Icon } from '@iconify/vue';
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";

const router = useRouter();

const email = ref('');
const username = ref('');
const password = ref('');

const register = ref(false);
</script>

<template>
  <div class="mx-4 mt-16">
    <div v-if="register">
      <p class="text-2xl text-center">Register an Account</p>
      <Form class="mt-6 mx-4">
        <Input v-model="email" type="email" placeholder="Email" class="my-4" />
        <Input v-model="username" type="text" placeholder="Username" class="my-4" />
        <Input v-model="password" type="password" placeholder="Password" class="my-4" />
        <Input v-model="password" type="password" placeholder="Confirm Password" class="my-4" />
        <div class="flex mt-2">
          <Checkbox id="terms" />
          <Label for="terms" class="ml-2 text-sm text-gray-800 dark:text-gray-100">
            I agree to the Terms of Service and Privacy Policy.
          </Label>
        </div>
        <div class="flex mt-2">
          <Checkbox id="opt-in" />
          <Label for="opt-in" class="ml-2 text-sm text-gray-800 dark:text-gray-100">
            Receive occasional product updates, user research invitations, company announcements, and digests.
          </Label>
        </div>
        <Button class="w-full my-4" @click="router.push('/')">Register</Button>
      </Form>
    </div>
    <div v-else>
      <p class="text-2xl text-center">Login to Proceed</p>
      <p class="text-sm ml-2 mt-2 text-gray-800 dark:text-gray-100">
        You may use any email and password to login, or proceed as a guest.
        You may also use Google or GitHub to login.
        To ensure a smooth experience, we recommend you to login to create a
        customized experience.
      </p>
      <Form class="mt-6 mx-4">
        <Input v-model="email" type="email" placeholder="Email" class="my-4" />
        <Input v-model="password" type="password" placeholder="Password" class="my-4" />
        <Button class="w-full my-4" @click="router.push('/')">Login</Button>
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
    <Button variant="outline" class="w-full my-4" @click="router.push('/')">
      Continue as Guest
    </Button>
    <Button variant="link" class="w-full" @click="register = !register">
      <Icon icon="tabler:arrow-right" class="w-4 h-4 mr-2" />
      {{ register ? 'Already have an account? Login' : "Don't have an account? Register" }}
    </Button>
  </div>
</template>

<style scoped></style>
