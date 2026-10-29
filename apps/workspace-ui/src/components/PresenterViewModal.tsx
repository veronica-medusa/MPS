import React, { useState, useEffect, useRef } from 'react';
import {
  Play,
  Pause,
  RotateCcw,
  Clock,
  ChevronLeft,
  ChevronRight,
  Maximize2,
  X,
  Highlighter,
  PenTool,
  MousePointer,
  Trash2,
  Monitor,
  Eye,
  Sliders,
} from 'lucide-react';
import { SlideInfo } from '../types';

interface PresenterViewModalProps {
  isOpen: boolean;
  onClose: () => void;
  slides: SlideInfo[];
  currentSlideIndex: number;
  onSlideChange: (index: number) => void;
  themeName: string;
}

export const PresenterViewModal: React.FC<PresenterViewModalProps> = ({
  isOpen,
  onClose,
  slides,
  currentSlideIndex,
  onSlideChange,
  themeName,
}) => {
  const [elapsedSeconds, setElapsedSeconds] = useState(0);
  const [isTimerRunning, setIsTimerRunning] = useState(true);
  const [activeTool, setActiveTool] = useState<'laser' | 'pen' | 'highlighter' | 'none'>('laser');
  const [penColor, setPenColor] = useState('#ff0033');
  const [isBlackout, setIsBlackout] = useState(false);
  const [isWhiteout, setIsWhiteout] = useState(false);

  // Inking canvas state
  const [inkStrokes, setInkStrokes] = useState<Array<{ points: Array<{ x: number; y: number }>; color: string; width: number; isHighlighter: boolean }>>([]);
  const [currentStroke, setCurrentStroke] = useState<Array<{ x: number; y: number }> | null>(null);
  const [laserPos, setLaserPos] = useState<{ x: number; y: number } | null>(null);

  const mainCanvasRef = useRef<HTMLCanvasElement | null>(null);
  const nextCanvasRef = useRef<HTMLCanvasElement | null>(null);

  // Timer tick
  useEffect(() => {
    if (!isOpen || !isTimerRunning) return;
    const interval = setInterval(() => {
      setElapsedSeconds((prev) => prev + 1);
    }, 1000);
    return () => clearInterval(interval);
  }, [isOpen, isTimerRunning]);

  // Clear ink when slide changes
  useEffect(() => {
    setInkStrokes([]);
    setCurrentStroke(null);
  }, [currentSlideIndex]);

  // Keyboard navigation
  useEffect(() => {
    if (!isOpen) return;
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'ArrowRight' || e.key === ' ' || e.key === 'PageDown') {
        if (currentSlideIndex + 1 < slides.length) {
          onSlideChange(currentSlideIndex + 1);
        }
      } else if (e.key === 'ArrowLeft' || e.key === 'PageUp') {
        if (currentSlideIndex > 0) {
          onSlideChange(currentSlideIndex - 1);
        }
      } else if (e.key === 'b' || e.key === 'B') {
        setIsBlackout((prev) => !prev);
      } else if (e.key === 'w' || e.key === 'W') {
        setIsWhiteout((prev) => !prev);
      } else if (e.key === 'Escape') {
        onClose();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, currentSlideIndex, slides.length, onSlideChange, onClose]);

  const currentSlide = slides[currentSlideIndex] || slides[0];
  const nextSlide = slides[currentSlideIndex + 1] || null;

  // Format timer HH:MM:SS
  const formatTimer = (secs: number) => {
    const h = Math.floor(secs / 3600);
    const m = Math.floor((secs % 3600) / 60);
    const s = secs % 60;
    return `${h > 0 ? `${h.toString().padStart(2, '0')}:` : ''}${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
  };

  // Render current slide canvas
  useEffect(() => {
    if (!isOpen || !mainCanvasRef.current || !currentSlide) return;
    const canvas = mainCanvasRef.current;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const width = canvas.width;
    const height = canvas.height;
    ctx.clearRect(0, 0, width, height);

    if (isBlackout) {
      ctx.fillStyle = '#000000';
      ctx.fillRect(0, 0, width, height);
      return;
    }
    if (isWhiteout) {
      ctx.fillStyle = '#ffffff';
      ctx.fillRect(0, 0, width, height);
      return;
    }

    // Slide Background
    ctx.fillStyle = '#ffffff';
    ctx.fillRect(0, 0, width, height);

    // Shapes
    for (const shape of currentSlide.shapes) {
      ctx.save();
      const sx = (shape.x / 960) * width;
      const sy = (shape.y / 540) * height;
      const sw = (shape.width / 960) * width;
      const sh = (shape.height / 540) * height;

      ctx.fillStyle = shape.fillColor || '#5266eb';
      ctx.strokeStyle = shape.strokeColor || '#28253b';
      ctx.lineWidth = shape.strokeWidth || 1.5;

      if (shape.geometry === 'RoundedRectangle' || shape.geometry === 'roundRect') {
        ctx.beginPath();
        ctx.roundRect(sx, sy, sw, sh, 12);
        ctx.fill();
        ctx.stroke();
      } else if (shape.geometry === 'Ellipse' || shape.geometry === 'ellipse') {
        ctx.beginPath();
        ctx.ellipse(sx + sw / 2, sy + sh / 2, sw / 2, sh / 2, 0, 0, Math.PI * 2);
        ctx.fill();
        ctx.stroke();
      } else if (shape.geometry === 'Diamond' || shape.geometry === 'diamond') {
        ctx.beginPath();
        ctx.moveTo(sx + sw / 2, sy);
        ctx.lineTo(sx + sw, sy + sh / 2);
        ctx.lineTo(sx + sw / 2, sy + sh);
        ctx.lineTo(sx, sy + sh / 2);
        ctx.closePath();
        ctx.fill();
        ctx.stroke();
      } else if (shape.geometry === 'Star5' || shape.geometry === 'star5') {
        const cx = sx + sw / 2;
        const cy = sy + sh / 2;
        const rOuter = sw / 2;
        const rInner = rOuter * 0.38;
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
        ctx.stroke();
      } else {
        ctx.fillRect(sx, sy, sw, sh);
        ctx.strokeRect(sx, sy, sw, sh);
      }

      // Text inside shape
      if (shape.text) {
        ctx.fillStyle = '#ffffff';
        ctx.font = '600 16px Inter, sans-serif';
        ctx.textAlign = 'center';
        const lines = shape.text.split('\n');
        lines.forEach((line, i) => {
          ctx.fillText(line, sx + sw / 2, sy + sh / 2 + (i - (lines.length - 1) / 2) * 22);
        });
      }
      ctx.restore();
    }

    // Connectors
    for (const conn of currentSlide.connectors) {
      ctx.save();
      ctx.strokeStyle = conn.color || '#28253b';
      ctx.lineWidth = conn.strokeWidth || 2;
      const s1 = currentSlide.shapes.find((s) => s.id === conn.startId);
      const s2 = currentSlide.shapes.find((s) => s.id === conn.endId);
      if (s1 && s2) {
        const x1 = ((s1.x + s1.width) / 960) * width;
        const y1 = ((s1.y + s1.height / 2) / 540) * height;
        const x2 = (s2.x / 960) * width;
        const y2 = ((s2.y + s2.height / 2) / 540) * height;
        const midX = (x1 + x2) / 2;

        ctx.beginPath();
        ctx.moveTo(x1, y1);
        ctx.lineTo(midX, y1);
        ctx.lineTo(midX, y2);
        ctx.lineTo(x2, y2);
        ctx.stroke();
      }
      ctx.restore();
    }

    // Render committed ink strokes
    for (const stroke of inkStrokes) {
      if (stroke.points.length < 2) continue;
      ctx.save();
      ctx.strokeStyle = stroke.color;
      ctx.lineWidth = stroke.width;
      ctx.lineCap = 'round';
      ctx.lineJoin = 'round';
      if (stroke.isHighlighter) {
        ctx.globalAlpha = 0.35;
      }
      ctx.beginPath();
      ctx.moveTo(stroke.points[0].x, stroke.points[0].y);
      for (let i = 1; i < stroke.points.length; i++) {
        ctx.lineTo(stroke.points[i].x, stroke.points[i].y);
      }
      ctx.stroke();
      ctx.restore();
    }

    // Render current in-progress ink stroke
    if (currentStroke && currentStroke.length >= 2) {
      ctx.save();
      ctx.strokeStyle = penColor;
      ctx.lineWidth = activeTool === 'highlighter' ? 14 : 3;
      ctx.lineCap = 'round';
      ctx.lineJoin = 'round';
      if (activeTool === 'highlighter') {
        ctx.globalAlpha = 0.35;
      }
      ctx.beginPath();
      ctx.moveTo(currentStroke[0].x, currentStroke[0].y);
      for (let i = 1; i < currentStroke.length; i++) {
        ctx.lineTo(currentStroke[i].x, currentStroke[i].y);
      }
      ctx.stroke();
      ctx.restore();
    }

    // Render Laser Pointer
    if (activeTool === 'laser' && laserPos) {
      ctx.save();
      ctx.fillStyle = 'rgba(255, 0, 51, 0.3)';
      ctx.beginPath();
      ctx.arc(laserPos.x, laserPos.y, 14, 0, Math.PI * 2);
      ctx.fill();

      ctx.fillStyle = '#ff0033';
      ctx.beginPath();
      ctx.arc(laserPos.x, laserPos.y, 4, 0, Math.PI * 2);
      ctx.fill();
      ctx.restore();
    }
  }, [isOpen, currentSlide, isBlackout, isWhiteout, inkStrokes, currentStroke, laserPos, activeTool, penColor]);

  // Render Next Slide preview thumbnail
  useEffect(() => {
    if (!isOpen || !nextCanvasRef.current) return;
    const canvas = nextCanvasRef.current;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    ctx.clearRect(0, 0, canvas.width, canvas.height);
    if (!nextSlide) {
      ctx.fillStyle = '#28253b';
      ctx.font = '500 13px Inter';
      ctx.textAlign = 'center';
      ctx.fillText('End of Slideshow', canvas.width / 2, canvas.height / 2);
      return;
    }

    ctx.fillStyle = '#ffffff';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    for (const shape of nextSlide.shapes) {
      const sx = (shape.x / 960) * canvas.width;
      const sy = (shape.y / 540) * canvas.height;
      const sw = (shape.width / 960) * canvas.width;
      const sh = (shape.height / 540) * canvas.height;
      ctx.fillStyle = shape.fillColor || '#e8590c';
      ctx.fillRect(sx, sy, sw, sh);
    }
  }, [isOpen, nextSlide]);

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 bg-[#171721] text-white flex flex-col font-sans select-none overflow-hidden">
      {/* Top Cockpit Header */}
      <div className="h-14 border-b border-[#28253b] px-6 flex items-center justify-between bg-[#1e1e2a]">
        <div className="flex items-center gap-3">
          <span className="w-3 h-3 rounded-full bg-[#e8590c] animate-pulse"></span>
          <span className="font-semibold text-base tracking-wide">Presenter View</span>
          <span className="text-xs px-2 py-0.5 rounded bg-[#28253b] text-[#9cb4e8]">Dual-Screen Cockpit</span>
          <span className="text-xs text-white/50 ml-2">Theme: {themeName}</span>
        </div>

        <div className="flex items-center gap-6">
          <div className="flex items-center gap-2 bg-[#28253b] px-3 py-1.5 rounded-lg border border-white/10">
            <Clock className="w-4 h-4 text-[#e8590c]" />
            <span className="font-mono text-sm font-bold tracking-wider">{formatTimer(elapsedSeconds)}</span>
            <button
              onClick={() => setIsTimerRunning(!isTimerRunning)}
              className="p-1 hover:bg-white/10 rounded transition text-white/80 hover:text-white"
              title={isTimerRunning ? 'Pause Timer' : 'Resume Timer'}
            >
              {isTimerRunning ? <Pause className="w-3.5 h-3.5" /> : <Play className="w-3.5 h-3.5" />}
            </button>
            <button
              onClick={() => setElapsedSeconds(0)}
              className="p-1 hover:bg-white/10 rounded transition text-white/80 hover:text-white"
              title="Reset Timer"
            >
              <RotateCcw className="w-3.5 h-3.5" />
            </button>
          </div>

          <div className="text-sm font-medium text-white/80">
            Slide <span className="font-bold text-[#e8590c]">{currentSlideIndex + 1}</span> of {slides.length}
          </div>

          <button
            onClick={onClose}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-red-600/80 hover:bg-red-600 text-white text-xs font-semibold transition"
          >
            <X className="w-4 h-4" />
            End Show
          </button>
        </div>
      </div>

      {/* Main Multi-Screen Layout */}
      <div className="flex-1 grid grid-cols-12 gap-6 p-6 overflow-hidden">
        {/* Left: Active Stage (7 cols) */}
        <div className="col-span-7 flex flex-col items-center justify-center bg-[#1e1e2a] rounded-xl border border-[#28253b] p-4 relative shadow-2xl">
          <div className="text-xs text-white/40 mb-2 font-mono flex items-center justify-between w-full px-2">
            <span>LIVE AUDIENCE SCREEN</span>
            <span>1920 × 1080 (16:9 Canvas)</span>
          </div>

          <div className="relative w-full max-w-[840px] aspect-[16/9] bg-white rounded-lg overflow-hidden shadow-2xl border border-white/20">
            <canvas
              ref={mainCanvasRef}
              width={960}
              height={540}
              className="w-full h-full cursor-crosshair"
              onMouseMove={(e) => {
                const rect = e.currentTarget.getBoundingClientRect();
                const x = (e.clientX - rect.left) * (960 / rect.width);
                const y = (e.clientY - rect.top) * (540 / rect.height);
                setLaserPos({ x, y });
                if (currentStroke) {
                  setCurrentStroke((prev) => (prev ? [...prev, { x, y }] : null));
                }
              }}
              onMouseLeave={() => setLaserPos(null)}
              onMouseDown={(e) => {
                if (activeTool === 'pen' || activeTool === 'highlighter') {
                  const rect = e.currentTarget.getBoundingClientRect();
                  const x = (e.clientX - rect.left) * (960 / rect.width);
                  const y = (e.clientY - rect.top) * (540 / rect.height);
                  setCurrentStroke([{ x, y }]);
                }
              }}
              onMouseUp={() => {
                if (currentStroke && currentStroke.length >= 2) {
                  setInkStrokes((prev) => [
                    ...prev,
                    {
                      points: currentStroke,
                      color: penColor,
                      width: activeTool === 'highlighter' ? 14 : 3,
                      isHighlighter: activeTool === 'highlighter',
                    },
                  ]);
                }
                setCurrentStroke(null);
              }}
            />
          </div>

          {/* Interactive Live Presenter Inking Bar */}
          <div className="absolute bottom-6 flex items-center gap-2 bg-[#28253b]/90 backdrop-blur-md px-4 py-2 rounded-xl border border-white/10 shadow-lg">
            <button
              onClick={() => setActiveTool('laser')}
              className={`p-2 rounded-lg transition ${
                activeTool === 'laser' ? 'bg-[#ff0033] text-white shadow-md' : 'text-white/70 hover:bg-white/10'
              }`}
              title="Laser Pointer"
            >
              <MousePointer className="w-4 h-4" />
            </button>
            <button
              onClick={() => setActiveTool('pen')}
              className={`p-2 rounded-lg transition ${
                activeTool === 'pen' ? 'bg-[#5266eb] text-white shadow-md' : 'text-white/70 hover:bg-white/10'
              }`}
              title="Draw Pen"
            >
              <PenTool className="w-4 h-4" />
            </button>
            <button
              onClick={() => {
                setActiveTool('highlighter');
                setPenColor('#ffd43b');
              }}
              className={`p-2 rounded-lg transition ${
                activeTool === 'highlighter' ? 'bg-amber-400 text-black shadow-md' : 'text-white/70 hover:bg-white/10'
              }`}
              title="Highlighter"
            >
              <Highlighter className="w-4 h-4" />
            </button>

            <div className="w-px h-5 bg-white/20 mx-1"></div>

            {/* Color Swatches */}
            {['#ff0033', '#3b5bdb', '#0ca678', '#ffd43b', '#ffffff'].map((color) => (
              <button
                key={color}
                onClick={() => setPenColor(color)}
                className={`w-5 h-5 rounded-full border border-white/20 transition ${
                  penColor === color ? 'scale-125 ring-2 ring-white ring-offset-1 ring-offset-[#28253b]' : 'hover:scale-110'
                }`}
                style={{ backgroundColor: color }}
              />
            ))}

            <div className="w-px h-5 bg-white/20 mx-1"></div>

            <button
              onClick={() => setInkStrokes([])}
              className="p-2 text-white/70 hover:text-red-400 hover:bg-white/10 rounded-lg transition"
              title="Clear Ink"
            >
              <Trash2 className="w-4 h-4" />
            </button>
            <button
              onClick={() => setIsBlackout(!isBlackout)}
              className={`px-2.5 py-1 text-xs font-semibold rounded-lg transition border ${
                isBlackout ? 'bg-black text-white border-white/50' : 'text-white/70 hover:bg-white/10 border-white/10'
              }`}
              title="Blackout Screen ('B')"
            >
              Blackout (B)
            </button>
            <button
              onClick={() => setIsWhiteout(!isWhiteout)}
              className={`px-2.5 py-1 text-xs font-semibold rounded-lg transition border ${
                isWhiteout ? 'bg-white text-black border-white/50' : 'text-white/70 hover:bg-white/10 border-white/10'
              }`}
              title="Whiteout Screen ('W')"
            >
              Whiteout (W)
            </button>
          </div>
        </div>

        {/* Right: Next Slide & Speaker Notes Cockpit (5 cols) */}
        <div className="col-span-5 flex flex-col gap-6 overflow-hidden">
          {/* Next Slide Preview Card */}
          <div className="bg-[#1e1e2a] rounded-xl border border-[#28253b] p-4 flex flex-col shadow-lg">
            <div className="flex items-center justify-between mb-3 text-xs text-white/50 font-mono">
              <span>NEXT SLIDE PREVIEW</span>
              {nextSlide?.transitionName && (
                <span className="px-2 py-0.5 rounded bg-[#e8590c]/20 text-[#e8590c] font-semibold border border-[#e8590c]/40">
                  {nextSlide.transitionName} Transition
                </span>
              )}
            </div>
            <div className="w-full aspect-[16/9] bg-white rounded-lg overflow-hidden border border-white/10 shadow">
              <canvas ref={nextCanvasRef} width={480} height={270} className="w-full h-full" />
            </div>
          </div>

          {/* Speaker Notes Card */}
          <div className="flex-1 bg-[#1e1e2a] rounded-xl border border-[#28253b] p-5 flex flex-col shadow-lg overflow-hidden">
            <div className="flex items-center justify-between mb-2 pb-2 border-b border-white/10">
              <span className="text-xs font-bold text-[#e8590c] tracking-wider uppercase">Speaker Notes</span>
              <span className="text-xs text-white/40">Markdown formatting supported</span>
            </div>
            <div className="flex-1 overflow-y-auto text-sm text-white/90 leading-relaxed font-sans pr-2">
              {currentSlide.speakerNotes ? (
                currentSlide.speakerNotes.split('\n').map((para, i) => (
                  <p key={i} className="mb-2">
                    {para}
                  </p>
                ))
              ) : (
                <span className="text-white/30 italic">No notes recorded for this slide.</span>
              )}
            </div>
          </div>
        </div>
      </div>

      {/* Bottom Master Navigation Bar */}
      <div className="h-16 border-t border-[#28253b] px-8 flex items-center justify-between bg-[#1e1e2a]">
        <div className="text-sm font-semibold truncate max-w-md text-white/90">
          {currentSlide.title || 'Untitled Slide'}
        </div>

        <div className="flex items-center gap-4">
          <button
            onClick={() => currentSlideIndex > 0 && onSlideChange(currentSlideIndex - 1)}
            disabled={currentSlideIndex === 0}
            className="flex items-center gap-1 px-4 py-2 rounded-lg bg-[#28253b] hover:bg-[#34304e] text-white disabled:opacity-40 disabled:cursor-not-allowed font-medium text-sm transition border border-white/10"
          >
            <ChevronLeft className="w-4 h-4" />
            Previous
          </button>

          <span className="text-sm font-mono text-white/70">
            {currentSlideIndex + 1} / {slides.length}
          </span>

          <button
            onClick={() => currentSlideIndex + 1 < slides.length && onSlideChange(currentSlideIndex + 1)}
            disabled={currentSlideIndex + 1 >= slides.length}
            className="flex items-center gap-1 px-4 py-2 rounded-lg bg-[#e8590c] hover:bg-[#d9480f] text-white disabled:opacity-40 disabled:cursor-not-allowed font-semibold text-sm transition shadow-lg shadow-[#e8590c]/20"
          >
            Next
            <ChevronRight className="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>
  );
};
