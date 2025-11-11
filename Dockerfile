# This Dockerfile is for the Harper website and web services.
# You do not need it to use Harper.

ARG NODE_VERSION=24

FROM rust:latest AS wasm-build
RUN rustup toolchain install

RUN mkdir -p /usr/build/
WORKDIR /usr/build/

RUN cargo install wasm-pack

COPY . .

WORKDIR /usr/build/harper-wasm
RUN wasm-pack build --target web

FROM node:${NODE_VERSION} AS node-build

RUN apt-get update && apt-get install git parallel -y
RUN corepack enable

RUN mkdir -p /usr/build/
WORKDIR /usr/build/

COPY . .
COPY --from=wasm-build /usr/build/harper-wasm/pkg /usr/build/harper-wasm/pkg

RUN pnpm install --shamefully-hoist

WORKDIR /usr/build/packages/harper.js

RUN pnpm build && ./docs.sh

WORKDIR /usr/build/packages/lint-framework
RUN pnpm build

WORKDIR /usr/build/packages/web
RUN pnpm install --shamefully-hoist
RUN pnpm build

FROM node:${NODE_VERSION}

COPY --from=node-build /usr/build/node_modules /usr/build/packages/web/node_modules
COPY --from=node-build /usr/build/packages/web/build /usr/build/packages/web/build
COPY ./packages/web/drizzle /usr/build/packages/web/build/drizzle
COPY --from=node-build /usr/build/packages/web/package.json /usr/build/packages/web/package.json

WORKDIR /usr/build/packages/web/build

ENV HOST=0.0.0.0
ENV PORT=3000

ENTRYPOINT ["node", "index"]
