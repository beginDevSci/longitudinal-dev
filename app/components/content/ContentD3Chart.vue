<script setup>
import { onMounted, ref } from "vue";
import * as d3 from "d3";

// Create refs for D3 charts
const fixedChart = ref(null);
const randomChart = ref(null);

onMounted(() => {
  // Chart dimensions
  const width = 300,
    height = 300;
  const margin = { top: 30, right: 30, bottom: 50, left: 50 };

  // Sample longitudinal data
  const subjects = d3.range(1, 11); // 10 subjects
  const timePoints = [1, 2, 3, 4, 5];

  const fixedData = [];
  const randomData = [];

  subjects.forEach((subject) => {
    const intercept = 2 + Math.random() * 2; // Random intercept
    const slope = 0.5 + Math.random() * 0.2; // Slight slope variation

    timePoints.forEach((time) => {
      const commonEffect = 2 + 0.6 * time + Math.random() * 0.3; // Fixed trend with noise
      const subjectEffect = intercept + slope * time + Math.random() * 0.3; // Individualized slopes

      fixedData.push({ subject, time, value: commonEffect });
      randomData.push({ subject, time, value: subjectEffect });
    });
  });

  // Define scales
  const xScale = d3
    .scaleLinear()
    .domain([1, 5])
    .range([margin.left, width - margin.right]);
  const yScale = d3
    .scaleLinear()
    .domain([1, 6])
    .range([height - margin.bottom, margin.top]);
  const colorScale = d3.scaleOrdinal(d3.schemeCategory10); // Color for subjects

  // Initialize SVGs
  const svgFixed = d3
    .select(fixedChart.value)
    .append("svg")
    .attr("width", width)
    .attr("height", height);

  const svgRandom = d3
    .select(randomChart.value)
    .append("svg")
    .attr("width", width)
    .attr("height", height);

  // Draw axes
  [svgFixed, svgRandom].forEach((svg) => {
    svg
      .append("g")
      .attr("transform", `translate(0,${height - margin.bottom})`)
      .call(d3.axisBottom(xScale).ticks(5));

    svg
      .append("g")
      .attr("transform", `translate(${margin.left},0)`)
      .call(d3.axisLeft(yScale).ticks(5));
  });

  // Fixed Effects Model (Single Regression Line)
  const fixedLine = d3
    .line()
    .x((d) => xScale(d.time))
    .y((d) => yScale(d.value))
    .curve(d3.curveMonotoneX);

  svgFixed
    .append("path")
    .datum(fixedData.filter((d) => d.subject === 1)) // Use any subject's data for common trend
    .attr("fill", "none")
    .attr("stroke", "black")
    .attr("stroke-width", 3)
    .attr("d", fixedLine);

  svgFixed
    .selectAll(".fixed-points")
    .data(fixedData)
    .enter()
    .append("circle")
    .attr("cx", (d) => xScale(d.time))
    .attr("cy", (d) => yScale(d.value))
    .attr("r", 4)
    .attr("fill", "black")
    .attr("opacity", 0.7);

  svgFixed
    .append("text")
    .attr("x", width / 2)
    .attr("y", height - 10)
    .attr("text-anchor", "middle")
    .text("Fixed Effect");

  // Random Effects Model (Multiple Lines per Subject)
  subjects.forEach((subject) => {
    const subjectData = randomData.filter((d) => d.subject === subject);

    const randomLine = d3
      .line()
      .x((d) => xScale(d.time))
      .y((d) => yScale(d.value))
      .curve(d3.curveMonotoneX);

    svgRandom
      .append("path")
      .datum(subjectData)
      .attr("fill", "none")
      .attr("stroke", colorScale(subject))
      .attr("stroke-width", 2)
      .attr("opacity", 0.7)
      .attr("d", randomLine);

    svgRandom
      .selectAll(`.random-points-${subject}`)
      .data(subjectData)
      .enter()
      .append("circle")
      .attr("cx", (d) => xScale(d.time))
      .attr("cy", (d) => yScale(d.value))
      .attr("r", 4)
      .attr("fill", colorScale(subject))
      .attr("opacity", 0.7);
  });

  svgRandom
    .append("text")
    .attr("x", width / 2)
    .attr("y", height - 10)
    .attr("text-anchor", "middle")
    .text("Random Effects");
});
</script>

<template>
  <div style="display: flex; justify-content: space-around">
    <div ref="fixedChart" />
    <div ref="randomChart" />
  </div>
</template>

<style scoped>
div {
  text-align: center;
}
</style>
