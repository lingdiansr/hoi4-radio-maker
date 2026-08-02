import { defineStore } from "pinia";
import { ref } from "vue";
import { invokeCommand } from "@/api/client";

export interface Project {
  id: string;
  name: string;
  version: string;
  supported_version: string;
  tags: string[];
  author?: string;
  output_dir: string;
}

export interface CreateProjectRequest {
  name: string;
  version: string;
  supported_version: string;
  tags: string[];
  author?: string;
  output_dir: string;
}

export interface UpdateProjectRequest {
  name: string;
  version: string;
  supported_version: string;
  tags: string[];
  author?: string;
  output_dir: string;
}

export const useProjectStore = defineStore("project", () => {
  const projects = ref<Project[]>([]);
  const currentProject = ref<Project | null>(null);

  async function loadProjects() {
    projects.value = await invokeCommand<Project[]>("list_projects");
  }

  async function createProject(req: CreateProjectRequest) {
    const p = await invokeCommand<Project>("create_project", { req });
    projects.value.unshift(p);
    return p;
  }

  async function updateProject(id: string, req: UpdateProjectRequest) {
    const p = await invokeCommand<Project>("update_project", { id, req });
    const idx = projects.value.findIndex((proj) => proj.id === id);
    if (idx !== -1) {
      projects.value[idx] = p;
    }
    if (currentProject.value?.id === id) {
      currentProject.value = p;
    }
    return p;
  }

  async function deleteProject(id: string, deleteFiles = false) {
    await invokeCommand("delete_project", { id, delete_files: deleteFiles });
    projects.value = projects.value.filter((p) => p.id !== id);
    if (currentProject.value?.id === id) {
      currentProject.value = null;
    }
  }

  function setCurrentProject(p: Project | null) {
    currentProject.value = p;
  }

  return {
    projects,
    currentProject,
    loadProjects,
    createProject,
    updateProject,
    deleteProject,
    setCurrentProject,
  };
});
