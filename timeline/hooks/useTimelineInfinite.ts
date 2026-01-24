import { useInfiniteQuery } from "@tanstack/react-query"
import {
  getGetTimelineQueryKey,
  getTimeline,
} from "../../api/timeline/timeline.ts"

const MAX_PAGES = 10 // Memory optimization: keep at most 10 pages (200 items)

/**
 * Hook for bidirectional infinite scrolling of timeline.
 *
 * This uses `useInfiniteQuery` with `maxPages` to limit memory usage.
 * Since timeline is recommendation-based (not chronological), we always
 * allow fetching more in both directions.
 */
export const useTimelineInfinite = () => {
  const query = useInfiniteQuery({
    queryKey: getGetTimelineQueryKey(),
    queryFn: ({ signal }) => getTimeline({ signal }),
    getNextPageParam: (lastPage) =>
      lastPage?.data && lastPage.data.length > 0 ? true : undefined,
    initialPageParam: undefined,
    maxPages: MAX_PAGES,
    // Disable automatic refetching as timeline is dynamic
    refetchOnMount: false,
    refetchOnReconnect: false,
    refetchOnWindowFocus: false,
  })

  // Flatten all pages into a single message array
  const messages = query.data?.pages.flatMap((page) => page.data) ?? []

  return {
    messages,
    ...query,
  }
}
