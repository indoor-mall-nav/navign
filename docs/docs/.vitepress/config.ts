import { defineConfig } from 'vitepress'

// https://vitepress.vuejs.org/config/app-configs
export default defineConfig({
  themeConfig: {
    siteTitle: 'Navign',
    nav: [
      {
        text: 'Home',
        link: '/',
      },
      {
        text: 'Components',
        items: [
          { text: 'Overview', link: '/components/' },
          { text: 'Admin', link: '/components/admin/' },
          { text: 'Beacon', link: '/components/beacon' },
          { text: 'Mobile', link: '/components/mobile/' },
          { text: 'Robot', link: '/components/robot/' },
          { text: 'Server', link: '/components/server/' },
          { text: 'Shared', link: '/components/shared' },
        ],
      },
      {
        text: 'Pipelines',
        link: '/pipelines/',
      },
      {
        text: 'Testing',
        link: '/testing/',
      },
      {
        text: 'Development',
        link: '/development/',
      },
    ],
    sidebar: {
      '/components/': [
        {
          text: 'Components Overview',
          link: '/components/',
        },
        {
          text: 'Admin',
          link: '/components/admin/',
          items: [
            {
              text: 'Quick Start',
              link: '/components/admin/quickstart',
            },
            {
              text: 'Deployment',
              link: '/components/admin/deployment',
            },
            {
              text: 'Orchestrator',
              link: '/components/admin/orchestrator',
            },
            {
              text: 'Tower',
              link: '/components/admin/tower',
            },
            {
              text: 'Client',
              link: '/components/admin/client',
            },
            {
              text: 'Protocol',
              link: '/components/admin/protocol',
            },
            {
              text: 'Implementation Guide',
              link: '/components/admin/implementation-guide',
            },
            {
              text: 'Vision',
              link: '/components/admin/vision',
            },
          ],
        },
        {
          text: 'Beacon',
          link: '/components/beacon',
        },
        {
          text: 'Mobile',
          link: '/components/mobile/',
          items: [
            {
              text: 'Admin Panel',
              link: '/components/mobile/admin-panel',
            },
            {
              text: 'gRPC-Web Integration',
              link: '/components/mobile/grpc-web-integration',
            },
          ],
        },
        {
          text: 'Miniapp',
          link: '/components/miniapp',
        },
        {
          text: 'Robot',
          link: '/components/robot/',
          items: [
            {
              text: 'Upper Layer',
              link: '/components/robot/upper/',
              items: [
                {
                  text: 'Audio',
                  link: '/components/robot/upper/audio',
                },
                {
                  text: 'Bluetooth',
                  link: '/components/robot/upper/bluetooth',
                },
                {
                  text: 'Navign',
                  link: '/components/robot/upper/navign',
                },
                {
                  text: 'Scheduler',
                  link: '/components/robot/upper/scheduler',
                },
                {
                  text: 'Serial',
                  link: '/components/robot/upper/serial',
                },
                {
                  text: 'Vision',
                  link: '/components/robot/upper/vision',
                },
              ],
            },
            {
              text: 'Lower Layer',
              link: '/components/robot/lower',
            },
          ],
        },
        {
          text: 'Server',
          link: '/components/server/',
          items: [
            {
              text: 'PostgreSQL Migration',
              link: '/components/server/postgres-migration',
            },
            {
              text: 'Migration Summary',
              link: '/components/server/postgres-migration-summary',
            },
          ],
        },
        {
          text: 'Shared',
          link: '/components/shared',
        },
      ],
      '/pipelines/': [
        {
          text: 'Pipelines Overview',
          link: '/pipelines/',
        },
        {
          text: 'Navigation',
          link: '/pipelines/navigation',
        },
        {
          text: 'Localization',
          link: '/pipelines/localization',
        },
        {
          text: 'Access Control (Unlock)',
          link: '/pipelines/unlock',
        },
        {
          text: 'Robot Control',
          link: '/pipelines/robot-control',
        },
        {
          text: 'OTA Updates',
          link: '/pipelines/ota',
        },
        {
          text: 'Firmware OTA',
          link: '/pipelines/firmware-ota',
        },
      ],
      '/testing/': [
        {
          text: 'Testing Overview',
          link: '/testing/',
        },
        {
          text: 'Firmware Testing',
          link: '/testing/firmware-testing',
        },
      ],
      '/development/': [
        {
          text: 'Development Overview',
          link: '/development/',
        },
        {
          text: 'Critical TODOs',
          link: '/development/critical-todos',
        },
        {
          text: 'Refactoring Plan',
          link: '/development/refactoring-plan',
        },
      ],
    },
  },
})
