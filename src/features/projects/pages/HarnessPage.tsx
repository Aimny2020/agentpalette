import React from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { getSkills, getProjectSkills, toggleProjectSkill } from '../../../shared/api/tauriClient';
import { useProjectStore } from '../../../shared/store/projectStore';
import { Card } from '../../../shared/ui/Card';
import { PageState } from '../../../shared/ui/PageState';
import './harness.css';

export function HarnessPage() {
  const queryClient = useQueryClient();
  const { activeProjectId } = useProjectStore();

  // Query global skills list
  const { data: skills = [], isLoading: skillsLoading } = useQuery({
    queryKey: ['skills'],
    queryFn: getSkills,
  });

  // Query enabled skills for the active project
  const { data: enabledSkillIds = [], isLoading: enabledLoading } = useQuery({
    queryKey: ['projectSkills', activeProjectId],
    queryFn: () => getProjectSkills(activeProjectId || ''),
    enabled: !!activeProjectId,
  });

  // Toggle skill mutation
  const toggleSkillMut = useMutation({
    mutationFn: ({ skillId, enabled }: { skillId: string; enabled: boolean }) =>
      toggleProjectSkill(activeProjectId || '', skillId, enabled),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['projectSkills', activeProjectId] });
    },
  });

  if (!activeProjectId) {
    return (
      <PageState
        state="empty"
        title="尚未选择任何项目"
        description="请在左侧侧边栏中选择或添加一个项目，以配置其工程规则与技能。"
      />
    );
  }

  if (skillsLoading || enabledLoading) {
    return (
      <div className="page-state">
        <div className="loading-dot" />
        <p>加载项目工程规则及技能项...</p>
      </div>
    );
  }

  const handleCheckboxChange = (skillId: string, isChecked: boolean) => {
    toggleSkillMut.mutate({ skillId, enabled: isChecked });
  };

  return (
    <div className="page-stack">
      <Card>
        <h2>项目工程规则 (Harness)</h2>
        <p className="muted-copy">在此项目中选择需要启用的技能，启动后相应的技能配置将同步到项目文件夹中。</p>
      </Card>

      <Card>
        <h3>启用技能项</h3>
        {skills.length === 0 ? (
          <p className="muted-copy" style={{ marginTop: '1rem' }}>
            全局技能库为空，请先前往 "Skills 管理" 页面导入一些技能。
          </p>
        ) : (
          <div className="harness-skills-list">
            {skills.map((skill) => {
              const isEnabled = enabledSkillIds.includes(skill.id);
              return (
                <div className="harness-skill-row" key={skill.id}>
                  <input
                    type="checkbox"
                    id={`skill-chk-${skill.id}`}
                    checked={isEnabled}
                    onChange={(e) => handleCheckboxChange(skill.id, e.target.checked)}
                  />
                  <label htmlFor={`skill-chk-${skill.id}`} style={{ cursor: 'pointer', userSelect: 'none' }}>
                    <strong>{skill.metadata.name}</strong>
                  </label>
                </div>
              );
            })}
          </div>
        )}
      </Card>
    </div>
  );
}
