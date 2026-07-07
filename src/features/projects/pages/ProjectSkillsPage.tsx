import React, { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { ChevronDown, ChevronRight } from 'lucide-react';
import { getSkills, getProjectSkills, toggleProjectSkill, trustSkill } from '../../../shared/api/tauriClient';
import type { Skill } from '../../../shared/api/types';
import { useProjectStore } from '../../../shared/store/projectStore';
import { Card } from '../../../shared/ui/Card';
import { PageState } from '../../../shared/ui/PageState';
import './harness.css';

export function ProjectSkillsPage() {
  const queryClient = useQueryClient();
  const { activeProjectId } = useProjectStore();
  
  // Track expanded packages
  const [expandedPacks, setExpandedPacks] = useState<Record<string, boolean>>({});

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
        description="请在左侧侧边栏中选择或添加一个项目，以配置该项目启用的技能。"
      />
    );
  }

  if (skillsLoading || enabledLoading) {
    return (
      <div className="page-state">
        <div className="loading-dot" />
        <p>加载项目技能配置...</p>
      </div>
    );
  }

  const togglePackExpand = (packId: string) => {
    setExpandedPacks((prev) => ({ ...prev, [packId]: !prev[packId] }));
  };

  const handleCheckboxChange = (skill: Skill, isChecked: boolean) => {
    toggleSkillMut.mutate({ skillId: skill.id, enabled: isChecked });
  };

  const handleSubSkillChange = (memberId: string, isChecked: boolean) => {
    toggleSkillMut.mutate({ skillId: memberId, enabled: isChecked });
  };

  const setIndeterminate = (isAnyEnabled: boolean, isAllEnabled: boolean) => (el: HTMLInputElement | null) => {
    if (el) {
      el.indeterminate = isAnyEnabled && !isAllEnabled;
    }
  };

  return (
    <div className="page-stack">
      <Card>
        <h3>选择启用技能</h3>
        {skills.length === 0 ? (
          <p className="muted-copy" style={{ marginTop: '1rem' }}>
            全局技能库为空，请先前往 "Skills 管理" 页面导入一些技能。
          </p>
        ) : (
          <div className="harness-skills-list">
            {skills.map((skill) => {
              const isPack = skill.kind === 'pack';
              const isUntrusted = skill.has_executable_content && !skill.trusted;
              
              // Calculate package status
              const enabledMembersCount = skill.members.filter((m) => enabledSkillIds.includes(m.id)).length;
              const isAllEnabled = isPack && skill.members.length > 0 && enabledMembersCount === skill.members.length;
              const isAnyEnabled = isPack && enabledMembersCount > 0;
              const isPackChecked = isPack ? isAllEnabled : enabledSkillIds.includes(skill.id);
              
              const isExpanded = !!expandedPacks[skill.id];

              return (
                <div key={skill.id} className="harness-skill-item-container" style={{ display: 'flex', flexDirection: 'column' }}>
                  <div 
                    className="harness-skill-row" 
                    data-enabled={isPackChecked || isAnyEnabled}
                    style={isUntrusted ? { opacity: 0.6 } : undefined}
                  >
                    {isPack && (
                      <button
                        type="button"
                        onClick={() => togglePackExpand(skill.id)}
                        style={{
                          background: 'none',
                          border: 'none',
                          padding: 0,
                          cursor: 'pointer',
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'center',
                          color: 'var(--color-muted)',
                          marginRight: '2px'
                        }}
                      >
                        {isExpanded ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
                      </button>
                    )}
                    <input
                      type="checkbox"
                      id={`skill-chk-${skill.id}`}
                      checked={isPackChecked}
                      disabled={isUntrusted}
                      ref={isPack ? setIndeterminate(isAnyEnabled, isAllEnabled) : undefined}
                      onChange={(e) => handleCheckboxChange(skill, e.target.checked)}
                    />
                    <label 
                      htmlFor={isUntrusted ? undefined : `skill-chk-${skill.id}`}
                      style={{ cursor: isUntrusted ? 'not-allowed' : 'pointer', userSelect: 'none' }}
                    >
                      <strong>{skill.metadata.name}</strong>
                      {isPack ? (
                        <span className="project-skill-pack-label">
                          技能扩展包 · {enabledMembersCount}/{skill.members.length} 启用
                        </span>
                      ) : (
                        <span className="project-skill-pack-label" style={{ visibility: 'hidden', userSelect: 'none' }}>
                          占位
                        </span>
                      )}
                      {isUntrusted && (
                        <span className="project-skill-pack-label" style={{ color: '#cf222e', marginLeft: '8px' }}>
                          (包含可执行内容，请在 Skills 管理页授权信任后启用)
                        </span>
                      )}
                    </label>
                  </div>
                  
                  {isPack && isExpanded && (
                    <div 
                      className="harness-sub-skills-list"
                      style={{
                        display: 'flex',
                        flexDirection: 'column',
                        gap: '4px',
                        paddingLeft: '28px',
                        borderLeft: '1px solid var(--color-outline, #e1e4e8)',
                        marginLeft: '18px',
                        marginTop: '4px',
                        marginBottom: '8px'
                      }}
                    >
                      {skill.members.map((member) => {
                        const isMemberEnabled = enabledSkillIds.includes(member.id);
                        return (
                          <div 
                            key={member.id} 
                            style={{ 
                              display: 'flex', 
                              alignItems: 'center', 
                              gap: '8px', 
                              opacity: isUntrusted ? 0.6 : 1, 
                              padding: '4px 0' 
                            }}
                          >
                            <input
                              type="checkbox"
                              id={`skill-chk-${member.id}`}
                              checked={isMemberEnabled}
                              disabled={isUntrusted}
                              onChange={(e) => handleSubSkillChange(member.id, e.target.checked)}
                              style={{ width: '16px', height: '16px', cursor: isUntrusted ? 'not-allowed' : 'pointer' }}
                            />
                            <label 
                              htmlFor={isUntrusted ? undefined : `skill-chk-${member.id}`}
                              style={{ 
                                cursor: isUntrusted ? 'not-allowed' : 'pointer', 
                                userSelect: 'none', 
                                fontSize: '0.85rem' 
                              }}
                            >
                              <strong style={{ color: 'var(--color-ink)', fontWeight: 500 }}>{member.metadata.name}</strong>
                            </label>
                          </div>
                        );
                      })}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </Card>
    </div>
  );
}
