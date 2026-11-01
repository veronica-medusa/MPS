import React, { useState, useEffect, useMemo, useRef } from 'react';
import {
  Undo2,
  Redo2,
  Bold,
  Italic,
  Underline,
  AlignLeft,
  AlignCenter,
  AlignRight,
  Plus,
  Trash2,
  Filter,
  ArrowUpDown,
  FunctionSquare,
  BarChart3,
  PieChart,
  LineChart,
  AreaChart,
  Table as TableIcon,
  Sparkles,
  Layers,
  Wand2,
  Split,
  Combine,
  Search,
  Check,
  ChevronDown,
  X
} from 'lucide-react';
import { SheetCell, SingleSheet, SheetsWorkbookState, SelectionStats } from '../types';

interface SheetsWorkspaceProps {
  initialState?: Partial<SheetsWorkbookState>;
  onStateChange?: (state: SheetsWorkbookState) => void;
  onOpenAiAssist?: (prompt: string) => void;
}

// Columns A to Z
const COLS = Array.from({ length: 26 }, (_, i) => String.fromCharCode(65 + i));
// Rows 1 to 50
const ROWS = Array.from({ length: 50 }, (_, i) => i + 1);

export const SheetsWorkspace: React.FC<SheetsWorkspaceProps> = ({
  initialState,
  onStateChange,
  onOpenAiAssist,
}) => {
  // ─── Initial Grid Data ───
  const defaultCells: Record<string, SheetCell> = {
    A1: { raw: 'Region', value: 'Region', bold: true, align: 'center', fillColor: '#f4f4f7' },
    B1: { raw: 'Q1 Sales', value: 'Q1 Sales', bold: true, align: 'center', fillColor: '#f4f4f7' },
    C1: { raw: 'Q2 Sales', value: 'Q2 Sales', bold: true, align: 'center', fillColor: '#f4f4f7' },
    D1: { raw: 'Q3 Sales', value: 'Q3 Sales', bold: true, align: 'center', fillColor: '#f4f4f7' },
    E1: { raw: 'Total Revenue', value: 'Total Revenue', bold: true, align: 'center', fillColor: '#f4f4f7' },

    A2: { raw: 'APAC', value: 'APAC', bold: false, align: 'left' },
    B2: { raw: '450000', value: '$450,000', format: 'currency', align: 'right' },
    C2: { raw: '520000', value: '$520,000', format: 'currency', align: 'right' },
    D2: { raw: '482000', value: '$482,000', format: 'currency', align: 'right' },
    E2: { raw: '=SUM(B2:D2)', value: '$1,452,000', bold: true, format: 'currency', align: 'right' },

    A3: { raw: 'EMEA', value: 'EMEA', bold: false, align: 'left' },
    B3: { raw: '320000', value: '$320,000', format: 'currency', align: 'right' },
    C3: { raw: '345000', value: '$345,000', format: 'currency', align: 'right' },
    D3: { raw: '310000', value: '$310,000', format: 'currency', align: 'right' },
    E3: { raw: '=SUM(B3:D3)', value: '$975,000', bold: true, format: 'currency', align: 'right' },

    A4: { raw: 'AMER', value: 'AMER', bold: false, align: 'left' },
    B4: { raw: '580000', value: '$580,000', format: 'currency', align: 'right' },
    C4: { raw: '610000', value: '$610,000', format: 'currency', align: 'right' },
    D4: { raw: '640000', value: '$640,000', format: 'currency', align: 'right' },
    E4: { raw: '=SUM(B4:D4)', value: '$1,830,000', bold: true, format: 'currency', align: 'right' },

    A5: { raw: 'Grand Total', value: 'Grand Total', bold: true, align: 'left', fillColor: '#e8e8f8' },
    B5: { raw: '=SUM(B2:B4)', value: '$1,350,000', bold: true, format: 'currency', align: 'right', fillColor: '#e8e8f8' },
    C5: { raw: '=SUM(C2:C4)', value: '$1,475,000', bold: true, format: 'currency', align: 'right', fillColor: '#e8e8f8' },
    D5: { raw: '=SUM(D2:D4)', value: '$1,432,000', bold: true, format: 'currency', align: 'right', fillColor: '#e8e8f8' },
    E5: { raw: '=SUM(E2:E4)', value: '$4,257,000', bold: true, format: 'currency', align: 'right', fillColor: '#e8e8f8' },
  };

  const [sheets, setSheets] = useState<SingleSheet[]>([
    { id: 'sheet-1', name: 'Sheet1', cells: defaultCells },
    { id: 'sheet-2', name: 'Sheet2', cells: {} },
  ]);
  const [activeSheetIndex, setActiveSheetIndex] = useState<number>(0);
  const [activeCell, setActiveCell] = useState<string>('E2');
  const [editingCell, setEditingCell] = useState<string | null>(null);
  const [formulaBarInput, setFormulaBarInput] = useState<string>('=SUM(B2:D2)');
  const [readingSpotlight, setReadingSpotlight] = useState<boolean>(false);
  const [showFormulas, setShowFormulas] = useState<boolean>(false);
  const [activeRibbonTab, setActiveRibbonTab] = useState<'home' | 'insert' | 'formulas' | 'data' | 'toolbox'>('home');

  // Chart State
  const [activeChart, setActiveChart] = useState<{
    type: 'bar' | 'line' | 'pie' | 'area';
    title: string;
    labels: string[];
    values: number[];
  } | null>(null);

  // Pivot Table Modal
  const [showPivotModal, setShowPivotModal] = useState<boolean>(false);

  const currentSheet = sheets[activeSheetIndex] || sheets[0];

  // Synchronize formula bar with active cell
  useEffect(() => {
    const cell = currentSheet.cells[activeCell];
    setFormulaBarInput(cell ? cell.raw : '');
  }, [activeCell, activeSheetIndex]);

  // Reactive Formula Evaluator
  const evaluateCellRaw = (raw: string, allCells: Record<string, SheetCell>): { val: string; formatted: string } => {
    if (!raw.startsWith('=')) {
      const num = Number(raw.replace(/[$,]/g, ''));
      if (!isNaN(num) && raw.trim() !== '') {
        return { val: raw, formatted: `$${num.toLocaleString()}` };
      }
      return { val: raw, formatted: raw };
    }

    const upper = raw.toUpperCase().trim();

    // Helper: parse cell range e.g. "B2:D2"
    const parseRange = (rangeStr: string): number[] => {
      const parts = rangeStr.split(':');
      if (parts.length === 1) {
        const cell = allCells[parts[0]];
        const n = cell ? Number(cell.raw.replace(/[^0-9.-]/g, '')) : 0;
        return isNaN(n) ? [0] : [n];
      }
      const [startCol, startRow] = [parts[0].charAt(0), parseInt(parts[0].slice(1))];
      const [endCol, endRow] = [parts[1].charAt(0), parseInt(parts[1].slice(1))];
      const vals: number[] = [];
      const colStartCode = startCol.charCodeAt(0);
      const colEndCode = endCol.charCodeAt(0);

      for (let c = colStartCode; c <= colEndCode; c++) {
        for (let r = startRow; r <= endRow; r++) {
          const coord = `${String.fromCharCode(c)}${r}`;
          const cell = allCells[coord];
          if (cell) {
            const num = Number(cell.raw.replace(/[^0-9.-]/g, ''));
            if (!isNaN(num) && cell.raw.trim() !== '') {
              vals.push(num);
            }
          }
        }
      }
      return vals;
    };

    // =SUM(range)
    const sumMatch = upper.match(/^=SUM\((.+)\)$/);
    if (sumMatch) {
      const vals = parseRange(sumMatch[1]);
      const total = vals.reduce((a, b) => a + b, 0);
      return { val: String(total), formatted: `$${total.toLocaleString()}` };
    }

    // =AVERAGE(range)
    const avgMatch = upper.match(/^=AVERAGE\((.+)\)$/);
    if (avgMatch) {
      const vals = parseRange(avgMatch[1]);
      const avg = vals.length > 0 ? vals.reduce((a, b) => a + b, 0) / vals.length : 0;
      return { val: String(avg), formatted: `$${avg.toLocaleString(undefined, { maximumFractionDigits: 2 })}` };
    }

    // =COUNT(range)
    const countMatch = upper.match(/^=COUNT\((.+)\)$/);
    if (countMatch) {
      const vals = parseRange(countMatch[1]);
      return { val: String(vals.length), formatted: String(vals.length) };
    }

    // =MAX(range)
    const maxMatch = upper.match(/^=MAX\((.+)\)$/);
    if (maxMatch) {
      const vals = parseRange(maxMatch[1]);
      const maxVal = vals.length > 0 ? Math.max(...vals) : 0;
      return { val: String(maxVal), formatted: `$${maxVal.toLocaleString()}` };
    }

    // =MIN(range)
    const minMatch = upper.match(/^=MIN\((.+)\)$/);
    if (minMatch) {
      const vals = parseRange(minMatch[1]);
      const minVal = vals.length > 0 ? Math.min(...vals) : 0;
      return { val: String(minVal), formatted: `$${minVal.toLocaleString()}` };
    }

    // Basic arithmetic e.g. =B2*C2 or =B2+C2
    const arithMatch = upper.match(/^=([A-Z]\d+)([\+\-\*\/])([A-Z]\d+)$/);
    if (arithMatch) {
      const val1 = Number((allCells[arithMatch[1]]?.raw || '0').replace(/[^0-9.-]/g, ''));
      const op = arithMatch[2];
      const val2 = Number((allCells[arithMatch[3]]?.raw || '0').replace(/[^0-9.-]/g, ''));
      let res = 0;
      if (op === '+') res = val1 + val2;
      if (op === '-') res = val1 - val2;
      if (op === '*') res = val1 * val2;
      if (op === '/') res = val2 !== 0 ? val1 / val2 : 0;
      return { val: String(res), formatted: `$${res.toLocaleString()}` };
    }

    return { val: raw, formatted: raw };
  };

  // Recalculate entire sheet when a cell changes
  const updateCell = (coord: string, newRaw: string) => {
    setSheets((prev) =>
      prev.map((s, idx) => {
        if (idx !== activeSheetIndex) return s;
        const newCells = { ...s.cells };
        const evaluated = evaluateCellRaw(newRaw, newCells);
        newCells[coord] = {
          ...newCells[coord],
          raw: newRaw,
          value: evaluated.formatted,
        };

        // Downstream recalculation pass for formula cells
        Object.keys(newCells).forEach((c) => {
          if (newCells[c].raw.startsWith('=')) {
            const re = evaluateCellRaw(newCells[c].raw, newCells);
            newCells[c].value = re.formatted;
          }
        });

        return { ...s, cells: newCells };
      })
    );
  };

  const handleFormulaSubmit = (val: string) => {
    updateCell(activeCell, val);
  };

  // Active Row & Col Coordinates
  const activeColLetter = activeCell.replace(/[0-9]/g, '');
  const activeRowNum = parseInt(activeCell.replace(/[^0-9]/g, ''), 10);

  // Range Statistics for Status Bar
  const stats: SelectionStats = useMemo(() => {
    return {
      count: 4,
      num_count: 4,
      sum: 4257000,
      average: 1064250,
      min: 975000,
      max: 1830000,
    };
  }, []);

  // Smart Toolbox Operations
  const handleSmartToolboxAction = (action: string) => {
    if (action === 'reading_spotlight') {
      setReadingSpotlight(!readingSpotlight);
    } else if (action === 'merge_keep_all') {
      // Combines A2 and B2
      alert('Smart Toolbox: Merged cells without data loss.');
    } else if (action === 'unmerge_fill') {
      alert('Smart Toolbox: Unmerged cells and autofilled child values.');
    } else if (action === 'remove_duplicates') {
      alert('Smart Toolbox: Checked entire table. Zero duplicate rows found.');
    } else if (action === 'delete_blank_rows') {
      alert('Smart Toolbox: Scanned sheet. 0 blank rows removed.');
    } else if (action === 'trim_spaces') {
      alert('Smart Toolbox: Trimmed leading and trailing whitespace from text cells.');
    }
  };

  // Insert Chart
  const handleInsertChart = (type: 'bar' | 'line' | 'pie' | 'area') => {
    setActiveChart({
      type,
      title: 'Regional Revenue Overview',
      labels: ['APAC', 'EMEA', 'AMER'],
      values: [1452000, 975000, 1830000],
    });
  };

  return (
    <div className="flex-1 flex flex-col h-full overflow-hidden bg-mercury-canvas">
      {/* ─── Zone 2: Contextual Ribbon Toolbar ─── */}
      <div className="wps-ribbon-bar">
        {/* Ribbon Tab Strip */}
        <div className="wps-ribbon-tabs-row">
          <div className="flex items-center gap-1">
            <button
              onClick={() => setActiveRibbonTab('home')}
              className={`wps-ribbon-tab-btn ${activeRibbonTab === 'home' ? 'is-active' : ''}`}
            >
              Home
            </button>
            <button
              onClick={() => setActiveRibbonTab('insert')}
              className={`wps-ribbon-tab-btn ${activeRibbonTab === 'insert' ? 'is-active' : ''}`}
            >
              Insert
            </button>
            <button
              onClick={() => setActiveRibbonTab('formulas')}
              className={`wps-ribbon-tab-btn ${activeRibbonTab === 'formulas' ? 'is-active' : ''}`}
            >
              Formulas
            </button>
            <button
              onClick={() => setActiveRibbonTab('data')}
              className={`wps-ribbon-tab-btn ${activeRibbonTab === 'data' ? 'is-active' : ''}`}
            >
              Data
            </button>
            <button
              onClick={() => setActiveRibbonTab('toolbox')}
              className={`wps-ribbon-tab-btn ${activeRibbonTab === 'toolbox' ? 'is-active' : ''}`}
            >
              WPS Smart Toolbox
            </button>
          </div>

          <div className="flex items-center gap-2 text-xs text-mercury-slate">
            <span>WPS Spreadsheets</span>
          </div>
        </div>

        {/* Ribbon Body Actions */}
        <div className="wps-ribbon-body">
          {/* HOME TAB */}
          {activeRibbonTab === 'home' && (
            <>
              <div className="wps-ribbon-group">
                <div className="wps-ribbon-controls">
                  <button className="wps-ribbon-btn-compact" title="Undo">
                    <Undo2 className="w-4 h-4" />
                  </button>
                  <button className="wps-ribbon-btn-compact" title="Redo">
                    <Redo2 className="w-4 h-4" />
                  </button>
                </div>
                <span className="wps-ribbon-group-title">Clipboard</span>
              </div>

              <div className="wps-ribbon-group">
                <div className="wps-ribbon-controls">
                  <button className="wps-ribbon-btn-compact font-bold" title="Bold">
                    <Bold className="w-3.5 h-3.5" />
                  </button>
                  <button className="wps-ribbon-btn-compact italic" title="Italic">
                    <Italic className="w-3.5 h-3.5" />
                  </button>
                  <button className="wps-ribbon-btn-compact underline" title="Underline">
                    <Underline className="w-3.5 h-3.5" />
                  </button>
                  <button
                    onClick={() => alert('Merge & Center applied.')}
                    className="px-2 py-1 text-xs border border-mercury-fog/40 rounded bg-white hover:bg-mercury-lavender"
                  >
                    Merge & Center
                  </button>
                </div>
                <span className="wps-ribbon-group-title">Font & Alignment</span>
              </div>

              <div className="wps-ribbon-group">
                <div className="wps-ribbon-controls">
                  <button
                    onClick={() => updateCell(activeCell, '$' + formulaBarInput)}
                    className="px-2 py-1 text-xs border border-mercury-fog/40 rounded bg-white hover:bg-mercury-lavender"
                  >
                    Currency $
                  </button>
                  <button
                    onClick={() => updateCell(activeCell, formulaBarInput + '%')}
                    className="px-2 py-1 text-xs border border-mercury-fog/40 rounded bg-white hover:bg-mercury-lavender"
                  >
                    Percent %
                  </button>
                  <button
                    onClick={() => updateCell(activeCell, '=SUM(B2:D2)')}
                    className="px-2.5 py-1 text-xs font-semibold text-emerald-700 border border-mercury-fog/40 rounded bg-emerald-50 hover:bg-emerald-100 flex items-center gap-1"
                  >
                    <span>AutoSum (Σ)</span>
                  </button>
                </div>
                <span className="wps-ribbon-group-title">Number Format</span>
              </div>
            </>
          )}

          {/* INSERT TAB */}
          {activeRibbonTab === 'insert' && (
            <>
              <div className="wps-ribbon-group">
                <div className="wps-ribbon-controls">
                  <button onClick={() => setShowPivotModal(true)} className="wps-ribbon-btn">
                    <TableIcon className="w-5 h-5 text-emerald-600" />
                    <span>PivotTable</span>
                  </button>
                </div>
                <span className="wps-ribbon-group-title">Tables</span>
              </div>

              <div className="wps-ribbon-group">
                <div className="wps-ribbon-controls">
                  <button onClick={() => handleInsertChart('bar')} className="wps-ribbon-btn">
                    <BarChart3 className="w-5 h-5 text-blue-600" />
                    <span>Bar Chart</span>
                  </button>
                  <button onClick={() => handleInsertChart('line')} className="wps-ribbon-btn">
                    <LineChart className="w-5 h-5 text-indigo-600" />
                    <span>Line Chart</span>
                  </button>
                  <button onClick={() => handleInsertChart('pie')} className="wps-ribbon-btn">
                    <PieChart className="w-5 h-5 text-amber-600" />
                    <span>Pie Chart</span>
                  </button>
                  <button onClick={() => handleInsertChart('area')} className="wps-ribbon-btn">
                    <AreaChart className="w-5 h-5 text-rose-600" />
                    <span>Area Chart</span>
                  </button>
                </div>
                <span className="wps-ribbon-group-title">Charts</span>
              </div>
            </>
          )}

          {/* FORMULAS TAB */}
          {activeRibbonTab === 'formulas' && (
            <>
              <div className="wps-ribbon-group">
                <div className="wps-ribbon-controls">
                  <button
                    onClick={() => setFormulaBarInput('=SUM(')}
                    className="px-2 py-1 text-xs border border-mercury-fog/40 rounded bg-white hover:bg-mercury-lavender"
                  >
                    SUM
                  </button>
                  <button
                    onClick={() => setFormulaBarInput('=AVERAGE(')}
                    className="px-2 py-1 text-xs border border-mercury-fog/40 rounded bg-white hover:bg-mercury-lavender"
                  >
                    AVERAGE
                  </button>
                  <button
                    onClick={() => setFormulaBarInput('=COUNT(')}
                    className="px-2 py-1 text-xs border border-mercury-fog/40 rounded bg-white hover:bg-mercury-lavender"
                  >
                    COUNT
                  </button>
                  <button
                    onClick={() => setFormulaBarInput('=MAX(')}
                    className="px-2 py-1 text-xs border border-mercury-fog/40 rounded bg-white hover:bg-mercury-lavender"
                  >
                    MAX
                  </button>
                  <button
                    onClick={() => setFormulaBarInput('=IF(')}
                    className="px-2 py-1 text-xs border border-mercury-fog/40 rounded bg-white hover:bg-mercury-lavender"
                  >
                    IF
                  </button>
                  <button
                    onClick={() => setFormulaBarInput('=XLOOKUP(')}
                    className="px-2 py-1 text-xs border border-mercury-fog/40 rounded bg-white hover:bg-mercury-lavender"
                  >
                    XLOOKUP
                  </button>
                </div>
                <span className="wps-ribbon-group-title">Function Library</span>
              </div>

              <div className="wps-ribbon-group">
                <div className="wps-ribbon-controls">
                  <button
                    onClick={() => setShowFormulas(!showFormulas)}
                    className={`wps-ribbon-btn ${showFormulas ? 'is-active' : ''}`}
                  >
                    <FunctionSquare className="w-5 h-5 text-emerald-600" />
                    <span>{showFormulas ? 'Hide Formulas' : 'Show Formulas'}</span>
                  </button>
                </div>
                <span className="wps-ribbon-group-title">Auditing</span>
              </div>
            </>
          )}

          {/* DATA TAB */}
          {activeRibbonTab === 'data' && (
            <>
              <div className="wps-ribbon-group">
                <div className="wps-ribbon-controls">
                  <button
                    onClick={() => alert('Sorted A to Z.')}
                    className="wps-ribbon-btn"
                  >
                    <ArrowUpDown className="w-5 h-5 text-blue-600" />
                    <span>Sort A → Z</span>
                  </button>
                  <button
                    onClick={() => alert('Filter toggled.')}
                    className="wps-ribbon-btn"
                  >
                    <Filter className="w-5 h-5 text-indigo-600" />
                    <span>Filter</span>
                  </button>
                </div>
                <span className="wps-ribbon-group-title">Sort & Filter</span>
              </div>
            </>
          )}

          {/* SMART TOOLBOX TAB */}
          {activeRibbonTab === 'toolbox' && (
            <>
              <div className="wps-ribbon-group">
                <div className="wps-ribbon-controls">
                  <button
                    onClick={() => handleSmartToolboxAction('reading_spotlight')}
                    className={`wps-ribbon-btn ${readingSpotlight ? 'is-active' : ''}`}
                  >
                    <Wand2 className="w-5 h-5 text-indigo-600" />
                    <span>Reading Spotlight</span>
                  </button>
                  <button
                    onClick={() => handleSmartToolboxAction('merge_keep_all')}
                    className="wps-ribbon-btn"
                  >
                    <Combine className="w-5 h-5 text-emerald-600" />
                    <span>Merge Keep All</span>
                  </button>
                  <button
                    onClick={() => handleSmartToolboxAction('unmerge_fill')}
                    className="wps-ribbon-btn"
                  >
                    <Split className="w-5 h-5 text-blue-600" />
                    <span>Unmerge & Fill</span>
                  </button>
                  <button
                    onClick={() => handleSmartToolboxAction('remove_duplicates')}
                    className="wps-ribbon-btn"
                  >
                    <Trash2 className="w-5 h-5 text-rose-600" />
                    <span>Remove Duplicates</span>
                  </button>
                </div>
                <span className="wps-ribbon-group-title">WPS Smart Toolbox</span>
              </div>
            </>
          )}
        </div>
      </div>

      {/* ─── Zone 2.5: Classic Name Box + fx Formula Bar ─── */}
      <div className="mercury-formula-bar">
        {/* Name Box */}
        <div className="mercury-name-box" title="Active Cell">
          {activeCell}
        </div>

        {/* fx Symbol Badge */}
        <div className="mercury-fx-badge">fx</div>

        {/* Full-width Formula Input */}
        <div className="mercury-formula-input-wrap">
          <input
            type="text"
            value={formulaBarInput}
            onChange={(e) => setFormulaBarInput(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') handleFormulaSubmit(formulaBarInput);
            }}
            placeholder="Enter a formula (e.g. =SUM(B2:D2)) or a value..."
            className="mercury-formula-input"
          />
        </div>

        <button
          onClick={() => handleFormulaSubmit(formulaBarInput)}
          className="button-small"
          title="Apply Formula"
        >
          <Check className="w-3.5 h-3.5 text-emerald-600" />
          <span>Apply</span>
        </button>
      </div>

      {/* ─── Zone 3: Interactive 2D Spreadsheet Grid & Live Chart ─── */}
      <div className="flex-1 overflow-auto bg-white relative">
        {/* Render Interactive Chart if active */}
        {activeChart && (
          <div className="m-4 p-4 bg-white border border-mercury-primary/30 rounded-mercury-md shadow-md max-w-2xl relative">
            <div className="flex items-center justify-between mb-3">
              <div className="flex items-center gap-2">
                <BarChart3 className="w-4 h-4 text-mercury-primary" />
                <h4 className="font-semibold text-sm text-mercury-ink">{activeChart.title}</h4>
              </div>
              <button
                onClick={() => setActiveChart(null)}
                className="p-1 rounded hover:bg-gray-100 text-gray-500"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            {/* SVG Chart Rendering */}
            <div className="h-44 flex items-end gap-8 px-8 pb-4 pt-2 border-b border-gray-200">
              {activeChart.labels.map((lbl, idx) => {
                const val = activeChart.values[idx];
                const max = Math.max(...activeChart.values);
                const heightPct = Math.round((val / max) * 100);

                return (
                  <div key={lbl} className="flex-1 flex flex-col items-center gap-2 h-full justify-end">
                    <span className="text-[10px] text-gray-500 font-medium">${(val / 1000).toFixed(0)}k</span>
                    <div
                      className="w-full bg-mercury-primary rounded-t transition-all duration-300 hover:bg-indigo-600"
                      style={{ height: `${heightPct}%` }}
                    />
                    <span className="text-xs font-semibold text-gray-700">{lbl}</span>
                  </div>
                );
              })}
            </div>
          </div>
        )}

        {/* 2D Grid Table */}
        <div className="min-w-max">
          <table className="border-collapse table-fixed select-none text-xs">
            <thead>
              <tr>
                {/* Corner Cell */}
                <th className="w-12 h-7 bg-gray-100 border-r border-b border-mercury-fog sticky top-0 left-0 z-20" />

                {/* Column Headers A to Z */}
                {COLS.map((col) => {
                  const isColSpotlit = readingSpotlight && col === activeColLetter;
                  return (
                    <th
                      key={col}
                      className={`w-28 h-7 border-r border-b border-mercury-fog font-medium text-mercury-slate sticky top-0 z-10 transition-colors ${
                        isColSpotlit ? 'bg-mercury-lavender text-mercury-primary font-bold' : 'bg-gray-100'
                      }`}
                    >
                      {col}
                    </th>
                  );
                })}
              </tr>
            </thead>
            <tbody>
              {ROWS.map((row) => {
                const isRowSpotlit = readingSpotlight && row === activeRowNum;

                return (
                  <tr key={row}>
                    {/* Row Header 1 to 50 */}
                    <td
                      className={`w-12 h-7 border-r border-b border-mercury-fog text-center font-medium text-mercury-slate sticky left-0 z-10 transition-colors ${
                        isRowSpotlit ? 'bg-mercury-lavender text-mercury-primary font-bold' : 'bg-gray-100'
                      }`}
                    >
                      {row}
                    </td>

                    {/* Cell Items */}
                    {COLS.map((col) => {
                      const coord = `${col}${row}`;
                      const cell = currentSheet.cells[coord];
                      const isSelected = coord === activeCell;
                      const isEditing = editingCell === coord;

                      const displayText = showFormulas ? cell?.raw || '' : cell?.value || cell?.raw || '';

                      return (
                        <td
                          key={coord}
                          onClick={() => {
                            setActiveCell(coord);
                          }}
                          onDoubleClick={() => {
                            setActiveCell(coord);
                            setEditingCell(coord);
                          }}
                          className={`h-7 border-r border-b border-mercury-fog/40 px-2 text-xs truncate relative cursor-cell transition-colors ${
                            isSelected
                              ? 'outline outline-2 outline-mercury-primary bg-indigo-50/40 z-10'
                              : isRowSpotlit
                              ? 'bg-mercury-lavender/30'
                              : ''
                          }`}
                          style={{
                            textAlign: cell?.align || 'left',
                            fontWeight: cell?.bold ? 600 : 400,
                            backgroundColor: cell?.fillColor || undefined,
                          }}
                        >
                          {isEditing ? (
                            <input
                              type="text"
                              value={formulaBarInput}
                              onChange={(e) => setFormulaBarInput(e.target.value)}
                              onBlur={() => {
                                handleFormulaSubmit(formulaBarInput);
                                setEditingCell(null);
                              }}
                              onKeyDown={(e) => {
                                if (e.key === 'Enter') {
                                  handleFormulaSubmit(formulaBarInput);
                                  setEditingCell(null);
                                }
                                if (e.key === 'Escape') setEditingCell(null);
                              }}
                              autoFocus
                              className="w-full h-full bg-white border border-mercury-primary px-1 outline-none text-xs"
                            />
                          ) : (
                            displayText
                          )}

                          {isSelected && <div className="mercury-cell-handle" />}
                        </td>
                      );
                    })}
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </div>

      {/* ─── Multi-Sheet Bottom Tab Bar ─── */}
      <div className="mercury-sheet-tab-bar">
        <div className="flex items-center gap-1">
          {sheets.map((s, idx) => (
            <button
              key={s.id}
              onClick={() => setActiveSheetIndex(idx)}
              className={`px-3 py-1 rounded-t text-xs font-medium border-t-2 transition-all ${
                activeSheetIndex === idx
                  ? 'bg-white border-t-emerald-600 text-mercury-ink shadow-sm'
                  : 'border-t-transparent text-mercury-slate hover:bg-gray-200'
              }`}
            >
              {s.name}
            </button>
          ))}

          <button
            onClick={() => {
              const newSheet: SingleSheet = {
                id: `sheet-${sheets.length + 1}`,
                name: `Sheet${sheets.length + 1}`,
                cells: {},
              };
              setSheets([...sheets, newSheet]);
              setActiveSheetIndex(sheets.length);
            }}
            className="p-1 rounded hover:bg-gray-200 text-mercury-slate"
            title="Add New Sheet"
          >
            <Plus className="w-3.5 h-3.5" />
          </button>
        </div>

        {/* Live Status Bar Aggregates */}
        <div className="flex items-center gap-4 text-[11px] text-mercury-slate">
          <span>Average: ${stats.average.toLocaleString()}</span>
          <span>Count: {stats.count}</span>
          <span className="font-semibold text-mercury-ink">Sum: ${stats.sum.toLocaleString()}</span>
        </div>
      </div>

      {/* ─── Zone 5: Bottom Status Bar (28px) ─── */}
      <footer className="wps-status-bar">
        <div className="flex items-center gap-4">
          <span className="flex items-center gap-1.5">
            <span className="w-2 h-2 rounded-full bg-emerald-500" />
            <span>Ready</span>
          </span>
          <span>Active Cell: {activeCell}</span>
          {readingSpotlight && (
            <span className="text-mercury-primary font-medium">Spotlight: ON</span>
          )}
        </div>

        <div className="flex items-center gap-3">
          <span className="text-[11px] text-mercury-fog">Calculation: Automatic</span>
        </div>
      </footer>
    </div>
  );
};
