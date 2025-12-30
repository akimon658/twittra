import { AppShell, Avatar, Group, Stack, Text } from "@mantine/core"
import { useUser } from "../auth/hooks/useUser.ts"

export const Layout = () => {
  const user = useUser()

  return (
    <AppShell navbar={{ breakpoint: "sm", width: 256 }} padding="md">
      <AppShell.Navbar>
        <AppShell.Section grow p="md">Home</AppShell.Section>

        <AppShell.Section p="md">
          <Group gap="sm" wrap="nowrap">
            <Avatar
              alt={`@${user.handle}のアイコン`}
              src={`https://image-proxy.trap.jp/icon/${user.handle}?width=128`}
            />

            <Stack gap={0}>
              <Text fw={700} span>{user.displayName}</Text>

              <Text c="dimmed" span>@{user.handle}</Text>
            </Stack>
          </Group>
        </AppShell.Section>
      </AppShell.Navbar>

      <AppShell.Main>
        {/* TODO: Replace this comment with `<Outlet />` */}
      </AppShell.Main>
    </AppShell>
  )
}
