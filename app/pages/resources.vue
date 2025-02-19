<template>
  <div class="wrapper">
    <header class="r-header flex justify-center items-center text-white h-64">
      <div class="r-logo-animation flex items-end">
        <img src="/images/r-logo.png" alt="Learn R" class="w-32 h-32" />
        <span class="text-4xl font-bold ml-2">esources</span>
      </div>
    </header>
    <header class="header bg-white p-4 text-black h-20">
      <nav class="navigation flex justify-center gap-4 h-full items-center">
        <NuxtLink to="#books" class="nav-button"> Books </NuxtLink>
        <NuxtLink to="#interactive" class="nav-button">
          Interactive tutorials
        </NuxtLink>
        <NuxtLink to="#videos" class="nav-button"> Videos </NuxtLink>
        <NuxtLink to="#cheatsheets" class="nav-button"> Cheatsheets </NuxtLink>
      </nav>
    </header>
    <main class="neve-main">
      <section id="books">
        <div class="container mx-auto">
          <h2 class="text-2xl font-bold text-center mb-4">Open Source R Books</h2>
          <div
            class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4"
          >
            <BookLandingCard
              v-for="(book, index) in books"
              :key="index"
              :book="book"
            />
          </div>
        </div>
      </section>

      <section id="videos" class="py-8 bg-gray-200 dark:bg-white">
        <div class="container mx-auto">
          <h2 class="text-2xl font-bold text-center text-black mb-4">
            Open Source R Videos
          </h2>
          <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-4">
            <VideoLandingCard
              v-for="(video, index) in videos"
              :key="index"
              :video="video"
            />
          </div>
        </div>
      </section>

      <section id="interactive" class="py-4">
        <div class="container mx-auto">
          <h2 class="text-2xl font-bold text-center mb-4">
            Open Source R Tutorials
          </h2>
          <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-2">
            <TutorialsLandingCard
              v-for="(tutorial, index) in tutorials"
              :key="index"
              :tutorial="tutorial"
            />
          </div>
        </div>
      </section>

      <section id="cheatsheets" class="py-8 bg-blue-100 dark:bg-white">
        <div class="container mx-auto">
          <h2 class="text-2xl font-bold text-center mb-4 text-black">
            Open Source R Cheatsheets
          </h2>
          <!-- Add cheatsheets content here -->
        </div>
      </section>


    </main>
  </div>
</template>

<script setup>
import { ref } from "vue";

definePageMeta({ documentDriven: { page: false } });

const books = ref([
  {
    title: "R for Data Science",
    author: "Hadley Wickham & Garrett Grolemund",
    description:
      "Data science with R, covering the entire data analysis workflow.",
    image: "/images/r-for-data-science-cover.jpg",
    link: "https://r4ds.hadley.nz/",
  },
  {
    title: "Advanced R",
    author: "Hadley Wickham",
    description:
      "A deep dive into the R language, helping you understand R's data structures and programming paradigms.",
    image: "/images/advanced-r.png",
    link: "https://adv-r.hadley.nz/",
  },
  {
    title: "Efficient R Programming",
    author: "Colin Gillespie & Robin Lovelace",
    description: "Tools and techniques to make your R code more efficient.",
    image: "/images/efficient-r-programming.png",
    link: "https://csgillespie.github.io/efficientR/",
  },
  {
    title: "R Programming for Data Science",
    author: "Roger D. Peng",
    description:
      "A comprehensive introduction to R programming for data science, covering fundamental concepts and techniques.",
    image: "/images/r-programming-for-data-science.png",
    link: "https://bookdown.org/rdpeng/RProgDA/",
  },
  {
    title: "Hands-On Programming with R",
    author: "Garrett Grolemund",
    description:
      "A practical introduction to programming with R, focusing on hands-on experience and real-world examples.",
    image: "/images/hands-on-programming-with-r.jpg",
    link: "https://rstudio-education.github.io/hopr/",
  },
  {
    title: "R Inferno",
    author: "Patrick Burns",
    description:
      "A guide to the pitfalls of R, providing insights into common errors and how to avoid them.",
    image: "/images/r-inferno.jpg",
    link: "https://www.burns-stat.com/pages/Tutor/R_inferno.pdf",
  },
  {
    title: "Software for Data Analysis with R",
    author: "John M. Chambers",
    description:
      "Software development for data analysis, with a focus on R and its applications.",
    image: "/images/software-for-data-analysis-with-r.jpg",
    link: "https://bookdown.org/rdpeng/RProgDA/",
  },
  {
    title: "The Pirate's Guide to R",
    author: "Nathaniel D. Phillips",
    description:
      "An accessible introduction to R, presented in a pirate-themed format to engage readers.",
    image: "/images/yarrr-the-pirates-guide-to-r.jpeg",
    link: "https://bookdown.org/ndphillips/YaRrr/",
  },
  {
    title: "The Art of R Programming",
    author: "Norman Matloff",
    description:
      "A comprehensive guide to R programming, covering everything from basic concepts to advanced techniques.",
    image: "/images/the-art-of-r-programming.jpg",
    link: "https://web.cs.ucdavis.edu/~matloff/matloff/public_html/145/PLN/RMaterials/NSPpart.pdf",
  },
  {
    title: "Deep R Programming",
    author: "Roger D. Peng",
    description:
      "An in-depth exploration of R programming, focusing on advanced topics and techniques.",
    image: "/images/deep-r-programming.png",
    link: "https://deepr.gagolewski.com/",
  },
  {
    title: "R Cookbook, 2nd Edition",
    author: "James (JD) Long",
    description:
      "This book is full of how-to recipes, each of which solves a specific problem.",
    image: "/images/rc2e.jpg",
    link: "https://rc2e.com/",
  },
  {
    title: "ggplot2: Elegant Graphics for Data Analysis (3e)",
    author: "Hadley Wickham",
    description:
      "This book is full of how-to recipes, each of which solves a specific problem.",
    image: "/images/ggplot2.jpg",
    link: "https://ggplot2-book.org/",
  },
]);

