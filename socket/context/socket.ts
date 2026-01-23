import { createContext } from "react"
import type { AppSocket } from "../lib/types.ts"

export const SocketContext = createContext<AppSocket | undefined>(undefined)
