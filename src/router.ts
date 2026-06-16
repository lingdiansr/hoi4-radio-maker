import { createRouter, createWebHistory } from "vue-router";
import WelcomeView from "@/views/WelcomeView.vue";
import ProjectView from "@/views/ProjectView.vue";

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: "/", name: "welcome", component: WelcomeView },
    {
      path: "/project/:id",
      name: "project",
      component: ProjectView,
      props: true,
    },
  ],
});

export default router;
