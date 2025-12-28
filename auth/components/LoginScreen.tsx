import { Button, Center, Container } from "@mantine/core"

export const LoginScreen = () => {
  return (
    <Container>
      <Center h="100dvh">
        <Button component="a" fullWidth href="/api/v1/auth/login" size="xl">
          traQでログイン
        </Button>
      </Center>
    </Container>
  )
}
