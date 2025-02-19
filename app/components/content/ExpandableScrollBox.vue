<template>
  <div class="p-0 rounded-md max-w-2xl mx-auto">
    <!-- Scrollable Box -->
    <div
      :style="{ maxHeight: expanded ? 'none' : '600px' }"
      class="overflow-y-auto p-2 rounded-md transition-all duration-300"
    >
      <slot />
    </div>
    <!-- Expand/Collapse Button with Arrow -->
    <button
      class="mt-2 mb-4 flex items-center justify-center mx-auto px-3 py-1.5 rounded-full border border-[#00DC82] text-green-500 text-sm font-medium transition-all duration-300 shadow-[0_0_8px_#00DC82] hover:text-[#007D57] hover:border-[#00B76D] hover:shadow-[0_0_10px_#00B76D] active:shadow-[0_0_12px_#00DC82] focus:outline-none"
      @click="expanded = !expanded"
    >
      <span class="text-white-100 font-medium">
        {{ expanded ? "Show Less" : "Show More" }}
      </span>
      <Icon
        :name="expanded ? 'i-heroicons-chevron-up' : 'i-heroicons-chevron-down'"
        class="w-5 h-5 text-green-500"
      />
    </button>
  </div>
</template>

<script setup>
import { ref } from "vue";

const expanded = ref(false);

const toggleExpand = () => {
  expanded.value = !expanded.value;
  console.log("Expanded state:", expanded.value);
};
</script>
