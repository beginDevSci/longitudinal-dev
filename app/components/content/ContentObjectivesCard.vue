<script setup>
import { computed } from "vue";

const props = defineProps({
  objectives: {
    type: Array,
    default: () => [],
  },
});

console.log("Received Objectives:", props.objectives);
</script>

<template>
  <div
    class="p-4 border-2 border-green-500 bg-gray-700 text-white rounded-lg shadow-lg"
  >
    <h2 class="text-lg font-bold text-green-400">Objectives</h2>
    <ul class="list-disc pl-5">
      <li v-for="(objective, index) in objectives" :key="index">
        <strong>{{ objective.action }}:</strong> {{ objective.description }}
      </li>
    </ul>
  </div>
</template>
