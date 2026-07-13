import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { CreateHarnessModal } from './CreateHarnessModal';

const codeModules = [
  {
    id: 'technical-design',
    name: 'Technical Design',
    description: 'Design first',
    files: [
      { path: 'docs/decision-record.md', kind: 'markdown', label: 'Decision Record', content: '' },
    ],
    agentInstructions: '',
  },
  {
    id: 'feature-development',
    name: 'Feature Development',
    description: 'Feature development',
    files: [
      { path: 'docs/feature_list.json', kind: 'json', label: 'Feature List', content: '' },
    ],
    agentInstructions: '',
  },
  {
    id: 'code-review',
    name: 'Code Review',
    description: 'Code review',
    files: [
      { path: 'docs/review-rubric.md', kind: 'markdown', label: 'Review Rubric', content: '' },
    ],
    agentInstructions: '',
  },
];

const codeSharedFiles = [
  { path: 'docs/architecture.md', kind: 'markdown', label: 'Architecture', content: '' },
];

const presets = [
  {
    id: 'doc-report',
    workType: 'document',
    name: 'Professional Report',
    description: 'Report preset',
    files: [
      { path: 'docs/document-brief.md', kind: 'markdown', label: 'Document Brief', content: '' },
    ],
  },
];

describe('CreateHarnessModal', () => {
  it('places template language beside the display name', () => {
    render(<CreateHarnessModal onClose={vi.fn()} onCreate={vi.fn()} presets={presets} codeModules={codeModules} codeSharedFiles={codeSharedFiles} />);

    fireEvent.click(screen.getByRole('button', { name: /下一步/i }));
    fireEvent.click(screen.getByRole('button', { name: /Technical Design/i }));
    fireEvent.click(screen.getByRole('button', { name: /下一步/i }));

    expect(screen.getByTestId('harness-identity-row')).toContainElement(screen.getByLabelText('模板语言'));
    expect(screen.getByLabelText('模板语言')).toHaveClass('harness-language-select');
  });

  it('allows multi-selecting Code Work modules and composing files', () => {
    const onCreate = vi.fn();
    render(
      <CreateHarnessModal
        onClose={vi.fn()}
        onCreate={onCreate}
        presets={presets}
        codeModules={codeModules}
        codeSharedFiles={codeSharedFiles}
      />
    );

    // Step 1: Select Code Work and click Next
    fireEvent.click(screen.getByRole('button', { name: /Code Work/i }));
    fireEvent.click(screen.getByRole('button', { name: /下一步/i }));

    // Step 2: Code Work module selection. Select Technical Design and Code Review
    fireEvent.click(screen.getByRole('button', { name: /Technical Design/i }));
    fireEvent.click(screen.getByRole('button', { name: /Code Review/i }));
    fireEvent.click(screen.getByRole('button', { name: /下一步/i }));

    // Step 3: Enter name
    fireEvent.change(screen.getByLabelText('显示名称'), { target: { value: 'Code Harness' } });
    fireEvent.click(screen.getByRole('button', { name: /下一步/i }));

    // Step 4: Files selection
    fireEvent.click(screen.getByRole('button', { name: /下一步/i }));

    // Step 5: Confirm Creation
    fireEvent.click(screen.getByRole('button', { name: /确认创建/i }));

    expect(onCreate).toHaveBeenCalledWith(expect.objectContaining({
      workType: 'code',
      presetId: undefined,
      selectedModules: ['technical-design', 'code-review'],
      optionalFiles: expect.arrayContaining([
        'docs/architecture.md',
        'docs/decision-record.md',
        'docs/review-rubric.md',
      ]),
    }));
  });

  it('toggles code module off when clicked twice, and prevents advancing if none selected', () => {
    const onCreate = vi.fn();
    const alertMock = vi.spyOn(window, 'alert').mockImplementation(() => {});
    render(
      <CreateHarnessModal
        onClose={vi.fn()}
        onCreate={onCreate}
        presets={presets}
        codeModules={codeModules}
        codeSharedFiles={codeSharedFiles}
      />
    );

    // Step 1: Select Code Work and click Next
    fireEvent.click(screen.getByRole('button', { name: /Code Work/i }));
    fireEvent.click(screen.getByRole('button', { name: /下一步/i }));

    // Step 2: Toggle module on and off
    const tdButton = screen.getByRole('button', { name: /Technical Design/i });
    fireEvent.click(tdButton); // On
    fireEvent.click(tdButton); // Off

    // Try to advance
    fireEvent.click(screen.getByRole('button', { name: /下一步/i }));
    expect(alertMock).toHaveBeenCalledWith(expect.stringContaining('请选择至少一个 Code 模块'));

    alertMock.mockRestore();
  });

  it('lets Custom Work choose from the complete standard file library', () => {
    const onCreate = vi.fn();
    render(
      <CreateHarnessModal
        onClose={vi.fn()}
        onCreate={onCreate}
        presets={presets}
        codeModules={codeModules}
        codeSharedFiles={codeSharedFiles}
      />
    );

    fireEvent.click(screen.getByRole('button', { name: /Custom Work/i }));
    fireEvent.click(screen.getByRole('button', { name: /下一步/i }));
    fireEvent.change(screen.getByLabelText('显示名称'), { target: { value: 'Custom Harness' } });
    fireEvent.click(screen.getByRole('button', { name: /下一步/i }));

    expect(screen.getByText('Decision Record')).toBeInTheDocument();
    expect(screen.getByText('Feature List')).toBeInTheDocument();
    expect(screen.getByText('Review Rubric')).toBeInTheDocument();
    expect(screen.getByText('Architecture')).toBeInTheDocument();
    expect(screen.getByText('Document Brief')).toBeInTheDocument();
  });

  it('renders correct step labels for Document Work vs Custom Work', () => {
    render(
      <CreateHarnessModal
        onClose={vi.fn()}
        onCreate={vi.fn()}
        presets={presets}
        codeModules={codeModules}
        codeSharedFiles={codeSharedFiles}
      />
    );

    // Default workType is 'code', hasPresetStep is true
    expect(screen.getByTitle('工作类型')).toBeInTheDocument();
    expect(screen.getByTitle('用途预设')).toBeInTheDocument();
    expect(screen.getByTitle('基本信息')).toBeInTheDocument();
    expect(screen.getByTitle('文件配置')).toBeInTheDocument();
    expect(screen.getByTitle('结构预览')).toBeInTheDocument();

    // Select Custom Work
    fireEvent.click(screen.getByRole('button', { name: /Custom Work/i }));

    // Now hasPresetStep is false, '用途预设' should not be present
    expect(screen.getByTitle('工作类型')).toBeInTheDocument();
    expect(screen.queryByTitle('用途预设')).not.toBeInTheDocument();
    expect(screen.getByTitle('基本信息')).toBeInTheDocument();
    expect(screen.getByTitle('文件配置')).toBeInTheDocument();
    expect(screen.getByTitle('结构预览')).toBeInTheDocument();
  });

  it('disables the "确认创建" button when registry is loading for document work', () => {
    const { rerender } = render(
      <CreateHarnessModal
        onClose={vi.fn()}
        onCreate={vi.fn()}
        presets={presets}
        codeModules={codeModules}
        codeSharedFiles={codeSharedFiles}
        isPresetsLoading={false}
      />
    );

    // Choose document work
    fireEvent.click(screen.getByRole('button', { name: /Document Work/i }));
    fireEvent.click(screen.getByRole('button', { name: /下一步/i }));

    // Select preset
    fireEvent.click(screen.getByRole('button', { name: /Professional Report/i }));
    fireEvent.click(screen.getByRole('button', { name: /下一步/i }));

    // Enter name
    fireEvent.change(screen.getByLabelText('显示名称'), { target: { value: 'Doc Harness' } });
    fireEvent.click(screen.getByRole('button', { name: /下一步/i }));

    // Files selection
    fireEvent.click(screen.getByRole('button', { name: /下一步/i }));

    // Confirm button should be disabled when presets are loading
    rerender(
      <CreateHarnessModal
        onClose={vi.fn()}
        onCreate={vi.fn()}
        presets={presets}
        codeModules={codeModules}
        codeSharedFiles={codeSharedFiles}
        isPresetsLoading={true}
      />
    );
    const confirmBtn = screen.getByRole('button', { name: /确认创建/i });
    expect(confirmBtn).toBeDisabled();

    // Rerender with isPresetsLoading = false
    rerender(
      <CreateHarnessModal
        onClose={vi.fn()}
        onCreate={vi.fn()}
        presets={presets}
        codeModules={codeModules}
        codeSharedFiles={codeSharedFiles}
        isPresetsLoading={false}
      />
    );
    expect(confirmBtn).not.toBeDisabled();
  });

  it('disables the "确认创建" button when registry is loading for code work', () => {
    const { rerender } = render(
      <CreateHarnessModal
        onClose={vi.fn()}
        onCreate={vi.fn()}
        presets={presets}
        codeModules={codeModules}
        codeSharedFiles={codeSharedFiles}
        isCodeModulesLoading={false}
        isCodeSharedFilesLoading={false}
      />
    );

    // Default is code work
    fireEvent.click(screen.getByRole('button', { name: /下一步/i }));

    // Select modules
    fireEvent.click(screen.getByRole('button', { name: /Technical Design/i }));
    fireEvent.click(screen.getByRole('button', { name: /下一步/i }));

    // Enter name
    fireEvent.change(screen.getByLabelText('显示名称'), { target: { value: 'Code Harness' } });
    fireEvent.click(screen.getByRole('button', { name: /下一步/i }));

    // Files selection
    fireEvent.click(screen.getByRole('button', { name: /下一步/i }));

    // Confirm button should be disabled because isCodeModulesLoading is true
    rerender(
      <CreateHarnessModal
        onClose={vi.fn()}
        onCreate={vi.fn()}
        presets={presets}
        codeModules={codeModules}
        codeSharedFiles={codeSharedFiles}
        isCodeModulesLoading={true}
        isCodeSharedFilesLoading={false}
      />
    );
    const confirmBtn = screen.getByRole('button', { name: /确认创建/i });
    expect(confirmBtn).toBeDisabled();

    // Rerender with isCodeModulesLoading = false and isCodeSharedFilesLoading = true
    rerender(
      <CreateHarnessModal
        onClose={vi.fn()}
        onCreate={vi.fn()}
        presets={presets}
        codeModules={codeModules}
        codeSharedFiles={codeSharedFiles}
        isCodeModulesLoading={false}
        isCodeSharedFilesLoading={true}
      />
    );
    expect(confirmBtn).toBeDisabled();

    // Rerender with both false
    rerender(
      <CreateHarnessModal
        onClose={vi.fn()}
        onCreate={vi.fn()}
        presets={presets}
        codeModules={codeModules}
        codeSharedFiles={codeSharedFiles}
        isCodeModulesLoading={false}
        isCodeSharedFilesLoading={false}
      />
    );
    expect(confirmBtn).not.toBeDisabled();
  });

  it('disables the "确认创建" button when registry is loading for custom work', () => {
    const { rerender } = render(
      <CreateHarnessModal
        onClose={vi.fn()}
        onCreate={vi.fn()}
        presets={presets}
        codeModules={codeModules}
        codeSharedFiles={codeSharedFiles}
        isPresetsLoading={false}
        isCodeModulesLoading={false}
        isCodeSharedFilesLoading={false}
      />
    );

    // Select Custom Work
    fireEvent.click(screen.getByRole('button', { name: /Custom Work/i }));
    fireEvent.click(screen.getByRole('button', { name: /下一步/i }));

    // Enter name
    fireEvent.change(screen.getByLabelText('显示名称'), { target: { value: 'Custom Harness' } });
    fireEvent.click(screen.getByRole('button', { name: /下一步/i }));

    // Files selection
    fireEvent.click(screen.getByRole('button', { name: /下一步/i }));

    // Confirm button should be disabled because isPresetsLoading is true
    rerender(
      <CreateHarnessModal
        onClose={vi.fn()}
        onCreate={vi.fn()}
        presets={presets}
        codeModules={codeModules}
        codeSharedFiles={codeSharedFiles}
        isPresetsLoading={true}
        isCodeModulesLoading={false}
        isCodeSharedFilesLoading={false}
      />
    );
    const confirmBtn = screen.getByRole('button', { name: /确认创建/i });
    expect(confirmBtn).toBeDisabled();

    // Rerender with isPresetsLoading = false and isCodeSharedFilesLoading = true
    rerender(
      <CreateHarnessModal
        onClose={vi.fn()}
        onCreate={vi.fn()}
        presets={presets}
        codeModules={codeModules}
        codeSharedFiles={codeSharedFiles}
        isPresetsLoading={false}
        isCodeModulesLoading={false}
        isCodeSharedFilesLoading={true}
      />
    );
    expect(confirmBtn).toBeDisabled();

    // Rerender with all false
    rerender(
      <CreateHarnessModal
        onClose={vi.fn()}
        onCreate={vi.fn()}
        presets={presets}
        codeModules={codeModules}
        codeSharedFiles={codeSharedFiles}
        isPresetsLoading={false}
        isCodeSharedFilesLoading={false}
        isCodeModulesLoading={false}
      />
    );
    expect(confirmBtn).not.toBeDisabled();
  });
});
