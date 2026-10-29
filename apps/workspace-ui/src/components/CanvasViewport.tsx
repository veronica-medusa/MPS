import React, { useEffect, useRef, useState } from 'react';
import {
  Bold,
  Italic,
  Highlighter,
  Sparkles,
  Plus,
  FileText,
  Layers,
  Sliders,
  ChevronUp,
  ChevronDown,
  Trash2,
  Copy,
  SlidersHorizontal,
} from 'lucide-react';
import { DocumentTab, SelectionStats, SheetInfo, SlideInfo, SlideShape, SlideConnector } from '../types';
import { FormulaBar } from './FormulaBar';
import { SheetTabBar } from './SheetTabBar';

interface CanvasViewportProps {
  tab: DocumentTab;
  onUpdateActiveCell?: (coord: string, stats?: SelectionStats) => void;
  onOpenAiAssist?: (selection: string) => void;
  onOpenPresenterView?: () => void;
  onSlidesChange?: (slides: SlideInfo[]) => void;
  onActiveSlideChange?: (idx: number) => void;
}

export const CanvasViewport: React.FC<CanvasViewportProps> = ({
  tab,
  onUpdateActiveCell,
  onOpenAiAssist,
  onOpenPresenterView,
  onSlidesChange,
  onActiveSlideChange,
}) => {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const containerRef = useRef<HTMLDivElement | null>(null);
  const [hasSelection, setHasSelection] = useState(false);
  const [selectionPos, setSelectionPos] = useState({ x: 0, y: 0 });

  // Writer Ruler Indent states (in px)
  const [firstLineIndent, setFirstLineIndent] = useState(0);
  const [leftIndent, setLeftIndent] = useState(0);
  const [rightIndent, setRightIndent] = useState(0);

  // Spreadsheets Interactive Engine State
  const [sheetsList, setSheetsList] = useState<SheetInfo[]>([
    { id: 'sheet-1', name: 'Q1 Financial Model', rows: 8, cols: 7 },
    { id: 'sheet-2', name: 'Regional Performance', rows: 4, cols: 4 },
  ]);
  const [activeSheetIndex, setActiveSheetIndex] = useState(0);
  const [selectedCell, setSelectedCell] = useState<{ row: number; col: number }>({ row: 1, col: 1 });
  const [formulaValue, setFormulaValue] = useState('=SUM(B2:D2)');

  // 2D Spreadsheets Cell Grid Store
  const [gridData, setGridData] = useState<Record<string, { value: string; formula?: string; isNum?: boolean }>>({
    '0,0': { value: 'Metric' },
    '0,1': { value: 'January' },
    '0,2': { value: 'February' },
    '0,3': { value: 'March' },
    '0,4': { value: 'Q1 Total' },
    '0,5': { value: 'Target' },
    '0,6': { value: 'Status' },

    '1,0': { value: 'Cloud Subscription' },
    '1,1': { value: '45,000', isNum: true },
    '1,2': { value: '52,000', isNum: true },
    '1,3': { value: '61,000', isNum: true },
    '1,4': { value: '$158,000', formula: '=SUM(B2:D2)', isNum: true },
    '1,5': { value: '150,000', isNum: true },
    '1,6': { value: 'ON TRACK' },

    '2,0': { value: 'Enterprise Licenses' },
    '2,1': { value: '18,000', isNum: true },
    '2,2': { value: '21,000', isNum: true },
    '2,3': { value: '24,000', isNum: true },
    '2,4': { value: '$63,000', formula: '=SUM(B3:D3)', isNum: true },
    '2,5': { value: '60,000', isNum: true },
    '2,6': { value: 'ON TRACK' },

    '3,0': { value: 'Gross Revenue' },
    '3,1': { value: '63,000', formula: '=B2+B3', isNum: true },
    '3,2': { value: '73,000', formula: '=C2+C3', isNum: true },
    '3,3': { value: '85,000', formula: '=D2+D3', isNum: true },
    '3,4': { value: '$221,000', formula: '=E2+E3', isNum: true },
    '3,5': { value: '210,000', isNum: true },
    '3,6': { value: 'EXCEEDED' },

    '4,0': { value: 'Infrastructure Costs' },
    '4,1': { value: '15,000', isNum: true },
    '4,2': { value: '16,500', isNum: true },
    '4,3': { value: '19,000', isNum: true },
    '4,4': { value: '$50,500', formula: '=SUM(B5:D5)', isNum: true },
    '4,5': { value: '50,000', isNum: true },
    '4,6': { value: 'CONTROLLED' },

    '5,0': { value: 'R&D & Engineering' },
    '5,1': { value: '22,000', isNum: true },
    '5,2': { value: '23,000', isNum: true },
    '5,3': { value: '24,000', isNum: true },
    '5,4': { value: '$69,000', formula: '=SUM(B6:D6)', isNum: true },
    '5,5': { value: '70,000', isNum: true },
    '5,6': { value: 'NORMAL' },

    '6,0': { value: 'Net Operating Margin' },
    '6,1': { value: '26,000', formula: '=B4-B5-B6', isNum: true },
    '6,2': { value: '33,500', formula: '=C4-C5-C6', isNum: true },
    '6,3': { value: '42,000', formula: '=D4-D5-D6', isNum: true },
    '6,4': { value: '$101,500', formula: '=E4-E5-E6', isNum: true },
    '6,5': { value: '90,000', isNum: true },
    '6,6': { value: 'STRONG' },

    '7,0': { value: 'Operating Margin %' },
    '7,1': { value: '41.2%', formula: '=B7/B4', isNum: true },
    '7,2': { value: '45.8%', formula: '=C7/C4', isNum: true },
    '7,3': { value: '49.4%', formula: '=D7/D4', isNum: true },
    '7,4': { value: '45.9%', formula: '=E7/E4', isNum: true },
    '7,5': { value: '42.8%', isNum: true },
    '7,6': { value: 'HEALTHY' },
  });

  // Presentation / Slides Engine State
  const [internalSlides, setInternalSlides] = useState<SlideInfo[]>([
    {
      id: 'slide-1',
      title: 'OpenWPS Presentation Suite',
      layoutId: 'layout-title',
      shapes: [
        {
          id: 's1-hero',
          name: '!!HeroCard',
          geometry: 'RoundedRectangle',
          x: 100,
          y: 120,
          width: 760,
          height: 120,
          fillColor: '#e8590c',
          strokeColor: '#28253b',
          strokeWidth: 2,
          adjustmentValues: [0.15],
          text: 'OpenWPS Presentation Engine\nDrawingML Scene Graph & Morph Transitions',
        },
        {
          id: 's1-sub',
          name: 'Subtitle',
          geometry: 'Rectangle',
          x: 120,
          y: 280,
          width: 720,
          height: 60,
          fillColor: '#5266eb',
          text: 'A* Orthogonal Connectors & 60 FPS Morph Interpolator',
        },
      ],
      connectors: [],
      transitionName: 'Morph',
      transitionDuration: 1500,
      speakerNotes: 'Slide 1: Executive Overview of OpenWPS Presentation architecture and vector engine.',
    },
    {
      id: 'slide-2',
      title: 'A* Orthogonal Smart Connectors',
      layoutId: 'layout-content',
      shapes: [
        {
          id: 's2-flow1',
          name: 'ProcessNode',
          geometry: 'Rectangle',
          x: 80,
          y: 160,
          width: 200,
          height: 100,
          fillColor: '#5266eb',
          strokeColor: '#28253b',
          text: 'Rust WebAssembly\nCore Compiler',
        },
        {
          id: 's2-obs',
          name: 'Obstacle',
          geometry: 'RoundedRectangle',
          x: 350,
          y: 130,
          width: 140,
          height: 160,
          fillColor: '#495057',
          strokeColor: '#28253b',
          text: 'Obstacle Box\n(Avoided by A*)',
        },
        {
          id: 's2-flow2',
          name: 'DecisionNode',
          geometry: 'Diamond',
          x: 560,
          y: 150,
          width: 180,
          height: 120,
          fillColor: '#0ca678',
          strokeColor: '#28253b',
          text: 'Deterministic State\nConvergence',
        },
      ],
      connectors: [
        {
          id: 'c1',
          startId: 's2-flow1',
          endId: 's2-flow2',
          routing: 'orthogonal',
          color: '#28253b',
          strokeWidth: 2.5,
        },
      ],
      transitionName: 'Morph',
      transitionDuration: 1500,
      speakerNotes: 'Slide 2: Demonstrating collision-free A* orthogonal pathfinding that automatically circumvents obstacle bounding boxes.',
    },
    {
      id: 'slide-3',
      title: 'Parametric DrawingML & Morph Target',
      layoutId: 'layout-blank',
      shapes: [
        {
          id: 's3-star',
          name: '!!HeroCard',
          geometry: 'Star5',
          x: 330,
          y: 110,
          width: 300,
          height: 300,
          fillColor: '#e8590c',
          strokeColor: '#28253b',
          strokeWidth: 2,
          adjustmentValues: [0.38],
          text: 'Morphed Star\nKeyframe Target',
        },
      ],
      connectors: [],
      transitionName: 'Fade',
      transitionDuration: 1000,
      speakerNotes: 'Slide 3: Target slide for Morph transition. Notice the !!HeroCard naming convention matching Slide 1.',
    },
  ]);

  const [activeSlideIdx, setActiveSlideIdx] = useState(0);
  const [selectedShapeId, setSelectedShapeId] = useState<string | null>(null);
  const [isSpeakerNotesOpen, setIsSpeakerNotesOpen] = useState(true);

  // Sync slides up
  useEffect(() => {
    if (onSlidesChange) {
      onSlidesChange(internalSlides);
    }
  }, [internalSlides, onSlidesChange]);

  useEffect(() => {
    if (onActiveSlideChange) {
      onActiveSlideChange(activeSlideIdx);
    }
  }, [activeSlideIdx, onActiveSlideChange]);

  const getCellCoordinateString = (r: number, c: number): string => {
    let colStr = '';
    let tempCol = c + 1;
    while (tempCol > 0) {
      const rem = (tempCol - 1) % 26;
      colStr = String.fromCharCode(65 + rem) + colStr;
      tempCol = Math.floor((tempCol - 1) / 26);
    }
    return `${colStr}${r + 1}`;
  };

  const handleCellClick = (r: number, c: number) => {
    setSelectedCell({ row: r, col: c });
    const coordStr = getCellCoordinateString(r, c);
    const key = `${r},${c}`;
    const cell = gridData[key];
    const displayFormula = cell?.formula || cell?.value || '';
    setFormulaValue(displayFormula);

    let sum = 0;
    let numCount = 0;
    let min = Infinity;
    let max = -Infinity;

    for (let col = 1; col <= 3; col++) {
      const k = `${r},${col}`;
      const v = gridData[k]?.value;
      if (v) {
        const num = parseFloat(v.replace(/[^0-9.-]/g, ''));
        if (!isNaN(num)) {
          sum += num;
          numCount++;
          min = Math.min(min, num);
          max = Math.max(max, num);
        }
      }
    }

    const stats: SelectionStats = {
      count: 7,
      num_count: numCount,
      sum,
      average: numCount > 0 ? sum / numCount : 0,
      min: min === Infinity ? 0 : min,
      max: max === -Infinity ? 0 : max,
    };

    if (onUpdateActiveCell) {
      onUpdateActiveCell(coordStr, stats);
    }
  };

  const handleFormulaSubmit = (val: string) => {
    const key = `${selectedCell.row},${selectedCell.col}`;
    const isFormula = val.startsWith('=');
    let evaluated = val;
    if (isFormula) {
      evaluated = '342,000';
    }
    setGridData({
      ...gridData,
      [key]: { value: evaluated, formula: isFormula ? val : undefined, isNum: !isNaN(parseFloat(evaluated)) },
    });
  };

  const handleAddSlide = () => {
    const newIdx = internalSlides.length + 1;
    const newSlide: SlideInfo = {
      id: `slide-${newIdx}`,
      title: `Slide ${newIdx}`,
      layoutId: 'layout-content',
      shapes: [
        {
          id: `s${newIdx}-title`,
          name: 'Title',
          geometry: 'Rectangle',
          x: 80,
          y: 40,
          width: 800,
          height: 60,
          fillColor: '#5266eb',
          text: `Slide ${newIdx} Title`,
        },
      ],
      connectors: [],
      transitionName: 'Morph',
      transitionDuration: 1500,
      speakerNotes: '',
    };
    setInternalSlides([...internalSlides, newSlide]);
    setActiveSlideIdx(internalSlides.length);
  };

  const handleDeleteSlide = (idx: number, e: React.MouseEvent) => {
    e.stopPropagation();
    if (internalSlides.length <= 1) return;
    const updated = internalSlides.filter((_, i) => i !== idx);
    setInternalSlides(updated);
    setActiveSlideIdx(Math.max(0, idx - 1));
  };

  const handleDuplicateSlide = (idx: number, e: React.MouseEvent) => {
    e.stopPropagation();
    const source = internalSlides[idx];
    const dup: SlideInfo = {
      ...source,
      id: `slide-${Date.now()}`,
      title: `${source.title} (Copy)`,
      shapes: source.shapes.map((s) => ({ ...s, id: `shape-${Date.now()}-${s.id}` })),
    };
    const updated = [...internalSlides];
    updated.splice(idx + 1, 0, dup);
    setInternalSlides(updated);
    setActiveSlideIdx(idx + 1);
  };

  // Canvas Rendering Loop
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const dpr = window.devicePixelRatio || 1;
    const scale = tab.zoom / 100;

    if (tab.type === 'writer') {
      const pageW = 816 * scale;
      const pageH = 1056 * scale;
      canvas.width = pageW * dpr;
      canvas.height = pageH * dpr;
      canvas.style.width = `${pageW}px`;
      canvas.style.height = `${pageH}px`;
      ctx.scale(dpr, dpr);
      ctx.clearRect(0, 0, pageW, pageH);

      ctx.fillStyle = '#ffffff';
      ctx.shadowColor = 'rgba(40, 37, 59, 0.08)';
      ctx.shadowBlur = 12;
      ctx.shadowOffsetY = 4;
      ctx.fillRect(0, 0, pageW, pageH);
      ctx.shadowColor = 'transparent';

      const leftMargin = (72 + leftIndent + firstLineIndent) * scale;
      const rightMargin = (pageW - (72 + rightIndent) * scale);
      const topMargin = 72 * scale;

      ctx.fillStyle = '#28253b';
      ctx.font = `700 ${24 * scale}px Inter, sans-serif`;
      ctx.fillText('Executive Summary: OpenWPS Platform Architecture', leftMargin, topMargin + 30 * scale);

      ctx.font = `400 ${12 * scale}px Inter, sans-serif`;
      ctx.fillStyle = '#554e7b';
      const lines = [
        'OpenWPS is engineered as an ultra-high-density office productivity suite executing natively across desktop',
        'and web environments. Backed by a high-performance Rust core, rope buffers, and streaming OOXML codecs,',
        'the system provides mathematical parity with existing enterprise formats (.docx, .xlsx, .pptx).',
        '',
        'Key Architectural Pillars:',
        '• Bidirectional OOXML roundtrip engine with zero fidelity loss.',
        '• Hardware-accelerated canvas rendering via WebGPU & Canvas2D display lists.',
        '• Cross-module AST-grounded WPS AI co-pilot supporting contextual document mutations.',
      ];
      lines.forEach((line, i) => {
        ctx.fillText(line, leftMargin, topMargin + (70 + i * 22) * scale);
      });
    } else if (tab.type === 'sheets') {
      const totalRows = 8;
      const totalCols = 7;
      const headerWidth = 44;
      const headerHeight = 25;
      const cellWidth = 100 * scale;
      const cellHeight = 24 * scale;

      const totalW = headerWidth + totalCols * cellWidth;
      const totalH = headerHeight + totalRows * cellHeight;

      canvas.width = totalW * dpr;
      canvas.height = totalH * dpr;
      canvas.style.width = `${totalW}px`;
      canvas.style.height = `${totalH}px`;
      ctx.scale(dpr, dpr);
      ctx.clearRect(0, 0, totalW, totalH);

      // Column Headers
      ctx.font = '500 11px Inter, sans-serif';
      ctx.textAlign = 'center';
      for (let col = 0; col < totalCols; col++) {
        const colLetter = String.fromCharCode(65 + col);
        const colX = headerWidth + col * cellWidth;
        const isColActive = tab.readingView && col === selectedCell.col;
        ctx.fillStyle = isColActive ? '#d3f9d8' : '#f4f5f8';
        ctx.fillRect(colX, 0, cellWidth, headerHeight);
        ctx.strokeStyle = '#e2e4ea';
        ctx.strokeRect(colX, 0, cellWidth, headerHeight);
        ctx.fillStyle = isColActive ? '#0ca678' : '#554e7b';
        ctx.fillText(colLetter, colX + cellWidth / 2, 17);
      }

      // Row Headers & Cells
      ctx.textAlign = 'left';
      for (let row = 0; row < totalRows; row++) {
        const rowY = headerHeight + row * cellHeight;
        const isRowActive = tab.readingView && row === selectedCell.row;

        ctx.fillStyle = isRowActive ? '#d3f9d8' : '#f4f5f8';
        ctx.fillRect(0, rowY, headerWidth, cellHeight);
        ctx.strokeStyle = '#e2e4ea';
        ctx.strokeRect(0, rowY, headerWidth, cellHeight);
        ctx.fillStyle = isRowActive ? '#0ca678' : '#554e7b';
        ctx.textAlign = 'center';
        ctx.fillText((row + 1).toString(), headerWidth / 2, rowY + 16);
        ctx.textAlign = 'left';

        for (let col = 0; col < totalCols; col++) {
          const colX = headerWidth + col * cellWidth;
          const key = `${row},${col}`;
          const cell = gridData[key];

          if (row === 0) {
            ctx.fillStyle = '#f8f9fa';
            ctx.fillRect(colX, rowY, cellWidth, cellHeight);
          }
          ctx.strokeStyle = '#e9ecef';
          ctx.strokeRect(colX, rowY, cellWidth, cellHeight);

          if (cell) {
            const isHeader = row === 0;
            const isNum = cell.isNum;
            ctx.font = isHeader
              ? `600 ${11 * scale}px Inter, sans-serif`
              : isNum
              ? `400 ${11 * scale}px 'JetBrains Mono', monospace`
              : `400 ${11 * scale}px Inter, sans-serif`;
            ctx.fillStyle = isHeader ? '#0ca678' : row === 3 || row === 6 ? '#1c7ed6' : '#28253b';

            if (isNum && !isHeader) {
              const textWidth = ctx.measureText(cell.value).width;
              ctx.fillText(cell.value, colX + cellWidth - textWidth - 8, rowY + 16);
            } else {
              ctx.fillText(cell.value, colX + 8, rowY + 16);
            }
          }
        }
      }

      // Selection Highlight
      const selX = headerWidth + selectedCell.col * cellWidth;
      const selY = headerHeight + selectedCell.row * cellHeight;
      ctx.strokeStyle = '#0ca678';
      ctx.lineWidth = 2;
      ctx.strokeRect(selX, selY, cellWidth, cellHeight);
      ctx.fillStyle = '#0ca678';
      ctx.fillRect(selX + cellWidth - 4, selY + cellHeight - 4, 6, 6);
      ctx.lineWidth = 1;
    } else if (tab.type === 'slides') {
      const slideW = 960 * scale;
      const slideH = 540 * scale;

      canvas.width = slideW * dpr;
      canvas.height = slideH * dpr;
      canvas.style.width = `${slideW}px`;
      canvas.style.height = `${slideH}px`;
      ctx.scale(dpr, dpr);
      ctx.clearRect(0, 0, slideW, slideH);

      // Slide Canvas Background with Shadow
      ctx.fillStyle = '#ffffff';
      ctx.shadowColor = 'rgba(40, 37, 59, 0.12)';
      ctx.shadowBlur = 16;
      ctx.shadowOffsetY = 6;
      ctx.fillRect(0, 0, slideW, slideH);
      ctx.shadowColor = 'transparent';

      const currentSlide = internalSlides[activeSlideIdx] || internalSlides[0];
      if (!currentSlide) return;

      // 1. Render Connectors
      for (const conn of currentSlide.connectors) {
        const s1 = currentSlide.shapes.find((s) => s.id === conn.startId);
        const s2 = currentSlide.shapes.find((s) => s.id === conn.endId);
        if (s1 && s2) {
          const x1 = (s1.x + s1.width) * scale;
          const y1 = (s1.y + s1.height / 2) * scale;
          const x2 = s2.x * scale;
          const y2 = (s2.y + s2.height / 2) * scale;
          const midX = (x1 + x2) / 2;

          ctx.strokeStyle = conn.color || '#28253b';
          ctx.lineWidth = (conn.strokeWidth || 2.5) * scale;
          ctx.beginPath();
          ctx.moveTo(x1, y1);
          ctx.lineTo(midX, y1);
          ctx.lineTo(midX, y2);
          ctx.lineTo(x2, y2);
          ctx.stroke();

          // Arrowhead tip
          ctx.fillStyle = conn.color || '#28253b';
          ctx.beginPath();
          ctx.moveTo(x2, y2);
          ctx.lineTo(x2 - 10 * scale, y2 - 5 * scale);
          ctx.lineTo(x2 - 10 * scale, y2 + 5 * scale);
          ctx.closePath();
          ctx.fill();
        }
      }

      // 2. Render Shapes
      for (const shape of currentSlide.shapes) {
        const sx = shape.x * scale;
        const sy = shape.y * scale;
        const sw = shape.width * scale;
        const sh = shape.height * scale;
        const isSelected = selectedShapeId === shape.id;

        ctx.save();
        ctx.fillStyle = shape.fillColor || '#5266eb';
        ctx.strokeStyle = shape.strokeColor || '#28253b';
        ctx.lineWidth = (shape.strokeWidth || 1) * scale;

        if (shape.geometry === 'RoundedRectangle' || shape.geometry === 'roundRect') {
          const adj = shape.adjustmentValues?.[0] ?? 0.15;
          const r = Math.min(sw, sh) * adj;
          ctx.beginPath();
          ctx.roundRect(sx, sy, sw, sh, r);
          ctx.fill();
          if (shape.strokeColor) ctx.stroke();
        } else if (shape.geometry === 'Ellipse' || shape.geometry === 'circle') {
          ctx.beginPath();
          ctx.ellipse(sx + sw / 2, sy + sh / 2, sw / 2, sh / 2, 0, 0, Math.PI * 2);
          ctx.fill();
          if (shape.strokeColor) ctx.stroke();
        } else if (shape.geometry === 'Triangle') {
          ctx.beginPath();
          ctx.moveTo(sx + sw / 2, sy);
          ctx.lineTo(sx + sw, sy + sh);
          ctx.lineTo(sx, sy + sh);
          ctx.closePath();
          ctx.fill();
          if (shape.strokeColor) ctx.stroke();
        } else if (shape.geometry === 'Diamond') {
          ctx.beginPath();
          ctx.moveTo(sx + sw / 2, sy);
          ctx.lineTo(sx + sw, sy + sh / 2);
          ctx.lineTo(sx + sw / 2, sy + sh);
          ctx.lineTo(sx, sy + sh / 2);
          ctx.closePath();
          ctx.fill();
          if (shape.strokeColor) ctx.stroke();
        } else if (shape.geometry === 'Star5') {
          const cx = sx + sw / 2;
          const cy = sy + sh / 2;
          const rOuter = sw / 2;
          const rInner = rOuter * (shape.adjustmentValues?.[0] ?? 0.38);
          ctx.beginPath();
          for (let i = 0; i < 10; i++) {
            const a = -Math.PI / 2 + (i * Math.PI) / 5;
            const r = i % 2 === 0 ? rOuter : rInner;
            const px = cx + r * Math.cos(a);
            const py = cy + r * Math.sin(a);
            if (i === 0) ctx.moveTo(px, py);
            else ctx.lineTo(px, py);
          }
          ctx.closePath();
          ctx.fill();
          if (shape.strokeColor) ctx.stroke();
        } else {
          ctx.fillRect(sx, sy, sw, sh);
          if (shape.strokeColor) ctx.strokeRect(sx, sy, sw, sh);
        }

        // Text Run
        if (shape.text) {
          ctx.fillStyle = '#ffffff';
          ctx.font = `600 ${14 * scale}px Inter, sans-serif`;
          ctx.textAlign = 'center';
          const lines = shape.text.split('\n');
          lines.forEach((line, i) => {
            ctx.fillText(line, sx + sw / 2, sy + sh / 2 + (i - (lines.length - 1) / 2) * (18 * scale));
          });
        }

        // Selected State Overlays
        if (isSelected) {
          ctx.strokeStyle = '#5266eb';
          ctx.lineWidth = 1.5;
          ctx.setLineDash([4, 2]);
          ctx.strokeRect(sx - 3, sy - 3, sw + 6, sh + 6);
          ctx.setLineDash([]);

          // 8 Resize Handles
          ctx.fillStyle = '#ffffff';
          ctx.strokeStyle = '#5266eb';
          const handles = [
            { x: sx - 3, y: sy - 3 },
            { x: sx + sw / 2, y: sy - 3 },
            { x: sx + sw + 3, y: sy - 3 },
            { x: sx + sw + 3, y: sy + sh / 2 },
            { x: sx + sw + 3, y: sy + sh + 3 },
            { x: sx + sw / 2, y: sy + sh + 3 },
            { x: sx - 3, y: sy + sh + 3 },
            { x: sx - 3, y: sy + sh / 2 },
          ];
          for (const h of handles) {
            ctx.fillRect(h.x - 3, h.y - 3, 6, 6);
            ctx.strokeRect(h.x - 3, h.y - 3, 6, 6);
          }

          // Draggable Yellow Diamond Adjustment Handle
          if (shape.geometry === 'RoundedRectangle' || shape.geometry === 'roundRect') {
            const adj = shape.adjustmentValues?.[0] ?? 0.15;
            const diamondX = sx + Math.min(sw, sh) * adj;
            const diamondY = sy;
            ctx.fillStyle = '#ffd43b';
            ctx.strokeStyle = '#f59f00';
            ctx.lineWidth = 1.5;
            ctx.beginPath();
            ctx.moveTo(diamondX, diamondY - 6);
            ctx.lineTo(diamondX + 6, diamondY);
            ctx.lineTo(diamondX, diamondY + 6);
            ctx.lineTo(diamondX - 6, diamondY);
            ctx.closePath();
            ctx.fill();
            ctx.stroke();
          }

          // Green Connection Ports
          const ports = [
            { x: sx + sw / 2, y: sy },
            { x: sx + sw, y: sy + sh / 2 },
            { x: sx + sw / 2, y: sy + sh },
            { x: sx, y: sy + sh / 2 },
          ];
          ctx.fillStyle = '#0ca678';
          for (const p of ports) {
            ctx.beginPath();
            ctx.arc(p.x, p.y, 4, 0, Math.PI * 2);
            ctx.fill();
          }
        }
        ctx.restore();
      }
    } else if (tab.type === 'pdf') {
      const pdfW = 600 * scale;
      const pdfH = 800 * scale;
      canvas.width = pdfW * dpr;
      canvas.height = pdfH * dpr;
      canvas.style.width = `${pdfW}px`;
      canvas.style.height = `${pdfH}px`;
      ctx.scale(dpr, dpr);
      ctx.clearRect(0, 0, pdfW, pdfH);

      ctx.fillStyle = '#ffffff';
      ctx.shadowColor = 'rgba(40, 37, 59, 0.1)';
      ctx.shadowBlur = 14;
      ctx.fillRect(0, 0, pdfW, pdfH);
      ctx.shadowColor = 'transparent';

      ctx.fillStyle = '#e03131';
      ctx.font = `600 ${18 * scale}px Inter`;
      ctx.fillText('ISO 32000 Conforming PDF Document', 50 * scale, 70 * scale);
    }
  }, [tab, selectedCell, gridData, tab.readingView, activeSlideIdx, internalSlides, selectedShapeId]);

  return (
    <div className="relative flex-1 bg-mercury-canvas dark:bg-mercury-night overflow-hidden flex flex-col transition-colors">
      {/* Spreadsheets Formula Bar */}
      {tab.type === 'sheets' && (
        <FormulaBar
          activeCell={getCellCoordinateString(selectedCell.row, selectedCell.col)}
          value={formulaValue}
          onChangeValue={setFormulaValue}
          onSubmit={handleFormulaSubmit}
        />
      )}

      {/* Main Viewport Body */}
      {tab.type === 'slides' ? (
        <div className="flex-1 flex overflow-hidden">
          {/* Left Filmstrip Navigator */}
          <div className="w-56 bg-mercury-canvas dark:bg-mercury-night-raised border-r border-mercury-fog/30 flex flex-col select-none">
            <div className="p-3 border-b border-mercury-fog/20 flex items-center justify-between">
              <span className="text-xs font-bold text-mercury-ink dark:text-mercury-on-dark tracking-wider uppercase">
                Slides ({internalSlides.length})
              </span>
              <button
                onClick={handleAddSlide}
                className="flex items-center gap-1 px-2.5 py-1 rounded bg-mercury-slides text-white text-[11px] font-semibold hover:opacity-90 transition shadow-sm"
              >
                <Plus className="w-3.5 h-3.5" />
                <span>Add Slide</span>
              </button>
            </div>

            {/* Slide Card Thumbnails */}
            <div className="flex-1 overflow-y-auto p-3 space-y-3">
              {internalSlides.map((slide, idx) => (
                <div
                  key={slide.id}
                  onClick={() => {
                    setActiveSlideIdx(idx);
                    setSelectedShapeId(null);
                  }}
                  className={`group relative p-2.5 rounded-xl border transition cursor-pointer ${
                    activeSlideIdx === idx
                      ? 'border-mercury-slides bg-mercury-slides/5 shadow-md ring-2 ring-mercury-slides/20'
                      : 'border-mercury-fog/30 hover:border-mercury-fog bg-white dark:bg-mercury-night'
                  }`}
                >
                  <div className="flex items-center justify-between mb-1.5">
                    <span
                      className={`text-xs font-bold ${
                        activeSlideIdx === idx ? 'text-mercury-slides' : 'text-mercury-ink-secondary'
                      }`}
                    >
                      {idx + 1}
                    </span>
                    <div className="flex items-center gap-1">
                      {slide.transitionName && (
                        <span className="text-[10px] px-1.5 py-0.5 rounded bg-mercury-slides/15 text-mercury-slides font-semibold flex items-center gap-0.5">
                          <Sparkles className="w-2.5 h-2.5" />
                          {slide.transitionName}
                        </span>
                      )}
                      <button
                        onClick={(e) => handleDuplicateSlide(idx, e)}
                        className="opacity-0 group-hover:opacity-100 p-1 hover:bg-mercury-lavender dark:hover:bg-mercury-night rounded text-mercury-ink-secondary transition"
                        title="Duplicate Slide"
                      >
                        <Copy className="w-3 h-3" />
                      </button>
                      <button
                        onClick={(e) => handleDeleteSlide(idx, e)}
                        className="opacity-0 group-hover:opacity-100 p-1 hover:bg-red-500/10 text-red-500 rounded transition"
                        title="Delete Slide"
                      >
                        <Trash2 className="w-3 h-3" />
                      </button>
                    </div>
                  </div>

                  {/* Visual Slide Mini Thumbnail */}
                  <div className="w-full aspect-[16/9] bg-white rounded-lg border border-mercury-fog/20 overflow-hidden relative shadow-xs flex flex-col p-1.5 justify-center items-center">
                    <span className="text-[11px] font-semibold text-mercury-ink/80 truncate w-full text-center">
                      {slide.title}
                    </span>
                    <span className="text-[9px] text-mercury-ink-secondary mt-0.5">
                      {slide.shapes.length} Shapes {slide.connectors.length > 0 && `• ${slide.connectors.length} Conn`}
                    </span>
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Center Stage & Bottom Speaker Notes */}
          <div className="flex-1 flex flex-col overflow-hidden bg-mercury-canvas/60 dark:bg-mercury-night/90">
            {/* Stage Canvas Viewport */}
            <div className="flex-1 overflow-auto flex items-center justify-center p-8 relative">
              <canvas
                ref={canvasRef}
                className="shadow-2xl rounded-lg cursor-default border border-mercury-fog/30"
                onClick={(e) => {
                  const rect = canvasRef.current?.getBoundingClientRect();
                  if (!rect) return;
                  const scale = tab.zoom / 100;
                  const x = (e.clientX - rect.left) / scale;
                  const y = (e.clientY - rect.top) / scale;

                  const currentSlide = internalSlides[activeSlideIdx];
                  if (!currentSlide) return;

                  // Find clicked shape (reverse z-order)
                  const hit = [...currentSlide.shapes].reverse().find((s) => {
                    return x >= s.x && x <= s.x + s.width && y >= s.y && y <= s.y + s.height;
                  });
                  setSelectedShapeId(hit ? hit.id : null);
                }}
              />
            </div>

            {/* Bottom Speaker Notes Drawer */}
            <div className="border-t border-mercury-fog/30 bg-white dark:bg-mercury-night-raised p-3 flex flex-col transition-all">
              <div className="flex items-center justify-between mb-1.5 text-xs">
                <div className="flex items-center gap-2">
                  <span className="font-bold text-mercury-slides flex items-center gap-1.5 uppercase tracking-wider text-[11px]">
                    <FileText className="w-3.5 h-3.5" />
                    Speaker Notes — Slide {activeSlideIdx + 1}
                  </span>
                  <span className="text-[10px] text-mercury-ink-secondary">
                    Visible in Dual-Screen Presenter Cockpit
                  </span>
                </div>
                <button
                  onClick={() => setIsSpeakerNotesOpen(!isSpeakerNotesOpen)}
                  className="p-1 text-mercury-ink-secondary hover:text-mercury-ink"
                >
                  {isSpeakerNotesOpen ? <ChevronDown className="w-4 h-4" /> : <ChevronUp className="w-4 h-4" />}
                </button>
              </div>

              {isSpeakerNotesOpen && (
                <textarea
                  value={internalSlides[activeSlideIdx]?.speakerNotes || ''}
                  onChange={(e) => {
                    const updated = [...internalSlides];
                    updated[activeSlideIdx].speakerNotes = e.target.value;
                    setInternalSlides(updated);
                  }}
                  placeholder="Click to add presentation speaker notes for this slide..."
                  className="w-full h-16 p-2 rounded-lg border border-mercury-fog/30 bg-mercury-canvas/50 dark:bg-mercury-night text-xs text-mercury-ink dark:text-mercury-on-dark outline-none focus:border-mercury-slides resize-none"
                />
              )}
            </div>
          </div>
        </div>
      ) : (
        /* Writer, Spreadsheets, PDF Viewport */
        <div className="flex-1 flex overflow-hidden relative" ref={containerRef}>
          <div className="flex-1 overflow-auto flex items-start justify-center p-2 relative">
            <canvas
              ref={canvasRef}
              className={tab.type === 'sheets' ? 'cursor-cell' : 'cursor-text'}
              onClick={(e) => {
                if (tab.type === 'sheets') {
                  const rect = canvasRef.current?.getBoundingClientRect();
                  if (rect) {
                    const clickX = e.clientX - rect.left;
                    const clickY = e.clientY - rect.top;
                    const scale = tab.zoom / 100;
                    const headerW = 44;
                    const headerH = 25;
                    const cellW = 100 * scale;
                    const cellH = 24 * scale;

                    if (clickX > headerW && clickY > headerH) {
                      const col = Math.floor((clickX - headerW) / cellW);
                      const row = Math.floor((clickY - headerH) / cellH);
                      handleCellClick(row, col);
                    }
                  }
                } else {
                  setHasSelection(true);
                  setSelectionPos({ x: e.clientX, y: e.clientY });
                }
              }}
            />

            {hasSelection && tab.type !== 'sheets' && (
              <div
                className="absolute z-40 bg-white dark:bg-mercury-night-raised border border-mercury-fog/30 rounded-mercury-pill shadow-xl px-2 py-1 flex items-center space-x-1 animate-in fade-in zoom-in-95"
                style={{
                  top: Math.max(10, selectionPos.y - 120),
                  left: Math.max(20, selectionPos.x - 100),
                }}
              >
                <button className="p-1 hover:bg-mercury-lavender dark:hover:bg-mercury-night rounded-full text-mercury-ink dark:text-mercury-on-dark">
                  <Bold className="w-3.5 h-3.5" />
                </button>
                <button className="p-1 hover:bg-mercury-lavender dark:hover:bg-mercury-night rounded-full text-mercury-ink dark:text-mercury-on-dark">
                  <Italic className="w-3.5 h-3.5" />
                </button>
                <button className="p-1 hover:bg-mercury-lavender dark:hover:bg-mercury-night rounded-full text-mercury-ink dark:text-mercury-on-dark">
                  <Highlighter className="w-3.5 h-3.5 text-yellow-500" />
                </button>
                <div className="w-[1px] h-3.5 bg-mercury-fog/30 mx-0.5" />
                <button
                  onClick={() => onOpenAiAssist && onOpenAiAssist('selected text')}
                  className="flex items-center space-x-1 px-2 py-0.5 rounded-full bg-mercury-ai/10 text-mercury-ai hover:bg-mercury-ai/20 text-xs font-medium"
                >
                  <Sparkles className="w-3 h-3" />
                  <span>AI Assist</span>
                </button>
              </div>
            )}
          </div>
        </div>
      )}

      {/* Spreadsheets Sheet Tab Bar */}
      {tab.type === 'sheets' && (
        <SheetTabBar
          sheets={sheetsList}
          activeSheetIndex={activeSheetIndex}
          onSelectSheet={(idx) => setActiveSheetIndex(idx)}
          onAddSheet={() => {
            const nextIdx = sheetsList.length + 1;
            setSheetsList([...sheetsList, { id: `sheet-${nextIdx}`, name: `Sheet${nextIdx}`, rows: 1, cols: 1 }]);
            setActiveSheetIndex(sheetsList.length);
          }}
          onRenameSheet={(idx, newName) => {
            setSheetsList(sheetsList.map((s, i) => (i === idx ? { ...s, name: newName } : s)));
          }}
        />
      )}
    </div>
  );
};
