import {
  AppShell,
  Box,
  Center,
  Group,
  SimpleGrid,
  Stack,
  Text,
} from "@mantine/core"
import { useMediaQuery } from "@mantine/hooks"
import { IconClick } from "@tabler/icons-react"
import { Outlet, useMatches } from "@tanstack/react-router"
import { useUser } from "../auth/hooks/useUser.ts"
import { Timeline } from "../timeline/components/Timeline.tsx"
import { UserAvatar } from "./UserAvatar.tsx"

const ChannelPlaceholder = () => (
  <Center h="100%">
    <Stack align="center" gap="md">
      <IconClick size={48} stroke={1.5} color="var(--mantine-color-dimmed)" />
      <Text c="dimmed" ta="center">
        左のタイムラインからメッセージをクリックすると、
        <br />
        そのチャンネルの投稿が表示されます
      </Text>
    </Stack>
  </Center>
)

export const Layout = () => {
  const user = useUser()
  const matches = useMatches()
  const isDesktop = useMediaQuery("(min-width: 768px)")

  // Check if we're on a channel route (not index)
  const isChannelRoute = matches.some((m) => m.routeId.includes("channels"))

  // On mobile, show only Timeline on index, only ChannelView on channel route
  // On desktop, always show split view
  const showTimeline = isDesktop || !isChannelRoute

  return (
    <AppShell
      navbar={{ breakpoint: "sm", collapsed: { mobile: true }, width: 256 }}
      padding="md"
    >
      <AppShell.Navbar>
        <AppShell.Section grow p="md">
          Home
        </AppShell.Section>

        <AppShell.Section p="md">
          <Group gap="sm" wrap="nowrap">
            <UserAvatar userId={user.id} username={user.handle} />

            <Stack gap={0}>
              <Text fw={500} span>
                {user.displayName}
              </Text>

              <Text c="dimmed" span>
                @{user.handle}
              </Text>
            </Stack>
          </Group>
        </AppShell.Section>
      </AppShell.Navbar>

      {/* Disable padding as we use virtual scrolling (scrollbar placed inside content) */}
      <AppShell.Main pr={0} py={0}>
        {isDesktop ? (
          <SimpleGrid cols={2} h="100%" spacing={0}>
            <Box
              h="100%"
              style={{ borderRight: "1px solid var(--mantine-color-dark-4)" }}
            >
              <Timeline />
            </Box>
            <Box h="100%">
              {isChannelRoute ? <Outlet /> : <ChannelPlaceholder />}
            </Box>
          </SimpleGrid>
        ) : showTimeline ? (
          <Timeline />
        ) : (
          <Outlet />
        )}
      </AppShell.Main>
    </AppShell>
  )
}
