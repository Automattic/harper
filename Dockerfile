ARG NODE_VERSION=slim

FROM rust:latest AS wasm-build
RUN rustup toolchain install stable-x86_64-unknown-linux-gnu

RUN mkdir -p /usr/build/
WORKDIR /usr/build/

RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

COPY . .

WORKDIR /usr/build/harper-wasm
RUN wasm-pack build --release --target web

FROM node:${NODE_VERSION} AS node-build

RUN apt-get update && apt-get install git pandoc -y

RUN mkdir -p /usr/build/
WORKDIR /usr/build/

RUN mkdir harper-wasm

COPY --from=wasm-build /usr/build/harper-wasm/pkg /usr/build/harper-wasm/pkg
COPY packages packages
COPY demo.md .

WORKDIR /usr/build/packages/harper.js

RUN yarn install && yarn build && ./docs.sh

WORKDIR /usr/build/packages/web

RUN yarn install && yarn build

FROM node:${NODE_VERSION}

COPY --from=node-build /usr/build/packages/web/build /usr/build/packages/web/build
COPY --from=node-build /usr/build/packages/web/package.json /usr/build/packages/web/package.json

WORKDIR /usr/build/packages/web

RUN yarn install

ENV HOST=0.0.0.0
ENV PORT=3000

ENTRYPOINT ["node", "build"]
