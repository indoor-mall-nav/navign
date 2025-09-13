<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { authenticate, type Status } from "@tauri-apps/plugin-biometric";
import { type BleDevice, startScan, stopScan } from "@mnlphlp/plugin-blec";
import { Button } from "@/components/ui/button";
import {
  checkPermissions,
  getCurrentPosition,
  requestPermissions,
} from "@tauri-apps/plugin-geolocation";
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { fetch } from "@tauri-apps/plugin-http";
import { baseUrl } from "@/lib/shared.ts";
import { Area, Beacon, Entity } from "@/schema";
import { useSessionStore } from "@/states/session.ts";
import { getIcon, Icon, loadIcon } from "@iconify/vue";
import { RouterView } from "vue-router";
import { unlockDevice } from "@/lib/unlocker";

const greetMsg = ref("");
const name = ref("");
const geolocation = ref<[number, number]>([0, 0]);
const devices = ref<
  (BleDevice & {
    distance: number;
  })[]
>([]);
const session = useSessionStore();
const status = ref<Status>();
const activeBeacon = ref<BleDevice>();
const activeBeaconDesc = ref<Beacon>();
const activeArea = ref<Area>();
const getLocationFailed = ref(false);
const polygonConfig = ref({
  points: [] as number[],
  fill: "#dddddd",
  stroke: "#aaaaaa",
  strokeWidth: 2,
  closed: true,
});
const locationImage = ref({
  url: "",
  width: 0,
  height: 0,
  path: null,
});

function getIconImage(icon: string): Promise<{
  url: string;
  width: number;
  height: number;
}> {
  return new Promise((res, _) => {
    const iconPath = getIcon(icon);
    const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${iconPath.width} ${iconPath.height}" fill="currentColor"><path d="${iconPath.body}"/></svg>`;
    const blob = new Blob([svg], { type: "image/svg+xml" });
    const url = URL.createObjectURL(blob);

    // Load into Image
    const img = new window.Image();
    img.onload = () => {
      res({
        url,
        width: img.width,
        height: img.height,
      });
    };
    img.src = url;
  });
}

const getLocationIcon = async () => {
  if (locationImage.value.url) return locationImage.value;
  console.log(await loadIcon("mdi:map-marker-radius"), "here");
  const icon = await getIconImage("mdi:map-marker-radius");
  console.log(icon, "icon");
  locationImage.value = icon;
  return icon;
};

onMounted(() => getLocationIcon());

function rssiToDistance(
  rssi: number,
  txPower: number = -59,
  pathLoss: number = 12.5,
) {
  if (rssi === 0) return -1.0;

  const ratio = (txPower - rssi) / (10.0 * pathLoss);
  return Math.pow(10, ratio);
}

async function getGeolocation() {
  // try {
  console.log("Checking permissions...");
  const permission = await checkPermissions();
  console.log("Current permissions:", permission);
  if (permission.location !== "granted") {
    console.log("Requesting permissions...");
    const request = await requestPermissions(["location", "coarseLocation"]);
    console.log("Permission request result:", request);
    if (request.location !== "granted") {
      greetMsg.value = "Location permission is required to proceed.";
      return;
    }
  }
  // } catch (error) {
  //   greetMsg.value = `Error obtaining geolocation: ${error}`;
  // }
  console.log("Requesting geolocation...");
  try {
    const position = await getCurrentPosition();
    console.log(position);
    geolocation.value = [position.coords.latitude, position.coords.longitude];
    greetMsg.value = `Geolocation obtained: ${geolocation.value}`;
  } catch (_) {
    getLocationFailed.value = true;
    greetMsg.value =
      "Failed to obtain geolocation. Please ensure location services are enabled.";
  }
}

async function startTask() {
  greetMsg.value = "Scanning Started";
  await authenticate("Please authenticate to start scanning")
    .then((res) => {
      console.log("Authentication successful:", res);
    })
    .catch((err) => {
      console.error("Authentication failed:", err);
      greetMsg.value = "Authentication failed. Cannot start scanning.";
    });
  await startScan(
    async (result) => {
      devices.value = result
        .filter((x) => x.name.includes("BEACON"))
        .map((x) => ({
          ...x,
          distance: rssiToDistance(x.rssi),
        }))
        .sort((a, b) => b.rssi - a.rssi);
      if (devices.value.length > 0 && session.entity) {
        activeBeacon.value = devices.value[0];
        const params = new URLSearchParams({
          query: activeBeacon.value.name,
        });
        console.log(
          baseUrl +
            "/api/entities/" +
            session.entity._id.$oid +
            "/beacons/?" +
            params.toString(),
        );
        greetMsg.value =
          baseUrl +
          "/api/entities/" +
          session.entity._id.$oid +
          "/beacons/?" +
          params.toString();
        activeBeaconDesc.value = (
          await fetch(
            baseUrl +
              "/api/entities/" +
              session.entity._id.$oid +
              "/beacons/?" +
              params.toString(),
            {
              method: "GET",
            },
          ).then((resp) => resp.json())
        )[0];
        console.log(activeBeaconDesc?.value?._id?.$oid, "beacon");
        if (activeBeaconDesc?.value?._id?.$oid) {
          console.log(
            baseUrl +
              "/api/entities/" +
              session.entity._id.$oid +
              "/areas/" +
              activeBeaconDesc?.value?.area.$oid,
          );
          const area: Area = await fetch(
            baseUrl +
              "/api/entities/" +
              session.entity._id.$oid +
              "/areas/" +
              activeBeaconDesc?.value?.area.$oid,
          ).then((x) => x.json());
          session.setArea(area);
          activeArea.value = area;
          polygonConfig.value.points = area.polygon
            .map(([a, b]) => [a * 2, b * 2])
            .flat(2) as number[];
        }
        // greetMsg.value = `Found ${devices.value.length} beacons.`;
      } else {
        // greetMsg.value = "No beacons found.";
      }
      await stopScan();
    },
    10000,
    true,
  );
}

