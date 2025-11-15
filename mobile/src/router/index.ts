import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      component: import('../views/HomeView.vue'),
    },
    {
      name: 'splash',
      path: '/splashscreen',
      component: () => import('../views/SplashScreen.vue'),
    },
    {
      path: '/about',
      name: 'about',
      component: import('../views/AboutView.vue'),
    },
    {
      path: '/login',
      name: 'login',
      component: () => import('../views/LoginView.vue'),
    },
    {
      path: '/navigation',
      name: 'navigation',
      component: () => import('../views/NavigationView.vue'),
    },
    {
      path: '/entity-details',
      name: 'entity-details',
      component: () => import('../views/EntityDetailsView.vue'),
    },
    // User profile routes
    {
      path: '/profile',
      name: 'profile',
      component: () => import('../views/ProfileView.vue'),
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('../views/SettingsView.vue'),
    },
    {
      path: '/favorites',
      name: 'favorites',
      component: () => import('../views/FavoritesView.vue'),
    },
    {
      path: '/history',
      name: 'history',
      component: () => import('../views/HistoryView.vue'),
    },
    // Admin routes
    {
      path: '/admin',
      name: 'admin',
      component: () => import('../views/admin/AdminDashboard.vue'),
    },
    {
      path: '/admin/beacons',
      name: 'admin-beacons',
      component: () => import('../views/admin/BeaconsView.vue'),
    },
    {
      path: '/admin/beacons/form',
      name: 'admin-beacons-form',
      component: () => import('../views/admin/BeaconFormView.vue'),
    },
    {
      path: '/admin/areas',
      name: 'admin-areas',
      component: () => import('../views/admin/AreasView.vue'),
    },
    {
      path: '/admin/areas/form',
      name: 'admin-areas-form',
      component: () => import('../views/admin/AreaFormView.vue'),
    },
    {
      path: '/admin/merchants',
      name: 'admin-merchants',
      component: () => import('../views/admin/MerchantsView.vue'),
    },
    {
      path: '/admin/merchants/form',
      name: 'admin-merchants-form',
      component: () => import('../views/admin/MerchantFormView.vue'),
    },
    {
      path: '/admin/connections',
      name: 'admin-connections',
      component: () => import('../views/admin/ConnectionsView.vue'),
    },
    {
      path: '/admin/connections/form',
      name: 'admin-connections-form',
      component: () => import('../views/admin/ConnectionFormView.vue'),
    },
  ],
})

export default router
