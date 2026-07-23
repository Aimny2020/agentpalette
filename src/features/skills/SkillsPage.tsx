import React, { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Plus, RefreshCw } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import {
  AppError,
  checkSkillUpdates,
  getSkills,
  getSkillDetail,
  getCategories,
  createCategory,
  renameCategory,
  deleteCategory,
  updateSkillMeta,
  deleteSkill,
  deleteSkillEverywhere,
  importSkill,
  trustSkill,
  updateSkill,
} from '../../shared/api/tauriClient';
import { SkillsSidebar } from './components/SkillsSidebar';
import { SkillCard } from './components/SkillCard';
import { SkillDetailModal } from './components/SkillDetailModal';
import { ImportSkillModal } from './components/ImportSkillModal';
import { ConfirmDeleteModal } from './components/ConfirmDeleteModal';
import { Skill, SkillMember } from '../../shared/api/types';
import { projectCatalog } from './skillCatalog';
import './components/skills.css';

export function SkillsPage() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [search, setSearch] = useState('');
  const [selectedCategoryId, setSelectedCategoryId] = useState<string | null>(null);
  const [activeDetailId, setActiveDetailId] = useState<{ skillId: string; memberId?: string } | null>(null);
  const [isImportOpen, setIsImportOpen] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<Skill | null>(null);

  // Queries
  const { data: skills = [], isLoading: skillsLoading } = useQuery({
    queryKey: ['skills'],
    queryFn: getSkills,
  });

  const { data: detailSkill } = useQuery({
    queryKey: ['skill-detail', activeDetailId?.skillId],
    queryFn: () => getSkillDetail(activeDetailId!.skillId),
    enabled: !!activeDetailId?.skillId,
  });

  const activeDetail = activeDetailId
    ? (() => {
        const skill = detailSkill ?? skills.find((s) => s.id === activeDetailId.skillId);
        if (!skill) return null;
        const member = skill.members.find((m) => m.id === activeDetailId.memberId);
        return { skill, member };
      })()
    : null;

  const { data: categories = [], isLoading: catsLoading } = useQuery({
    queryKey: ['categories'],
    queryFn: getCategories,
  });

  const { data: skillUpdates = [], refetch: refetchUpdates, isFetching: updatesLoading } = useQuery({
    queryKey: ['skill-updates'],
    queryFn: checkSkillUpdates,
    staleTime: 5 * 60 * 1000,
    enabled: false,
  });
  const updateStatus = new Map(skillUpdates.map((update) => [update.skill_id, update.status]));

  // Mutations
  const updateMetaMut = useMutation({
    mutationFn: ({ id, cat, notes }: { id: string; cat: string | null; notes: string | null }) =>
      updateSkillMeta(id, cat, notes),
    onSuccess: (_, { id }) => {
      queryClient.invalidateQueries({ queryKey: ['skills'] });
      queryClient.invalidateQueries({ queryKey: ['skill-detail', id] });
    },
  });

  const deleteSkillMut = useMutation({
    mutationFn: deleteSkill,
    onSuccess: (_, skillId) => {
      queryClient.invalidateQueries({ queryKey: ['skills'] });
      queryClient.invalidateQueries({ queryKey: ['skill-detail', skillId] });
    },
    onError: (error, skillId) => {
      if (error instanceof AppError && error.details?.includes('enabled in projects')) {
        if (confirm(t('skills.occupiedDetail'))) {
          deleteEverywhereMut.mutate(skillId);
        }
      }
    },
  });

  const deleteEverywhereMut = useMutation({
    mutationFn: deleteSkillEverywhere,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['skills'] });
      queryClient.invalidateQueries({ queryKey: ['skill-detail'] });
    },
  });

  const updateSkillMut = useMutation({
    mutationFn: updateSkill,
    onSuccess: (_, skillId) => {
      queryClient.invalidateQueries({ queryKey: ['skills'] });
      queryClient.invalidateQueries({ queryKey: ['skill-detail', skillId] });
      queryClient.invalidateQueries({ queryKey: ['skill-updates'] });
    },
  });

  const trustSkillMut = useMutation({
    mutationFn: (id: string) => trustSkill(id),
    onMutate: (skillId) => {
      queryClient.cancelQueries({ queryKey: ['skills'] });
      queryClient.cancelQueries({ queryKey: ['skill-detail', skillId] });

      const previousSkills = queryClient.getQueryData<Skill[]>(['skills']);
      const previousDetail = queryClient.getQueryData<Skill>(['skill-detail', skillId]);

      queryClient.setQueryData<Skill[]>(['skills'], (current = []) =>
        current.map((skill) =>
          skill.id === skillId
            ? { ...skill, trusted: true }
            : skill
        )
      );

      if (previousDetail) {
        queryClient.setQueryData<Skill>(['skill-detail', skillId], {
          ...previousDetail,
          trusted: true,
        });
      }

      return { previousSkills, previousDetail };
    },
    onSuccess: (_, skillId) => {
      queryClient.invalidateQueries({ queryKey: ['skills'] });
      queryClient.invalidateQueries({ queryKey: ['skill-detail', skillId] });
      queryClient.invalidateQueries({ queryKey: ['projectSkills'] });
    },
    onError: (error, skillId, context) => {
      if (context?.previousSkills) {
        queryClient.setQueryData(['skills'], context.previousSkills);
      }
      if (context?.previousDetail) {
        queryClient.setQueryData(['skill-detail', skillId], context.previousDetail);
      }
      if (error instanceof AppError) {
        alert(t('skills.trustFailed', { details: error.details ?? error.message }));
        return;
      }
      alert(t('skills.trustFailed', { details: error instanceof Error ? error.message : String(error) }));
    },
  });

  const importSkillMut = useMutation({
    mutationFn: ({ source, type }: { source: string; type: 'folder' | 'git' }) => importSkill(source, type),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['skills'] }),
  });

  const createCatMut = useMutation({
    mutationFn: createCategory,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['categories'] }),
  });

  const renameCatMut = useMutation({
    mutationFn: ({ id, name }: { id: string; name: string }) => renameCategory(id, name),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['categories'] });
      queryClient.invalidateQueries({ queryKey: ['skills'] });
    },
  });

  const deleteCatMut = useMutation({
    mutationFn: deleteCategory,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['categories'] });
      queryClient.invalidateQueries({ queryKey: ['skills'] });
    },
  });

  if (skillsLoading || catsLoading) {
    return (
      <div className="page-state">
        <div className="loading-dot" />
        <p>{t('skills.loading')}</p>
      </div>
    );
  }

  // Process counts for sidebar
  const skillsCountMap: Record<string, number> = { all: skills.length };
  let uncategorizedCount = 0;
  skills.forEach((s) => {
    if (!s.category_id) {
      uncategorizedCount++;
    } else {
      skillsCountMap[s.category_id] = (skillsCountMap[s.category_id] || 0) + 1;
    }
  });
  skillsCountMap['uncategorized'] = uncategorizedCount;

  const catalogResults = projectCatalog(skills, search, selectedCategoryId);

  const getCategoryName = (catId?: string) => {
    if (!catId) return t('skills.uncategorized');
    return categories.find((c) => c.id === catId)?.name || t('skills.uncategorized');
  };

  return (
    <div className="page-stack skills-page-container">
      <header className="page-header" style={{ minHeight: 'auto', alignItems: 'center' }}>
        <div style={{ display: 'flex', alignItems: 'baseline', gap: '1rem', flexWrap: 'wrap' }}>
          <h1 style={{ fontSize: '1.75rem', lineHeight: 1 }}>{t('skills.title')}</h1>
          <span style={{ color: 'var(--color-muted)', fontSize: '0.85rem' }}>
            {t('skills.description')}
          </span>
        </div>
      </header>

      <div className="content-grid" style={{ gridTemplateColumns: '16rem 1fr' }}>
        <SkillsSidebar
          categories={categories}
          skillsCountMap={skillsCountMap}
          selectedCategoryId={selectedCategoryId}
          onSelectCategory={setSelectedCategoryId}
          onCreateCategory={(name) => createCatMut.mutate(name)}
          onRenameCategory={(id, name) => renameCatMut.mutate({ id, name })}
          onDeleteCategory={(id) => deleteCatMut.mutate(id)}
        />

        <main className="skills-main-area">
          <div className="skills-toolbar">
            <input
              className="search-input"
              placeholder={t('skills.search')}
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
            <div className="skills-toolbar__actions">
              <button className="button button--secondary" onClick={() => refetchUpdates()} disabled={updatesLoading}>
                <RefreshCw size={16} className={updatesLoading ? 'is-spinning' : ''} />
                {t('skills.checkUpdates')}
              </button>
              <button className="button button--primary" onClick={() => setIsImportOpen(true)}>
                <Plus size={16} /> {t('skills.import')}
              </button>
            </div>
          </div>

          {catalogResults.length === 0 ? (
            <div className="page-state">
              <p>{t('skills.noResults')}</p>
            </div>
          ) : (
            <div className="skills-cards-grid">
              {catalogResults.map((result) => (
                <SkillCard
                  key={result.type === 'member' ? result.member.id : result.skill.id}
                  skill={result.skill}
                  member={result.type === 'member' ? result.member : undefined}
                  categoryName={getCategoryName(result.skill.category_id)}
                  updateStatus={updateStatus.get(result.skill.id) ?? result.skill.update_status}
                  onOpenDetail={() => setActiveDetailId(result.type === 'member' ? { skillId: result.skill.id, memberId: result.member.id } : { skillId: result.skill.id })}
                  onUpdate={result.skill.source.kind === 'git' ? (e) => {
                    e.stopPropagation();
                    if (confirm(t('skills.updatePrompt', { name: result.skill.metadata.name }))) {
                      updateSkillMut.mutate(result.skill.id);
                    }
                  } : undefined}
                  onDelete={result.type === 'skill' ? (e) => {
                    e.stopPropagation();
                    setDeleteTarget(result.skill);
                  } : undefined}
                />
              ))}
            </div>
          )}
        </main>
      </div>

      {activeDetail && (
        <SkillDetailModal
          skill={activeDetail.skill}
          initialMember={activeDetail.member}
          categories={categories}
          updateStatus={updateStatus.get(activeDetail.skill.id) ?? activeDetail.skill.update_status}
          onClose={() => setActiveDetailId(null)}
          onUpdate={(cat, notes) =>
            updateMetaMut.mutate({ id: activeDetail.skill.id, cat, notes })
          }
          onTrust={() => trustSkillMut.mutate(activeDetail.skill.id)}
          onInstallUpdate={() => updateSkillMut.mutate(activeDetail.skill.id)}
        />
      )}

      {isImportOpen && (
        <ImportSkillModal
          onClose={() => setIsImportOpen(false)}
          onImport={(source, type) => importSkillMut.mutate({ source, type })}
        />
      )}

      {deleteTarget && (
        <ConfirmDeleteModal
          skill={deleteTarget}
          onClose={() => setDeleteTarget(null)}
          onConfirm={async (force) => {
            if (force) {
              await deleteEverywhereMut.mutateAsync(deleteTarget.id);
            } else {
              await deleteSkillMut.mutateAsync(deleteTarget.id);
            }
          }}
        />
      )}
    </div>
  );
}
