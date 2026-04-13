const BASE = '/api';

export interface QueryResponse {
  question: string;
  sql: string;
  columns: string[];
  rows: (string | number | null)[][];
  error: string | null;
  provider?: string | null;
}

export interface SavedQuery {
  id?: number;
  question: string;
  sql_query: string;
  created_at?: string;
  updated_at?: string;
  color?: string | null;
}

async function checked<T>(res: Response): Promise<T> {
  if (!res.ok) {
    const text = await res.text().catch(() => res.statusText);
    throw new Error(`${res.status}: ${text}`);
  }
  return res.json();
}

export async function submitQuery(question: string): Promise<QueryResponse> {
  const res = await fetch(`${BASE}/query`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ question }),
  });
  return checked(res);
}

export async function getSavedQueries(): Promise<SavedQuery[]> {
  const res = await fetch(`${BASE}/saved-queries`);
  return checked(res);
}

export async function createSavedQuery(q: SavedQuery): Promise<SavedQuery> {
  const res = await fetch(`${BASE}/saved-queries`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(q),
  });
  return checked(res);
}

export async function updateSavedQuery(id: number, q: SavedQuery): Promise<SavedQuery> {
  const res = await fetch(`${BASE}/saved-queries/${id}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(q),
  });
  return checked(res);
}

export async function deleteSavedQuery(id: number): Promise<void> {
  const res = await fetch(`${BASE}/saved-queries/${id}`, { method: 'DELETE' });
  if (!res.ok) {
    const text = await res.text().catch(() => res.statusText);
    throw new Error(`${res.status}: ${text}`);
  }
}

export async function executeDirect(question: string, sql: string): Promise<QueryResponse> {
  const res = await fetch(`${BASE}/execute`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ question, sql }),
  });
  return checked(res);
}

export async function getTables(): Promise<string[]> {
  const res = await fetch(`${BASE}/tables`);
  return checked(res);
}

export async function getTableData(name: string): Promise<QueryResponse> {
  const res = await fetch(`${BASE}/tables/${name}`);
  return checked(res);
}
