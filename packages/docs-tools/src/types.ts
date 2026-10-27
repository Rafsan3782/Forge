export interface DocPageMeta {
  id: string;
  title: string;
  summary: string;
  tags: string[];
  audience: "human" | "agent" | "both";
  engine_version: string;
  last_verified: string;
  related_ids: string[];
  token_count: number;
  category: "agents" | "cookbook" | "concepts" | "reference";
  file_path: string;
  content: string;
  sections?: Record<string, string>;
}

export interface DocsIndexData {
  pages: DocPageMeta[];
  recipes: Array<{
    id: string;
    title: string;
    summary: string;
    tags: string[];
    token_count: number;
    file_path: string;
  }>;
  errors: Record<
    string | number,
    {
      code: string | number;
      meaning: string;
      common_cause: string;
      fix: string;
    }
  >;
  agent_packs: Record<string, string>;
}

export interface SearchResultItem {
  id: string;
  title: string;
  summary: string;
  token_count: number;
  score: number;
}

export interface ReadDocResult {
  id: string;
  title: string;
  section?: string;
  content: string;
  token_count: number;
  truncated: boolean;
}

export interface ErrorHelpResult {
  code: string | number;
  meaning: string;
  common_cause: string;
  fix: string;
}
