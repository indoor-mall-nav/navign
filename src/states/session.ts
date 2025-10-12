import { defineStore } from "pinia";
import type { Area, Beacon, Connection, Entity, Merchant } from "@/schema";

export const useSessionStore = defineStore("session", {
  state: () => ({
    entity: {} as Entity,
    area: {} as Area,
    nearestMerchants: [] as Merchant[],
    beacons: [] as Beacon[],
    connections: [] as Connection[],
    userToken: "" as string,
    userId: "" as string,
    currentLocation: null as { x: number; y: number } | null,
    isAuthenticated: false as boolean,
  }),
  getters: {
    isEntitySet: (state) => Object.keys(state.entity).length > 0,
    isAreaSet: (state) => Object.keys(state.area).length > 0,
    isNearestMerchantSet: (state) =>
      Object.keys(state.nearestMerchants).length > 0,
    isBeaconsSet: (state) => state.beacons.length > 0,
    isConnectionsSet: (state) => state.connections.length > 0,
    hasValidToken: (state) => state.userToken.length > 0 && state.isAuthenticated,
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
    setConnections(connections: Connection[]) {
      this.connections = connections;
    },
    setUserToken(token: string) {
      this.userToken = token;
      this.isAuthenticated = token.length > 0;
    },
    setUserId(userId: string) {
      this.userId = userId;
    },
    setCurrentLocation(location: { x: number; y: number } | null) {
      this.currentLocation = location;
    },
    clearSession() {
      this.entity = {} as Entity;
      this.area = {} as Area;
      this.nearestMerchants = [] as Merchant[];
      this.beacons = [];
      this.userToken = "";
      this.userId = "";
      this.currentLocation = null;
      this.isAuthenticated = false;
    },
  },
});
