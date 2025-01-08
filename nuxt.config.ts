export default defineNuxtConfig({
  target: 'static',
  extends: ['@nuxt/ui-pro'],
  components: true,
  modules: [
    '@nuxt/content',
    '@nuxt/eslint',
    '@nuxt/fonts',
    '@nuxt/image',
    '@nuxt/ui',
    '@nuxthq/studio',
    'nuxt-og-image',
    '@nuxtjs/tailwindcss',
    '@nuxt/icon' // Add the @nuxt/icon module
  ],

  hooks: {
    // Define `@nuxt/ui` components as global to use them in `.md`
    'components:extend': (components) => {
      const globals = components.filter(c => ['UButton', 'UIcon', 'ArticleList', 'SimpleCarousel'].includes(c.pascalName))
      globals.forEach(c => c.global = true)
    }
  },

  ui: {
    icons: ['heroicons', 'simple-icons']
  },

  icon: {
    clientBundle: {
      // List of frequently used icons to include in the client bundle
      icons: [
        'vscode-icons:file-type-r',
        'vscode-icons:file-type-python',
        'vscode-icons:file-type-javascript',
        'heroicons:moon-20-solid',
        'simple-icons:github'
      ],
      // Enable scanning of all components in the project and include icons
      scan: true,

      // Include all custom collections in the client bundle
      includeCustomCollections: true,

      // Guard to ensure the bundle does not exceed 256KB
      sizeLimitKb: 256
    }
  },

  colorMode: {
    preference: 'dark', // Set dark mode as the default
    fallback: 'dark', // Fallback to dark if no system preference is found
    classSuffix: '', // This means the class applied will be `dark` without a suffix
    disableTransition: true
  },

  nitro: {
    prerender: {
      routes: [
        '/', // Home page
        '/abcd-study/'
      ],
      crawlLinks: true, // Enable crawling to find more links automatically
      ignore: []
    }
  },

  app: {
    baseURL: process.env.NUXT_APP_BASE_URL || '/', // Set this to your GitHub repo slug
    trailingSlash: true // Ensure URLs maintain consistency with or without a trailing slash
  },

  devtools: {
    enabled: true
  },

  typescript: {
    strict: false
  },

  future: {
    compatibilityVersion: 4
  },

  eslint: {
    config: {
      stylistic: {
        commaDangle: 'never',
        braceStyle: '1tbs'
      }
    }
  },

  content: {
    documentDriven: true, // Ensuring document-driven mode is correctly enabled
    highlight: {
      langs: ['json', 'js', 'ts', 'html', 'css', 'vue', 'shell', 'md', 'yaml', 'r', 'mdc', 'python']
    }
  },

  css: [
    '@fortawesome/fontawesome-free/css/all.min.css',
    '@/assets/css/main.css'
  ],

  build: {
    postcss: {
      plugins: {
        tailwindcss: {},
        autoprefixer: {}
      }
    }
  },

  compatibilityDate: '2024-07-11',

  head: {
    script: [
      {
        src: 'https://cdnjs.cloudflare.com/ajax/libs/mathjax/2.7.7/MathJax.js',
        type: 'text/javascript',
        async: true
      }
    ]
  }
})
