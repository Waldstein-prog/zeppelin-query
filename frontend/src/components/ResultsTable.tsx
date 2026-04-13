import type { QueryResponse } from '../api';

interface Props {
  result: QueryResponse | null;
}

export default function ResultsTable({ result }: Props) {
  if (!result) return null;

  return (
    <div className="results">
      <div className="sql-display">
        <strong>SQL:</strong>
        <code>{result.sql}</code>
      </div>

      {result.error && (
        <div className="error">
          {result.error}
        </div>
      )}

      {result.columns.length > 0 && (
        <div className="table-wrapper">
          <table>
            <thead>
              <tr>
                {result.columns.map((col, i) => (
                  <th key={i}>{col}</th>
                ))}
              </tr>
            </thead>
            <tbody>
              {result.rows.map((row, i) => (
                <tr key={i}>
                  {row.map((cell, j) => (
                    <td key={j}>{cell === null ? '-' : String(cell)}</td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
          <div className="row-count">{result.rows.length} rij(en)</div>
        </div>
      )}
    </div>
  );
}
