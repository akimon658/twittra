FROM denoland/deno:alpine-2.6.6 AS builder

WORKDIR /usr/src/app

COPY . .

RUN deno task build

FROM caddy:2.10.2-alpine

COPY --from=builder /usr/src/app/dist /usr/share/caddy
COPY Caddyfile /etc/caddy/Caddyfile

EXPOSE 80
