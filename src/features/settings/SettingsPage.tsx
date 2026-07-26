import { useEffect, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
  AppWindow,
  Check,
  CheckCircle2,
  ChevronDown,
  Copy,
  Languages,
  Monitor,
  MonitorCog,
  Moon,
  Palette,
  ShieldCheck,
  Sun,
  TerminalSquare,
} from 'lucide-react';
import { useBlocker } from 'react-router-dom';
import { useTranslation } from 'react-i18next';

import appIcon from '../../../src-tauri/icons/icon.png';
import { getHealth, getLaunchPreferences, saveLaunchPreferences } from '../../shared/api/tauriClient';
import type { LaunchPreferences, TerminalPreference } from '../../shared/api/types';
import { useThemeStore, type ThemePreference } from '../../shared/theme/themeStore';
import { useLanguageStore } from '../../shared/i18n/languageStore';
import type { LanguagePreference } from '../../shared/i18n/i18n';
import { Card } from '../../shared/ui/Card';
import { PageState } from '../../shared/ui/PageState';
import { StatusBadge } from '../../shared/ui/StatusBadge';

type SettingsSection = 'general' | 'launch' | 'about';

const macosTerminals: Array<{ value: TerminalPreference; label: string; detail: string }> = [
  { value: 'auto', label: '自动选择', detail: '启动时选择首个可用终端。' },
  { value: 'terminal', label: 'Terminal', detail: '使用 macOS 自带终端。' },
  { value: 'iterm', label: 'iTerm2', detail: '适合多标签与开发工作流。' },
  { value: 'warp', label: 'Warp', detail: '使用 Warp 的原生工作区。' },
  { value: 'vscode', label: 'VS Code', detail: '将 CLI 交给集成终端。' },
];

const windowsTerminals: Array<{ value: TerminalPreference; label: string; detail: string }> = [
  { value: 'auto', label: '自动选择', detail: '优先使用 Windows Terminal；不可用时回退到 PowerShell。' },
  { value: 'windows_terminal', label: 'Windows Terminal', detail: '使用 Windows Terminal 新标签页。' },
  { value: 'powershell', label: 'PowerShell', detail: '使用 PowerShell 启动 CLI。' },
];

function PreferenceToggle({ checked, onChange, label, detail }: { checked: boolean; onChange: (checked: boolean) => void; label: string; detail: string }) {
  return (
    <label className="launch-preference-toggle">
      <input type="checkbox" checked={checked} onChange={(event) => onChange(event.target.checked)} />
      <span><strong>{label}</strong><small>{detail}</small></span>
    </label>
  );
}

