import React, { useEffect } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Trash2 } from 'lucide-react';
import { getProjects, addProject, selectDirectory, deleteProject } from '../../shared/api/tauriClient';
import { useProjectStore } from '../../shared/store/projectStore';

export function ProjectSidebar() {
  const queryClient = useQueryClient();
  const { activeProjectId, setActiveProjectId } = useProjectStore();

  // Query projects list
  const { data: projects = [] } = useQuery({
    queryKey: ['projects'],
    queryFn: getProjects,
  });

  // Auto-select first project if none is active or active is deleted
  useEffect(() => {
    if (projects.length > 0) {
      const activeExists = projects.some((p) => p.id === activeProjectId);
      if (!activeProjectId || !activeExists) {
        const firstId = projects[0].id;
        if (activeProjectId !== firstId) {
          setActiveProjectId(firstId);
        }
      }
    } else {
      if (activeProjectId !== null) {
        setActiveProjectId(null);
      }
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

  // Mutation to delete project
  const deleteProjectMut = useMutation({
    mutationFn: (id: string) => deleteProject(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['projects'] });
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

  const handleDeleteProject = (e: React.MouseEvent, id: string, name: string) => {
    e.stopPropagation();
    if (confirm(`确定要在当前系统中删除项目 "${name}" 吗？此操作不会物理删除该项目对应的文件夹。`)) {
      deleteProjectMut.mutate(id);
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
              <div style={{ paddingRight: '2rem' }}>
                <strong>{project.name}</strong>
                <small>{project.path}</small>
              </div>
              <button
                type="button"
                className="delete-project-btn"
                onClick={(e) => handleDeleteProject(e, project.id, project.name)}
                title="从系统中删除项目"
              >
                <Trash2 size={14} />
              </button>
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
