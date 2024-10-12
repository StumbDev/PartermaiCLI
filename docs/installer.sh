#!/bin/bash

# Function to check if Rust is installed
check_rust_install() {
  if ! command -v cargo &>/dev/null; then
    echo "Rust not found. Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  else
    echo "Rust is already installed."
  fi
}

# Function to build the Partermai binary
build_partermai() {
  echo "Cloning Partermai repository..."
  git clone https://github.com/StumbDev/PartermaiCLI.git
  cd partermai

  echo "Building Partermai..."
  cargo build --release
  if [ $? -ne 0 ]; then
    echo "Error: Failed to build Partermai."
    exit 1
  fi
}

# Function to move the binary to /usr/local/bin
install_partermai() {
  echo "Installing Partermai..."
  sudo cp target/release/partermai /usr/local/bin/partermai

  if [ $? -eq 0 ]; then
    echo "Partermai installed successfully!"
  else
    echo "Error: Failed to install Partermai."
    exit 1
  fi
}

# Function to clean up after installation
cleanup() {
  echo "Cleaning up..."
  cd ..
  rm -rf partermai
}

# Main installer function
main() {
  check_rust_install
  build_partermai
  install_partermai
  cleanup
}

# Execute the main function
main