function GeneralSettings() {
  const { t } = useTranslation();
  const theme = useThemeStore((state) => state.theme);
  const setTheme = useThemeStore((state) => state.setTheme);
  const language = useLanguageStore((state) => state.language);
  const setLanguage = useLanguageStore((state) => state.setLanguage);
  const themes: Array<{ value: ThemePreference; label: string; detail: string; icon: typeof Monitor }> = [
    { value: 'system', label: t('theme.system'), detail: t('settings.appearanceSystem'), icon: Monitor },
    { value: 'light', label: t('theme.light'), detail: t('settings.appearanceLight'), icon: Sun },
    { value: 'dark', label: t('theme.dark'), detail: t('settings.appearanceDark'), icon: Moon },
  ];

  return (
    <div className="settings-general" aria-labelledby="general-settings-title">
      <div className="settings-section-intro">
        <div className="settings-section-icon"><Palette aria-hidden="true" size={20} /></div>
        <div>
          <h2 id="general-settings-title">{t('settings.generalTitle')}</h2>
          <p>{t('settings.generalDescription')}</p>
        </div>
      </div>

      <section className="settings-group" aria-labelledby="appearance-title">
        <div className="settings-group-heading">
          <div><h3 id="appearance-title">{t('theme.label')}</h3><p>{t('settings.appearanceDescription')}</p></div>
          <StatusBadge>{t('settings.autoSaved')}</StatusBadge>
        </div>
        <div className="theme-choice-grid" role="radiogroup" aria-label={t('theme.label')}>
          {themes.map((option) => {
            const Icon = option.icon;
            return (
              <button
                type="button"
                role="radio"
                aria-checked={theme === option.value}
                className={`theme-choice theme-choice--${option.value}${theme === option.value ? ' is-selected' : ''}`}
                key={option.value}
                onClick={() => setTheme(option.value)}
              >
                <span className="theme-choice__preview" aria-hidden="true">
                  <span className="theme-choice__sidebar" />
                  <span className="theme-choice__canvas"><i /><i /><i /></span>
                </span>
                <span className="theme-choice__copy"><Icon aria-hidden="true" size={17} /><strong>{option.label}</strong><small>{option.detail}</small></span>
                {theme === option.value && <Check className="theme-choice__check" aria-hidden="true" size={17} />}
              </button>
            );
          })}
        </div>
      </section>

      <section className="settings-group settings-language" aria-labelledby="language-title">
        <div className="settings-language__copy">
          <Languages aria-hidden="true" size={19} />
          <div><h3 id="language-title">{t('settings.language')}</h3><p>{t('settings.languageDescription')}</p></div>
        </div>
        <label className="settings-language__select">
          <span className="sr-only">{t('settings.languageLabel')}</span>
          <select aria-label={t('settings.languageLabel')} value={language} onChange={(event) => setLanguage(event.target.value as LanguagePreference)}>
            <option value="system">{t('settings.languageSystem')}</option>
            <option value="zh-CN">{t('settings.languageChinese')}</option>
            <option value="en">{t('settings.languageEnglish')}</option>
          </select>
          <ChevronDown aria-hidden="true" size={17} />
        </label>
      </section>
    </div>
  );
}

function AboutSettings({ version, loading, error, onRetry }: { version?: string; loading: boolean; error: boolean; onRetry: () => void }) {
  const { t } = useTranslation();
  return (
    <div className="settings-about" aria-labelledby="about-settings-title">
      <div className="about-product">
        <img src={appIcon} alt={t('settings.aboutProductAlt')} />
        <div>
          <h2 id="about-settings-title">AgentPalette</h2>
          <p>{t('settings.aboutTagline')}</p>
        </div>
      </div>

      <section className="about-details" aria-label={t('settings.aboutDetails')}>
        <div><span>{t('settings.currentVersion')}</span>{loading ? <small>{t('settings.readingVersion')}</small> : error ? <small className="settings-error-copy">{t('settings.versionReadFailed')}</small> : <strong>{version}</strong>}</div>
        <div><span>{t('settings.githubRepository')}</span><a className="about-github-text-link" href="https://github.com/Aimny2020/agentpalette" target="_blank" rel="noreferrer" aria-label={t('settings.openGithub')}>github.com/Aimny2020/agentpalette ↗</a></div>
        <div><span>{t('settings.updates')}</span><button type="button" className="button button--secondary" disabled>{t('settings.checkUpdatesUnavailable')}</button></div>
      </section>
      {error && <button type="button" className="button button--secondary about-retry" onClick={onRetry}>{t('settings.retryVersion')}</button>}
    </div>
  );
}

function LeaveSettingsDialog({ saving, error, onSave, onDiscard, onContinue }: { saving: boolean; error: boolean; onSave: () => void; onDiscard: () => void; onContinue: () => void }) {
  return (
    <div className="settings-dialog-backdrop">
      <div className="settings-dialog" role="dialog" aria-modal="true" aria-labelledby="leave-settings-title">
        <div className="settings-dialog__icon"><AppWindow aria-hidden="true" size={22} /></div>
        <h2 id="leave-settings-title">保存启动偏好？</h2>
        <p>你对平台启动偏好的修改尚未保存。离开后可以保存，也可以放弃这些修改。</p>
        {error && <p className="settings-error-copy" role="alert">保存失败，请重试或继续编辑。</p>}
        <div className="settings-dialog__actions">
          <button type="button" className="button button--secondary" disabled={saving} onClick={onContinue}>继续编辑</button>
          <button type="button" className="button button--secondary" disabled={saving} onClick={onDiscard}>放弃修改</button>
          <button type="button" className="button button--primary" disabled={saving} onClick={onSave}>{saving ? '正在保存...' : '保存并离开'}</button>
        </div>
      </div>
    </div>
  );
}

