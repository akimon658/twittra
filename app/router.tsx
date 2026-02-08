import {
    Outlet,
    createRootRoute,
    createRoute,
    createRouter,
} from "@tanstack/react-router"

import { Layout } from "../components/Layout.tsx"
import { ChannelView } from "../timeline/components/ChannelView.tsx"

const rootRoute = createRootRoute({
    component: () => <Layout />,
})

const indexRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "/",
    component: () => null,
})

const channelRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "/channels/$channelId",
    validateSearch: (search: Record<string, unknown>) => ({
        messageId: typeof search.messageId === "string" ? search.messageId : undefined,
    }),
    component: () => <ChannelView />,
})

const routeTree = rootRoute.addChildren([indexRoute, channelRoute])

export const router = createRouter({
    routeTree,
    defaultPreload: "intent",
})

declare module "@tanstack/react-router" {
    interface Register {
        router: typeof router
    }
}
