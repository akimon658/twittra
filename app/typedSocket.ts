import type { Socket } from "socket.io-client"
import type { ServerEvent } from "../api/twittra.schemas.ts"

// Automatically derive event map from ServerEvent discriminated union
type ServerToClientEvents = {
    [E in ServerEvent as E["type"]]: (payload: E["payload"]) => void
}

// Typed wrapper around Socket.io client
export type TypedSocket = Socket<ServerToClientEvents>

