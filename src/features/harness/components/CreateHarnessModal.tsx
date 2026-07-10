import React, { useState } from 'react';
import { X, ArrowRight, ArrowLeft } from 'lucide-react';
import { CreateHarnessTemplateInput } from '../../../shared/api/types';

interface CreateHarnessModalProps {
  onClose: () => void;
  onCreate: (input: CreateHarnessTemplateInput) => void;
}

type WorkType = 'code' | 'documentation' | 'presentation' | 'review' | 'custom';

const WORK_TYPES: { type: WorkType; name: string; desc: string; defaultFiles: string[] }[] = [
  {
    type: 'code',
    name: 'Code Work',
    desc: '代码编写、缺陷修复、重构与单元测试。',
    defaultFiles: ['docs/architecture.md', 'docs/feature_list.json', 'docs/verification.md', 'docs/risk-rules.md'],
  },
  {
    type: 'documentation',
    name: 'Documentation Work',
    desc: '需求规约、设计文档、知识库与长文写作。',
    defaultFiles: ['docs/feature_list.json', 'docs/task-status.md', 'docs/verification.md'],
  },
  {
    type: 'presentation',
    name: 'Presentation Work',
    desc: '汇报幻灯片内容、故事线与宣讲文档。',
    defaultFiles: ['docs/feature_list.json', 'docs/task-status.md', 'docs/agent-profile.md'],
  },
  {
    type: 'review',
    name: 'Review Work',
    desc: '代码审查、实施方案评审与技术文档校对。',
    defaultFiles: ['docs/architecture.md', 'docs/verification.md', 'docs/risk-rules.md'],
  },
  {
    type: 'custom',
    name: 'Custom Work',
    desc: '极简起点。默认不包含任何标准可选文件，后续可自由添加。',
    defaultFiles: [],
  },
];

const OPTIONAL_FILES = [
  { path: 'docs/architecture.md', label: 'docs/architecture.md (架构边界与设计规约)' },
  { path: 'docs/feature_list.json', label: 'docs/feature_list.json (机器可读的特性列表)' },
  { path: 'docs/task-status.md', label: 'docs/task-status.md (当前任务状态及决策记录)' },
  { path: 'docs/verification.md', label: 'docs/verification.md (完工验收标准与测试命令)' },
  { path: 'docs/risk-rules.md', label: 'docs/risk-rules.md (高风险操作与安全边界限制)' },
  { path: 'docs/agent-profile.md', label: 'docs/agent-profile.md (Agent 行为风格与工具指引)' },
];

