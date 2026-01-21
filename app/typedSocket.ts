import type { Socket } from "socket.io-client"
import { customReviver } from "../api/reviver.ts"
import type { ServerEvent } from "../api/twittra.schemas.ts"

// Automatically derive event map from ServerEvent discriminated union
type ServerToClientEvents = {
    [E in ServerEvent as E["type"]]: (payload: E["payload"]) => void
}

// Typed wrapper around Socket.io client
export type TypedSocket = Socket<ServerToClientEvents>

/**
 * Helper to parse and revive dates in Socket.io payloads
 */
export function revivePayload<T>(payload: unknown): T {
    // Convert to JSON and back with custom reviver to handle dates
    const jsonString = JSON.stringify(payload)
    return JSON.parse(jsonString, customReviver)
}
