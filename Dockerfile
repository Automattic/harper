ARG NODE_VERSION=slim

FROM rust:latest AS wasm-build
RUN rustup toolchain install

RUN mkdir -p /usr/build/
WORKDIR /usr/build/

RUN cargo install wasm-pack

COPY . .

WORKDIR /usr/build/harper-wasm
RUN wasm-pack build --release --target web

FROM node:${NODE_VERSION} AS node-build

RUN apt-get update && apt-get install git pandoc parallel -y
RUN corepack enable

RUN mkdir -p /usr/build/
WORKDIR /usr/build/

COPY . .
COPY --from=wasm-build /usr/build/harper-wasm/pkg /usr/build/harper-wasm/pkg

RUN pnpm install

WORKDIR /usr/build/packages/harper.js

RUN pnpm build && ./docs.sh

WORKDIR /usr/build/packages/web

RUN pnpm build

FROM node:${NODE_VERSION}

COPY --from=node-build /usr/build/packages/web/build /usr/build/packages/web/build
COPY --from=node-build /usr/build/packages/web/package.json /usr/build/packages/web/package.json

WORKDIR /usr/build/packages/web/build

ENV HOST=0.0.0.0
ENV PORT=3000

ENTRYPOINT ["node", "index"]
