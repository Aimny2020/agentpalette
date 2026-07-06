import React, { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Plus } from 'lucide-react';
import {
  getSkills,
  getCategories,
  createCategory,
  renameCategory,
  deleteCategory,
  updateSkillMeta,
  deleteSkill,
  importSkill,
} from '../../shared/api/tauriClient';
import { SkillsSidebar } from './components/SkillsSidebar';
import { SkillCard } from './components/SkillCard';
import { SkillDetailModal } from './components/SkillDetailModal';
import { ImportSkillModal } from './components/ImportSkillModal';
import { Skill } from '../../shared/api/types';
import './components/skills.css';

export function SkillsPage() {
  const queryClient = useQueryClient();
  const [search, setSearch] = useState('');
  const [selectedCategoryId, setSelectedCategoryId] = useState<string | null>(null);
  const [activeDetailSkill, setActiveDetailSkill] = useState<Skill | null>(null);
  const [isImportOpen, setIsImportOpen] = useState(false);

  // Queries
  const { data: skills = [], isLoading: skillsLoading } = useQuery({
    queryKey: ['skills'],
    queryFn: getSkills,
  });

  const { data: categories = [], isLoading: catsLoading } = useQuery({
    queryKey: ['categories'],
    queryFn: getCategories,
  });

  // Mutations
  const updateMetaMut = useMutation({
    mutationFn: ({ id, cat, notes }: { id: string; cat: string | null; notes: string | null }) =>
      updateSkillMeta(id, cat, notes),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['skills'] }),
  });

  const deleteSkillMut = useMutation({
    mutationFn: deleteSkill,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['skills'] }),
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
        <p>加载技能目录...</p>
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

  // Filter skills
  const filteredSkills = skills.filter((s) => {
    // Category filter
    if (selectedCategoryId === 'uncategorized' && s.category_id) return false;
    if (selectedCategoryId !== null && selectedCategoryId !== 'uncategorized' && s.category_id !== selectedCategoryId) return false;
    
    // Search text filter
    if (search.trim()) {
      const query = search.toLowerCase();
      const nameMatch = s.metadata.name.toLowerCase().includes(query);
      const descMatch = s.metadata.description.toLowerCase().includes(query);
      return nameMatch || descMatch;
    }
    return true;
  });

  const getCategoryName = (catId?: string) => {
    if (!catId) return '未分类';
    return categories.find((c) => c.id === catId)?.name || '未分类';
  };

  return (
    <div className="page-stack">
      <header className="page-header">
        <div>
          <p className="eyebrow">CAPABILITY CATALOG</p>
          <h1>Skills 管理</h1>
          <p className="page-description">管理全局 AI 技能，自定义分类并将其启用至平台。</p>
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
              placeholder="搜索技能名称或描述..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
            <button className="button button--primary" onClick={() => setIsImportOpen(true)}>
              <Plus size={16} style={{ marginRight: '8px', verticalAlign: 'middle' }} />
              导入技能
            </button>
          </div>

          {filteredSkills.length === 0 ? (
            <div className="page-state">
              <p>没有找到匹配的技能</p>
            </div>
          ) : (
            <div className="skills-cards-grid">
              {filteredSkills.map((s) => (
                <SkillCard
                  key={s.id}
                  skill={s}
                  categoryName={getCategoryName(s.category_id)}
                  onOpenDetail={() => setActiveDetailSkill(s)}
                  onDelete={(e) => {
                    e.stopPropagation();
                    if (confirm(`确定要删除技能 "${s.metadata.name}" 吗？此操作物理删除本地文件且不可逆。`)) {
                      deleteSkillMut.mutate(s.id);
                    }
                  }}
                />
              ))}
            </div>
          )}
        </main>
      </div>

      {activeDetailSkill && (
        <SkillDetailModal
          skill={activeDetailSkill}
          categories={categories}
          onClose={() => setActiveDetailSkill(null)}
          onUpdate={(cat, notes) =>
            updateMetaMut.mutate({ id: activeDetailSkill.id, cat, notes })
          }
        />
      )}

      {isImportOpen && (
        <ImportSkillModal
          onClose={() => setIsImportOpen(false)}
          onImport={(source, type) => importSkillMut.mutate({ source, type })}
        />
      )}
    </div>
  );
}
