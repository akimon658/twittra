import { useContext } from "react"
import { UserContext } from "../context/user.ts"

/**
 * @throws {Error} if used outside of an AuthProvider
 */
export const useUser = () => {
  const context = useContext(UserContext)

  if (!context) {
    throw new Error("useUser must be used within an AuthProvider")
  }

  return context
}
