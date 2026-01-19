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
      mock: {
        type: "msw",
        delay: 100,
      },
      override: {
        fetch: {
          forceSuccessResponse: true,
          jsonReviver: {
            path: "./api/reviver.ts",
            name: "customReviver",
          },
        },
        query: {
          useSuspenseQuery: true,
          useQuery: false,
        },
        useDates: true,
      },
      target: "./api/twittra.ts",
    },
  },
})
