import React, { useEffect } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { getProjects, addProject, selectDirectory } from '../../shared/api/tauriClient';
import { useProjectStore } from '../../shared/store/projectStore';

export function ProjectSidebar() {
  const queryClient = useQueryClient();
  const { activeProjectId, setActiveProjectId } = useProjectStore();

  // Query projects list
  const { data: projects = [] } = useQuery({
    queryKey: ['projects'],
    queryFn: getProjects,
  });

  // Auto-select first project if none is active
  useEffect(() => {
    if (!activeProjectId && projects.length > 0) {
      setActiveProjectId(projects[0].id);
    }
  }, [projects, activeProjectId, setActiveProjectId]);

  // Mutation to add project
  const addProjectMut = useMutation({
    mutationFn: (path: string) => addProject(path),
    onSuccess: (newProj) => {
      queryClient.invalidateQueries({ queryKey: ['projects'] });
      // Set the newly added project as active
      setActiveProjectId(newProj.id);
    },
  });

  const handleAddProject = async () => {
    try {
      const selectedPath = await selectDirectory();
      if (selectedPath) {
        addProjectMut.mutate(selectedPath);
      }
    } catch (err) {
      console.error('Failed to select or add project', err);
    }
  };

  return (
    <aside className="project-sidebar" aria-label="项目列表">
      <div className="sidebar-heading">
        <h2>我的项目</h2>
        <button type="button" onClick={handleAddProject} disabled={addProjectMut.isPending}>
          {addProjectMut.isPending ? '添加中...' : '＋ 添加'}
        </button>
      </div>
      <ul className="project-list">
        {projects.map((project) => {
          const isActive = activeProjectId === project.id;
          return (
            <li
              className="project-item"
              data-active={isActive}
              key={project.id}
              onClick={() => setActiveProjectId(project.id)}
              style={{ cursor: 'pointer' }}
            >
              <strong>{project.name}</strong>
              <small>{project.path}</small>
            </li>
          );
        })}
        {projects.length === 0 && (
          <li className="project-item-empty" style={{ padding: '1rem', textAlign: 'center', opacity: 0.5 }}>
            暂无项目，请点击上方按钮添加。
          </li>
        )}
      </ul>
    </aside>
  );
}