export function CreateHarnessModal({ onClose, onCreate }: CreateHarnessModalProps) {
  const [step, setStep] = useState(1);
  const [workType, setWorkType] = useState<WorkType>('code');
  const [id, setId] = useState('');
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [selectedFiles, setSelectedFiles] = useState<string[]>(WORK_TYPES[0].defaultFiles);

  const handleWorkTypeSelect = (type: WorkType) => {
    setWorkType(type);
    const defaults = WORK_TYPES.find((w) => w.type === type)?.defaultFiles || [];
    setSelectedFiles(defaults);
  };

  const handleToggleFile = (path: string) => {
    setSelectedFiles((prev) =>
      prev.includes(path) ? prev.filter((p) => p !== path) : [...prev, path]
    );
  };

  const handleNext = () => {
    if (step === 2) {
      // Validate metadata
      if (!id.trim() || !name.trim()) {
        alert('请输入模板 ID 和名称！');
        return;
      }
      if (!/^[a-z0-9-_]+$/.test(id)) {
        alert('模板 ID 只能包含小写字母、数字、连字符(-)和下划线(_)！');
        return;
      }
    }
    setStep((prev) => prev + 1);
  };

  const handleBack = () => {
    setStep((prev) => prev - 1);
  };

  const handleSubmit = () => {
    onCreate({
      id: id.trim(),
      name: name.trim(),
      description: description.trim(),
      workType,
      optionalFiles: selectedFiles,
    });
  };

  return (
    <div className="modal-overlay" onClick={onClose} style={{ zIndex: 1000 }}>
      <div className="modal-body" onClick={(e) => e.stopPropagation()} style={{ maxWidth: '32rem', height: 'auto' }}>
        <div className="modal-header">
          <h3>新建 Harness 模板</h3>
          <button type="button" className="close-btn" onClick={onClose}>
            <X size={20} />
          </button>
        </div>

        <div className="harness-modal-content" style={{ marginTop: 'var(--space-2)' }}>
          {/* Progress Indicator */}
          <div className="harness-wizard-steps">
            <div className="harness-wizard-step" data-active={step === 1} data-completed={step > 1}>1</div>
            <div className="harness-wizard-step" data-active={step === 2} data-completed={step > 2}>2</div>
            <div className="harness-wizard-step" data-active={step === 3} data-completed={step > 3}>3</div>
            <div className="harness-wizard-step" data-active={step === 4}>4</div>
          </div>

          {/* Step 1: Work Type */}
          {step === 1 && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--space-2)' }}>
              <p style={{ fontSize: '0.9rem', color: 'var(--color-muted)', margin: 0 }}>
                第一步：请选择该 Harness 模板面向的 AI 工作类型：
              </p>
              <div className="harness-type-grid">
                {WORK_TYPES.map((wt) => (
                  <div
                    key={wt.type}
                    className="harness-type-card"
                    data-selected={workType === wt.type}
                    onClick={() => handleWorkTypeSelect(wt.type)}
                  >
                    <strong>{wt.name}</strong>
                    <span>{wt.desc}</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Step 2: Metadata */}
          {step === 2 && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--space-2)' }}>
              <p style={{ fontSize: '0.9rem', color: 'var(--color-muted)', margin: 0 }}>
                第二步：输入模板的基本识别信息（ID 决定了磁盘存储文件夹名称）：
              </p>
              <div className="harness-form-group">
                <label htmlFor="harness-id">模板唯一 ID (英文线框标识)</label>
                <input
                  id="harness-id"
                  placeholder="例如: standard-web-coding"
                  value={id}
                  onChange={(e) => setId(e.target.value)}
                />
              </div>
              <div className="harness-form-group">
                <label htmlFor="harness-name">显示名称 (中文可读)</label>
                <input
                  id="harness-name"
                  placeholder="例如: Web 前端标准开发规范"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                />
              </div>
              <div className="harness-form-group">
                <label htmlFor="harness-desc">描述信息</label>
                <textarea
                  id="harness-desc"
                  placeholder="该模板的用途与主要约束规则介绍..."
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
                  rows={3}
                />
              </div>
            </div>
          )}

          {/* Step 3: Optional Files Checklist */}
          {step === 3 && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--space-2)' }}>
              <p style={{ fontSize: '0.9rem', color: 'var(--color-muted)', margin: 0 }}>
                第三步：选择需要生成的标准可选文档。我们已基于工作类型进行了初始推荐勾选：
              </p>
              <div className="harness-checklist">
                {OPTIONAL_FILES.map((file) => {
                  const isChecked = selectedFiles.includes(file.path);
                  return (
                    <div
                      key={file.path}
                      className="harness-checklist-item"
                      onClick={() => handleToggleFile(file.path)}
                    >
                      <input
                        type="checkbox"
                        checked={isChecked}
                        onChange={() => {}} // toggled by parent div click
                      />
                      <span>{file.label}</span>
                    </div>
                  );
                })}
              </div>
            </div>
          )}

          {/* Step 4: Preview File Tree */}
          {step === 4 && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--space-2)' }}>
              <p style={{ fontSize: '0.9rem', color: 'var(--color-muted)', margin: 0 }}>
                第四步：检查即将为您生成的模板目录结构预览：
              </p>
              <div className="harness-tree-preview">
                <div>📁 ~/.agent-forge/harnesses/{id || 'template-id'}/</div>
                <div> &nbsp; 📄 AGENTS.md <span style={{ color: 'var(--color-muted)', fontSize: '0.75rem' }}>(必填 Agent 规则总纲)</span></div>
                <div> &nbsp; 📁 docs/</div>
                <div> &nbsp; &nbsp; 📄 harness.toml <span style={{ color: 'var(--color-muted)', fontSize: '0.75rem' }}>(系统管理元数据配置)</span></div>
                {selectedFiles.map((file) => {
                  const nameOnly = file.replace('docs/', '');
                  return (
                    <div key={file}>
                       &nbsp; &nbsp; 📄 {nameOnly}
                    </div>
                  );
                })}
              </div>
            </div>
          )}

          {/* Wizard Footer Navigation */}
          <div style={{ display: 'flex', justifyContent: 'flex-end', gap: 'var(--space-2)', marginTop: 'var(--space-2)', paddingTop: 'var(--space-2)', borderTop: '1px solid var(--color-outline)' }}>
            {step > 1 && (
              <button type="button" className="button button--secondary" onClick={handleBack}>
                <ArrowLeft size={16} style={{ marginRight: '0.35rem' }} /> 上一步
              </button>
            )}
            {step < 4 ? (
              <button type="button" className="button button--primary" onClick={handleNext}>
                下一步 <ArrowRight size={16} style={{ marginLeft: '0.35rem' }} />
              </button>
            ) : (
              <button type="button" className="button button--primary" onClick={handleSubmit}>
                确认创建
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
