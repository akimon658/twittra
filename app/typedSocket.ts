import type { Socket } from "socket.io-client"
import type { ClientEvent, ServerEvent } from "../api/twittra.schemas.ts"

// Automatically derive server-to-client event map from ServerEvent discriminated union
type ServerToClientEvents = {
  [E in ServerEvent as E["type"]]: (payload: E["payload"]) => void
}

// Automatically derive client-to-server event map from ClientEvent discriminated union
type ClientToServerEvents = {
  [E in ClientEvent as E["type"]]: (payload: E["payload"]) => void
}

// Typed wrapper around Socket.io client
export type TypedSocket = Socket<ServerToClientEvents, ClientToServerEvents>
