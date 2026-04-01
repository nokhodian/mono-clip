export function relativeTime(dateStr: string): string {
  // SQLite stores UTC but without a 'Z' suffix — append it so JS parses correctly
  const normalized = dateStr.endsWith("Z") ? dateStr : dateStr + "Z";
  const date = new Date(normalized);
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const seconds = Math.floor(diff / 1000);

  if (seconds < 60) return "just now";
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
  if (seconds < 604800) return `${Math.floor(seconds / 86400)}d ago`;

  return date.toLocaleDateString("en-US", { month: "short", day: "numeric" });
}

export function formatDate(dateStr: string): string {
  const normalized = dateStr.endsWith("Z") ? dateStr : dateStr + "Z";
  return new Date(normalized).toLocaleDateString("en-US", {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}
