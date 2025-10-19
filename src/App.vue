<script setup lang="ts">
import { RouterView } from "vue-router";
import {
  startScan,
  stopScan,
  connect,
  disconnect,
  read,
  send,
  type BleDevice,
  subscribe
} from "@mnlphlp/plugin-blec";
import { Button } from "@/components/ui/button";

function scan(timeout: number = 5000): Promise<BleDevice[]> {
  return new Promise((resolve, reject) => {
    startScan(
      (devices) => {
        stopScan()
          .then(() => {
            resolve(devices);
          })
          .catch(reject);
      },
      timeout,
      true,
    ).catch(reject);
  });
}

// Example usage of the BLEC plugin functions
async function useBLEC() {
  try {
    let id = ''
    while (!id) {
      const result = await scan(10000);
      console.log(
        "Scan result:",
        result, result.filter(x => x.name.includes('BEACON'))
      );

      id = result.find(x => x.name.includes('BEACON'))?.address ?? "f3c0db4c-bf08-6332-c5dc-995b759b556c";
    }
    await connect(id, null, true);
    console.log("Connected to BLEC device");

    const dataToSend = [0xFF, 0x01];
    await send(
      "99d92823-9e38-72ff-6cf1-d2d593316af8",
      dataToSend,
      "withoutResponse",
      "134b1d88-cd91-8134-3e94-5c4052743845",
    );
    console.log("Data sent:", dataToSend);

    const receivedData = await read(
      "99d92823-9e38-72ff-6cf1-d2d593316af8",
      "134b1d88-cd91-8134-3e94-5c4052743845",
    );
    console.log("Data received:", receivedData);

    await disconnect();
    console.log("Disconnected from BLEC device");
  } catch (error) {
    console.error("BLEC operation failed:", error);
  }
}
</script>

<template>
  <main class="container">
    <Button @click="useBLEC" class="pt-10">Use BLEC Plugin</Button>
    <RouterView />
  </main>
</template>
