<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  type Status,
} from "@tauri-apps/plugin-biometric";
import {
  startScan,
  type BleDevice,
} from "@mnlphlp/plugin-blec";
import { Button } from "@/components/ui/button";
import {Card, CardAction, CardContent, CardDescription, CardHeader, CardTitle} from "@/components/ui/card";

const greetMsg = ref("");
const name = ref("");
const devices = ref<(BleDevice & {
  distance: number
})[]>([]);
const permission = ref(false);
const status = ref<Status>();

const options = {
  // Set true if you want the user to be able to authenticate using phone password
  allowDeviceCredential: false,
  cancelTitle: "Feature won't work if Canceled",

  // iOS only feature
  fallbackTitle: "Sorry, authentication failed",

  // Android only features
  title: "Tauri feature",
  subtitle: "Authenticate to access the locked Tauri function",
  confirmationRequired: true,
};

function rssiToDistance(rssi: number, txPower: number = -59, pathLoss: number = 12.5) {
  if (rssi === 0) return -1.0;

  const ratio = (txPower - rssi) / (10.0 * pathLoss);
  return Math.pow(10, ratio);
}


async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  // await authenticate("Please authenticate to greet", options);
  greetMsg.value = await invoke("greet", { name: name.value });
}

async function startTask() {
  greetMsg.value = "Scanning Started";
  await startScan(
    (result) => {
      devices.value = result.filter(x => x.name.startsWith('BEACON')).map(x => ({
        ...x,
        distance: rssiToDistance(x.rssi),
      })).sort((a, b) => b.rssi - a.rssi);
    },
    10000,
    true,
  );
}
</script>

<template>
  <main class="container">
    <p class="text-2xl ml-4 mt-8">Bluetooth Low Energy</p>
    Biometric status: {{ JSON.stringify(status?.biometryType) }} We
    {{ permission ? "have" : "don't have" }} the permission.
    <Button @click="startTask">Start Scanning</Button>
    {{ JSON.stringify(devices.map(x => x.name).filter(Boolean)) }}
    <Card v-for="device in devices" :key="device.name">
      <CardHeader>
        <CardTitle>{{ device.name }}</CardTitle>
        <CardDescription>{{ device.address }}</CardDescription>
      </CardHeader>
      <CardContent>
        <p>RSSI: {{ device.rssi }}</p>
        <p>Distance: {{ device.distance.toFixed(2) }} m</p>
        <p>UUID: {{ JSON.stringify(device.serviceData) }}</p>
        <p>Manufacturer Data: {{ JSON.stringify(device.manufacturerData) }}</p>
      </CardContent>
      <CardAction>
        <Button>Connect</Button>
      </CardAction>
    </Card>
    {{ greetMsg }}
  </main>
</template>
