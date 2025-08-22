import { defineStore } from "pinia";
import type { Area, Beacon, Connection, Entity, Merchant } from "@/schema";

export const useSessionStore = defineStore("session", {
  state: () => ({
    entity: {} as Entity,
    area: {} as Area,
    nearestMerchants: [] as Merchant[],
    beacons: [] as Beacon[],
    connections: [] as Connection[], // Assuming connections are stored as an array of strings
  }),
  getters: {
    isEntitySet: (state) => Object.keys(state.entity).length > 0,
    isAreaSet: (state) => Object.keys(state.area).length > 0,
    isNearestMerchantSet: (state) =>
      Object.keys(state.nearestMerchants).length > 0,
    isBeaconsSet: (state) => state.beacons.length > 0,
    isConnectionsSet: (state) => state.connections.length > 0,
  },
  actions: {
    setEntity(entity: Entity) {
      this.entity = entity;
    },
    setArea(area: Area) {
      this.area = area;
    },
    setNearestMerchants(merchant: Merchant[]) {
      this.nearestMerchants = merchant;
    },
    setBeacons(beacons: Beacon[]) {
      this.beacons = beacons;
    },
    clearSession() {
      this.entity = {} as Entity;
      this.area = {} as Area;
      this.nearestMerchants = [] as Merchant[];
      this.beacons = [];
    },
    setConnections(connections: Connection[]) {
      this.connections = connections;
    },
  },
});
