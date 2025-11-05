import { defineConfig } from 'vitepress'

// https://vitepress.vuejs.org/config/app-configs
export default defineConfig({
    themeConfig: {
        siteTitle: 'Navign',
        nav: [
            {
                text: 'Home',
                link: '/'
            },
            {
                text: 'Components',
                items: [
                    { text: 'Admin', link: '/components/admin/' },
                    { text: 'Beacon', link: '/components/beacon' },
                    { text: 'Mobile', link: '/components/mobile' },
                    { text: 'Miniapp', link: '/components/miniapp' },
                    { text: 'Robot', link: '/components/robot' },
                    { text: 'Server', link: '/components/server' },
                ]
            }
        ],
        sidebar: {
            '/components/': [
                {
                    text: 'Admin',
                    link: '/components/admin/',
                    items: [
                        {
                            text: 'Orchestrator',
                            link: '/components/admin/orchestrator'
                        },
                        {
                            text: 'Tower',
                            link: '/components/admin/tower'
                        },
                        {
                            text: 'Client',
                            link: '/components/admin/client'
                        },
                        {
                            text: 'Vision',
                            link: '/components/admin/vision'
                        }
                    ]
                },
                {
                    text: 'Beacon',
                    link: '/components/beacon'
                },
                {
                    text: 'Mobile',
                    link: '/components/mobile'
                },
                {
                    text: 'Miniapp',
                    link: '/components/miniapp'
                },
                {
                    text: 'Robot',
                    link: '/components/robot',
                    items: [
                        {
                            text: 'Upper Layer',
                            link: '/components/robot/upper',
                            items: [
                                {
                                    text: 'Audio',
                                    link: '/components/robot/upper/audio'
                                },
                                {
                                    text: 'Bluetooth',
                                    link: '/components/robot/upper/bluetooth'
                                },
                                {
                                    text: 'Navign',
                                    link: '/components/robot/upper/navign'
                                },
                                {
                                    text: 'Scheduler',
                                    link: '/components/robot/upper/scheduler'
                                },
                                {
                                    text: 'Serial',
                                    link: '/components/robot/upper/serial'
                                },
                                {
                                    text: 'Vision',
                                    link: '/components/robot/upper/vision'
                                }
                            ]
                        },
                        {
                            text: 'Lower Layer',
                            link: '/components/robot/lower'
                        }
                    ]
                },
                {
                    text: 'Server',
                    link: '/components/server'
                }
            ]
        }
    }
})
