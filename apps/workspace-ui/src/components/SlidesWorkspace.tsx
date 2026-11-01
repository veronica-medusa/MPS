import React, { useState, useRef, useEffect } from 'react';
import {
  Play,
  Monitor,
  Plus,
  Copy,
  Trash2,
  Square,
  Circle,
  ArrowRight,
  MessageSquare,
  Star,
  Type,
  ImageIcon,
  Table as TableIcon,
  Palette,
  Sparkles,
  ChevronLeft,
  ChevronRight,
  Maximize2,
  X,
  Compass
} from 'lucide-react';
import { SlideData, SlideElement, PresentationState } from '../types';

interface SlidesWorkspaceProps {
  initialState?: Partial<PresentationState>;
  onStateChange?: (state: PresentationState) => void;
  onOpenAiAssist?: (prompt: string) => void;
  onOpenPresenterView?: () => void;
}

export const SlidesWorkspace: React.FC<SlidesWorkspaceProps> = ({
  initialState,
  onStateChange,
  onOpenAiAssist,
  onOpenPresenterView,
}) => {
  // ─── Default Slides ───
  const defaultSlides: SlideData[] = [
    {
      id: 'slide-1',
      title: 'OpenWPS Executive Strategy',
      layout: 'title',
      transition: 'morph',
      speakerNotes: 'Welcome the board. Emphasize that OpenWPS delivers complete native performance with 100% ECMA-376 fidelity.',
      elements: [
        {
          id: 'el-1',
          type: 'title',
          x: 10,
          y: 28,
          width: 80,
          height: 20,
          text: 'OpenWPS Enterprise Suite',
          fontSize: 38,
          bold: true,
          align: 'center',
          textColor: '#ffffff',
        },
        {
          id: 'el-2',
          type: 'subtitle',
          x: 15,
          y: 50,
          width: 70,
          height: 15,
          text: 'High-performance document processing, calculations, and AI co-pilots',
          fontSize: 18,
          align: 'center',
          textColor: '#9cb4e8',
        },
      ],
    },
    {
      id: 'slide-2',
      title: 'Core Architectural Pillars',
      layout: 'title_content',
      transition: 'fade',
      speakerNotes: 'Walk through the 3 core pillars: unified Rust kernel, zero-jargon office UX, and private offline AI co-pilot.',
      elements: [
        {
          id: 'el-201',
          type: 'title',
          x: 8,
          y: 10,
          width: 84,
          height: 14,
          text: 'Architectural Capabilities',
          fontSize: 28,
          bold: true,
          align: 'left',
          textColor: '#ffffff',
        },
        {
          id: 'el-202',
          type: 'shape',
          shapeType: 'rounded',
          x: 8,
          y: 30,
          width: 26,
          height: 48,
          text: '1. Unified Core\n\nInstant startup and 120fps rendering across all 4 document types.',
          fillColor: 'rgba(82, 102, 235, 0.25)',
          strokeColor: '#5266eb',
          textColor: '#ededf3',
          fontSize: 14,
          align: 'center',
        },
        {
          id: 'el-203',
          type: 'shape',
          shapeType: 'rounded',
          x: 37,
          y: 30,
          width: 26,
          height: 48,
          text: '2. Intuitive UX\n\nZero compiler jargon. Authentic 5-zone WPS Office layout.',
          fillColor: 'rgba(33, 115, 70, 0.25)',
          strokeColor: '#217346',
          textColor: '#ededf3',
          fontSize: 14,
          align: 'center',
        },
        {
          id: 'el-204',
          type: 'shape',
          shapeType: 'rounded',
          x: 66,
          y: 30,
          width: 26,
          height: 48,
          text: '3. Built-In AI\n\nLocal assistance for writing, formulas, slides, and PDF RAG.',
          fillColor: 'rgba(210, 71, 38, 0.25)',
          strokeColor: '#d24726',
          textColor: '#ededf3',
          fontSize: 14,
          align: 'center',
        },
      ],
    },
    {
      id: 'slide-3',
      title: 'Rollout Roadmap',
      layout: 'title_content',
      transition: 'push',
      speakerNotes: 'Conclude with rollout milestones for Q1 and Q2.',
      elements: [
        {
          id: 'el-301',
          type: 'title',
          x: 8,
          y: 10,
          width: 84,
          height: 14,
          text: 'Rollout Milestones',
          fontSize: 28,
          bold: true,
          align: 'left',
          textColor: '#ffffff',
        },
        {
          id: 'el-302',
          type: 'text',
          x: 8,
          y: 30,
          width: 84,
          height: 50,
          text: '• Phase 1: Deploy Desktop & Web builds across enterprise pilot groups.\n• Phase 2: Connect real-time collaboration and cloud roaming storage.\n• Phase 3: Roll out AI Co-Pilot with specialized company document templates.',
          fontSize: 16,
          textColor: '#ededf3',
          align: 'left',
        },
      ],
    },
  ];

  const [slides, setSlides] = useState<SlideData[]>(initialState?.slides || defaultSlides);
  const [activeSlideIndex, setActiveSlideIndex] = useState<number>(0);
  const [theme, setTheme] = useState<'mercury-night' | 'mercury-canvas' | 'mercury-lavender' | 'executive-indigo'>('mercury-night');
  const [selectedElementId, setSelectedElementId] = useState<string | null>(null);
  const [activeRibbonTab, setActiveRibbonTab] = useState<'home' | 'insert' | 'design' | 'transitions' | 'slideshow'>('home');
  const [isPlayingSlideshow, setIsPlayingSlideshow] = useState<boolean>(false);
  const [slideshowSlideIdx, setSlideshowSlideIdx] = useState<number>(0);
  const [laserActive, setLaserActive] = useState<boolean>(false);
  const [mousePos, setMousePos] = useState<{ x: number; y: number }>({ x: 0, y: 0 });

  const activeSlide = slides[activeSlideIndex] || slides[0];

  // Add New Slide
  const handleAddSlide = () => {
    const newSlide: SlideData = {
      id: `slide-${Date.now()}`,
      title: `New Slide ${slides.length + 1}`,
      layout: 'title_content',
      transition: 'fade',
      speakerNotes: 'Click to add speaker notes...',
      elements: [
        {
          id: `el-${Date.now()}-1`,
          type: 'title',
          x: 8,
          y: 10,
          width: 84,
          height: 14,
          text: 'Click to Add Title',
          fontSize: 28,
          bold: true,
          align: 'left',
          textColor: theme === 'mercury-canvas' ? '#28253b' : '#ffffff',
        },
        {
          id: `el-${Date.now()}-2`,
          type: 'text',
          x: 8,
          y: 30,
          width: 84,
          height: 50,
          text: 'Click to add slide content or bullet points...',
          fontSize: 16,
          align: 'left',
          textColor: theme === 'mercury-canvas' ? '#554e7b' : '#ededf3',
        },
      ],
    };
    setSlides([...slides, newSlide]);
    setActiveSlideIndex(slides.length);
  };

  // Duplicate Slide
  const handleDuplicateSlide = () => {
    const current = activeSlide;
    const duplicated: SlideData = {
      ...current,
      id: `slide-${Date.now()}`,
      title: `${current.title} (Copy)`,
      elements: current.elements.map((el) => ({ ...el, id: `el-${Date.now()}-${Math.random()}` })),
    };
    const newSlides = [...slides];
    newSlides.splice(activeSlideIndex + 1, 0, duplicated);
    setSlides(newSlides);
    setActiveSlideIndex(activeSlideIndex + 1);
  };

  // Delete Slide
  const handleDeleteSlide = () => {
    if (slides.length <= 1) return;
    const newSlides = slides.filter((_, idx) => idx !== activeSlideIndex);
    setSlides(newSlides);
    setActiveSlideIndex(Math.max(0, activeSlideIndex - 1));
  };

  // Insert Shape
  const handleInsertShape = (shapeType: 'rectangle' | 'rounded' | 'circle' | 'arrow' | 'callout' | 'star') => {
    const newEl: SlideElement = {
      id: `shape-${Date.now()}`,
      type: 'shape',
      shapeType,
      x: 35,
      y: 35,
      width: 30,
      height: 25,
      text: `${shapeType.toUpperCase()}`,
      fillColor: 'rgba(82, 102, 235, 0.3)',
      strokeColor: '#5266eb',
      textColor: '#ffffff',
      fontSize: 14,
      align: 'center',
    };
    const updated = {
      ...activeSlide,
      elements: [...activeSlide.elements, newEl],
    };
    const newSlides = [...slides];
    newSlides[activeSlideIndex] = updated;
    setSlides(newSlides);
    setSelectedElementId(newEl.id);
  };

  // Update text of element
  const handleElementTextChange = (elId: string, newText: string) => {
    const updated = {
      ...activeSlide,
      elements: activeSlide.elements.map((el) => (el.id === elId ? { ...el, text: newText } : el)),
    };
    const newSlides = [...slides];
    newSlides[activeSlideIndex] = updated;
    setSlides(newSlides);
  };

  // Update speaker notes
  const handleSpeakerNotesChange = (notes: string) => {
    const updated = { ...activeSlide, speakerNotes: notes };
    const newSlides = [...slides];
    newSlides[activeSlideIndex] = updated;
    setSlides(newSlides);
  };

  // Keyboard navigation for slideshow
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (!isPlayingSlideshow) return;
      if (e.key === 'ArrowRight' || e.key === 'Space') {
        if (slideshowSlideIdx < slides.length - 1) {
          setSlideshowSlideIdx((prev) => prev + 1);
        }
      } else if (e.key === 'ArrowLeft') {
        if (slideshowSlideIdx > 0) {
          setSlideshowSlideIdx((prev) => prev - 1);
        }
      } else if (e.key === 'Escape') {
        setIsPlayingSlideshow(false);
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isPlayingSlideshow, slideshowSlideIdx, slides.length]);

  // Theme background styles
  const getThemeBackground = () => {
    switch (theme) {
      case 'mercury-night':
        return '#1e1e2a';
      case 'mercury-canvas':
        return '#ffffff';
      case 'mercury-lavender':
        return '#edecfb';
      case 'executive-indigo':
        return '#252945';
    }
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
              onClick={() => setActiveRibbonTab('design')}
              className={`wps-ribbon-tab-btn ${activeRibbonTab === 'design' ? 'is-active' : ''}`}
            >
              Design & Themes
            </button>
            <button
              onClick={() => setActiveRibbonTab('transitions')}
              className={`wps-ribbon-tab-btn ${activeRibbonTab === 'transitions' ? 'is-active' : ''}`}
            >
              Transitions
            </button>
            <button
              onClick={() => setActiveRibbonTab('slideshow')}
              className={`wps-ribbon-tab-btn ${activeRibbonTab === 'slideshow' ? 'is-active' : ''}`}
            >
              Slide Show
            </button>
          </div>

          <div className="flex items-center gap-2 text-xs text-mercury-slate">
            <span>WPS Presentation</span>
          </div>
        </div>

        {/* Ribbon Body Actions */}
        <div className="wps-ribbon-body">
          {/* HOME TAB */}
          {activeRibbonTab === 'home' && (
            <>
              <div className="wps-ribbon-group">
                <div className="wps-ribbon-controls">
                  <button onClick={handleAddSlide} className="wps-ribbon-btn">
                    <Plus className="w-5 h-5 text-orange-600" />
                    <span>New Slide</span>
                  </button>
                  <button onClick={handleDuplicateSlide} className="wps-ribbon-btn">
                    <Copy className="w-5 h-5 text-mercury-slate" />
                    <span>Duplicate</span>
                  </button>
                  <button onClick={handleDeleteSlide} className="wps-ribbon-btn text-rose-700">
                    <Trash2 className="w-5 h-5 text-rose-600" />
                    <span>Delete Slide</span>
                  </button>
                </div>
                <span className="wps-ribbon-group-title">Slide Actions</span>
              </div>

              <div className="wps-ribbon-group">
                <div className="wps-ribbon-controls">
                  <button
                    onClick={() => {
                      setSlideshowSlideIdx(0);
                      setIsPlayingSlideshow(true);
                    }}
                    className="button-primary h-9 px-4 text-xs flex items-center gap-2"
                  >
                    <Play className="w-4 h-4 fill-white" />
                    <span>Present</span>
                  </button>
                </div>
                <span className="wps-ribbon-group-title">Quick Present</span>
              </div>
            </>
          )}

          {/* INSERT TAB */}
          {activeRibbonTab === 'insert' && (
            <>
              <div className="wps-ribbon-group">
                <div className="wps-ribbon-controls">
                  <button onClick={() => handleInsertShape('rectangle')} className="wps-ribbon-btn">
                    <Square className="w-5 h-5 text-blue-600" />
                    <span>Rectangle</span>
                  </button>
                  <button onClick={() => handleInsertShape('circle')} className="wps-ribbon-btn">
                    <Circle className="w-5 h-5 text-emerald-600" />
                    <span>Circle</span>
                  </button>
                  <button onClick={() => handleInsertShape('arrow')} className="wps-ribbon-btn">
                    <ArrowRight className="w-5 h-5 text-amber-600" />
                    <span>Arrow</span>
                  </button>
                  <button onClick={() => handleInsertShape('star')} className="wps-ribbon-btn">
                    <Star className="w-5 h-5 text-orange-600" />
                    <span>Star</span>
                  </button>
                </div>
                <span className="wps-ribbon-group-title">Shapes</span>
              </div>
            </>
          )}

          {/* DESIGN TAB */}
          {activeRibbonTab === 'design' && (
            <>
              <div className="wps-ribbon-group">
                <div className="wps-ribbon-controls">
                  <button
                    onClick={() => setTheme('mercury-night')}
                    className={`px-3 py-1 text-xs border rounded transition-all ${
                      theme === 'mercury-night' ? 'border-mercury-primary bg-indigo-50 font-semibold' : 'border-mercury-fog/40'
                    }`}
                  >
                    Mercury Night (#171721)
                  </button>
                  <button
                    onClick={() => setTheme('mercury-canvas')}
                    className={`px-3 py-1 text-xs border rounded transition-all ${
                      theme === 'mercury-canvas' ? 'border-mercury-primary bg-indigo-50 font-semibold' : 'border-mercury-fog/40'
                    }`}
                  >
                    Light Canvas (#fbfcfd)
                  </button>
                  <button
                    onClick={() => setTheme('mercury-lavender')}
                    className={`px-3 py-1 text-xs border rounded transition-all ${
                      theme === 'mercury-lavender' ? 'border-mercury-primary bg-indigo-50 font-semibold' : 'border-mercury-fog/40'
                    }`}
                  >
                    Lavender (#edecfb)
                  </button>
                  <button
                    onClick={() => setTheme('executive-indigo')}
                    className={`px-3 py-1 text-xs border rounded transition-all ${
                      theme === 'executive-indigo' ? 'border-mercury-primary bg-indigo-50 font-semibold' : 'border-mercury-fog/40'
                    }`}
                  >
                    Executive Indigo (#5266eb)
                  </button>
                </div>
                <span className="wps-ribbon-group-title">Theme Presets</span>
              </div>
            </>
          )}

          {/* TRANSITIONS TAB */}
          {activeRibbonTab === 'transitions' && (
            <>
              <div className="wps-ribbon-group">
                <div className="wps-ribbon-controls">
                  {(['none', 'fade', 'push', 'wipe', 'morph'] as const).map((trans) => (
                    <button
                      key={trans}
                      onClick={() => {
                        const updated = { ...activeSlide, transition: trans };
                        const newSlides = [...slides];
                        newSlides[activeSlideIndex] = updated;
                        setSlides(newSlides);
                      }}
                      className={`px-2.5 py-1 text-xs border rounded capitalize ${
                        activeSlide.transition === trans ? 'border-mercury-primary bg-indigo-50 font-semibold' : 'border-mercury-fog/40'
                      }`}
                    >
                      {trans}
                    </button>
                  ))}
                </div>
                <span className="wps-ribbon-group-title">Slide Transition</span>
              </div>
            </>
          )}

          {/* SLIDESHOW TAB */}
          {activeRibbonTab === 'slideshow' && (
            <>
              <div className="wps-ribbon-group">
                <div className="wps-ribbon-controls">
                  <button
                    onClick={() => {
                      setSlideshowSlideIdx(0);
                      setIsPlayingSlideshow(true);
                    }}
                    className="wps-ribbon-btn"
                  >
                    <Play className="w-5 h-5 text-emerald-600" />
                    <span>From Beginning</span>
                  </button>
                  <button
                    onClick={() => {
                      setSlideshowSlideIdx(activeSlideIndex);
                      setIsPlayingSlideshow(true);
                    }}
                    className="wps-ribbon-btn"
                  >
                    <Play className="w-5 h-5 text-orange-600" />
                    <span>From Current Slide</span>
                  </button>
                  <button
                    onClick={onOpenPresenterView}
                    className="wps-ribbon-btn"
                  >
                    <Monitor className="w-5 h-5 text-indigo-600" />
                    <span>Presenter View</span>
                  </button>
                </div>
                <span className="wps-ribbon-group-title">Presentation Modes</span>
              </div>
            </>
          )}
        </div>
      </div>

      {/* ─── Zone 3: 3-Panel Presentation Workspace ─── */}
      <div className="flex-1 flex overflow-hidden">
        {/* Panel 1: Left Slide Filmstrip (200px) */}
        <aside className="w-52 bg-white border-r border-mercury-fog/40 flex flex-col p-3 gap-3 overflow-y-auto select-none flex-shrink-0">
          <div className="flex items-center justify-between text-xs text-mercury-slate font-medium px-1">
            <span>Slides ({slides.length})</span>
            <button
              onClick={handleAddSlide}
              className="p-1 rounded hover:bg-gray-100 text-mercury-slate hover:text-mercury-primary"
              title="Add New Slide"
            >
              <Plus className="w-3.5 h-3.5" />
            </button>
          </div>

          <div className="flex flex-col gap-2.5">
            {slides.map((s, idx) => (
              <div
                key={s.id}
                onClick={() => setActiveSlideIndex(idx)}
                className={`flex items-start gap-2 cursor-pointer p-1.5 rounded-mercury-sm border-2 transition-all ${
                  activeSlideIndex === idx
                    ? 'border-mercury-primary bg-indigo-50/30 shadow-sm'
                    : 'border-transparent hover:border-mercury-fog/60'
                }`}
              >
                <span className="text-[11px] font-semibold text-mercury-slate mt-1 w-4 text-right">
                  {idx + 1}
                </span>

                <div
                  className="flex-1 aspect-video rounded border border-mercury-fog/50 p-2 overflow-hidden flex flex-col justify-between text-[8px]"
                  style={{ backgroundColor: getThemeBackground() }}
                >
                  <span
                    className="font-bold truncate"
                    style={{ color: theme === 'mercury-canvas' ? '#28253b' : '#ffffff' }}
                  >
                    {s.title || `Slide ${idx + 1}`}
                  </span>
                  <span className="text-gray-400 self-end text-[7px]">{s.transition || 'fade'}</span>
                </div>
              </div>
            ))}
          </div>
        </aside>

        {/* Panel 2 & 3: Center 16:9 Canvas + Bottom Speaker Notes */}
        <div className="flex-1 flex flex-col overflow-hidden bg-mercury-canvas">
          {/* Main 16:9 Slide Canvas Viewport */}
          <div className="flex-1 flex items-center justify-center p-6 overflow-auto">
            <div
              className="w-full max-w-4xl aspect-video rounded-mercury-md shadow-2xl relative overflow-hidden p-8 select-none transition-all duration-300 border border-mercury-glass-dark-border"
              style={{ backgroundColor: getThemeBackground() }}
            >
              {activeSlide.elements.map((el) => {
                const isSelected = selectedElementId === el.id;

                return (
                  <div
                    key={el.id}
                    onClick={(e) => {
                      e.stopPropagation();
                      setSelectedElementId(el.id);
                    }}
                    className={`absolute p-2 rounded transition-all cursor-move ${
                      isSelected ? 'ring-2 ring-mercury-primary shadow-lg' : 'hover:ring-1 hover:ring-white/30'
                    }`}
                    style={{
                      left: `${el.x}%`,
                      top: `${el.y}%`,
                      width: `${el.width}%`,
                      height: `${el.height}%`,
                      backgroundColor: el.fillColor || undefined,
                      border: el.strokeColor ? `1px solid ${el.strokeColor}` : undefined,
                      borderRadius: el.shapeType === 'circle' ? '50%' : el.shapeType === 'rounded' ? '12px' : '4px',
                      color: el.textColor || (theme === 'mercury-canvas' ? '#28253b' : '#ffffff'),
                      textAlign: el.align || 'left',
                      fontSize: `${el.fontSize || 16}px`,
                      fontWeight: el.bold ? 600 : 400,
                    }}
                  >
                    <div
                      contentEditable
                      suppressContentEditableWarning
                      onBlur={(e) => handleElementTextChange(el.id, e.currentTarget.innerText)}
                      className="w-full h-full outline-none leading-relaxed"
                    >
                      {el.text}
                    </div>

                    {/* Resize / Selection Handles */}
                    {isSelected && (
                      <>
                        <div className="absolute -top-1.5 -left-1.5 w-3 h-3 bg-white border border-mercury-primary rounded-full cursor-nw-resize" />
                        <div className="absolute -top-1.5 -right-1.5 w-3 h-3 bg-white border border-mercury-primary rounded-full cursor-ne-resize" />
                        <div className="absolute -bottom-1.5 -left-1.5 w-3 h-3 bg-white border border-mercury-primary rounded-full cursor-sw-resize" />
                        <div className="absolute -bottom-1.5 -right-1.5 w-3 h-3 bg-white border border-mercury-primary rounded-full cursor-se-resize" />
                      </>
                    )}
                  </div>
                );
              })}
            </div>
          </div>

          {/* Panel 3: Bottom Speaker Notes Bar */}
          <div className="h-28 bg-white border-t border-mercury-fog/40 p-3 flex flex-col gap-1.5">
            <div className="flex items-center justify-between text-xs font-semibold text-mercury-slate">
              <span>Speaker Notes (Slide {activeSlideIndex + 1})</span>
              <span className="text-[10px] text-gray-400">Only visible to presenter</span>
            </div>
            <textarea
              value={activeSlide.speakerNotes}
              onChange={(e) => handleSpeakerNotesChange(e.target.value)}
              placeholder="Click to add speaker notes for this slide..."
              className="flex-1 w-full p-2 text-xs text-mercury-ink border border-mercury-fog/30 rounded resize-none outline-none focus:border-mercury-primary"
            />
          </div>
        </div>
      </div>

      {/* ─── Full-Screen Slideshow Mode Overlay ─── */}
      {isPlayingSlideshow && (
        <div
          className="fixed inset-0 z-50 bg-black flex flex-col items-center justify-center cursor-default"
          onMouseMove={(e) => setMousePos({ x: e.clientX, y: e.clientY })}
        >
          {/* Slideshow Top Bar */}
          <div className="absolute top-4 right-4 flex items-center gap-3 text-white/70 z-50">
            <button
              onClick={() => setLaserActive(!laserActive)}
              className={`p-2 rounded-full border border-white/20 hover:bg-white/20 ${
                laserActive ? 'bg-red-600 text-white border-red-600' : ''
              }`}
              title="Virtual Laser Pointer"
            >
              <Compass className="w-4 h-4" />
            </button>
            <span className="text-xs">
              Slide {slideshowSlideIdx + 1} of {slides.length}
            </span>
            <button
              onClick={() => setIsPlayingSlideshow(false)}
              className="p-1.5 rounded-full hover:bg-white/20 text-white"
            >
              <X className="w-5 h-5" />
            </button>
          </div>

          {/* Active Slideshow Slide Frame */}
          <div
            className="w-11/12 max-w-6xl aspect-video rounded-xl shadow-2xl relative overflow-hidden p-12 border border-white/10"
            style={{ backgroundColor: getThemeBackground() }}
          >
            {slides[slideshowSlideIdx]?.elements.map((el) => (
              <div
                key={el.id}
                className="absolute p-4 rounded"
                style={{
                  left: `${el.x}%`,
                  top: `${el.y}%`,
                  width: `${el.width}%`,
                  height: `${el.height}%`,
                  backgroundColor: el.fillColor || undefined,
                  border: el.strokeColor ? `1px solid ${el.strokeColor}` : undefined,
                  borderRadius: el.shapeType === 'circle' ? '50%' : el.shapeType === 'rounded' ? '12px' : '4px',
                  color: el.textColor || (theme === 'mercury-canvas' ? '#28253b' : '#ffffff'),
                  textAlign: el.align || 'left',
                  fontSize: `${(el.fontSize || 16) * 1.3}px`,
                  fontWeight: el.bold ? 600 : 400,
                }}
              >
                {el.text}
              </div>
            ))}

            {/* Virtual Laser Pointer dot */}
            {laserActive && (
              <div
                className="fixed w-3 h-3 rounded-full bg-red-500 shadow-[0_0_12px_#ff0000] pointer-events-none z-50"
                style={{ left: mousePos.x - 6, top: mousePos.y - 6 }}
              />
            )}
          </div>

          {/* Navigation Arrows */}
          <div className="absolute bottom-6 flex items-center gap-4 text-white/70">
            <button
              onClick={() => {
                if (slideshowSlideIdx > 0) setSlideshowSlideIdx((prev) => prev - 1);
              }}
              disabled={slideshowSlideIdx === 0}
              className="p-2 rounded-full bg-white/10 hover:bg-white/20 disabled:opacity-30"
            >
              <ChevronLeft className="w-5 h-5" />
            </button>
            <span className="text-xs">Press Space or Right Arrow to advance</span>
            <button
              onClick={() => {
                if (slideshowSlideIdx < slides.length - 1) setSlideshowSlideIdx((prev) => prev + 1);
              }}
              disabled={slideshowSlideIdx === slides.length - 1}
              className="p-2 rounded-full bg-white/10 hover:bg-white/20 disabled:opacity-30"
            >
              <ChevronRight className="w-5 h-5" />
            </button>
          </div>
        </div>
      )}

      {/* ─── Zone 5: Bottom Status Bar (28px) ─── */}
      <footer className="wps-status-bar">
        <div className="flex items-center gap-4">
          <span>Slide {activeSlideIndex + 1} of {slides.length}</span>
          <span className="capitalize">Theme: {theme.replace('-', ' ')}</span>
          <span className="capitalize">Transition: {activeSlide.transition || 'None'}</span>
        </div>

        <div className="flex items-center gap-3">
          <button
            onClick={() => {
              setSlideshowSlideIdx(activeSlideIndex);
              setIsPlayingSlideshow(true);
            }}
            className="text-xs text-white bg-mercury-primary px-2.5 py-0.5 rounded-full hover:bg-indigo-600 flex items-center gap-1"
          >
            <Play className="w-3 h-3 fill-white" />
            <span>Play Slideshow</span>
          </button>
        </div>
      </footer>
    </div>
  );
};
