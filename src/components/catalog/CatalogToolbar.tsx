import { Search, X } from 'lucide-react'
import type { CatalogSort } from './sort'
import { CATALOG_SORTS } from './sort'

export function CatalogToolbar({
  query,
  onQueryChange,
  sort,
  onSortChange,
}: {
  query: string
  onQueryChange: (v: string) => void
  sort: CatalogSort
  onSortChange: (v: CatalogSort) => void
}) {
  return (
    <>
      <div className="relative w-full max-w-56 min-w-32">
        <Search size={13} className="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-ink-faint" />
        <input
          data-catalog-search
          value={query}
          onChange={(e) => onQueryChange(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Escape') {
              e.stopPropagation()
              if (query) onQueryChange('')
              else e.currentTarget.blur()
            }
          }}
          placeholder="Search threads…"
          className="w-full rounded-lg border border-transparent bg-surface-3 py-1.5 pl-8 pr-7 text-xs text-ink outline-none transition-colors placeholder:text-ink-faint focus:border-accent/50"
        />
        {query && (
          <button
            type="button"
            onClick={() => onQueryChange('')}
            title="Clear search"
            className="absolute right-1.5 top-1/2 flex size-5 -translate-y-1/2 items-center justify-center rounded-md text-ink-faint transition-colors hover:text-ink"
          >
            <X size={12} />
          </button>
        )}
      </div>

      <select
        value={sort}
        onChange={(e) => onSortChange(e.target.value as CatalogSort)}
        title="Sort threads"
        className="cursor-pointer rounded-lg border border-transparent bg-surface-3 py-1.5 pl-2.5 pr-1.5 text-xs text-ink-dim outline-none transition-colors hover:text-ink focus:border-accent/50"
      >
        {CATALOG_SORTS.map((s) => (
          <option key={s.value} value={s.value}>
            {s.label}
          </option>
        ))}
      </select>
    </>
  )
}
