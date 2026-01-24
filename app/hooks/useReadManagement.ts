import { useEffect, useRef } from "react"
import { useMarkMessagesAsRead } from "../../api/message/message.ts"

/**
 * Custom hook to manage marking messages as read with batching and debouncing.
 * It collects message IDs to be marked as read and sends them in batches
 * after a delay, optimizing network requests and memory usage.
 */
export const useReadManagement = () => {
  const { mutate } = useMarkMessagesAsRead()
  const pendingIdsRef = useRef<Set<string>>(new Set())
  /** Set of message IDs that have already been sent to avoid unnecessary re-sends */
  const sentIdsRef = useRef<Set<string>>(new Set())
  const timerRef = useRef<number | null>(null)

  const flush = () => {
    if (pendingIdsRef.current.size === 0) {
      return
    }

    mutate({ data: { message_ids: Array.from(pendingIdsRef.current) } })

    // Clear sent IDs if they grow too large.
    if (sentIdsRef.current.size > 5000) {
      sentIdsRef.current.clear()
    }

    // Move pending IDs to sent IDs
    sentIdsRef.current = sentIdsRef.current.union(pendingIdsRef.current)
    pendingIdsRef.current.clear()
    timerRef.current = null
  }

  const markAsRead = (id: string) => {
    if (sentIdsRef.current.has(id) || pendingIdsRef.current.has(id)) {
      return
    }

    pendingIdsRef.current.add(id)

    if (!timerRef.current) {
      timerRef.current = setTimeout(flush, 2000)
    }
  }

  // Flush on unmount
  useEffect(() => {
    return () => {
      if (timerRef.current) {
        clearTimeout(timerRef.current)
        flush()
      }
    }
  }, [])

  return { markAsRead }
}
