FROM ubuntu:24.04

RUN apt-get update && apt-get install -y \
    curl tar jq ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /wpp

RUN set -eux; \
    LATEST_URL=$(curl -s https://api.github.com/repos/sinisterMage/WPlusPlus/releases/latest \
      | jq -r '.assets[] | select(.name | test("wpp-cli-x86_64-unknown-linux-gnu\\.tar\\.gz$")) | .browser_download_url'); \
    echo "📦 Downloading: $LATEST_URL"; \
    curl -L "$LATEST_URL" -o wpp-cli.tar.gz; \
    tar -xzf wpp-cli.tar.gz; \
    mv wpp-cli /usr/local/bin/wpp; \
    chmod +x /usr/local/bin/wpp; \
    rm -f wpp-cli.tar.gz

WORKDIR /app
ENTRYPOINT ["wpp"]
CMD ["--help"]
