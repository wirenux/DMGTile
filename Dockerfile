FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=Etc/UTC

RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    clang \
    lld \
    gcc \
    g++ \
    pkg-config \
    curl \
    ca-certificates \
    libasound2-dev \
    libfontconfig-dev \
    libfreetype-dev \
    libgit2-dev \
    libglib2.0-dev \
    libssl-dev \
    libsqlite3-dev \
    libva-dev \
    libvulkan1 \
    libvulkan-dev \
    libwayland-dev \
    libx11-xcb-dev \
    libxcb1-dev \
    libxkbcommon-x11-dev \
    libxkbcommon-dev \
    libxcomposite-dev \
    libxdamage-dev \
    libxext-dev \
    libxfixes-dev \
    libxrandr-dev \
    libxi-dev \
    libxcursor-dev \
    libdrm-dev \
    libgbm-dev \
    libzstd-dev \
    vulkan-tools \
    mesa-vulkan-drivers \
    libgl1-mesa-dri \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /workspace

CMD ["/bin/bash"]
