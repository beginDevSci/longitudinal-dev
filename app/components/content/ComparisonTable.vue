<script setup>
defineProps({
  data: {
    type: Array,
    required: true,
  },
});
</script>

<template>
  <table class="min-w-full bg-white shadow-md rounded-lg">
    <thead class="bg-gray-700 text-white">
      <tr>
        <th class="py-2 px-4 text-left">Approach</th>
        <th class="py-2 px-4 text-left">How It Differs from LMMs</th>
      </tr>
    </thead>
    <tbody class="text-gray-700 text-sm">
      <tr
        v-for="(item, index) in data"
        :key="index"
        class="border-b border-gray-400"
      >
        <td class="py-2 px-4 font-semibold">
          {{ item.approach }}
        </td>
        <td class="py-2 px-4">
          {{ item.difference }}
        </td>
      </tr>
    </tbody>
  </table>
</template>
