import tailwindcss from '@tailwindcss/vite'

export default defineNuxtConfig({
  app: {
    head: {
      htmlAttrs: { lang: 'pl' },
      link: [
        { rel: 'icon', type: 'image/svg+xml', href: '/assets/ferris/ferris.svg' },
        { rel: 'icon', href: '/favicon.ico', sizes: '48x48' },
        { rel: 'apple-touch-icon', href: '/apple-touch-icon.png' },
        { rel: 'preconnect', href: 'https://fonts.googleapis.com' },
        { rel: 'preconnect', href: 'https://fonts.gstatic.com', crossorigin: '' },
        {
          rel: 'stylesheet',
          href: 'https://fonts.googleapis.com/css2?family=Bricolage+Grotesque:opsz,wght@12..96,500..800&display=swap',
        },
      ],
    },
  },
  compatibilityDate: '2025-07-15',
  css: ['~/assets/css/main.css'],
  devtools: { enabled: process.env.NODE_ENV === 'development' && process.env.NUXT_DEVTOOLS !== 'false' },
  modules: ['@nuxt/eslint'],
  runtimeConfig: {
    public: {
      apiMode: process.env.NUXT_PUBLIC_API_MODE
        ?? (process.env.NODE_ENV === 'production' ? 'production' : 'development'),
      gatewayBase: process.env.NUXT_PUBLIC_GATEWAY_BASE ?? '',
      slotBaseTemplate: process.env.NUXT_PUBLIC_SLOT_BASE_TEMPLATE ?? '',
    },
  },
  typescript: {
    strict: true,
    typeCheck: process.env.NUXT_TYPECHECK !== 'false',
  },
  vite: {
    plugins: [tailwindcss()],
  },
})
