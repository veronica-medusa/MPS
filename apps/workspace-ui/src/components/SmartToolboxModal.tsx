import React, { useState } from 'react';
import { X, Combine, Split, Layers, Search, Wand2, Check, Calculator, Type, Table as TableIcon, CheckCircle2 } from 'lucide-react';
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
  const [mathOp, setMathOp] = useState<string>('+');
  const [mathOperand, setMathOperand] = useState<number>(10);
  const [prefix, setPrefix] = useState<string>('');
  const [suffix, setSuffix] = useState<string>('');
  const [caseMode, setCaseMode] = useState<string>('upper');
  const [regexPattern, setRegexPattern] = useState<string>('\\d+');
  const [targetCol, setTargetCol] = useState<string>('D');

  // Pivot Wizard state
  const [pivotRowCol, setPivotRowCol] = useState<number>(0);
  const [pivotValCol, setPivotValCol] = useState<number>(2);
  const [pivotAgg, setPivotAgg] = useState<string>('Sum');
  const [pivotShowAs, setPivotShowAs] = useState<string>('Normal');

  // Data Validation state
  const [valType, setValType] = useState<string>('whole_number');
  const [valMin, setValMin] = useState<number>(0);
  const [valMax, setValMax] = useState<number>(1000000);
  const [valList, setValList] = useState<string>('High,Medium,Low');

  const [hygieneOptions, setHygieneOptions] = useState({
    trimSpaces: true,
    removeDuplicates: true,
    stripNonPrintable: true,
    extractEmails: false,
  });

  if (!action) return null;

  const handleApply = () => {
    onApply(action, {
      delimiter: delimiter === 'custom' ? customDelimiter : delimiter,
      splitColumn,
      mathOp,
      mathOperand,
      prefix,
      suffix,
      caseMode,
      regexPattern,
      targetCol,
      pivotRowCol,
      pivotValCol,
      pivotAgg,
      pivotShowAs,
      valType,
      valMin,
      valMax,
      valList,
      hygieneOptions,
    });
    onClose();
  };

  const getTitle = () => {
    switch (action) {
      case 'merge_no_loss':
        return 'Merge Cells Without Data Loss';
      case 'unmerge_fill':
        return 'Unmerge & Autofill Content';
      case 'split_sheet':
        return 'Split Sheet Wizard';
      case 'combine_sheets':
        return 'Combine Workbooks & Sheets';
      case 'compare_sheets':
        return 'Sheet Visual Comparator';
      case 'data_hygiene':
        return 'One-Click Data Hygiene';
      case 'range_arithmetic':
        return 'Batch Range Arithmetic';
      case 'add_prefix_suffix':
        return 'Batch Add Prefix / Suffix';
      case 'case_conversion':
        return 'Batch Text Case Conversion';
      case 'regex_extract':
        return 'Batch Regex / Text Extractor';
      case 'pivot_wizard':
        return 'Pivot Table Cube Wizard';
      case 'data_validation':
        return 'Data Validation Rules';
      default:
        return 'Smart Toolbox Tool';
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

        {action === 'unmerge_fill' && (
          <div className="space-y-4 text-xs">
            <p className="text-mercury-ink-secondary dark:text-mercury-fog">
              Splits merged cells in the selected range and automatically duplicates the top-left value into all unmerged cells to eliminate blank gaps.
            </p>
            <div className="p-3 bg-mercury-sheets/10 rounded-mercury-md border border-mercury-sheets/20 text-mercury-sheets font-medium">
              &check; Prevents blank cells from breaking VLOOKUP and Pivot Tables.
            </div>
          </div>
        )}

        {action === 'range_arithmetic' && (
          <div className="space-y-4 text-xs">
            <p className="text-mercury-ink-secondary dark:text-mercury-fog">
              Performs rapid in-place arithmetic on all numeric cells within the selected range without modifying non-numeric labels.
            </p>
            <div className="flex items-center space-x-3">
              <div>
                <label className="block font-medium text-mercury-ink dark:text-mercury-on-dark mb-1">Operator:</label>
                <select
                  value={mathOp}
                  onChange={(e) => setMathOp(e.target.value)}
                  className="px-2 py-1.5 border border-mercury-fog/40 rounded bg-white dark:bg-mercury-night text-xs font-mono font-bold"
                >
                  <option value="+">+ Add</option>
                  <option value="-">- Subtract</option>
                  <option value="*">&times; Multiply</option>
                  <option value="/">&divide; Divide</option>
                </select>
              </div>
              <div className="flex-1">
                <label className="block font-medium text-mercury-ink dark:text-mercury-on-dark mb-1">Operand Value:</label>
                <input
                  type="number"
                  value={mathOperand}
                  onChange={(e) => setMathOperand(Number(e.target.value))}
                  className="w-full px-2 py-1.5 border border-mercury-fog/40 rounded bg-white dark:bg-mercury-night text-xs font-mono"
                />
              </div>
            </div>
          </div>
        )}

        {action === 'add_prefix_suffix' && (
          <div className="space-y-4 text-xs">
            <p className="text-mercury-ink-secondary dark:text-mercury-fog">
              Batch prepend a prefix or append a suffix to every cell in the active range.
            </p>
            <div>
              <label className="block font-medium text-mercury-ink dark:text-mercury-on-dark mb-1">Prefix to add:</label>
              <input
                type="text"
                value={prefix}
                onChange={(e) => setPrefix(e.target.value)}
                placeholder="e.g. SKU-"
                className="w-full px-2 py-1.5 border border-mercury-fog/40 rounded bg-white dark:bg-mercury-night text-xs font-mono"
              />
            </div>
            <div>
              <label className="block font-medium text-mercury-ink dark:text-mercury-on-dark mb-1">Suffix to add:</label>
              <input
                type="text"
                value={suffix}
                onChange={(e) => setSuffix(e.target.value)}
                placeholder="e.g. -2026"
                className="w-full px-2 py-1.5 border border-mercury-fog/40 rounded bg-white dark:bg-mercury-night text-xs font-mono"
              />
            </div>
          </div>
        )}

        {action === 'case_conversion' && (
          <div className="space-y-4 text-xs">
            <p className="text-mercury-ink-secondary dark:text-mercury-fog">
              Select desired letter case conversion for all text cells in range:
            </p>
            <div className="space-y-2">
              {[
                { id: 'upper', label: 'UPPERCASE (e.g. JOHN DOE)' },
                { id: 'lower', label: 'lowercase (e.g. john doe)' },
                { id: 'proper', label: 'Proper Case (e.g. John Doe)' },
              ].map((c) => (
                <button
                  key={c.id}
                  onClick={() => setCaseMode(c.id)}
                  className={`w-full py-2 px-3 rounded-mercury-md border text-left flex items-center justify-between transition-colors ${
                    caseMode === c.id
                      ? 'border-mercury-sheets bg-mercury-sheets/10 text-mercury-sheets font-medium'
                      : 'border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night text-mercury-ink dark:text-mercury-on-dark'
                  }`}
                >
                  <span>{c.label}</span>
                  {caseMode === c.id && <Check className="w-3.5 h-3.5" />}
                </button>
              ))}
            </div>
          </div>
        )}

        {action === 'regex_extract' && (
          <div className="space-y-4 text-xs">
            <p className="text-mercury-ink-secondary dark:text-mercury-fog">
              Extract matched regex patterns from source cells into an adjacent target column.
            </p>
            <div>
              <label className="block font-medium text-mercury-ink dark:text-mercury-on-dark mb-1">Regex Pattern:</label>
              <input
                type="text"
                value={regexPattern}
                onChange={(e) => setRegexPattern(e.target.value)}
                className="w-full px-2 py-1.5 border border-mercury-fog/40 rounded bg-white dark:bg-mercury-night text-xs font-mono"
              />
            </div>
            <div>
              <label className="block font-medium text-mercury-ink dark:text-mercury-on-dark mb-1">Target Output Column:</label>
              <input
                type="text"
                value={targetCol}
                onChange={(e) => setTargetCol(e.target.value.toUpperCase())}
                className="w-20 px-2 py-1.5 border border-mercury-fog/40 rounded bg-white dark:bg-mercury-night text-xs font-mono"
              />
            </div>
          </div>
        )}

        {action === 'pivot_wizard' && (
          <div className="space-y-4 text-xs">
            <p className="text-mercury-ink-secondary dark:text-mercury-fog">
              Generate a multi-dimensional Pivot Table Cube aggregated from the active worksheet data.
            </p>
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="block font-medium text-mercury-ink dark:text-mercury-on-dark mb-1">Row Field Column:</label>
                <select
                  value={pivotRowCol}
                  onChange={(e) => setPivotRowCol(Number(e.target.value))}
                  className="w-full px-2 py-1.5 border border-mercury-fog/40 rounded bg-white dark:bg-mercury-night text-xs"
                >
                  <option value={0}>Col A (Category / Metric)</option>
                  <option value={1}>Col B (Region / Q1)</option>
                  <option value={2}>Col C (Status)</option>
                </select>
              </div>
              <div>
                <label className="block font-medium text-mercury-ink dark:text-mercury-on-dark mb-1">Value Field Column:</label>
                <select
                  value={pivotValCol}
                  onChange={(e) => setPivotValCol(Number(e.target.value))}
                  className="w-full px-2 py-1.5 border border-mercury-fog/40 rounded bg-white dark:bg-mercury-night text-xs"
                >
                  <option value={1}>Col B (Amount)</option>
                  <option value={2}>Col C (Amount)</option>
                  <option value={3}>Col D (Total)</option>
                </select>
              </div>
              <div>
                <label className="block font-medium text-mercury-ink dark:text-mercury-on-dark mb-1">Aggregation Function:</label>
                <select
                  value={pivotAgg}
                  onChange={(e) => setPivotAgg(e.target.value)}
                  className="w-full px-2 py-1.5 border border-mercury-fog/40 rounded bg-white dark:bg-mercury-night text-xs"
                >
                  <option value="Sum">Sum</option>
                  <option value="Count">Count</option>
                  <option value="Average">Average</option>
                  <option value="Min">Min</option>
                  <option value="Max">Max</option>
                  <option value="Product">Product</option>
                  <option value="StdDev">StdDev</option>
                  <option value="Var">Variance</option>
                </select>
              </div>
              <div>
                <label className="block font-medium text-mercury-ink dark:text-mercury-on-dark mb-1">Show Values As:</label>
                <select
                  value={pivotShowAs}
                  onChange={(e) => setPivotShowAs(e.target.value)}
                  className="w-full px-2 py-1.5 border border-mercury-fog/40 rounded bg-white dark:bg-mercury-night text-xs"
                >
                  <option value="Normal">Normal</option>
                  <option value="PercentOfGrandTotal">% of Grand Total</option>
                  <option value="PercentOfRowTotal">% of Row Total</option>
                  <option value="PercentOfColumnTotal">% of Column Total</option>
                </select>
              </div>
            </div>
          </div>
        )}

        {action === 'data_validation' && (
          <div className="space-y-4 text-xs">
            <p className="text-mercury-ink-secondary dark:text-mercury-fog">
              Define input validation criteria to restrict data entry in selected cells.
            </p>
            <div>
              <label className="block font-medium text-mercury-ink dark:text-mercury-on-dark mb-1">Allow Criterion:</label>
              <select
                value={valType}
                onChange={(e) => setValType(e.target.value)}
                className="w-full px-2 py-1.5 border border-mercury-fog/40 rounded bg-white dark:bg-mercury-night text-xs"
              >
                <option value="whole_number">Whole Number (Integer)</option>
                <option value="decimal">Decimal Number</option>
                <option value="list">List of Values (Dropdown)</option>
              </select>
            </div>
            {valType === 'list' ? (
              <div>
                <label className="block font-medium text-mercury-ink dark:text-mercury-on-dark mb-1">Comma-separated Values:</label>
                <input
                  type="text"
                  value={valList}
                  onChange={(e) => setValList(e.target.value)}
                  className="w-full px-2 py-1.5 border border-mercury-fog/40 rounded bg-white dark:bg-mercury-night text-xs font-mono"
                />
              </div>
            ) : (
              <div className="flex items-center space-x-2">
                <div>
                  <label className="block font-medium text-mercury-ink dark:text-mercury-on-dark mb-1">Minimum:</label>
                  <input
                    type="number"
                    value={valMin}
                    onChange={(e) => setValMin(Number(e.target.value))}
                    className="w-full px-2 py-1.5 border border-mercury-fog/40 rounded bg-white dark:bg-mercury-night text-xs font-mono"
                  />
                </div>
                <div>
                  <label className="block font-medium text-mercury-ink dark:text-mercury-on-dark mb-1">Maximum:</label>
                  <input
                    type="number"
                    value={valMax}
                    onChange={(e) => setValMax(Number(e.target.value))}
                    className="w-full px-2 py-1.5 border border-mercury-fog/40 rounded bg-white dark:bg-mercury-night text-xs font-mono"
                  />
                </div>
              </div>
            )}
          </div>
        )}

        {action === 'split_sheet' && (
          <div className="space-y-4 text-xs">
            <p className="text-mercury-ink-secondary dark:text-mercury-fog">
              Splits the active worksheet into separate worksheets or files based on unique values in a chosen column.
            </p>
            <div>
              <label className="block font-medium text-mercury-ink dark:text-mercury-on-dark mb-1">
                Column containing key categories:
              </label>
              <input
                type="text"
                value={splitColumn}
                onChange={(e) => setSplitColumn(e.target.value.toUpperCase())}
                className="w-20 px-2 py-1.5 border border-mercury-fog/40 rounded bg-white dark:bg-mercury-night text-xs font-mono font-bold"
              />
            </div>
          </div>
        )}

        {action === 'compare_sheets' && (
          <div className="space-y-4 text-xs">
            <p className="text-mercury-ink-secondary dark:text-mercury-fog">
              Visual side-by-side sheet comparator. Highlights insertions, modifications, and deletions with color-coded diff overlays.
            </p>
            <div className="p-3 bg-mercury-lavender/30 dark:bg-mercury-night rounded-mercury-md border border-mercury-fog/20 space-y-1">
              <div className="flex items-center space-x-2">
                <span className="w-2.5 h-2.5 rounded-full bg-emerald-500" />
                <span>Green: Newly added values</span>
              </div>
              <div className="flex items-center space-x-2">
                <span className="w-2.5 h-2.5 rounded-full bg-amber-500" />
                <span>Amber: Value modified / formula changed</span>
              </div>
              <div className="flex items-center space-x-2">
                <span className="w-2.5 h-2.5 rounded-full bg-rose-500" />
                <span>Red: Missing or deleted entries</span>
              </div>
            </div>
          </div>
        )}

        {action === 'data_hygiene' && (
          <div className="space-y-4 text-xs">
            <p className="text-mercury-ink-secondary dark:text-mercury-fog">
              Select automated cleaning routines to sanitize active worksheet cells:
            </p>
            <div className="space-y-2">
              {[
                { key: 'trimSpaces', label: 'Trim leading & trailing whitespace' },
                { key: 'removeDuplicates', label: 'Purge duplicate rows automatically' },
                { key: 'stripNonPrintable', label: 'Strip non-printable / control ASCII characters' },
                { key: 'extractEmails', label: 'Standardize email and domain formats' },
              ].map((opt) => (
                <label key={opt.key} className="flex items-center space-x-2 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={(hygieneOptions as any)[opt.key]}
                    onChange={(e) =>
                      setHygieneOptions({ ...hygieneOptions, [opt.key]: e.target.checked })
                    }
                    className="rounded border-mercury-fog text-mercury-sheets focus:ring-mercury-sheets"
                  />
                  <span className="text-mercury-ink dark:text-mercury-on-dark">{opt.label}</span>
                </label>
              ))}
            </div>
          </div>
        )}

        {/* Footer */}
        <div className="flex items-center justify-end space-x-2 pt-4 border-t border-mercury-fog/20 mt-5">
          <button
            onClick={onClose}
            className="px-3 py-1.5 rounded-mercury-sm border border-mercury-fog/30 text-xs text-mercury-ink-secondary hover:bg-mercury-lavender dark:hover:bg-mercury-night"
          >
            Cancel
          </button>
          <button
            onClick={handleApply}
            className="px-4 py-1.5 rounded-mercury-sm bg-mercury-sheets text-white text-xs font-medium hover:bg-mercury-sheets/90 shadow-sm"
          >
            Apply Action
          </button>
        </div>
      </div>
    </div>
  );
};
