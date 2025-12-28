import { createContext } from "react"
import type { User } from "../../api/twittra.schemas.ts"

export const UserContext = createContext<User | undefined>(undefined)
