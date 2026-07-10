import React, { useState } from 'react';
import { Plus, Trash2, Edit2, FolderOpen } from 'lucide-react';
import { Category } from '../../../shared/api/types';
import './sidebar.css';

interface Props {
  categories: Category[];
  skillsCountMap: Record<string, number>; // Maps categoryId to count
  selectedCategoryId: string | null; // null = all, 'uncategorized' = uncategorized
  onSelectCategory: (id: string | null) => void;
  onCreateCategory: (name: string) => void;
  onRenameCategory: (id: string, name: string) => void;
  onDeleteCategory: (id: string) => void;
}

export function SkillsSidebar({
  categories,
  skillsCountMap,
  selectedCategoryId,
  onSelectCategory,
  onCreateCategory,
  onRenameCategory,
  onDeleteCategory,
}: Props) {
  const [newCatName, setNewCatName] = useState('');
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editingName, setEditingName] = useState('');
  const [deletingId, setDeletingId] = useState<string | null>(null);
  const [isCreatingCat, setIsCreatingCat] = useState(false);

  return (
    <aside className="skills-sidebar">
      <div className="sidebar-heading-row">
        <h3>技能分类</h3>
        <button
          type="button"
          className="create-cat-btn"
          onClick={() => {
            setIsCreatingCat(true);
            setNewCatName('');
          }}
          title="新建分类"
        >
          <Plus size={14} />
        </button>
      </div>
      <ul className="sidebar-cat-list">
        <li
          data-active={selectedCategoryId === null}
          onClick={() => onSelectCategory(null)}
        >
          <FolderOpen size={16} />
          <span className="cat-name">全部技能</span>
          <span className="count-badge">{skillsCountMap['all'] || 0}</span>
        </li>
        <li
          data-active={selectedCategoryId === 'uncategorized'}
          onClick={() => onSelectCategory('uncategorized')}
        >
          <FolderOpen size={16} />
          <span className="cat-name">未分类</span>
          <span className="count-badge">{skillsCountMap['uncategorized'] || 0}</span>
        </li>
        {categories.map((cat) => {
          const isDeletingConfirm = deletingId === cat.id;
          return (
            <li
              key={cat.id}
              data-active={selectedCategoryId === cat.id}
              data-deleting={isDeletingConfirm}
              onClick={() => {
                if (isDeletingConfirm) return;
                onSelectCategory(cat.id);
              }}
            >
              <FolderOpen size={16} />
              {editingId === cat.id ? (
                <input
                  value={editingName}
                  onChange={(e) => setEditingName(e.target.value)}
                  onBlur={() => {
                    if (editingName.trim()) onRenameCategory(cat.id, editingName.trim());
                    setEditingId(null);
                  }}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') {
                      if (editingName.trim()) onRenameCategory(cat.id, editingName.trim());
                      setEditingId(null);
                    }
                  }}
                  onClick={(e) => e.stopPropagation()}
                  autoFocus
                />
              ) : (
                <>
                  <span className="cat-name">{cat.name}</span>
                  {isDeletingConfirm ? (
                    <div className="cat-confirm-actions" onClick={(e) => e.stopPropagation()}>
                      <button
                        type="button"
                        className="cat-confirm-btn cat-confirm-btn--yes"
                        title="确认删除"
                        onClick={() => {
                          onDeleteCategory(cat.id);
                          setDeletingId(null);
                        }}
                      >
                        确认
                      </button>
                      <button
                        type="button"
                        className="cat-confirm-btn cat-confirm-btn--no"
                        title="取消"
                        onClick={() => setDeletingId(null)}
                      >
                        取消
                      </button>
                    </div>
                  ) : (
                    <div className="cat-actions" onClick={(e) => e.stopPropagation()}>
                      <Edit2
                        size={12}
                        onClick={() => {
                          setEditingId(cat.id);
                          setEditingName(cat.name);
                        }}
                      />
                      <Trash2
                        size={12}
                        onClick={() => {
                          setDeletingId(cat.id);
                        }}
                      />
                    </div>
                  )}
                </>
              )}
              {!isDeletingConfirm && (
                <span className="count-badge">{skillsCountMap[cat.id] || 0}</span>
              )}
            </li>
          );
        })}

        {isCreatingCat && (
          <li className="sidebar-cat-item--creating">
            <FolderOpen size={16} />
            <input
              type="text"
              className="cat-create-input"
              value={newCatName}
              onChange={(e) => setNewCatName(e.target.value)}
              placeholder="新分类名称..."
              autoFocus
              onBlur={() => {
                setTimeout(() => {
                  setIsCreatingCat(false);
                  setNewCatName('');
                }, 150);
              }}
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  const val = newCatName.trim();
                  if (val) {
                    onCreateCategory(val);
                  }
                  setIsCreatingCat(false);
                  setNewCatName('');
                } else if (e.key === 'Escape') {
                  setIsCreatingCat(false);
                  setNewCatName('');
                }
              }}
            />
          </li>
        )}
      </ul>
    </aside>
  );
}