export function SettingsPage() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [activeSection, setActiveSection] = useState<SettingsSection>('general');
  const sections: Array<{ id: SettingsSection; label: string }> = [
    { id: 'general', label: t('settings.sections.general') },
    { id: 'launch', label: t('settings.sections.launch') },
    { id: 'about', label: t('settings.sections.about') },
  ];
  const health = useQuery({ queryKey: ['health'], queryFn: getHealth, enabled: activeSection !== 'general' });
  const preferences = useQuery({ queryKey: ['launchPreferences'], queryFn: getLaunchPreferences, enabled: activeSection === 'launch' });
  const [draft, setDraft] = useState<LaunchPreferences | null>(null);

  useEffect(() => {
    if (preferences.data) setDraft(preferences.data);
  }, [preferences.data]);

  const saveMutation = useMutation({
    mutationFn: saveLaunchPreferences,
    onSuccess: (saved) => {
      queryClient.setQueryData(['launchPreferences'], saved);
      setDraft(saved);
    },
  });

  const dirty = Boolean(draft && preferences.data && JSON.stringify(draft) !== JSON.stringify(preferences.data));
  const blocker = useBlocker(dirty);
  const isMac = health.data?.platform === 'macos';
  const terminalOptions = isMac ? macosTerminals : windowsTerminals;
  const terminalKey = isMac ? 'macosTerminal' : 'windowsTerminal';
  const activeTerminal = draft?.[terminalKey];
  const update = (changes: Partial<LaunchPreferences>) => setDraft((current) => current ? { ...current, ...changes } : current);

  const saveAndLeave = async () => {
    if (!draft) return;
    try {
      await saveMutation.mutateAsync(draft);
      blocker.proceed?.();
    } catch {
      // The mutation exposes the contextual error inside the dialog.
    }
  };

  const discardAndLeave = () => {
    if (preferences.data) setDraft(preferences.data);
    blocker.proceed?.();
  };

  return (
    <div className="page-stack fixed-workspace-page settings-page-container">
      <header className="page-header settings-page-header">
        <div><h1>{t('settings.title')}</h1><p>{t('settings.description')}</p></div>
      </header>

      <nav className="settings-section-tabs" aria-label={t('settings.sectionsLabel')}>
        {sections.map((section) => (
          <button
            type="button"
            className={activeSection === section.id ? 'is-active' : undefined}
            aria-current={activeSection === section.id ? 'page' : undefined}
            key={section.id}
            onClick={() => setActiveSection(section.id)}
          >
            {section.label}
          </button>
        ))}
      </nav>

      <div className="settings-content">
        {activeSection === 'general' && <GeneralSettings />}

        {activeSection === 'launch' && (
          <>
            {(preferences.isLoading || health.isLoading) && <PageState state="loading" label="正在读取启动偏好..." />}
            {(preferences.isError || health.isError) && <PageState state="error" title="无法读取启动偏好" description="基础设置与关于不受影响。请确认 AgentPalette 后端已启动，然后重试。" onRetry={() => { void preferences.refetch(); void health.refetch(); }} />}
            {preferences.isSuccess && health.isSuccess && draft && (
              <div className="settings-grid settings-grid--launch">
                <Card className="launch-preferences-card">
                  <div className="settings-card-heading"><TerminalSquare size={18} /><div><h2>默认终端</h2><p>为当前系统选择打开 Agent CLI 的终端。</p></div></div>
                  <fieldset className="terminal-choice-grid"><legend className="sr-only">默认终端</legend>
                    {terminalOptions.map((option) => <label className={`terminal-choice${activeTerminal === option.value ? ' is-selected' : ''}`} key={option.value}>
                      <input type="radio" name="terminal" value={option.value} checked={activeTerminal === option.value} onChange={() => update({ [terminalKey]: option.value })} />
                      <span><strong>{option.label}</strong><small>{option.detail}</small></span>
                      {activeTerminal === option.value && <CheckCircle2 aria-hidden="true" size={18} />}
                    </label>)}
                  </fieldset>
                  <div className="settings-select-row"><label htmlFor="launch-presentation"><strong>启动位置</strong><small>项目会以同一个目录交给所选终端，不会复制工程。</small></label><select id="launch-presentation" value={draft.launchPresentation} onChange={(event) => update({ launchPresentation: event.target.value as LaunchPreferences['launchPresentation'] })}><option value="new_tab">在新标签页启动</option><option value="new_window">在新窗口启动</option></select></div>
                </Card>

                <Card className="launch-preferences-card">
                  <div className="settings-card-heading"><ShieldCheck size={18} /><div><h2>启动前检查</h2><p>在项目交接给 Agent 前发现环境与权限风险。</p></div></div>
                  <div className="launch-toggle-list">
                    <PreferenceToggle checked={draft.showCommandPreview} onChange={(showCommandPreview) => update({ showCommandPreview })} label="显示命令预览" detail="启动前展示项目目录与安全处理后的命令。" />
                    <PreferenceToggle checked={draft.checkEnvironment} onChange={(checkEnvironment) => update({ checkEnvironment })} label="检查项目环境" detail="后续将检查 Git、CLI 与项目运行时。" />
                    <PreferenceToggle checked={draft.checkPermissions} onChange={(checkPermissions) => update({ checkPermissions })} label="检查权限摘要" detail="启动前展示文件、网络与 MCP 访问范围。" />
                    <PreferenceToggle checked={draft.allowCopyCommandFallback} onChange={(allowCopyCommandFallback) => update({ allowCopyCommandFallback })} label="允许复制命令兜底" detail="终端交接失败时仍可复制完整命令继续工作。" />
                  </div>
                </Card>

                <Card className="launch-preferences-card launch-preferences-card--summary">
                  <div className="settings-card-heading"><MonitorCog size={18} /><div><h2>启动体验</h2><p>此设置会成为项目主页中 Agent 启动操作的全局默认值。</p></div></div>
                  <div className="launch-preference-summary"><span>当前默认</span><strong>{terminalOptions.find((option) => option.value === activeTerminal)?.label}</strong><span>{draft.launchPresentation === 'new_tab' ? '新标签页' : '新窗口'}，{draft.showCommandPreview ? '先预览命令' : '直接启动'}</span></div>
                  <p className="muted-copy"><Copy size={14} /> 项目级覆盖与受管运行会在 Agent Control Center 中提供。</p>
                </Card>
              </div>
            )}
          </>
        )}

        {activeSection === 'about' && <AboutSettings version={health.data?.version} loading={health.isLoading} error={health.isError} onRetry={() => void health.refetch()} />}
      </div>

      {activeSection === 'launch' && draft && preferences.isSuccess && (
        <div className="settings-save-bar" aria-live="polite">
          <span>{saveMutation.isError ? '保存失败，请重试。' : saveMutation.isSuccess ? '启动偏好已保存。' : dirty ? '你有未保存的启动偏好。' : '所有启动偏好已保存。'}</span>
          <button type="button" className="button button--primary" disabled={!dirty || saveMutation.isPending} onClick={() => saveMutation.mutate(draft)}>{saveMutation.isPending ? '正在保存...' : '保存启动偏好'}</button>
        </div>
      )}

      {blocker.state === 'blocked' && <LeaveSettingsDialog saving={saveMutation.isPending} error={saveMutation.isError} onSave={() => void saveAndLeave()} onDiscard={discardAndLeave} onContinue={() => blocker.reset?.()} />}
    </div>
  );
}
