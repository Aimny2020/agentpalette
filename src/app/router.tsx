import { createBrowserRouter, type RouteObject } from 'react-router-dom';

import { DashboardPage } from '../features/dashboard/DashboardPage';
import { McpPage } from '../features/mcp/McpPage';
import { AgentsPage } from '../features/projects/pages/AgentsPage';
import { EnvironmentPage } from '../features/projects/pages/EnvironmentPage';
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
      { index: true, element: <DashboardPage /> },
      {
        path: 'projects',
        element: <ProjectsPage />,
        children: [
          { index: true, element: <ProjectOverview /> },
          { path: 'harness', element: <HarnessPage /> },
          { path: 'skills', element: <ProjectSkillsPage /> },
          { path: 'agents', element: <AgentsPage /> },
          { path: 'environment', element: <EnvironmentPage /> },
        ],
      },
      { path: 'skills', element: <SkillsPage /> },
      { path: 'harness', element: <GlobalHarnessPage /> },
      { path: 'mcp', element: <McpPage /> },
      { path: 'tasks', element: <TasksPage /> },
      { path: 'settings', element: <SettingsPage /> },
    ],
  },
];

export const router = createBrowserRouter(appRoutes);
