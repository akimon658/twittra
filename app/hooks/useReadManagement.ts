import { useEffect, useRef } from 'react'
import { useMarkMessagesAsRead } from '../../api/message/message'

export function useReadManagement() {
    const { mutate } = useMarkMessagesAsRead()
    const pendingIdsRef = useRef<Set<string>>(new Set())
    const sentIdsRef = useRef<Set<string>>(new Set())
    const timerRef = useRef<number | null>(null)

    const flush = () => {
        const ids = Array.from(pendingIdsRef.current)
        if (ids.length === 0) return

        mutate({ data: { message_ids: ids } })

        // Memory usage optimization: Clear sent IDs if they grow too large.
        // This prevents memory leaks in long-running sessions (e.g., infinite scroll).
        if (sentIdsRef.current.size > 5000) {
            sentIdsRef.current.clear()
        }

        // Use Set.prototype.union (ESNext) to merge pending IDs into sent IDs
        sentIdsRef.current = sentIdsRef.current.union(pendingIdsRef.current)
        pendingIdsRef.current.clear()
        timerRef.current = null
    }

    const markAsRead = (id: string) => {
        if (sentIdsRef.current.has(id) || pendingIdsRef.current.has(id)) return

        pendingIdsRef.current.add(id)

        if (!timerRef.current) {
            timerRef.current = window.setTimeout(flush, 2000)
        }
    }

    // Flush on unmount
    useEffect(() => {
        return () => {
            if (timerRef.current) {
                window.clearTimeout(timerRef.current)
                flush()
            }
        }
    }, [])

    return { markAsRead }
}
