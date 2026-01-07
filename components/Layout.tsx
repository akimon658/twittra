import { AppShell, Group, Stack, Text } from "@mantine/core"
import { useUser } from "../auth/hooks/useUser.ts"
import { Timeline } from "../timeline/components/Timeline.tsx"
import { UserAvatar } from "./UserAvatar.tsx"

export const Layout = () => {
  const user = useUser()

  return (
    <AppShell navbar={{ breakpoint: "sm", width: 256 }} padding="md">
      <AppShell.Navbar>
        <AppShell.Section grow p="md">Home</AppShell.Section>

        <AppShell.Section p="md">
          <Group gap="sm" wrap="nowrap">
            <UserAvatar username={user.handle} />

            <Stack gap={0}>
              <Text fw={500} span>{user.displayName}</Text>

              <Text c="dimmed" span>@{user.handle}</Text>
            </Stack>
          </Group>
        </AppShell.Section>
      </AppShell.Navbar>

      <AppShell.Main>
        {/* TODO: Replace below with `<Outlet />` */}
        <Timeline />
      </AppShell.Main>
    </AppShell>
  )
}
