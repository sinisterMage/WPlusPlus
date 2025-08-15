FROM debian:bookworm-slim

# Install dependencies (including libicu for .NET)
RUN apt-get update && apt-get install -y \
    curl \
    unzip \
    libicu72 \
    && rm -rf /var/lib/apt/lists/*

# Working directory inside container
WORKDIR /wpp

# Fetch latest release of ingot (linux-x64) and install it
RUN curl -s https://api.github.com/repos/sinisterMage/WPlusPlus/releases/latest \
    | grep "browser_download_url.*ingot-linux-x64.zip" \
    | cut -d '"' -f 4 \
    | xargs curl -L -o ingot-linux-x64.zip \
    && unzip ingot-linux-x64.zip -d /usr/local/bin \
    && rm ingot-linux-x64.zip \
    && chmod +x /usr/local/bin/ingot

# Run "ingot run-file <your-file>" by default
ENTRYPOINT ["ingot", "run-file"]
CMD []
