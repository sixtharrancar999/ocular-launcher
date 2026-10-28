interface ProgressProps {
  value: number;
  label?: string;
  detail?: string;
  indeterminate?: boolean;
}

export function Progress({ value, label, detail, indeterminate }: ProgressProps) {
  const percent = Math.max(0, Math.min(100, value));
  return (
    <div className="progress-block">
      {(label || detail) && (
        <div className="progress-meta">
          <span>{label}</span>
          <span className="progress-detail">{detail}</span>
        </div>
      )}
      <div className={`progress ${indeterminate ? "indeterminate" : ""}`}>
        <div className="progress-fill" style={{ width: indeterminate ? "35%" : `${percent}%` }} />
      </div>
    </div>
  );
}

export function formatBytes(bytes: number): string {
  if (!bytes) {
    return "";
  }
  const mb = bytes / (1024 * 1024);
  if (mb >= 1024) {
    return `${(mb / 1024).toFixed(2)} ГБ`;
  }
  return `${mb.toFixed(mb >= 100 ? 0 : 1)} МБ`;
}
