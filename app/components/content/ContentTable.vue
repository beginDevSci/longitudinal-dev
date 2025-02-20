<template>
  <div class="mb-6">
    <!-- Table Title (Optional) -->
    <h3
      v-if="title"
      class="text-xl font-semibold text-gray-800 dark:text-white mb-4 text-center border-b-2 border-gray-300 pb-2"
    >
      {{ title }}
    </h3>

    <div class="overflow-x-auto flex justify-center">
      <table
        class="w-auto bg-white shadow-md rounded-lg border border-gray-300"
      >
        <!-- Table Header -->
        <thead class="bg-gray-700 text-white uppercase text-lg">
          <tr>
            <th
              v-for="(column, index) in columns"
              :key="index"
              class="py-3 px-6 text-left border border-gray-400"
            >
              {{ column }}
            </th>
          </tr>
        </thead>

        <!-- Table Body -->
        <tbody class="text-gray-700 text-sm">
          <tr
            v-for="(row, rowIndex) in rows"
            :key="rowIndex"
            class="border-b border-gray-400 hover:bg-gray-200"
          >
            <td
              v-for="(value, colIndex) in row"
              :key="colIndex"
              class="py-3 px-6 border border-gray-300"
              :class="getCellStyle(value)"
            >
              <!-- Render HTML & Markdown -->
              <span v-html="formatValue(value)" />
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup>
import { computed } from "vue";

const props = defineProps({
  title: String, // Optional table title
  columns: Array, // Array of column names
  rows: Array, // Array of table row data
});

// Function to support Markdown formatting in table cells
const formatValue = (value) => {
  if (typeof value !== "string") return value;

  return value
    .replace(/\*\*(.*?)\*\*/g, "<strong>$1</strong>") // Bold text
    .replace(/_(.*?)_/g, "<em>$1</em>"); // Italics
};

// Function to dynamically style specific values
const getCellStyle = (value) => {
  if (typeof value !== "string") return "";

  if (value.includes("✅ Yes")) return "text-green-500 font-semibold";
  if (value.includes("❌ No")) return "text-red-500 font-semibold";
  return "text-gray-600 text-sm";
};
</script>

<style scoped>
table {
  border-collapse: collapse;
  width: auto;
}
th,
td {
  padding: 10px;
  text-align: left;
}
</style>
