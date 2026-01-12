# Twittra

**Twittra** is an unofficial, Twitter-like client for
[**traQ**](https://github.com/traPtitech/traQ) (a Slack-like chat platform). It
provides a cross-channel timeline view with a recommendation algorithm, focusing
on content discovery and a fluid browsing experience.

## Project Philosophy & Objectives

1. **Discovery:** Solve the issue where interesting discussions are buried in
   specific channels. Provide a unified feed based on user interests and social
   graphs.
1. **Not a Replacement:** Complement the official traQ client. Focus on timeline
   browsing rather than chat functionalities.
1. **Performance under Constraints:** Designed to run on low-resource servers
   (approx. **100MB RAM**).
1. **Modern DX:** Prioritize type safety and automated code generation between
   backend and frontend.

## Architecture Overview

### Backend (Rust)

- **Structure:** Cargo Workspace with Clean Architecture.
- `crates/app`: Entry point, API handlers (axum), DI container, OpenAPI
  definition.
- `crates/domain`: Business logic, Traits (Interfaces), Domain models. **Pure
  Rust, no external IO.**
- `crates/infra`: DB implementation (sqlx), API clients (`traq` crate wrapper),
  Concrete repositories.
- **API Definition:** **Code-first**. OpenAPI JSON is generated from Rust code
  using `utoipa`.

### Frontend (React)

- **Runtime:** Deno.
- **Fetching:** **Schema-driven**. Uses `Orval` to generate React Query hooks
  from the backend's OpenAPI JSON. Configured to generate only
  suspense-compatible hooks so that loading states are handled by React
  Suspense.
- **UI:** Mantine v8.

### Authentication & Session Management

- **OAuth2:** Users authenticate via traQ OAuth2.
- **Session:** Managed by `tower-sessions` (backed by MariaDB).

## Data & Algorithm Strategy

### Data Sync

- **Worker:** A background task in the Rust application crawls recently posted
  messages from traQ.
- **Token Strategy:** As traQ does not have a concept of private channels
  (except for DMs), there is no privacy issue in using tokens of users who
  authorized the app to crawl messages. In other words, no matter which user's
  token is used, the same set of messages can be accessed. Therefore, we use a
  random authorized user's token for crawling.
- **On-Demand Fetching:** If a user accesses a resource (e.g., user profile) not
  yet in the local DB, it is transparently fetched from traQ API and stored
  locally.

### Recommendation Scoring

The timeline is constructed by merging 4 buckets:

1. **Recency:** Latest posts.
1. **Popularity:** Posts with stamps from many users.
1. **Affinity:** Posts from users frequently interacted with.
1. **Interest:** Full-text search matches based on user-defined keywords.

## Development Setup

### Local traQ Environment

This project uses a local instance of traQ for development to ensure a
consistent environment and avoid impacting the production server.

1. Start the Docker environment:
   ```bash
   docker compose up -d
   ```
   This will start traQ, MariaDB, Elasticsearch, and other necessary services.
1. Set API Base URL: Add the local traQ API base URL to your `.env.local` file
   (first time only):
   ```bash
   echo "TRAQ_API_BASE_URL=http://localhost:3000/api/v3" >> .env.local
   ```
1. Create an OAuth2 Client and Configure Environment Variables (first time
   only): Run the following script to create an OAuth2 client on the local traQ
   instance. This script logs in to the local traQ and registers a client with
   the necessary scopes. Append the output directly to your `.env.local` file:
   ```bash
   deno run --allow-net scripts/create_oauth_client.ts >> .env.local
   ```
1. Run the Backend:
   ```bash
   cargo run -p app
   ```
