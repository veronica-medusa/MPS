import React, { useState } from 'react';
import { X, Combine, Split, Layers, Search, Wand2, Check } from 'lucide-react';
import { SmartToolboxAction } from '../types';

interface SmartToolboxModalProps {
  action: SmartToolboxAction | null;
  onClose: () => void;
  onApply: (action: SmartToolboxAction, config: any) => void;
}

export const SmartToolboxModal: React.FC<SmartToolboxModalProps> = ({ action, onClose, onApply }) => {
  const [delimiter, setDelimiter] = useState<string>('comma');
  const [customDelimiter, setCustomDelimiter] = useState<string>('');
  const [splitColumn, setSplitColumn] = useState<string>('A');
  const [hygieneOptions, setHygieneOptions] = useState({
    trimSpaces: true,
    removeDuplicates: true,
    stripNonPrintable: true,
    extractEmails: false,
  });

  if (!action) return null;

  const handleApply = () => {
    onApply(action, { delimiter: delimiter === 'custom' ? customDelimiter : delimiter, splitColumn, hygieneOptions });
    onClose();
  };

  const getTitle = () => {
    switch (action) {
      case 'merge_no_loss':
        return 'Merge Cells Without Data Loss';
      case 'split_sheet':
        return 'Split Sheet Wizard';
      case 'combine_sheets':
        return 'Combine Workbooks & Sheets';
      case 'compare_sheets':
        return 'Sheet Visual Comparator';
      case 'data_hygiene':
        return 'One-Click Data Hygiene';
    }
  };

  return (
    <div className="fixed inset-0 bg-black/40 backdrop-blur-sm z-50 flex items-center justify-center p-4 select-none">
      <div className="w-full max-w-md bg-white dark:bg-mercury-night-raised border border-mercury-fog/30 rounded-mercury-lg shadow-2xl p-5 animate-in fade-in zoom-in-95">
        {/* Header */}
        <div className="flex items-center justify-between pb-3 border-b border-mercury-fog/20 mb-4">
          <div className="flex items-center space-x-2">
            <Combine className="w-5 h-5 text-mercury-sheets" />
            <h3 className="font-semibold text-sm text-mercury-ink dark:text-mercury-on-dark">{getTitle()}</h3>
          </div>
          <button
            onClick={onClose}
            className="p-1 rounded-mercury-sm text-mercury-ink-secondary dark:text-mercury-fog hover:bg-mercury-lavender dark:hover:bg-mercury-night"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Content based on action */}
        {action === 'merge_no_loss' && (
          <div className="space-y-4 text-xs">
            <p className="text-mercury-ink-secondary dark:text-mercury-fog">
              Combines all selected cells into a single cell, concatenating all text values without discarding any data.
            </p>
            <div>
              <label className="block font-medium text-mercury-ink dark:text-mercury-on-dark mb-2">
                Select Delimiter:
              </label>
              <div className="grid grid-cols-2 gap-2">
                {[
                  { id: 'comma', label: 'Comma (,)' },
                  { id: 'space', label: 'Space ( )' },
                  { id: 'semicolon', label: 'Semicolon (;)' },
                  { id: 'newline', label: 'New Line (\\n)' },
                ].map((item) => (
                  <button
                    key={item.id}
                    onClick={() => setDelimiter(item.id)}
                    className={`py-2 px-3 rounded-mercury-md border text-left flex items-center justify-between transition-colors ${
                      delimiter === item.id
                        ? 'border-mercury-sheets bg-mercury-sheets/10 text-mercury-sheets font-medium'
                        : 'border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night text-mercury-ink dark:text-mercury-on-dark'
                    }`}
                  >
                    <span>{item.label}</span>
                    {delimiter === item.id && <Check className="w-3.5 h-3.5" />}
                  </button>
                ))}
              </div>
            </div>
          </div>
        )}

        {action === 'split_sheet' && (
          <div className="space-y-4 text-xs">
            <p className="text-mercury-ink-secondary dark:text-mercury-fog">
              Splits the active worksheet into separate worksheets or files based on unique values in a chosen column.
            </p>
            <div>
              <label className="block font-medium text-mercury-ink dark:text-mercury-on-dark mb-1">
                Partition by Column:
              </label>
              <select
                value={splitColumn}
                onChange={(e) => setSplitColumn(e.target.value)}
                className="w-full px-3 py-2 border border-mercury-fog/40 rounded-mercury-md bg-white dark:bg-mercury-night text-mercury-ink dark:text-mercury-on-dark"
              >
                <option value="A">Column A (Region)</option>
                <option value="B">Column B (Sales Rep)</option>
                <option value="C">Column C (Department)</option>
              </select>
            </div>
          </div>
        )}

        {action === 'data_hygiene' && (
          <div className="space-y-3 text-xs">
            <p className="text-mercury-ink-secondary dark:text-mercury-fog">
              Batch clean, deduplicate, and normalize cell data across the selected range.
            </p>
            <div className="space-y-2">
              <label className="flex items-center space-x-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={hygieneOptions.trimSpaces}
                  onChange={(e) => setHygieneOptions({ ...hygieneOptions, trimSpaces: e.target.checked })}
                  className="rounded text-mercury-sheets accent-mercury-sheets"
                />
                <span className="text-mercury-ink dark:text-mercury-on-dark">Trim excess whitespace & trailing spaces</span>
              </label>
              <label className="flex items-center space-x-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={hygieneOptions.removeDuplicates}
                  onChange={(e) => setHygieneOptions({ ...hygieneOptions, removeDuplicates: e.target.checked })}
                  className="rounded text-mercury-sheets accent-mercury-sheets"
                />
                <span className="text-mercury-ink dark:text-mercury-on-dark">Remove duplicate rows across key columns</span>
              </label>
              <label className="flex items-center space-x-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={hygieneOptions.stripNonPrintable}
                  onChange={(e) => setHygieneOptions({ ...hygieneOptions, stripNonPrintable: e.target.checked })}
                  className="rounded text-mercury-sheets accent-mercury-sheets"
                />
                <span className="text-mercury-ink dark:text-mercury-on-dark">Strip non-printable ASCII characters</span>
              </label>
            </div>
          </div>
        )}

        {/* Action Buttons */}
        <div className="mt-6 flex items-center justify-end space-x-2 pt-3 border-t border-mercury-fog/20">
          <button
            onClick={onClose}
            className="px-3 py-1.5 rounded-mercury-md text-xs font-medium text-mercury-ink-secondary dark:text-mercury-fog hover:bg-mercury-lavender dark:hover:bg-mercury-night"
          >
            Cancel
          </button>
          <button
            onClick={handleApply}
            className="px-4 py-1.5 rounded-mercury-md text-xs font-medium bg-mercury-sheets text-white hover:bg-mercury-sheets/90 shadow-sm"
          >
            Execute Tool
          </button>
        </div>
      </div>
    </div>
  );
};