const videos = ref([
  {
    title: "R Programming Tutorial by freeCodeCamp.org",
    link: "https://www.youtube.com/embed/_V8eKsto3Ug?si=Xcpi5ieqGAL1EvGv",
    description: "Learn the Basics of Statistical Computing",
  },
  {
    title: "An introduction to programming using a language called R",
    link: "https://www.youtube.com/embed/videoseries?si=zuDZo4eQIMhSYvXw&amp;list=PLhQjrBD2T382yfNp_-xzX244d-O9W6YmD",
    description: "CS50's Introduction to Programming with R",
  },
  {
    title: "R Programming by R Programming 101 ",
    link: "https://www.youtube.com/embed/videoseries?si=63KVaFKRiu2D9v6P&amp;list=PLtL57Fdbwb_Chn-dNR0qBjH3esKS2MXY3",
    description: "R Programming For Beginners",
  },
  {
    title: "R Programming by Simplilearn ",
    link: "https://www.youtube.com/embed/videoseries?list=PLm-R300a1uRfH-cy2-KuOD7gHzSRFnX-h",
    description: "An R Tutorial For Beginners",
  },
  {
    title: "R Analysis by Equitable Equations ",
    link: "https://www.youtube.com/embed/videoseries?si=hWHp7QvCXjWq3HHE&amp;list=PLKBUk9FL4nBbnAFHU3AZfsGHShsMC2AJC",
    description: "Full data analyses with R",
  },
  {
    title: "Statistical Learning with R",
    link: "https://www.youtube.com/embed/videoseries?si=rKw0bKInPkgcUjIH&amp;list=PLoROMvodv4rOzrYsAxzQyHb8n_RWNuS1e",
    description:
      "Introductory-level course in supervised learning, with a focus on regression and classification methods.",
  },
]);

const tutorials = ref([
  {
    title: "Datacamp R Programming Tutorials",
    link: "https://www.datacamp.com/tutorial/category/r-programming",
    description:
      "R tutorials, with helpful tips and use cases to grow your programming skills.",
  },
  {
    title: " {swirl} Learn R, in R.",
    link: "https://swirlstats.com/",
    description:
      "swirl teaches you R programming and data science interactively, at your own pace, and right in the R console!",
  },
  {
    title: "HarvardX: CS50's Introduction to Programming with R",
    link: "https://cs50.harvard.edu/r/2024/",
    description:
      "An introduction to programming using a language called R, a popular language for statistical computing and graphics in data science and other domains.",
  },
  {
    title: "fasteR",
    link: "https://github.com/matloff/fasteR",
    description:
      "fasteR: Fast Lane to Learning R! Becoming productive in R, as fast as possible",
  },
]);
</script>

<style scoped>
.r-logo-animation {
  display: flex;
  align-items: flex-end; /* Align items to the bottom */
}

.r-logo-animation img {
  margin-bottom: -5px; /* Adjust this value as needed */
}

.text-4xl {
  font-size: 2.25rem; /* Tailwind text-4xl equivalent */
}

.font-bold {
  font-weight: 700; /* Tailwind font-bold equivalent */
}

.ml-2 {
  margin-left: 0.5rem; /* Tailwind ml-2 equivalent */
}

.nav-button {
  @apply inline-block px-4 py-2 bg-gray-800 text-white rounded-md transition duration-300 ease-in-out hover:bg-gray-600;
}

.wrapper {
  font-family: "Arial", sans-serif;
}

.header {
  background-color: #ffffff;
}

.neve-main {
  padding-top: 20px;
  padding-bottom: 20px;
}

/* .book-gallery-section, */
/* .video-gallery-section, */
.tutorial-gallery-section {
  padding: 20px;
  margin-bottom: 20px;
}

/* .book-gallery-section {
  background-color: #f9f9f9;
} */

/* .video-gallery-section {
  background-color: #e9ecef;
} */

.tutorial-gallery-section {
  background-color: #d3f9d8;
}

/* .book-gallery, */
/* .video-gallery, */
.tutorial-gallery {
  display: flex;
  flex-wrap: wrap;
  gap: 20px;
  justify-content: space-around;
}

/* .book, */
/* .video, */
.tutorial {
  text-align: center;
  max-width: 200px;
}

/* .book-cover, */
/* .video iframe {
  width: 150px;
  height: 50px;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  border-radius: 5px;
} */

h3 {
  margin: 10px 0 5px;
  font-size: 1.1em;
}

p {
  font-size: 0.9em;
  color: #555;
}
</style>
