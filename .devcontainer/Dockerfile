FROM mcr.microsoft.com/devcontainers/rust:1-1-bullseye

# Include lld linker to improve build times either by using environment variable
# RUSTFLAGS="-C link-arg=-fuse-ld=lld" or with Cargo's configuration file (i.e see .cargo/config.toml).
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
   && apt-get -y install \
      clang \
      lld \
      # For Rust RFD
      libgtk-3-dev \
      zstd

# Install Nomad
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
   && apt-get -y install wget gpg coreutils \
   && wget -O- https://apt.releases.hashicorp.com/gpg | gpg --dearmor -o /usr/share/keyrings/hashicorp-archive-keyring.gpg \
   && echo "deb [signed-by=/usr/share/keyrings/hashicorp-archive-keyring.gpg] https://apt.releases.hashicorp.com $(lsb_release -cs) main" | sudo tee /etc/apt/sources.list.d/hashicorp.list \
   && apt-get update && apt-get install nomad \
   && apt-get autoremove -y && apt-get clean -y

# Change the user to vscode to avoid permission issues when installing packages.
USER vscode

RUN cargo install cargo-quickinstall \
   && cargo quickinstall sea-orm-cli \
   && cargo quickinstall trunk \
   && cargo quickinstall wasm-bindgen-cli \
   && cargo quickinstall wasm-pack \
   && cargo quickinstall just \
   && rustup target add wasm32-unknown-unknown
