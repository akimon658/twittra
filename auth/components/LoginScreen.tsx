import { Button, Center, Container } from "@mantine/core"
import { TraqLogo } from "../../components/TraqLogo.tsx"

export const LoginScreen = () => {
  return (
    <Container>
      <Center h="100dvh">
        <Button
          component="a"
          fullWidth
          href="/api/v1/auth/login"
          leftSection={<TraqLogo size={24} />}
          size="xl"
        >
          traQでログイン
        </Button>
      </Center>
    </Container>
  )
}
