#!/bin/bash
export PATH="$HOME/.cargo/bin:$PATH"

install_cross() {
    if ! command -v cross &> /dev/null; then
        echo "Installing cross..."
        cargo install cross
    fi
}

install_mingw() {
    if command -v x86_64-w64-mingw32-gcc &> /dev/null; then
        return
    fi
    if command -v apt-get &> /dev/null; then
        sudo apt-get update && sudo apt-get install -y mingw-w64
    elif command -v dnf &> /dev/null; then
        sudo dnf install -y mingw64-gcc
    elif command -v pacman &> /dev/null; then
        sudo pacman -S --needed mingw-w64-gcc
    fi
}

try_build() {
    local cmd="$1"
    local target="$2"
    
    if command -v cross &> /dev/null; then
        if cross build --release --offline --target "$target" 2>/dev/null; then
            return 0
        fi
        if cross build --release --target "$target" 2>/dev/null; then
            return 0
        fi
    fi
    
    if cargo build --release --offline --target "$target" 2>/dev/null; then
        return 0
    fi
    
    if cargo build --release --target "$target" 2>/dev/null; then
        return 0
    fi
    
    if RUSTFLAGS="-C linker=cc" cargo build --release --target "$target" 2>/dev/null; then
        return 0
    fi
    
    if RUSTFLAGS="-C linker=clang" cargo build --release --target "$target" 2>/dev/null; then
        return 0
    fi
    
    return 1
}

targets=(
    "x86_64-unknown-linux-gnu"
    "x86_64-pc-windows-gnu"
    "aarch64-unknown-linux-gnu"
    "aarch64-pc-windows-msvc"
)

for target in "${targets[@]}"; do
    if try_build "cargo" "$target"; then
        echo "Built $target"
    else
        if [[ "$target" == *"windows"* ]]; then
            install_mingw
            if try_build "cargo" "$target"; then
                echo "Built $target"
            else
                echo "Failed to build $target"
            fi
        elif [[ "$target" == "aarch64-"* ]]; then
            install_cross
            if try_build "cargo" "$target"; then
                echo "Built $target"
            else
                echo "Failed to build $target"
            fi
        else
            echo "Failed to build $target"
        fi
    fi
done