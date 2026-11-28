# This Dockerfile is for the Harper website and web services.
# You do not need it to use Harper.

ARG NODE_VERSION=24

FROM rust:latest AS wasm-build
RUN rustup toolchain install
RUN apt-get update -y && apt-get install clang -y

RUN mkdir -p /usr/build/
WORKDIR /usr/build/

# Download a prebuilt wasm-pack instead of compiling it, which took about a minute. Update the
# checksums whenever the version changes.
ARG WASM_PACK_VERSION=0.15.1
RUN set -eu; \
    arch="$(uname -m)"; \
    case "$arch" in \
      x86_64) sha256=410101009739b9d01c7d5affaaad6e9c9a24b4cba6791a4805db8e89b9bf7af2 ;; \
      aarch64) sha256=7004853c7925ae1984033b5f25866ec604bbc921ef956d5e6c5595bc5af6d30c ;; \
      *) echo "No prebuilt wasm-pack for $arch" >&2; exit 1 ;; \
    esac; \
    name="wasm-pack-v${WASM_PACK_VERSION}-${arch}-unknown-linux-musl"; \
    curl -sSfL -o /tmp/wasm-pack.tar.gz \
      "https://github.com/wasm-bindgen/wasm-pack/releases/download/v${WASM_PACK_VERSION}/${name}.tar.gz"; \
    echo "${sha256}  /tmp/wasm-pack.tar.gz" | sha256sum -c -; \
    tar -xzf /tmp/wasm-pack.tar.gz -C /tmp; \
    install "/tmp/${name}/wasm-pack" /usr/local/cargo/bin/wasm-pack; \
    rm -rf /tmp/wasm-pack.tar.gz "/tmp/${name}"

COPY . .

WORKDIR /usr/build/harper-wasm
RUN wasm-pack build --target web --out-name harper_wasm
RUN wasm-pack build --target web --out-name harper_wasm_slim --no-default-features 
RUN cargo clean

FROM node:${NODE_VERSION} AS node-build

RUN apt-get update && apt-get install git parallel -y
RUN npm install --global corepack@0.36.0 && corepack enable

RUN mkdir -p /usr/build/
WORKDIR /usr/build/

COPY . .
COPY --from=wasm-build /usr/build/harper-wasm/pkg /usr/build/harper-wasm/pkg

RUN pnpm install --engine-strict=false --shamefully-hoist

WORKDIR /usr/build/packages/components
RUN pnpm install --engine-strict=false --shamefully-hoist
RUN pnpm build

WORKDIR /usr/build/packages/harper.js

RUN pnpm build && ./docs.sh

WORKDIR /usr/build/packages/lint-framework
RUN pnpm build

WORKDIR /usr/build/packages/harper-editor
RUN pnpm build

WORKDIR /usr/build/packages/web
ENV ENABLE_ADMIN_ROUTES=false
RUN pnpm install --engine-strict=false --shamefully-hoist
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
