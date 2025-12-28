import { QueryErrorResetBoundary } from "@tanstack/react-query"
import { type PropsWithChildren, Suspense } from "react"
import { ErrorBoundary } from "react-error-boundary"
import { useGetMeSuspense } from "../../api/user/user.ts"
import { UserContext } from "../context/user.ts"

const AuthUserContextProvider = ({ children }: PropsWithChildren) => {
  const { data: { data } } = useGetMeSuspense()

  return (
    <UserContext.Provider value={data}>
      {children}
    </UserContext.Provider>
  )
}

/**
 * @throws {Error} if `/api/v1/me` returns an error other than 401 Unauthorized
 */
export const AuthProvider = ({ children }: PropsWithChildren) => {
  return (
    <QueryErrorResetBoundary>
      {({ reset }) => (
        <ErrorBoundary
          onReset={reset}
          fallbackRender={({ error }) => {
            if (error.status !== 401) {
              throw new Error(
                `retrieving authenticated user`,
                { cause: error },
              )
            }

            return <a href="/api/v1/auth/login">Login</a>
          }}
        >
          <Suspense fallback={<div>Loading...</div>}>
            <AuthUserContextProvider>
              {children}
            </AuthUserContextProvider>
          </Suspense>
        </ErrorBoundary>
      )}
    </QueryErrorResetBoundary>
  )
}
