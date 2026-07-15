import { Outlet, useLocation } from 'react-router-dom';

import { ProjectSidebar } from './ProjectSidebar';
import { TopNavigation } from './TopNavigation';

export function AppShell() {
  const location = useLocation();
  const showSidebar = !location.pathname.startsWith('/skills') && !location.pathname.startsWith('/harness') && !location.pathname.startsWith('/agents') && !location.pathname.startsWith('/settings');

  return (
    <div className="app-shell">
      <TopNavigation />
      <div className={`shell-body ${showSidebar ? '' : 'shell-body--no-sidebar'}`}>
        {showSidebar && <ProjectSidebar />}
        <main className="workspace-main">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
