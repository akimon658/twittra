import { useInfiniteQuery } from "@tanstack/react-query"
import { useMemo } from "react"
import {
  getGetTimelineQueryKey,
  getTimeline,
} from "../../api/timeline/timeline.ts"
import type { MessageListItem } from "../../api/twittra.schemas.ts"

const MAX_PAGES = 10 // Memory optimization: keep at most 10 pages (200 items)

export interface UseTimelineInfiniteResult {
  /** Flattened list of all messages from all pages */
  messages: MessageListItem[]
  /** Fetch next page (scroll down) */
  fetchNextPage: () => void
  /** Fetch previous page (scroll up) */
  fetchPreviousPage: () => void
  /** Loading states */
  isFetchingNextPage: boolean
  isFetchingPreviousPage: boolean
  hasNextPage: boolean
  hasPreviousPage: boolean
  /** Overall query status */
  isLoading: boolean
  error: Error | null
}

/**
 * Hook for bidirectional infinite scrolling of timeline.
 *
 * This uses `useInfiniteQuery` with `maxPages` to limit memory usage.
 * Since timeline is recommendation-based (not chronological), we always
 * allow fetching more in both directions.
 */
export function useTimelineInfinite(): UseTimelineInfiniteResult {
  const query = useInfiniteQuery({
    queryKey: getGetTimelineQueryKey(),
    queryFn: async ({ signal }) => {
      const response = await getTimeline({ signal })
      return response
    },
    // Always allow fetching in both directions since content is recommendation-based
    getNextPageParam: () => true,
    getPreviousPageParam: () => true,
    initialPageParam: undefined,
    maxPages: MAX_PAGES,
    // Disable automatic refetching as timeline is dynamic
    refetchOnMount: false,
    refetchOnReconnect: false,
    refetchOnWindowFocus: false,
  })

  // Flatten all pages into a single message array
  const messages = useMemo(() => {
    if (!query.data) return []
    return query.data.pages.flatMap((page) => page.data)
  }, [query.data])

  return {
    messages,
    fetchNextPage: () => query.fetchNextPage(),
    fetchPreviousPage: () => query.fetchPreviousPage(),
    isFetchingNextPage: query.isFetchingNextPage,
    isFetchingPreviousPage: query.isFetchingPreviousPage,
    hasNextPage: query.hasNextPage,
    hasPreviousPage: query.hasPreviousPage,
    isLoading: query.isLoading,
    error: query.error,
  }
}
