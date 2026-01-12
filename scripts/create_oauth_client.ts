/// <reference lib="deno.ns" />

const TRAQ_BASE_URL = "http://localhost:3000/api/v3"
const CALLBACK_URL = "http://localhost:5173/api/v1/auth/callback"

// 1. Login
const loginRes = await fetch(`${TRAQ_BASE_URL}/login`, {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({ name: "traq", password: "traq" }),
})

if (!loginRes.ok) {
  console.error(
    `Login failed: ${loginRes.status} ${loginRes.statusText}`,
    await loginRes.text(),
  )
  Deno.exit(1)
}

const cookie = loginRes.headers.get("set-cookie")
if (!cookie) {
  console.error("No cookie received")
  Deno.exit(1)
}

// 2. Create Client
const clientRes = await fetch(`${TRAQ_BASE_URL}/clients`, {
  method: "POST",
  headers: {
    "Content-Type": "application/json",
    "Cookie": cookie,
  },
  body: JSON.stringify({
    name: "Twittra Dev",
    description: "Development client for Twittra",
    callbackUrl: CALLBACK_URL,
    scopes: ["read", "write", "manage_bot"],
  }),
})

if (!clientRes.ok) {
  console.error("Failed to create client:", await clientRes.text())
  Deno.exit(1)
}

const client = await clientRes.json()
console.log("TRAQ_CLIENT_ID=" + client.id)
console.log("TRAQ_CLIENT_SECRET=" + client.secret)
