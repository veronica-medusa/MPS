export type DocumentType = 'writer' | 'sheets' | 'slides' | 'pdf';

export interface Collaborator {
  id: string;
  name: string;
  color: string;
  avatar: string;
}

export interface DocumentTab {
  id: string;
  title: string;
  type: DocumentType;
  isDirty: boolean;
  filePath?: string;
  zoom: number;
  collaborators: Collaborator[];
  activeSheet?: string;
  currentPage?: number;
  totalPages?: number;
  currentSlide?: number;
  totalSlides?: number;
  wordCount?: number;
  activeCell?: string;
  selectionSummary?: string;
}

export type RibbonMode = 'classic' | 'compact';

export type ActiveRibbonTab =
  | 'home'
  | 'insert'
  | 'layout'
  | 'formulas'
  | 'data'
  | 'toolbox'
  | 'slides'
  | 'transitions'
  | 'animations'
  | 'pdf_annotate'
  | 'pdf_edit'
  | 'pdf_organize'
  | 'pdf_convert'
  | 'review'
  | 'view'
  | 'ai';

export interface CommandItem {
  id: string;
  title: string;
  category: string;
  shortcut?: string;
  icon?: string;
  action: () => void;
}

export type SmartToolboxAction =
  | 'merge_no_loss'
  | 'split_sheet'
  | 'combine_sheets'
  | 'compare_sheets'
  | 'data_hygiene';
