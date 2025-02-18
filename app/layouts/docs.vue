<script setup lang="ts">
import type { NavItem } from "@nuxt/content";

const navigation = inject<Ref<NavItem[]>>("navigation");
</script>

<template>
  this is the docs layout page
  <UContainer>
    <UPage>
      <template #left>
        <UAside>
          <UNavigationTree
            :links="mapContentNavigation(navigation)"
            default-open
          />
        </UAside>
      </template>

      <slot />
    </UPage>
  </UContainer>
</template>
