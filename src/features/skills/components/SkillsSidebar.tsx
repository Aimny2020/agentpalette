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

  const handleCreate = (e: React.FormEvent) => {
    e.preventDefault();
    if (newCatName.trim()) {
      onCreateCategory(newCatName.trim());
      setNewCatName('');
    }
  };

  return (
    <aside className="skills-sidebar">
      <h3>技能分类</h3>
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
        {categories.map((cat) => (
          <li
            key={cat.id}
            data-active={selectedCategoryId === cat.id}
            onClick={() => onSelectCategory(cat.id)}
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
                <div className="cat-actions" onClick={(e) => e.stopPropagation()}>
                  <Edit2
                    size={12}
                    onClick={() => {
                      setEditingId(cat.id);
                      setEditingName(cat.name);
                    }}
                  />
                  <Trash2 size={12} onClick={() => onDeleteCategory(cat.id)} />
                </div>
              </>
            )}
            <span className="count-badge">{skillsCountMap[cat.id] || 0}</span>
          </li>
        ))}
      </ul>
      <form onSubmit={handleCreate} className="create-category-form">
        <input
          placeholder="新建分类..."
          value={newCatName}
          onChange={(e) => setNewCatName(e.target.value)}
        />
        <button type="submit">
          <Plus size={16} />
        </button>
      </form>
    </aside>
  );
}
