import { createBrowserRouter, type RouteObject } from 'react-router-dom';

import { AgentsPage } from '../features/agents/AgentsPage';
import { HarnessPage } from '../features/projects/pages/HarnessPage';
import { ProjectSkillsPage } from '../features/projects/pages/ProjectSkillsPage';
import { ProjectOverview } from '../features/projects/pages/ProjectOverview';
import { ProjectsPage } from '../features/projects/ProjectsPage';
import { SettingsPage } from '../features/settings/SettingsPage';
import { SkillsPage } from '../features/skills/SkillsPage';
import { GlobalHarnessPage } from '../features/harness/GlobalHarnessPage';
import { TasksPage } from '../features/tasks/TasksPage';
import { AppShell } from './layout/AppShell';

export const appRoutes: RouteObject[] = [
  {
    path: '/',
    element: <AppShell />,
    children: [
      { index: true, element: <ProjectsPage /> },
      {
        path: 'projects',
        element: <ProjectsPage />,
        children: [
          { index: true, element: <ProjectOverview /> },
          { path: 'harness', element: <HarnessPage /> },
          { path: 'skills', element: <ProjectSkillsPage /> },
        ],
      },
      { path: 'skills', element: <SkillsPage /> },
      { path: 'agents', element: <AgentsPage /> },
      { path: 'harness', element: <GlobalHarnessPage /> },
      { path: 'tasks', element: <TasksPage /> },
      { path: 'settings', element: <SettingsPage /> },
    ],
  },
];

export const router = createBrowserRouter(appRoutes);
