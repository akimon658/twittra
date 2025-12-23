import { defineConfig } from "orval"

export default defineConfig({
  twittra: {
    input: "./api/openapi.json",
    hooks: {
      afterAllFilesWrite: "deno fmt",
    },
    output: {
      baseUrl: "/api/v1",
      client: "react-query",
      httpClient: "fetch",
      mode: "tags-split",
      override: {
        fetch: {
          includeHttpResponseReturnType: false,
        },
        query: {
          useSuspenseQuery: true,
          useQuery: false,
        },
      },
      target: "./api/twittra.ts",
    },
  },
})
