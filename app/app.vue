<script setup lang="ts">
import type { ParsedContent } from "@nuxt/content";

const { seo } = useAppConfig();

const { data: navigation } = await useAsyncData("navigation", () =>
  fetchContentNavigation()
);
const { data: files } = useLazyFetch<ParsedContent[]>("/api/search.json", {
  default: () => [],
  server: false,
});

useHead({
  meta: [{ name: "viewport", content: "width=device-width, initial-scale=1" }],
  link: [{ rel: "icon", href: "/logo.svg" }],
  htmlAttrs: {
    lang: "en",
  },
});

useSeoMeta({
  titleTemplate: `%s - ${seo?.siteName}`,
  ogSiteName: seo?.siteName,
  ogImage: "https://docs-template.nuxt.dev/social-card.png",
});

provide("navigation", navigation);
</script>

<template>
  <div>
    <NuxtLoadingIndicator />
    <AppHeader />
    <div
      class="border-l-6 border-yellow-500 bg-yellow-100 p-5 shadow-lg text-center rounded-lg"
    >
      <p
        class="text-2xl font-extrabold text-yellow-700 flex items-center justify-center"
      >
        ⚠️ Caution: Site Under Construction ⚠️
      </p>
      <p class="text-lg font-semibold text-yellow-800 mt-2">
        Examples may be incomplete or incorrect.
      </p>
    </div>
    <UMain>
      <NuxtLayout>
        <NuxtPage />
      </NuxtLayout>
    </UMain>
    <AppFooter />

    <ClientOnly>
      <LazyUContentSearch :files="files" :navigation="navigation" />
    </ClientOnly>

    <UNotifications />
  </div>
</template>
