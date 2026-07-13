import React, { useMemo, useState, useEffect } from 'react';
import { ArrowLeft, ArrowRight, X } from 'lucide-react';
import { CreateHarnessTemplateInput, HarnessPreset, CodeWorkModule, HarnessPresetFile } from '../../../shared/api/types';

interface CreateHarnessModalProps {
  onClose: () => void;
  onCreate: (input: CreateHarnessTemplateInput) => void;
  presets: HarnessPreset[];
  isPresetsLoading?: boolean;
  codeModules?: CodeWorkModule[];
  isCodeModulesLoading?: boolean;
  codeSharedFiles?: HarnessPresetFile[];
  isCodeSharedFilesLoading?: boolean;
}

type WorkType = 'code' | 'document' | 'presentation' | 'custom';

const WORK_TYPES: { type: WorkType; name: string; description: string }[] = [
  { type: 'code', name: 'Code Work', description: '编码、测试、代码审查与技术设计。' },
  { type: 'document', name: 'Document Work', description: '专业报告、论文与证据型长文写作。' },
  { type: 'presentation', name: 'Presentation Work', description: '汇报演示、叙事结构与讲稿准备。' },
  { type: 'custom', name: 'Custom Work', description: '从最小结构开始，自由配置文件。' },
];

export function CreateHarnessModal({
  onClose,
  onCreate,
  presets,
  isPresetsLoading = false,
  codeModules = [],
  isCodeModulesLoading = false,
  codeSharedFiles = [],
  isCodeSharedFilesLoading = false,
}: CreateHarnessModalProps) {
  const [step, setStep] = useState(1);
  const [workType, setWorkType] = useState<WorkType>('code');
  const [language, setLanguage] = useState<'zh-CN' | 'en'>('zh-CN');
  const [presetId, setPresetId] = useState<string | undefined>();
  const [selectedModules, setSelectedModules] = useState<string[]>([]);
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [selectedFiles, setSelectedFiles] = useState<string[]>([]);

  const hasPresetStep = workType !== 'custom';

  const currentStepLabels = useMemo(() => {
    if (hasPresetStep) {
      return ['工作类型', '用途预设', '基本信息', '文件配置', '结构预览'];
    }
    return ['工作类型', '基本信息', '文件配置', '结构预览'];
  }, [hasPresetStep]);

  const isRegistryLoading = useMemo(() => {
    switch (workType) {
      case 'code':
        return isCodeModulesLoading || isCodeSharedFilesLoading;
      case 'custom':
        return isPresetsLoading || isCodeModulesLoading || isCodeSharedFilesLoading;
      case 'document':
      case 'presentation':
        return isPresetsLoading;
      default:
        return false;
    }
  }, [workType, isPresetsLoading, isCodeModulesLoading, isCodeSharedFilesLoading]);
  const selectedPreset = presets.find((preset) => preset.id === presetId);
  const availablePresets = presets.filter((preset) => preset.workType === workType);

  // Preselect docs/ paths in codeSharedFiles when they load or when workType changes to code
  useEffect(() => {
    if (workType === 'code' && codeSharedFiles.length > 0 && selectedModules.length === 0 && selectedFiles.length === 0) {
      const preselected = codeSharedFiles
        .filter((f) => f.path.startsWith('docs/'))
        .map((f) => f.path);
      setSelectedFiles(preselected);
    }
  }, [workType, codeSharedFiles, selectedModules]);

  const fileOptions = useMemo(() => {
    if (workType === 'code') {
      const filesMap = new Map<string, HarnessPresetFile>();
      codeSharedFiles.forEach((f) => filesMap.set(f.path, f));
      selectedModules.forEach((modId) => {
        const mod = codeModules.find((m) => m.id === modId);
        mod?.files.forEach((f) => filesMap.set(f.path, f));
      });
      return Array.from(filesMap.values());
    }

    if (workType === 'custom') {
      const filesMap = new Map<string, HarnessPresetFile>();
      presets.forEach((preset) => {
        preset.files.forEach((f) => filesMap.set(f.path, f));
      });
      codeSharedFiles.forEach((f) => filesMap.set(f.path, f));
      codeModules.forEach((mod) => {
        mod.files.forEach((f) => filesMap.set(f.path, f));
      });
      return Array.from(filesMap.values());
    }

    // document or presentation
    return selectedPreset?.files ?? [];
  }, [workType, selectedPreset, presets, codeModules, codeSharedFiles, selectedModules]);

  const chooseWorkType = (type: WorkType) => {
    setWorkType(type);
    setPresetId(undefined);
    setSelectedModules([]);
    if (type === 'code') {
      const preselected = codeSharedFiles
        .filter((f) => f.path.startsWith('docs/'))
        .map((f) => f.path);
      setSelectedFiles(preselected);
    } else {
      setSelectedFiles([]);
    }
  };

  const choosePreset = (preset: HarnessPreset) => {
    setPresetId(preset.id);
    setSelectedFiles(preset.files.map((file) => file.path));
  };

  const handleToggleModule = (moduleId: string) => {
    const isAdding = !selectedModules.includes(moduleId);
    const nextModules = isAdding
      ? [...selectedModules, moduleId]
      : selectedModules.filter((id) => id !== moduleId);

    const getOfferedPaths = (mods: string[]) => {
      const paths = new Set<string>();
      codeSharedFiles.forEach((f) => paths.add(f.path));
      mods.forEach((modId) => {
        const mod = codeModules.find((m) => m.id === modId);
        mod?.files.forEach((f) => paths.add(f.path));
      });
      return paths;
    };

    const prevOffered = getOfferedPaths(selectedModules);
    const nextOffered = getOfferedPaths(nextModules);

    setSelectedFiles((current) => {
      const nextSelected: string[] = [];
      nextOffered.forEach((path) => {
        if (prevOffered.has(path)) {
          if (current.includes(path)) {
            nextSelected.push(path);
          }
        } else {
          if (path.startsWith('docs/')) {
            nextSelected.push(path);
          }
        }
      });
      return nextSelected;
    });

    setSelectedModules(nextModules);
  };

  const toggleFile = (path: string) => {
    setSelectedFiles((current) =>
      current.includes(path) ? current.filter((item) => item !== path) : [...current, path],
    );
  };

  const validateCurrentStep = () => {
    if (hasPresetStep && step === 2) {
      if (workType === 'code') {
        if (selectedModules.length === 0) {
          alert('请选择至少一个 Code 模块。');
          return false;
        }
      } else {
        if (!presetId) {
          alert('请选择一个用途预设。');
          return false;
        }
      }
    }
    const metadataStep = currentStepLabels.length - 2;
    if (step === metadataStep && !name.trim()) {
      alert('请输入模板名称。');
      return false;
    }
    return true;
  };

  const handleNext = () => {
    if (!validateCurrentStep()) return;
    setStep((current) => Math.min(currentStepLabels.length, current + 1));
  };

  const handleSubmit = () => {
    onCreate({
      name: name.trim(),
      description: description.trim(),
      workType,
      language,
      presetId: workType === 'code' ? undefined : presetId,
      selectedModules: workType === 'code' ? selectedModules : [],
      optionalFiles: selectedFiles,
    });
  };

  const previewStep = currentStepLabels.length;
  const filesStep = currentStepLabels.length - 1;

  return (
    <div className="modal-overlay" onClick={onClose} style={{ zIndex: 1000 }}>
      <div className="modal-body" onClick={(event) => event.stopPropagation()} style={{ width: '42rem', maxWidth: '90vw', height: 'auto' }}>
        <div className="modal-header">
          <h3>新建 Harness 模板</h3>
          <button type="button" className="close-btn" onClick={onClose} aria-label="关闭">
            <X size={20} />
          </button>
        </div>

        <div className="harness-modal-content" style={{ marginTop: 'var(--space-2)' }}>
          <div className="harness-wizard-steps" aria-label="创建进度">
            {currentStepLabels.map((label, index) => (
              <div
                key={label}
                className="harness-wizard-step"
                data-active={step === index + 1}
                data-completed={step > index + 1}
                title={label}
              >
                {index + 1}
              </div>
            ))}
          </div>

          {step === 1 && (
            <section className="harness-wizard-panel">
              <p className="harness-wizard-hint">选择这个模板长期服务的 AI 工作类型。</p>
              <div className="harness-type-grid">
                {WORK_TYPES.map((type) => (
                  <button
                    type="button"
                    key={type.type}
                    className="harness-type-card"
                    data-selected={workType === type.type}
                    onClick={() => chooseWorkType(type.type)}
                  >
                    <strong>{type.name}</strong>
                    <span>{type.description}</span>
                  </button>
                ))}
              </div>
            </section>
          )}

          {hasPresetStep && step === 2 && (
            <section className="harness-wizard-panel">
              {workType === 'code' ? (
                <>
                  <p className="harness-wizard-hint">选择要启用的 Code 模块。模块定义了初始文件和专属指令规范。</p>
                  {isCodeModulesLoading ? (
                    <p>正在加载系统模块...</p>
                  ) : (
                    <div className="harness-type-grid">
                      {codeModules.map((module) => (
                        <button
                          type="button"
                          key={module.id}
                          className="harness-type-card"
                          data-selected={selectedModules.includes(module.id)}
                          onClick={() => handleToggleModule(module.id)}
                        >
                          <strong>{module.name}</strong>
                          <span>{module.description}</span>
                        </button>
                      ))}
                    </div>
                  )}
                </>
              ) : (
                <>
                  <p className="harness-wizard-hint">选择系统维护的只读用途预设。预设决定初始文件和内容骨架。</p>
                  {isPresetsLoading ? (
                    <p>正在加载系统预设...</p>
                  ) : (
                    <div className="harness-type-grid">
                      {availablePresets.map((preset) => (
                        <button
                          type="button"
                          key={preset.id}
                          className="harness-type-card"
                          data-selected={presetId === preset.id}
                          onClick={() => choosePreset(preset)}
                        >
                          <strong>{preset.name}</strong>
                          <span>{preset.description}</span>
                        </button>
                      ))}
                    </div>
                  )}
                </>
              )}
            </section>
          )}

          {step === (hasPresetStep ? 3 : 2) && (
            <section className="harness-wizard-panel">
              <p className="harness-wizard-hint">填写模板的显示信息。模板 ID 将由系统自动生成。</p>
              <div className="harness-identity-row" data-testid="harness-identity-row">
                <div className="harness-form-group">
                  <label htmlFor="harness-name">显示名称</label>
                  <input id="harness-name" placeholder="例如：Web 前端标准开发规范" value={name} onChange={(event) => setName(event.target.value)} />
                </div>
                <div className="harness-form-group harness-form-group--language">
                  <label htmlFor="harness-language">模板语言</label>
                  <select className="harness-language-select" id="harness-language" value={language} onChange={(event) => setLanguage(event.target.value as 'zh-CN' | 'en')}>
                    <option value="zh-CN">简体中文</option>
                    <option value="en">English</option>
                  </select>
                </div>
              </div>
              <div className="harness-form-group">
                <label htmlFor="harness-desc">描述信息</label>
                <textarea id="harness-desc" placeholder="该模板的用途与主要约束规则介绍..." value={description} onChange={(event) => setDescription(event.target.value)} rows={4} />
              </div>
            </section>
          )}

          {step === filesStep && (
            <section className="harness-wizard-panel">
              <p className="harness-wizard-hint">确认要生成的标准文件。预设文件默认全选，创建后仍可自由编辑、增加或删除。</p>
              <div className="harness-checklist">
                {fileOptions.map((file) => {
                  let badgeText = '';
                  if (workType === 'code') {
                    const contributors: string[] = [];
                    const isShared = codeSharedFiles.some((f) => f.path === file.path);
                    if (isShared) {
                      contributors.push('Shared');
                    }
                    selectedModules.forEach((modId) => {
                      const mod = codeModules.find((m) => m.id === modId);
                      if (mod && mod.files.some((f) => f.path === file.path)) {
                        contributors.push(mod.name);
                      }
                    });
                    badgeText = contributors.join(', ');
                  }

                  return (
                    <label key={file.path} className="harness-checklist-item" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', width: '100%' }}>
                      <div style={{ display: 'flex', alignItems: 'center', gap: '0.55rem' }}>
                        <input type="checkbox" checked={selectedFiles.includes(file.path)} onChange={() => toggleFile(file.path)} />
                        <span>{file.label} <small style={{ color: 'var(--color-muted)', marginLeft: 'var(--space-1)' }}>{file.path}</small></span>
                      </div>
                      {badgeText && (
                        <span className="harness-file-contributor">
                          {badgeText}
                        </span>
                      )}
                    </label>
                  );
                })}
              </div>
            </section>
          )}

          {step === previewStep && (
            <section className="harness-wizard-panel">
              <p className="harness-wizard-hint">检查即将生成的目录结构。</p>
              <div className="harness-tree-preview">
                <div>📁 ~/.agent-forge/harnesses/&lt;system-generated-id&gt;/</div>
                <div>&nbsp; 📄 AGENTS.md <small>(必填 Agent 入口)</small></div>
                <div>&nbsp; 📁 docs/</div>
                <div>&nbsp;&nbsp; 📄 harness.toml <small>(必填系统元数据)</small></div>
                {Array.from(new Set(selectedFiles)).map((file) => (
                  <div key={file}>&nbsp;&nbsp; 📄 {file.replace(/^docs\//, '')}</div>
                ))}
              </div>
            </section>
          )}

          <div className="harness-wizard-footer">
            {step > 1 && <button type="button" className="button button--secondary" onClick={() => setStep((current) => current - 1)}><ArrowLeft size={16} /> 上一步</button>}
            {step < previewStep ? (
              <button type="button" className="button button--primary" onClick={handleNext}>下一步 <ArrowRight size={16} /></button>
            ) : (
              <button type="button" className="button button--primary" onClick={handleSubmit} disabled={!name.trim() || isRegistryLoading}>确认创建</button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