const entity = ref<Entity>();
const entityString = ref("");
const entities = ref<Entity[]>([]);
const switchEntities = ref(false);

async function requestEntity() {
  const query = new URLSearchParams({
    country: "China",
    region: "Zhejiang",
    city: "Ningbo",
    entity: entityString.value,
  });
  if (
    geolocation.value &&
    geolocation.value.length !== 2 &&
    geolocation.value[0] !== 0.0 &&
    geolocation.value[1] !== 0.0
  ) {
    query.append("latitude", geolocation.value[0].toString());
    query.append("longitude", geolocation.value[1].toString());
  }
  console.log(baseUrl + "/api/entities/?" + query.toString());
  // Here you would typically make a request to your backend or service to get the entity details.
  // For now, we will just simulate a response.
  await fetch(baseUrl + "/api/entities/?" + query.toString(), {
    method: "GET",
  })
    .then((response) => response.json())
    .then((data: Entity[]) => {
      console.log(data);
      entities.value = data;
      if (data.length == 1) {
        entity.value = data[0]; // Set the first entity as the default
        session.setEntity(entity.value);
        greetMsg.value = `Found ${data.length} entities.`;
      } else if (data.length > 1) {
        greetMsg.value = `Found ${data.length} entities. Please select one.`;
        switchEntities.value = true; // Show the entity selection dialog
      } else {
        greetMsg.value = "No entities found.";
      }
    });
}

watch(entity, (newEntity) => {
  if (newEntity) {
    greetMsg.value = `Selected entity: ${newEntity.name}`;
    session.setEntity(newEntity);
  }
});
const stageSize = ref({
  width: window.innerWidth * 0.9,
  height: window.innerHeight * 0.45,
});
</script>

<template>
  <main class="container">
    <RouterView />
    <p class="text-2xl text-center mt-16">Indoor Mall Nav System</p>
    <div class="mx-6">
      To get started with, please allow us to locate your device to find which
      mall/store/any indoor location you are in.
      <Button @click="getGeolocation" v-if="!entity">Get Geolocation </Button>
      <Input
        v-if="!entity"
        v-model="entityString"
        placeholder="Enter entity name (e.g., mall, store)"
        class="mt-4"
      />
      <Button @click="requestEntity" v-if="!entity"
        >Get Entity <Icon icon="mdi:map-marker-radius"
      /></Button>

      <Dialog>
        <DialogTrigger as-child>
          <Button class="mt-4">Start Scanning</Button>
        </DialogTrigger>
        <DialogContent>
          <DialogTitle>Scanning for Beacons</DialogTitle>
          <p class="text-sm text-muted-foreground">
            Please wait while we scan for nearby beacons...
          </p>
          <Button @click="startTask" class="mt-4">Start Scan</Button>
          <p v-if="getLocationFailed" class="text-red-500 mt-2">
            Failed to obtain geolocation. Please ensure location services are
            enabled.
          </p>
          <p v-if="greetMsg" class="text-green-500 mt-2">{{ greetMsg }}</p>
          <Button @click="getGeolocation" class="mt-4"
            >Retry Geolocation
          </Button>
        </DialogContent>
      </Dialog>
    </div>

    <Card v-if="activeArea" class="mx-4">
      <CardHeader>
        <CardTitle>Active Area</CardTitle>
        <CardDescription>{{ activeArea.name }}</CardDescription>
      </CardHeader>
      <CardContent> </CardContent>
    </Card>
    <!--    <Button @click="startTask">Start Scanning</Button>-->
    <!--    {{ // JSON.stringify(devices.map((x) => x.name).filter(Boolean)) }}-->
    <!--    {{ JSON.stringify(geolocation) }}-->
    <Card v-for="device in [] as BleDevice[]" :key="device.name">
      <CardHeader>
        <CardTitle>{{ device.name }}</CardTitle>
        <CardDescription>{{ device.address }}</CardDescription>
      </CardHeader>
      <CardContent>
        <p>RSSI: {{ device.rssi }}</p>
        <p>UUID: {{ JSON.stringify(device.serviceData) }}</p>
        <p>Manufacturer Data: {{ JSON.stringify(device.manufacturerData) }}</p>
      </CardContent>
      <CardAction>
        <Button @click="unlockDevice(device)">Unlock</Button>
      </CardAction>
    </Card>
    <Card class="mx-2" v-if="switchEntities && entities.length > 0">
      <CardHeader>
        <CardTitle>Several Entities Found</CardTitle>
        <CardDescription
          >Check out which entity (i.e., mall) you are in.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <Card
          v-for="entityItem in entities"
          :key="entityItem._id.$oid"
          class="mb-2"
        >
          <CardHeader>
            <CardTitle>{{ entityItem.name }}</CardTitle>
          </CardHeader>
          <CardContent>
            <CardDescription>{{ entityItem.description }}</CardDescription>
          </CardContent>
          <CardAction class="ml-4">
            <Button @click="entity = entityItem">Select</Button>
          </CardAction>
        </Card>
      </CardContent>
    </Card>
  </main>
</template>
