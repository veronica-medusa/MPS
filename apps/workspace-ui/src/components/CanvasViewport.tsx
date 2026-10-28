import React, { useEffect, useRef, useState } from 'react';
import { Bold, Italic, Underline, Highlighter, Sparkles, MessageSquare } from 'lucide-react';
import { DocumentTab } from '../types';

interface CanvasViewportProps {
  tab: DocumentTab;
  onUpdateActiveCell?: (coord: string) => void;
  onOpenAiAssist?: (selection: string) => void;
}

export const CanvasViewport: React.FC<CanvasViewportProps> = ({ tab, onUpdateActiveCell, onOpenAiAssist }) => {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const containerRef = useRef<HTMLDivElement | null>(null);
  const [hasSelection, setHasSelection] = useState(false);
  const [selectionPos, setSelectionPos] = useState({ x: 0, y: 0 });

  // Writer Ruler Indent states (in px)
  const [firstLineIndent, setFirstLineIndent] = useState(0);
  const [leftIndent, setLeftIndent] = useState(0);
  const [rightIndent, setRightIndent] = useState(0);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const dpr = window.devicePixelRatio || 1;
    const scale = tab.zoom / 100;

    if (tab.type === 'writer') {
      // Multi-Page A4 Setup (595 x 842 pt per page)
      const pageWidth = 595 * scale;
      const pageHeight = 842 * scale;
      const pageGap = 32 * scale;
      const totalPages = tab.totalPages || 2;

      const canvasW = Math.max(800, pageWidth + 160);
      const canvasH = (pageHeight * totalPages) + (pageGap * (totalPages + 1));

      canvas.width = canvasW * dpr;
      canvas.height = canvasH * dpr;
      canvas.style.width = `${canvasW}px`;
      canvas.style.height = `${canvasH}px`;
      ctx.scale(dpr, dpr);

      ctx.clearRect(0, 0, canvasW, canvasH);

      const pageX = (canvasW - pageWidth) / 2;

      for (let p = 0; p < totalPages; p++) {
        const pageY = pageGap + p * (pageHeight + pageGap);

        // 1. Paper Drop Shadow & Card Fill
        ctx.fillStyle = '#ffffff';
        ctx.shadowColor = 'rgba(40, 37, 59, 0.08)';
        ctx.shadowBlur = 16 * scale;
        ctx.shadowOffsetY = 6 * scale;
        ctx.fillRect(pageX, pageY, pageWidth, pageHeight);
        ctx.shadowColor = 'transparent';

        // 2. Paper Hairline Border
        ctx.strokeStyle = '#e2e4ea';
        ctx.lineWidth = 1;
        ctx.strokeRect(pageX, pageY, pageWidth, pageHeight);

        // 3. Margin boundaries
        const marginL = 72 * scale; // 1 inch
        const marginR = 72 * scale;
        const marginT = 72 * scale;
        const marginB = 72 * scale;

        // 4. Redline Revision Indicator in Left Margin (Track Changes)
        if (tab.trackChanges || p === 0) {
          ctx.strokeStyle = '#e03131';
          ctx.lineWidth = 2.5;
          ctx.beginPath();
          ctx.moveTo(pageX + 18 * scale, pageY + marginT + 8 * scale);
          ctx.lineTo(pageX + 18 * scale, pageY + marginT + 220 * scale);
          ctx.stroke();
          ctx.lineWidth = 1;
        }

        if (p === 0) {
          // ==================== PAGE 1 ====================
          // Document Title (Heading 1)
          ctx.fillStyle = '#171721';
          ctx.font = `700 ${18 * scale}px Inter, sans-serif`;
          ctx.fillText('Executive Summary & Architecture', pageX + marginL, pageY + marginT + 24 * scale);

          // Subtitle
          ctx.fillStyle = '#554e7b';
          ctx.font = `500 ${11 * scale}px Inter, sans-serif`;
          ctx.fillText('OpenWPS Production Suite • Phase 2 Verified Kernel', pageX + marginL, pageY + marginT + 46 * scale);

          // Paragraph 1
          ctx.fillStyle = '#28253b';
          ctx.font = `400 ${10.5 * scale}px Inter, sans-serif`;
          const p1Lines = [
            'OpenWPS Writer provides zero-stub document processing with a deterministic Rust pagination',
            'engine. Features include UTF-8 Rope buffers, multi-level numbering, tracked revisions, threaded',
            'comments, and ECMA-376 bidirectional DOCX OOXML fidelity.',
          ];
          p1Lines.forEach((line, i) => {
            ctx.fillText(line, pageX + marginL, pageY + marginT + 76 * scale + i * 18 * scale);
          });

          // Heading 2
          ctx.fillStyle = '#3b5bdb'; // Writer Cobalt Accent
          ctx.font = `600 ${13 * scale}px Inter, sans-serif`;
          ctx.fillText('Performance Invariants & Benchmarks', pageX + marginL, pageY + marginT + 154 * scale);

          // Paragraph 2
          ctx.fillStyle = '#28253b';
          ctx.font = `400 ${10 * scale}px Inter, sans-serif`;
          ctx.fillText('Below is the verified performance envelope across core computational sub-systems:', pageX + marginL, pageY + marginT + 176 * scale);

          // Render Table
          const tableX = pageX + marginL;
          const tableY = pageY + marginT + 194 * scale;
          const tableW = pageWidth - marginL - marginR;
          const colW = tableW / 3;
          const rowH = 26 * scale;

          // Header Row
          ctx.fillStyle = '#f1f3f5';
          ctx.fillRect(tableX, tableY, tableW, rowH);

          const headers = ['Subsystem', 'Latency Target', 'Verified Status'];
          ctx.fillStyle = '#28253b';
          ctx.font = `600 ${9.5 * scale}px Inter, sans-serif`;
          headers.forEach((h, ci) => {
            ctx.fillText(h, tableX + ci * colW + 8 * scale, tableY + 17 * scale);
          });

          // Table Rows
          const rowData = [
            ['Rope Buffer Slicing', '< 0.05 ms', 'Deterministic O(log N)'],
            ['Incremental Line Layout', '< 16.6 ms', '60 FPS Verified'],
            ['OOXML DOCX Roundtrip', 'Lossless', 'ECMA-376 Compliant'],
          ];

          rowData.forEach((row, ri) => {
            const ry = tableY + (ri + 1) * rowH;
            ctx.fillStyle = ri % 2 === 1 ? '#fafbfc' : '#ffffff';
            ctx.fillRect(tableX, ry, tableW, rowH);

            ctx.font = `400 ${9.5 * scale}px Inter, sans-serif`;
            ctx.fillStyle = '#28253b';
            ctx.fillText(row[0], tableX + 8 * scale, ry + 17 * scale);
            ctx.fillText(row[1], tableX + colW + 8 * scale, ry + 17 * scale);

            // Emerald status
            ctx.fillStyle = '#0ca678';
            ctx.fillText(row[2], tableX + 2 * colW + 8 * scale, ry + 17 * scale);
          });

          // Grid Lines
          ctx.strokeStyle = '#dee2e6';
          ctx.lineWidth = 1;
          for (let r = 0; r <= 4; r++) {
            ctx.beginPath();
            ctx.moveTo(tableX, tableY + r * rowH);
            ctx.lineTo(tableX + tableW, tableY + r * rowH);
            ctx.stroke();
          }
          for (let c = 0; c <= 3; c++) {
            ctx.beginPath();
            ctx.moveTo(tableX + c * colW, tableY);
            ctx.lineTo(tableX + c * colW, tableY + 4 * rowH);
            ctx.stroke();
          }

          // Footer
          ctx.fillStyle = '#868e96';
          ctx.font = `400 ${9 * scale}px Inter, sans-serif`;
          ctx.textAlign = 'center';
          ctx.fillText(`Page 1 of ${totalPages}`, pageX + pageWidth / 2, pageY + pageHeight - 32 * scale);
          ctx.textAlign = 'left';
        } else {
          // ==================== PAGE 2 ====================
          ctx.fillStyle = '#3b5bdb';
          ctx.font = `600 ${14 * scale}px Inter, sans-serif`;
          ctx.fillText('Section II: Track Changes & Revision Governance', pageX + marginL, pageY + marginT + 24 * scale);

          // Revision Markup demo
          ctx.fillStyle = '#28253b';
          ctx.font = `400 ${10.5 * scale}px Inter, sans-serif`;
          ctx.fillText('All mutations are tracked via ', pageX + marginL, pageY + marginT + 54 * scale);

          const w1 = ctx.measureText('All mutations are tracked via ').width;
          // Deleted text in Red with strike
          ctx.fillStyle = '#e03131';
          ctx.fillText('legacy string buffers', pageX + marginL + w1, pageY + marginT + 54 * scale);
          const w2 = ctx.measureText('legacy string buffers').width;
          ctx.strokeStyle = '#e03131';
          ctx.beginPath();
          ctx.moveTo(pageX + marginL + w1, pageY + marginT + 50 * scale);
          ctx.lineTo(pageX + marginL + w1 + w2, pageY + marginT + 50 * scale);
          ctx.stroke();

          // Inserted text in Green with underline
          ctx.fillStyle = '#0ca678';
          ctx.fillText(' reversible interval trees', pageX + marginL + w1 + w2, pageY + marginT + 54 * scale);
          const w3 = ctx.measureText(' reversible interval trees').width;
          ctx.strokeStyle = '#0ca678';
          ctx.beginPath();
          ctx.moveTo(pageX + marginL + w1 + w2, pageY + marginT + 56 * scale);
          ctx.lineTo(pageX + marginL + w1 + w2 + w3, pageY + marginT + 56 * scale);
          ctx.stroke();

          // Threaded Comment Card in Right Margin
          const commentX = pageX + pageWidth - marginR + 12 * scale;
          const commentY = pageY + marginT + 40 * scale;
          const commentW = 120 * scale;
          const commentH = 68 * scale;

          ctx.fillStyle = '#ffffff';
          ctx.shadowColor = 'rgba(40, 37, 59, 0.1)';
          ctx.shadowBlur = 8 * scale;
          ctx.fillRect(commentX, commentY, commentW, commentH);
          ctx.shadowColor = 'transparent';

          ctx.strokeStyle = '#3b5bdb';
          ctx.lineWidth = 1.5;
          ctx.strokeRect(commentX, commentY, commentW, commentH);

          ctx.fillStyle = '#3b5bdb';
          ctx.font = `600 ${8.5 * scale}px Inter, sans-serif`;
          ctx.fillText('Alice Chen (Lead)', commentX + 8 * scale, commentY + 16 * scale);

          ctx.fillStyle = '#554e7b';
          ctx.font = `400 ${7.5 * scale}px Inter, sans-serif`;
          ctx.fillText('10 mins ago', commentX + 8 * scale, commentY + 28 * scale);

          ctx.fillStyle = '#28253b';
          ctx.font = `400 ${8 * scale}px Inter, sans-serif`;
          ctx.fillText('Verified ECMA-376', commentX + 8 * scale, commentY + 44 * scale);
          ctx.fillText('strict conformance.', commentX + 8 * scale, commentY + 56 * scale);

          // Footer
          ctx.fillStyle = '#868e96';
          ctx.font = `400 ${9 * scale}px Inter, sans-serif`;
          ctx.textAlign = 'center';
          ctx.fillText(`Page 2 of ${totalPages}`, pageX + pageWidth / 2, pageY + pageHeight - 32 * scale);
          ctx.textAlign = 'left';
        }
      }
    } else if (tab.type === 'sheets') {
      const width = canvas.clientWidth || 800;
      const height = canvas.clientHeight || 600;
      canvas.width = width * dpr;
      canvas.height = height * dpr;
      ctx.scale(dpr, dpr);
      ctx.clearRect(0, 0, width, height);

      const cellWidth = 90 * scale;
      const cellHeight = 24 * scale;
      const headerWidth = 44;
      const headerHeight = 24;

      ctx.fillStyle = '#f4f5f8';
      ctx.fillRect(0, 0, headerWidth, headerHeight);

      ctx.font = '500 11px Inter';
      ctx.fillStyle = '#554e7b';
      ctx.textAlign = 'center';
      for (let col = 0; col < 15; col++) {
        const colLetter = String.fromCharCode(65 + col);
        const colX = headerWidth + col * cellWidth;
        ctx.fillStyle = '#f4f5f8';
        ctx.fillRect(colX, 0, cellWidth, headerHeight);
        ctx.strokeStyle = '#e2e4ea';
        ctx.strokeRect(colX, 0, cellWidth, headerHeight);
        ctx.fillStyle = '#554e7b';
        ctx.fillText(colLetter, colX + cellWidth / 2, 16);
      }

      ctx.textAlign = 'left';
      for (let row = 1; row <= 35; row++) {
        const rowY = headerHeight + (row - 1) * cellHeight;
        ctx.fillStyle = '#f4f5f8';
        ctx.fillRect(0, rowY, headerWidth, cellHeight);
        ctx.strokeStyle = '#e2e4ea';
        ctx.strokeRect(0, rowY, headerWidth, cellHeight);
        ctx.fillStyle = '#554e7b';
        ctx.textAlign = 'center';
        ctx.fillText(row.toString(), headerWidth / 2, rowY + 16);
        ctx.textAlign = 'left';

        for (let col = 0; col < 15; col++) {
          const colX = headerWidth + col * cellWidth;
          ctx.strokeStyle = '#f0f1f4';
          ctx.strokeRect(colX, rowY, cellWidth, cellHeight);

          if (row === 1 && col === 0) {
            ctx.fillStyle = '#0ca678';
            ctx.fillText('Region', colX + 8, rowY + 16);
          } else if (row === 1 && col === 1) {
            ctx.fillStyle = '#0ca678';
            ctx.fillText('Q1 Revenue', colX + 8, rowY + 16);
          } else if (row === 2 && col === 0) {
            ctx.fillStyle = '#28253b';
            ctx.fillText('North America', colX + 8, rowY + 16);
          } else if (row === 2 && col === 1) {
            ctx.fillStyle = '#28253b';
            ctx.fillText('$1,420,500', colX + 8, rowY + 16);
          } else if (row === 3 && col === 0) {
            ctx.fillStyle = '#28253b';
            ctx.fillText('EMEA', colX + 8, rowY + 16);
          } else if (row === 3 && col === 1) {
            ctx.fillStyle = '#28253b';
            ctx.fillText('$985,200', colX + 8, rowY + 16);
          }
        }
      }

      const selCol = 1;
      const selRow = 2;
      const selX = headerWidth + selCol * cellWidth;
      const selY = headerHeight + (selRow - 1) * cellHeight;
      ctx.strokeStyle = '#0ca678';
      ctx.lineWidth = 2;
      ctx.strokeRect(selX, selY, cellWidth, cellHeight);
      ctx.fillStyle = '#0ca678';
      ctx.fillRect(selX + cellWidth - 5, selY + cellHeight - 5, 5, 5);
      ctx.lineWidth = 1;
    } else if (tab.type === 'slides') {
      const width = canvas.clientWidth || 800;
      const height = canvas.clientHeight || 600;
      canvas.width = width * dpr;
      canvas.height = height * dpr;
      ctx.scale(dpr, dpr);
      ctx.clearRect(0, 0, width, height);

      const slideW = 760 * scale;
      const slideH = 427.5 * scale;
      const slideX = Math.max(40, (width - slideW) / 2);
      const slideY = 40;

      ctx.fillStyle = '#ffffff';
      ctx.shadowColor = 'rgba(40, 37, 59, 0.12)';
      ctx.shadowBlur = 16;
      ctx.shadowOffsetY = 6;
      ctx.fillRect(slideX, slideY, slideW, slideH);
      ctx.shadowColor = 'transparent';

      ctx.fillStyle = '#e8590c';
      ctx.font = `700 ${28 * scale}px Inter`;
      ctx.fillText('WPS Presentation Engine', slideX + 60, slideY + 100);

      ctx.fillStyle = '#554e7b';
      ctx.font = `400 ${14 * scale}px Inter`;
      ctx.fillText('3-Tier Master Slide Inheritance & DrawingML Connectors', slideX + 60, slideY + 140);
    } else if (tab.type === 'pdf') {
      const width = canvas.clientWidth || 800;
      const height = canvas.clientHeight || 600;
      canvas.width = width * dpr;
      canvas.height = height * dpr;
      ctx.scale(dpr, dpr);
      ctx.clearRect(0, 0, width, height);

      const pdfW = 600 * scale;
      const pdfH = 800 * scale;
      const pdfX = Math.max(40, (width - pdfW) / 2);
      const pdfY = 40;

      ctx.fillStyle = '#ffffff';
      ctx.shadowColor = 'rgba(40, 37, 59, 0.1)';
      ctx.shadowBlur = 14;
      ctx.fillRect(pdfX, pdfY, pdfW, pdfH);
      ctx.shadowColor = 'transparent';

      ctx.fillStyle = '#e03131';
      ctx.font = `600 ${18 * scale}px Inter`;
      ctx.fillText('ISO 32000 Conforming PDF Document', pdfX + 50, pdfY + 70);

      ctx.strokeStyle = 'rgba(224, 49, 49, 0.3)';
      ctx.setLineDash([4, 2]);
      ctx.strokeRect(pdfX + 45, pdfY + 95, pdfW - 90, 80);
      ctx.setLineDash([]);

      ctx.fillStyle = '#28253b';
      ctx.font = `400 ${12 * scale}px Inter`;
      ctx.fillText('[Editable Paragraph Box Reconstructed via DBSCAN]', pdfX + 50, pdfY + 120);
      ctx.fillText('Click to edit text directly inline with font-matching reflow.', pdfX + 50, pdfY + 145);
    }
  }, [tab, firstLineIndent, leftIndent, rightIndent]);

  return (
    <div className="relative flex-1 bg-mercury-canvas dark:bg-mercury-night overflow-hidden flex flex-col transition-colors">
      {/* Interactive Horizontal Ruler (Writer Only) */}
      {tab.type === 'writer' && (
        <div className="h-5 bg-white dark:bg-mercury-night-raised border-b border-mercury-fog/30 flex items-center relative select-none pl-6 text-[9px] text-mercury-ink-secondary dark:text-mercury-fog">
          {/* Top-left corner box */}
          <div className="absolute left-0 top-0 w-6 h-5 border-r border-mercury-fog/30 bg-mercury-lavender/30 dark:bg-mercury-night" />

          {/* Ruler Markings Track */}
          <div className="flex-1 h-full relative overflow-hidden flex items-center">
            {/* Inch tick marks */}
            {Array.from({ length: 9 }).map((_, inch) => (
              <div
                key={inch}
                className="absolute flex items-start"
                style={{ left: `${inch * 72 * (tab.zoom / 100)}px` }}
              >
                <div className="w-[1px] h-3 bg-mercury-fog/60" />
                {inch > 0 && <span className="ml-0.5 -mt-0.5 font-mono">{inch}</span>}
                <div className="absolute left-[36px] w-[1px] h-2 bg-mercury-fog/40" />
                <div className="absolute left-[18px] w-[1px] h-1.5 bg-mercury-fog/30" />
                <div className="absolute left-[54px] w-[1px] h-1.5 bg-mercury-fog/30" />
              </div>
            ))}

            {/* Draggable Indent Triangles */}
            {/* First-Line Indent (Top Triangle) */}
            <div
              className="absolute top-0 cursor-ew-resize text-mercury-primary hover:text-mercury-primary/80 transition-colors"
              style={{ left: `${Math.max(0, firstLineIndent)}px` }}
              title="First Line Indent"
              onMouseDown={(e) => {
                const startX = e.clientX;
                const startVal = firstLineIndent;
                const onMouseMove = (ev: MouseEvent) => {
                  setFirstLineIndent(Math.max(-36, Math.min(144, startVal + (ev.clientX - startX))));
                };
                const onMouseUp = () => {
                  window.removeEventListener('mousemove', onMouseMove);
                  window.removeEventListener('mouseup', onMouseUp);
                };
                window.addEventListener('mousemove', onMouseMove);
                window.addEventListener('mouseup', onMouseUp);
              }}
            >
              <svg width="8" height="6" viewBox="0 0 8 6" fill="currentColor">
                <polygon points="0,0 8,0 4,6" />
              </svg>
            </div>

            {/* Left Hanging Indent (Bottom Triangle) */}
            <div
              className="absolute bottom-0 cursor-ew-resize text-mercury-primary hover:text-mercury-primary/80 transition-colors"
              style={{ left: `${Math.max(0, leftIndent)}px` }}
              title="Left Hanging Indent"
              onMouseDown={(e) => {
                const startX = e.clientX;
                const startVal = leftIndent;
                const onMouseMove = (ev: MouseEvent) => {
                  setLeftIndent(Math.max(0, Math.min(144, startVal + (ev.clientX - startX))));
                };
                const onMouseUp = () => {
                  window.removeEventListener('mousemove', onMouseMove);
                  window.removeEventListener('mouseup', onMouseUp);
                };
                window.addEventListener('mousemove', onMouseMove);
                window.addEventListener('mouseup', onMouseUp);
              }}
            >
              <svg width="8" height="6" viewBox="0 0 8 6" fill="currentColor">
                <polygon points="4,0 8,6 0,6" />
              </svg>
            </div>
          </div>
        </div>
      )}

      {/* Main Viewport Body */}
      <div className="flex-1 flex overflow-hidden relative" ref={containerRef}>
        {/* Interactive Vertical Ruler (Writer Only) */}
        {tab.type === 'writer' && (
          <div className="w-6 bg-white dark:bg-mercury-night-raised border-r border-mercury-fog/30 select-none relative shrink-0 text-[8px] text-mercury-ink-secondary dark:text-mercury-fog font-mono">
            {Array.from({ length: 12 }).map((_, inch) => (
              <div
                key={inch}
                className="absolute w-full flex items-center justify-end pr-1"
                style={{ top: `${inch * 72 * (tab.zoom / 100)}px` }}
              >
                {inch > 0 && <span>{inch}</span>}
                <div className="w-2 h-[1px] bg-mercury-fog/60 ml-0.5" />
              </div>
            ))}
          </div>
        )}

        {/* Scrollable Canvas Container */}
        <div className="flex-1 overflow-auto flex items-start justify-center p-6 relative">
          <canvas
            ref={canvasRef}
            className="cursor-text"
            onClick={(e) => {
              setHasSelection(true);
              setSelectionPos({ x: e.clientX, y: e.clientY });
            }}
          />

          {/* Floating Mercury Micro-Toolbar */}
          {hasSelection && (
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
    </div>
  );
};
