export interface Section {
  id: string;
  heading: string;
  level: number;
  content: string;
}

export interface TaskItem {
  id: string;
  title: string;
  status: "todo" | "in_progress" | "done" | "blocked";
  acceptance_checks: string[];
  notes?: string;
}

export interface DecisionItem {
  date: string;
  decision: string;
  why: string;
}

export interface ContextPackOptions {
  tokenBudget?: number;
}

export interface ContextPack {
  gameSummary: string;
  currentTasks: TaskItem[];
  recentDecisions: DecisionItem[];
  mapOverview: Record<string, unknown> | null;
  memoryNotes?: string[];
  totalEstimatedTokens: number;
}

export interface MergeConflict {
  sectionId: string;
  baseContent: string;
  userContent: string;
  agentContent: string;
}

export interface MergeResult {
  mergedContent: string;
  conflicts: MergeConflict[];
}

export interface ProjectBrainContract {
  recordKnowledge(topic: string, content: string): Promise<void>;
  queryKnowledge(query: string): Promise<string[]>;
  getContextSnapshot(): Promise<Record<string, unknown>>;
}
